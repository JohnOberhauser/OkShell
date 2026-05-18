use std::fmt;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use tokio::sync::{Mutex, mpsc, oneshot, watch};
use tokio_stream::wrappers::WatchStream;
use tracing::{error, info, warn};
use wayland_client::protocol::{wl_compositor, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
};

static INSTANCE: OnceLock<IdleInhibitor> = OnceLock::new();

fn cache_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("okshell").join("idle_inhibitor"))
}

fn read_cached_state() -> bool {
    let Some(path) = cache_path() else {
        return false;
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => s.trim() == "1",
        Err(_) => false,
    }
}

fn write_cached_state(enabled: bool) {
    let Some(path) = cache_path() else {
        warn!("cannot determine cache path for idle inhibitor state");
        return;
    };
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        warn!("failed to create cache dir {}: {}", parent.display(), e);
        return;
    }
    let contents = if enabled { "1" } else { "0" };
    if let Err(e) = std::fs::write(&path, contents) {
        warn!(
            "failed to write idle inhibitor cache {}: {}",
            path.display(),
            e
        );
    }
}

// === === === === === === === === === ===
// ===       Wayland Internals         ===
// === === === === === === === === === ===

struct AppData {
    compositor: Option<wl_compositor::WlCompositor>,
    manager: Option<ZwpIdleInhibitManagerV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let v = version.min(4);
                    state.compositor =
                        Some(proxy.bind::<wl_compositor::WlCompositor, _, _>(name, v, qh, ()));
                }
                "zwp_idle_inhibit_manager_v1" => {
                    let v = version.min(1);
                    state.manager =
                        Some(proxy.bind::<ZwpIdleInhibitManagerV1, _, _>(name, v, qh, ()));
                }
                _ => {}
            }
        }
    }
}

wayland_client::delegate_noop!(AppData: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(AppData: ignore wl_surface::WlSurface);
wayland_client::delegate_noop!(AppData: ignore ZwpIdleInhibitManagerV1);
wayland_client::delegate_noop!(AppData: ignore ZwpIdleInhibitorV1);

/// Commands sent from the async API to the dedicated Wayland thread.
enum Command {
    SetInhibit { on: bool, ack: oneshot::Sender<()> },
}

/// Eventfd-based waker — writing to it makes poll() wake on this fd.
struct Waker {
    fd: RawFd,
}

impl Waker {
    fn new() -> std::io::Result<Self> {
        // SAFETY: eventfd syscall — no caller-side invariants.
        let fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self { fd })
    }

    fn wake(&self) {
        let val: u64 = 1;
        // SAFETY: 8-byte write to a valid eventfd.
        unsafe {
            libc::write(self.fd, &val as *const u64 as *const _, 8);
        }
    }

    fn drain(&self) {
        let mut buf = [0u8; 8];
        // SAFETY: 8-byte read from a valid eventfd; nonblocking, may EAGAIN.
        unsafe {
            libc::read(self.fd, buf.as_mut_ptr() as *mut _, 8);
        }
    }
}

impl Drop for Waker {
    fn drop(&mut self) {
        // SAFETY: fd is owned.
        unsafe { libc::close(self.fd) };
    }
}

unsafe impl Send for Waker {}
unsafe impl Sync for Waker {}

/// Dedicated Wayland thread: owns the connection, queue, dummy surface, manager,
/// and the live inhibitor. Wakes on either compositor events or commands.
fn wayland_thread(
    mut rx: mpsc::UnboundedReceiver<Command>,
    waker: Arc<Waker>,
    ready_tx: oneshot::Sender<Result<(), IdleError>>,
) {
    let connection = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(e) => {
            warn!("idle inhibitor: failed to connect to wayland: {}", e);
            let _ = ready_tx.send(Err(IdleError));
            return;
        }
    };

    let mut queue: EventQueue<AppData> = connection.new_event_queue();
    let qh = queue.handle();
    connection.display().get_registry(&qh, ());

    let mut data = AppData {
        compositor: None,
        manager: None,
    };

    if queue.roundtrip(&mut data).is_err() {
        let _ = ready_tx.send(Err(IdleError));
        return;
    }

    let Some(compositor) = data.compositor.clone() else {
        warn!("idle inhibitor: compositor missing wl_compositor");
        let _ = ready_tx.send(Err(IdleError));
        return;
    };
    let Some(manager) = data.manager.clone() else {
        warn!("idle inhibitor: compositor missing zwp_idle_inhibit_manager_v1");
        let _ = ready_tx.send(Err(IdleError));
        return;
    };

    // Dummy surface — never attached, never committed, never mapped.
    let surface = compositor.create_surface(&qh, ());
    let _ = connection.flush();

    let wl_fd = connection.as_fd().as_raw_fd();
    let waker_fd = waker.fd;

    let _ = ready_tx.send(Ok(()));

    let mut inhibitor: Option<ZwpIdleInhibitorV1> = None;

    'outer: loop {
        // 1. Drain any queued commands.
        loop {
            match rx.try_recv() {
                Ok(Command::SetInhibit { on, ack }) => {
                    match (on, inhibitor.is_some()) {
                        (true, false) => {
                            inhibitor = Some(manager.create_inhibitor(&surface, &qh, ()));
                        }
                        (false, true) => {
                            if let Some(i) = inhibitor.take() {
                                i.destroy();
                            }
                        }
                        _ => {}
                    }
                    let _ = connection.flush();
                    let _ = ack.send(());
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => break 'outer,
            }
        }

        // 2. Flush any pending requests and dispatch already-buffered events.
        let _ = connection.flush();
        if queue.dispatch_pending(&mut data).is_err() {
            error!("idle inhibitor: dispatch_pending failed");
            break;
        }

        // 3. Prepare to read from the wayland fd.
        // Returns None if events arrived between dispatch_pending and now —
        // in which case loop back and dispatch them.
        let read_guard = match connection.prepare_read() {
            Some(g) => g,
            None => continue,
        };

        // 4. Poll both fds, indefinite timeout.
        let mut fds = [
            libc::pollfd {
                fd: wl_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: waker_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        // SAFETY: valid fds, valid array, indefinite timeout.
        let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                drop(read_guard);
                continue;
            }
            error!("idle inhibitor: poll failed: {}", err);
            break;
        }

        // 5. Handle whichever fd(s) woke us.
        if fds[0].revents & libc::POLLIN != 0 {
            if read_guard.read().is_err() {
                error!("idle inhibitor: read_events failed");
                break;
            }
        } else {
            drop(read_guard);
        }

        if fds[1].revents & libc::POLLIN != 0 {
            waker.drain();
        }
    }

    if let Some(i) = inhibitor.take() {
        i.destroy();
    }
    surface.destroy();
    let _ = connection.flush();
}

// === === === === === === === === === ===
// ===          Public API             ===
// === === === === === === === === === ===

/// Global singleton idle inhibitor.
///
/// Owns a dedicated Wayland connection (independent of GTK/GDK) and a
/// background thread that drains its event queue via poll(2) on the
/// wayland fd plus an eventfd waker for commands. Backed by
/// `zwp_idle_inhibit_manager_v1` on a dummy surface — compositor-wide
/// in effect on Hyprland.
pub struct IdleInhibitor {
    cmd_tx: Mutex<Option<mpsc::UnboundedSender<Command>>>,
    waker: Mutex<Option<Arc<Waker>>>,
    state_tx: watch::Sender<bool>,
    state_rx: watch::Receiver<bool>,
    initialized: AtomicBool,
}

impl IdleInhibitor {
    pub fn global() -> &'static Self {
        INSTANCE.get_or_init(|| {
            let (state_tx, state_rx) = watch::channel(false);
            Self {
                cmd_tx: Mutex::new(None),
                waker: Mutex::new(None),
                state_tx,
                state_rx,
                initialized: AtomicBool::new(false),
            }
        })
    }

    pub async fn init(&self) -> Result<(), IdleError> {
        let mut cmd_guard = self.cmd_tx.lock().await;
        if cmd_guard.is_some() {
            return Ok(());
        }

        let waker = Arc::new(Waker::new().map_err(|_| IdleError)?);
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (ready_tx, ready_rx) = oneshot::channel();

        let waker_thread = Arc::clone(&waker);
        thread::Builder::new()
            .name("okshell-idle-inhibit".into())
            .spawn(move || wayland_thread(cmd_rx, waker_thread, ready_tx))
            .map_err(|_| IdleError)?;

        ready_rx.await.map_err(|_| IdleError)??;

        *cmd_guard = Some(cmd_tx);
        *self.waker.lock().await = Some(waker);
        drop(cmd_guard);

        self.initialized.store(true, Ordering::Release);

        if read_cached_state() {
            self.enable().await?;
        }
        info!("Idle inhibitor initialized");
        Ok(())
    }

    pub fn get(&self) -> bool {
        *self.state_rx.borrow()
    }

    pub fn watch(&self) -> WatchStream<bool> {
        WatchStream::new(self.state_rx.clone())
    }

    async fn send(&self, on: bool) -> Result<(), IdleError> {
        let cmd_guard = self.cmd_tx.lock().await;
        let tx = cmd_guard.as_ref().ok_or(IdleError)?;
        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(Command::SetInhibit { on, ack: ack_tx })
            .map_err(|_| IdleError)?;

        // Wake the wayland thread out of poll so it picks up the command.
        if let Some(w) = self.waker.lock().await.as_ref() {
            w.wake();
        }

        ack_rx.await.map_err(|_| IdleError)
    }

    pub async fn enable(&self) -> Result<(), IdleError> {
        self.send(true).await?;
        let _ = self.state_tx.send(true);
        write_cached_state(true);
        info!("Idle inhibitor enabled");
        Ok(())
    }

    pub async fn disable(&self) {
        let _ = self.send(false).await;
        let _ = self.state_tx.send(false);
        write_cached_state(false);
        info!("Idle inhibitor disabled");
    }

    pub async fn toggle(&self) -> Result<bool, IdleError> {
        if self.get() {
            self.disable().await;
            Ok(false)
        } else {
            self.enable().await?;
            Ok(true)
        }
    }
}

#[derive(Debug)]
pub struct IdleError;

impl fmt::Display for IdleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to set up idle inhibitor")
    }
}

impl std::error::Error for IdleError {}

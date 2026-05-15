use std::ffi::c_void;
use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;

use gtk::gdk;
use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::{NativeExt, ObjectExt};
use relm4::gtk;
use tokio::sync::{Mutex, watch};
use tokio_stream::wrappers::WatchStream;
use tracing::{info, warn};
use wayland_client::backend::{Backend, ObjectId};
use wayland_client::protocol::{wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
};

#[link(name = "gtk-4")]
unsafe extern "C" {
    fn gdk_wayland_display_get_wl_display(display: *mut c_void) -> *mut c_void;
    fn gdk_wayland_surface_get_wl_surface(surface: *mut c_void) -> *mut c_void;
}

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
            if interface == "zwp_idle_inhibit_manager_v1" {
                let v = version.min(1);
                let manager = proxy.bind::<ZwpIdleInhibitManagerV1, _, _>(name, v, qh, ());
                state.manager = Some(manager);
            }
        }
    }
}

impl Dispatch<ZwpIdleInhibitManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ZwpIdleInhibitManagerV1,
        _: <ZwpIdleInhibitManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpIdleInhibitorV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ZwpIdleInhibitorV1,
        _: <ZwpIdleInhibitorV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: <wl_surface::WlSurface as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

/// Holds the Wayland connection, manager, and anchor surface. Once constructed,
/// supports creating/destroying inhibitors on the anchor surface.
struct WaylandInner {
    connection: Connection,
    queue: EventQueue<AppData>,
    data: AppData,
    surface: wl_surface::WlSurface,
    inhibitor: Option<ZwpIdleInhibitorV1>,
}

impl WaylandInner {
    fn new(window: &gtk::Window) -> Result<Self, IdleError> {
        let display = gdk::Display::default().ok_or(IdleError)?;
        if display.type_().name() != "GdkWaylandDisplay" {
            return Err(IdleError);
        }

        // SAFETY: GObject pointer for a verified GdkWaylandDisplay.
        let wl_display_ptr = unsafe {
            let gdk_ptr: *mut gdk::ffi::GdkDisplay = display.to_glib_none().0;
            gdk_wayland_display_get_wl_display(gdk_ptr as *mut c_void)
        };
        if wl_display_ptr.is_null() {
            return Err(IdleError);
        }

        // SAFETY: GDK owns this wl_display and keeps it alive.
        let backend = unsafe { Backend::from_foreign_display(wl_display_ptr as *mut _) };
        let connection = Connection::from_backend(backend);

        let mut queue: EventQueue<AppData> = connection.new_event_queue();
        let qh = queue.handle();
        connection.display().get_registry(&qh, ());

        let mut data = AppData { manager: None };
        queue.roundtrip(&mut data).map_err(|_| IdleError)?;

        if data.manager.is_none() {
            return Err(IdleError);
        }

        let gdk_surface = window.surface().ok_or(IdleError)?;
        if !gdk_surface.type_().name().starts_with("GdkWayland") {
            return Err(IdleError);
        }

        // SAFETY: GObject pointer for a verified GdkWaylandSurface.
        let wl_surface_ptr = unsafe {
            let gdk_ptr: *mut gdk::ffi::GdkSurface = gdk_surface.to_glib_none().0;
            gdk_wayland_surface_get_wl_surface(gdk_ptr as *mut c_void)
        };
        if wl_surface_ptr.is_null() {
            return Err(IdleError);
        }

        // SAFETY: Valid wl_proxy for wl_surface, owned by GDK.
        let surface_id = unsafe {
            ObjectId::from_ptr(wl_surface::WlSurface::interface(), wl_surface_ptr as *mut _)
                .map_err(|_| IdleError)?
        };
        let surface =
            wl_surface::WlSurface::from_id(&connection, surface_id).map_err(|_| IdleError)?;

        Ok(Self {
            connection,
            queue,
            data,
            surface,
            inhibitor: None,
        })
    }

    fn set_inhibit(&mut self, on: bool) {
        match (on, self.inhibitor.is_some()) {
            (true, false) => {
                if let Some(manager) = &self.data.manager {
                    let inhibitor =
                        manager.create_inhibitor(&self.surface, &self.queue.handle(), ());
                    self.inhibitor = Some(inhibitor);
                }
            }
            (false, true) => {
                if let Some(inhibitor) = self.inhibitor.take() {
                    inhibitor.destroy();
                }
            }
            _ => {}
        }
        let _ = self.connection.flush();
    }
}

impl Drop for WaylandInner {
    fn drop(&mut self) {
        if let Some(inhibitor) = self.inhibitor.take() {
            inhibitor.destroy();
            let _ = self.connection.flush();
        }
    }
}

// SAFETY: WaylandInner is only accessed through Mutex in IdleInhibitor.
// The wayland-client Connection is Send + Sync; the proxy objects are too.
// gdk::Surface pointer wrapping happens at construction and we don't retain
// any non-Send GDK references.
unsafe impl Send for WaylandInner {}

// === === === === === === === === === ===
// ===          Public API             ===
// === === === === === === === === === ===

/// Global singleton idle inhibitor.
///
/// Backed by the Wayland `zwp_idle_inhibit_manager_v1` protocol. Call
/// [`init`] once at startup, after the anchor window is realized, to set
/// up the Wayland connection and apply the cached state.
///
/// State is cached to `$XDG_CACHE_HOME/okshell/idle_inhibitor`.
pub struct IdleInhibitor {
    inner: Mutex<Option<WaylandInner>>,
    state_tx: watch::Sender<bool>,
    state_rx: watch::Receiver<bool>,
}

impl IdleInhibitor {
    pub fn global() -> &'static Self {
        INSTANCE.get_or_init(|| {
            let (state_tx, state_rx) = watch::channel(false);
            Self {
                inner: Mutex::new(None),
                state_tx,
                state_rx,
            }
        })
    }

    /// Set up the Wayland connection anchored to `window` and apply cached state.
    /// Call once at startup, after the window is realized. Safe to call again;
    /// subsequent calls are no-ops.
    pub async fn init(&self, window: &gtk::Window) -> Result<(), IdleError> {
        let mut guard = self.inner.lock().await;
        if guard.is_some() {
            return Ok(());
        }
        let inner = WaylandInner::new(window)?;
        *guard = Some(inner);
        drop(guard);

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

    pub async fn enable(&self) -> Result<(), IdleError> {
        let mut guard = self.inner.lock().await;
        let inner = guard.as_mut().ok_or(IdleError)?;
        inner.set_inhibit(true);
        let _ = self.state_tx.send(true);
        write_cached_state(true);
        info!("Idle inhibitor enabled");
        Ok(())
    }

    pub async fn disable(&self) {
        let mut guard = self.inner.lock().await;
        if let Some(inner) = guard.as_mut() {
            inner.set_inhibit(false);
        }
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

//! Singleton idle inhibitor using systemd-logind.
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::{Mutex, watch};
use tokio_stream::wrappers::WatchStream;
use tracing::warn;
use zbus::{Connection, proxy, zvariant::OwnedFd};

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait LogindManager {
    fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> zbus::Result<OwnedFd>;
}

static INSTANCE: OnceLock<IdleInhibitor> = OnceLock::new();

fn cache_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("okshell").join("idle_inhibitor"))
}

/// Read cached state. Returns `false` on any error (missing file, parse failure, etc).
fn read_cached_state() -> bool {
    let Some(path) = cache_path() else {
        return false;
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => s.trim() == "1",
        Err(_) => false,
    }
}

/// Persist state to cache. Best-effort; logs on failure but doesn't propagate errors.
fn write_cached_state(enabled: bool) {
    let Some(path) = cache_path() else {
        warn!("cannot determine cache path for idle inhibitor state");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            warn!("failed to create cache dir {}: {}", parent.display(), e);
            return;
        }
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

/// Global singleton idle inhibitor.
///
/// Holds a logind inhibitor file descriptor while enabled. Dropping the
/// FD (via [`disable`]) releases the inhibitor. State is observable via
/// a `tokio::sync::watch` channel compatible with the `watch!` macros.
///
/// The enabled/disabled state is cached to `$XDG_CACHE_HOME/okshell/idle_inhibitor`
/// (falling back to `~/.cache/okshell/idle_inhibitor`). Call [`init`] at startup
/// to apply the cached state.
pub struct IdleInhibitor {
    /// Held FD when active. `Mutex` because enable/disable are async
    /// and may race; the lock serializes D-Bus calls.
    fd: Mutex<Option<OwnedFd>>,
    state_tx: watch::Sender<bool>,
    state_rx: watch::Receiver<bool>,
    who: String,
}

impl IdleInhibitor {
    pub fn global() -> &'static Self {
        INSTANCE.get_or_init(|| {
            let (state_tx, state_rx) = watch::channel(false);
            Self {
                fd: Mutex::new(None),
                state_tx,
                state_rx,
                who: "okshell".to_string(),
            }
        })
    }

    /// Apply the cached enabled state. Call once at startup after constructing
    /// the tokio runtime. Safe to call multiple times — if already enabled,
    /// subsequent calls are no-ops.
    pub async fn init(&self) -> zbus::Result<()> {
        if read_cached_state() {
            self.enable().await?;
        }
        Ok(())
    }

    pub fn get(&self) -> bool {
        *self.state_rx.borrow()
    }

    pub fn watch(&self) -> WatchStream<bool> {
        WatchStream::new(self.state_rx.clone())
    }

    pub async fn enable(&self) -> zbus::Result<()> {
        let mut guard = self.fd.lock().await;
        if guard.is_some() {
            return Ok(());
        }
        let conn = Connection::system().await?;
        let proxy = LogindManagerProxy::new(&conn).await?;
        let fd = proxy
            .inhibit("idle", &self.who, "User enabled the inhibitor", "block")
            .await?;
        *guard = Some(fd);
        let _ = self.state_tx.send(true);
        write_cached_state(true);
        Ok(())
    }

    pub async fn disable(&self) {
        let mut guard = self.fd.lock().await;
        if guard.take().is_some() {
            let _ = self.state_tx.send(false);
            write_cached_state(false);
        }
    }

    pub async fn toggle(&self) -> zbus::Result<bool> {
        if self.get() {
            self.disable().await;
            Ok(false)
        } else {
            self.enable().await?;
            Ok(true)
        }
    }
}

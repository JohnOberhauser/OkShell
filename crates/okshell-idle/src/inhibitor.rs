//! Singleton idle inhibitor using systemd-logind.

use std::sync::OnceLock;

use tokio::sync::{Mutex, watch};
use tokio_stream::wrappers::WatchStream;
use zbus::{Connection, proxy, zvariant::OwnedFd};

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait LogindManager {
    fn inhibit(
        &self,
        what: &str,
        who: &str,
        why: &str,
        mode: &str,
    ) -> zbus::Result<OwnedFd>;
}

static INSTANCE: OnceLock<IdleInhibitor> = OnceLock::new();

/// Global singleton idle inhibitor.
///
/// Holds a logind inhibitor file descriptor while enabled. Dropping the
/// FD (via [`disable`]) releases the inhibitor. State is observable via
/// a `tokio::sync::watch` channel compatible with the `watch!` macros.
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
        let fd = proxy.inhibit("idle", &self.who, "User enabled the inhibitor", "block").await?;

        *guard = Some(fd);
        let _ = self.state_tx.send(true);
        Ok(())
    }

    pub async fn disable(&self) {
        let mut guard = self.fd.lock().await;
        if guard.take().is_some() {
            let _ = self.state_tx.send(false);
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
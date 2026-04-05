use std::time::Duration;
use tracing::info;
use zbus::proxy;

#[derive(Debug, Clone)]
pub enum FingerprintEvent {
    Ready,
    Scanning,
    Match,
    NoMatch,
    UnknownError,
    Error(String),
}

#[proxy(
    interface = "net.reactivated.Fprint.Manager",
    default_service = "net.reactivated.Fprint",
    default_path = "/net/reactivated/Fprint/Manager"
)]
trait FprintManager {
    fn get_default_device(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "net.reactivated.Fprint.Device",
    default_service = "net.reactivated.Fprint"
)]
pub trait FprintDevice {
    fn claim(&self, username: &str) -> zbus::Result<()>;
    fn verify_start(&self, finger_name: &str) -> zbus::Result<()>;
    fn verify_stop(&self) -> zbus::Result<()>;
    fn release(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn verify_status(&self, result: &str, done: bool) -> zbus::Result<()>;

    #[zbus(signal)]
    fn verify_finger_required(&self, finger_name: &str) -> zbus::Result<()>;
}

pub struct FingerprintAuth {
    pub device: FprintDeviceProxy<'static>,
}

impl FingerprintAuth {
    pub async fn new() -> zbus::Result<Self> {
        let conn = zbus::Connection::system().await?;

        let manager = FprintManagerProxy::new(&conn).await?;
        let device_path = manager.get_default_device().await?;

        let device = FprintDeviceProxy::builder(&conn)
            .path(device_path)?
            .build()
            .await?;

        Ok(Self { device })
    }

    pub async fn start(&self, username: &str) -> zbus::Result<()> {
        let _ = self.device.verify_stop().await;
        let _ = self.device.release().await;

        let mut delay = std::time::Duration::from_millis(200);
        for attempt in 1..=5 {
            match self.device.claim(username).await {
                Ok(()) => {
                    self.device.verify_start("any").await?;
                    return Ok(());
                }
                Err(e) if attempt < 5 => {
                    let msg = e.to_string();
                    if msg.contains("busy") || msg.contains("Internal") {
                        info!("fprintd claim attempt {attempt}/5: {msg}, retrying in {delay:?}");
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                    } else {
                        return Err(e);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    /// Blocks until a verify result comes in. Returns the event.
    pub async fn wait_for_result(&self) -> zbus::Result<FingerprintEvent> {
        use futures::StreamExt;

        let mut stream = self.device.receive_verify_status().await?;

        if let Some(signal) = stream.next().await {
            let args = signal.args()?;
            info!("fprintd verify status: {} done: {}", args.result, args.done);

            let event = match args.result {
                "verify-match" => FingerprintEvent::Match,
                "verify-no-match" |
                "verify-retry-scan" |
                "verify-swipe-too-short" |
                "verify-finger-not-centered" |
                "verify-remove-and-retry" => FingerprintEvent::NoMatch,
                "verify-unknown-error" => FingerprintEvent::UnknownError,
                "verify-disconnected" => FingerprintEvent::Error("Device disconnected".into()),
                other => FingerprintEvent::Error(format!("Unexpected: {other}")),
            };

            Ok(event)
        } else {
            Ok(FingerprintEvent::Error("Signal stream ended".into()))
        }
    }

    pub async fn stop(&self) -> zbus::Result<()> {
        let _ = self.device.verify_stop().await;
        let _ = self.device.release().await;
        Ok(())
    }
}

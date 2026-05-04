pub type Result<T> = std::result::Result<T, ScreenshotError>;

/// Errors that can occur during screenshot operations.
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    #[error("wayland connection failed: {0}")]
    WaylandConnect(String),

    #[error("wlr-screencopy protocol not supported by compositor")]
    ProtocolNotSupported,

    #[error("output not found: {0}")]
    OutputNotFound(String),

    #[error("capture failed: {0}")]
    CaptureFailed(String),

    #[error("image encoding failed: {0}")]
    EncodingFailed(String),

    #[error("clipboard operation failed: {0}")]
    ClipboardFailed(String),

    #[error("file I/O failed: {0}")]
    IoFailed(#[from] std::io::Error),

    #[error("hyprland IPC failed: {0}")]
    HyprlandIpc(String),

    #[error("user cancelled selection")]
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale: f64,
}

/// Identifies a Hyprland window for capture.
#[derive(Debug, Clone)]
pub struct HyprlandWindow {
    /// Hyprland window address (hex string like "0x5678abcd").
    pub address: String,
    /// Output the window is on — needed to know which output to screencopy.
    pub output: String,
    /// Window geometry in global compositor coordinates.
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

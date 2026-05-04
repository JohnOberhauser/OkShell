use crate::relm_app::{Shell, ShellInput};
use okshell_cache::wallpaper::set_wallpaper;
use okshell_services::{audio_service, brightness_service, hyprland_service};
use okshell_session::session_lock::session_lock;
use okshell_settings::{close_settings, open_settings};
use okshell_sounds::play_audio_volume_change;
use relm4::gtk::glib;
use relm4::{ComponentSender, gtk};
use std::path::PathBuf;
use tokio::sync::mpsc;
use wayle_audio::volume::types::Volume;
use wayle_brightness::Percentage;
use zbus::connection;
use zbus::interface;

pub fn init_ipc_shell_service(sender: &ComponentSender<Shell>) {
    let (shell_tx, mut shell_rx) = mpsc::unbounded_channel();

    tokio::spawn(start_shell_service(shell_tx));

    let app_sender = sender.input_sender().clone();
    glib::spawn_future_local(async move {
        while let Some(cmd) = shell_rx.recv().await {
            match cmd {
                IPCCommand::Quit => app_sender.emit(ShellInput::Quit),
                IPCCommand::AppLauncher => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleAppLauncher(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::ToggleAppLauncher(None));
                    }
                }
                IPCCommand::QuickSettings => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleQuickSettings(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::ToggleQuickSettings(None));
                    }
                }
                IPCCommand::Clock => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleClockMenu(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::ToggleClockMenu(None));
                    }
                }
                IPCCommand::Clipboard => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleClipboard(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::ToggleClipboard(None));
                    }
                }
                IPCCommand::Notifications => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleNotifications(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::ToggleNotifications(None));
                    }
                }
                IPCCommand::Screenshot => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleScreenshotMenu(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::ToggleScreenshotMenu(None));
                    }
                }
                IPCCommand::Wallpaper => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleWallpaperMenu(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::ToggleWallpaperMenu(None));
                    }
                }
                IPCCommand::CloseAllMenus => app_sender.emit(ShellInput::CloseAllMenus),
                IPCCommand::VolumeUp => {
                    if let Some(output) = audio_service().default_output.get() {
                        let current_volume = output.volume.get();
                        let max_volume: f64 = 1.0;
                        let new_volume = max_volume.min(current_volume.average() + 0.05);
                        let _ = output
                            .set_volume(Volume::stereo(new_volume, new_volume))
                            .await;
                    }
                    play_audio_volume_change();
                }
                IPCCommand::VolumeDown => {
                    if let Some(output) = audio_service().default_output.get() {
                        let current_volume = output.volume.get();
                        let min_volume: f64 = 0.0;
                        let new_volume = min_volume.max(current_volume.average() - 0.05);
                        let _ = output
                            .set_volume(Volume::stereo(new_volume, new_volume))
                            .await;
                    }
                    play_audio_volume_change();
                }
                IPCCommand::Mute => {
                    if let Some(output) = audio_service().default_output.get() {
                        let _ = output.set_mute(!output.muted.get()).await;
                    }
                    play_audio_volume_change();
                }
                IPCCommand::BrightnessUp => {
                    if let Some(brightness_service) = brightness_service() {
                        if let Some(primary) = brightness_service.primary.get() {
                            let current_brightness = primary.percentage().value();
                            let max_brightness: f64 = 100.0;
                            let new_brightness = max_brightness.min(current_brightness + 5.0);
                            let _ = primary
                                .set_percentage(Percentage::new(new_brightness))
                                .await;
                        }
                    }
                }
                IPCCommand::BrightnessDown => {
                    if let Some(brightness_service) = brightness_service() {
                        if let Some(primary) = brightness_service.primary.get() {
                            let current_brightness = primary.percentage().value();
                            let min_brightness: f64 = 0.0;
                            let new_brightness = min_brightness.max(current_brightness - 5.0);
                            let _ = primary
                                .set_percentage(Percentage::new(new_brightness))
                                .await;
                        }
                    }
                }
                IPCCommand::SetWallpaper(path) => {
                    set_wallpaper(&path);
                }
                IPCCommand::Lock => {
                    session_lock().lock();
                }
                IPCCommand::CheckLock(reply) => {
                    let _ = reply.send(session_lock().is_locked());
                }
                IPCCommand::Screenshare(reply, payload) => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::ToggleScreenshareMenu(
                            Some(active_workspace.monitor.get()),
                            reply,
                            payload,
                        ));
                    } else {
                        app_sender.emit(ShellInput::ToggleScreenshareMenu(None, reply, payload));
                    }
                }
                IPCCommand::OpenSettings => {
                    open_settings();
                }
                IPCCommand::CloseSettings => {
                    close_settings();
                }
                IPCCommand::Inspect => {
                    gtk::Window::set_interactive_debugging(true);
                }
                IPCCommand::BarToggleTop => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::BarToggleTop(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::BarToggleTop(None));
                    }
                }
                IPCCommand::BarToggleBottom => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::BarToggleBottom(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::BarToggleBottom(None));
                    }
                }
                IPCCommand::BarToggleLeft => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::BarToggleLeft(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::BarToggleLeft(None));
                    }
                }
                IPCCommand::BarToggleRight => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::BarToggleRight(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::BarToggleRight(None));
                    }
                }
                IPCCommand::BarToggleAll => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::BarToggleAll(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::BarToggleAll(None));
                    }
                }
                IPCCommand::BarRevealAll => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender.emit(ShellInput::BarRevealAll(Some(
                            active_workspace.monitor.get(),
                        )));
                    } else {
                        app_sender.emit(ShellInput::BarRevealAll(None));
                    }
                }
                IPCCommand::BarHideAll => {
                    if let Some(active_workspace) = hyprland_service().active_workspace().await {
                        app_sender
                            .emit(ShellInput::BarHideAll(Some(active_workspace.monitor.get())));
                    } else {
                        app_sender.emit(ShellInput::BarHideAll(None));
                    }
                }
            }
        }
    });
}

enum IPCCommand {
    Quit,
    QuickSettings,
    AppLauncher,
    Clock,
    Clipboard,
    Notifications,
    Screenshot,
    Wallpaper,
    CloseAllMenus,
    VolumeUp,
    VolumeDown,
    Mute,
    BrightnessUp,
    BrightnessDown,
    SetWallpaper(PathBuf),
    Lock,
    CheckLock(tokio::sync::oneshot::Sender<bool>),
    Screenshare(tokio::sync::oneshot::Sender<String>, String),
    OpenSettings,
    CloseSettings,
    Inspect,
    BarToggleTop,
    BarToggleBottom,
    BarToggleLeft,
    BarToggleRight,
    BarToggleAll,
    BarRevealAll,
    BarHideAll,
}

struct IPCService {
    tx: mpsc::UnboundedSender<IPCCommand>,
}

impl IPCService {
    pub fn new(tx: mpsc::UnboundedSender<IPCCommand>) -> Self {
        Self { tx }
    }
}

#[interface(name = "com.okshell.Shell")]
impl IPCService {
    async fn quit(&self) {
        let _ = self.tx.send(IPCCommand::Quit);
    }
    async fn quick_settings(&self) {
        let _ = self.tx.send(IPCCommand::QuickSettings);
    }
    async fn app_launcher(&self) {
        let _ = self.tx.send(IPCCommand::AppLauncher);
    }
    async fn clock(&self) {
        let _ = self.tx.send(IPCCommand::Clock);
    }
    async fn clipboard(&self) {
        let _ = self.tx.send(IPCCommand::Clipboard);
    }
    async fn notifications(&self) {
        let _ = self.tx.send(IPCCommand::Notifications);
    }
    async fn screenshot(&self) {
        let _ = self.tx.send(IPCCommand::Screenshot);
    }
    async fn wallpaper(&self) {
        let _ = self.tx.send(IPCCommand::Wallpaper);
    }
    async fn close_all_menus(&self) {
        let _ = self.tx.send(IPCCommand::CloseAllMenus);
    }
    async fn volume_up(&self) {
        let _ = self.tx.send(IPCCommand::VolumeUp);
    }
    async fn volume_down(&self) {
        let _ = self.tx.send(IPCCommand::VolumeDown);
    }
    async fn mute(&self) {
        let _ = self.tx.send(IPCCommand::Mute);
    }
    async fn brightness_up(&self) {
        let _ = self.tx.send(IPCCommand::BrightnessUp);
    }
    async fn brightness_down(&self) {
        let _ = self.tx.send(IPCCommand::BrightnessDown);
    }
    async fn set_wallpaper(&self, path: &str) {
        let _ = self.tx.send(IPCCommand::SetWallpaper(PathBuf::from(path)));
    }
    async fn lock(&self) {
        let _ = self.tx.send(IPCCommand::Lock);
    }
    async fn check_lock(&self) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(IPCCommand::CheckLock(tx));
        rx.await.unwrap_or(false)
    }
    async fn screenshare(&self, payload: &str) -> String {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self
            .tx
            .send(IPCCommand::Screenshare(tx, payload.to_string()));
        rx.await.unwrap_or(String::new())
    }
    async fn open_settings(&self) {
        let _ = self.tx.send(IPCCommand::OpenSettings);
    }
    async fn close_settings(&self) {
        let _ = self.tx.send(IPCCommand::CloseSettings);
    }
    async fn inspect(&self) {
        let _ = self.tx.send(IPCCommand::Inspect);
    }
    async fn bar_toggle_top(&self) {
        let _ = self.tx.send(IPCCommand::BarToggleTop);
    }
    async fn bar_toggle_bottom(&self) {
        let _ = self.tx.send(IPCCommand::BarToggleBottom);
    }
    async fn bar_toggle_left(&self) {
        let _ = self.tx.send(IPCCommand::BarToggleLeft);
    }
    async fn bar_toggle_right(&self) {
        let _ = self.tx.send(IPCCommand::BarToggleRight);
    }
    async fn bar_toggle_all(&self) {
        let _ = self.tx.send(IPCCommand::BarToggleAll);
    }
    async fn bar_reveal_all(&self) {
        let _ = self.tx.send(IPCCommand::BarRevealAll);
    }
    async fn bar_hide_all(&self) {
        let _ = self.tx.send(IPCCommand::BarHideAll);
    }
}

async fn start_shell_service(tx: mpsc::UnboundedSender<IPCCommand>) -> zbus::Result<()> {
    let service = IPCService::new(tx);
    let _connection = connection::Builder::session()?
        .name("com.okshell.Shell")?
        .serve_at("/com/okshell/Shell", service)?
        .build()
        .await?;
    std::future::pending::<()>().await;
    Ok(())
}

use crate::ipc::init_ipc_shell_service;
use crate::monitors;
use gtk4_layer_shell::{Layer, LayerShell};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{BarsStoreFields, ConfigStoreFields, FrameStoreFields};
use okshell_frame::frame::{Frame, FrameInit, FrameInput};
use okshell_lockscreen::lock_screen_manager::{LockScreenManagerInit, LockScreenManagerModel};
use okshell_notification_popups::popup_notifications::{
    PopupNotificationsInit, PopupNotificationsModel,
};
use okshell_osd::brightness_osd::{BrightnessOsdInit, BrightnessOsdModel};
use okshell_osd::sound_alerts::SoundAlertsModel;
use okshell_osd::volume_osd::{VolumeOsdInit, VolumeOsdModel};
use okshell_polkit::PolkitPromptModel;
use okshell_style::style_manager::{StyleManagerModel, StyleManagerOutput};
use okshell_wallpaper::wallpaper::{WallpaperInit, WallpaperModel};
use reactive_graph::effect::Effect;
use reactive_graph::traits::{Get, GetUntracked};
use relm4::gtk::gdk::Monitor;
use relm4::{gtk::prelude::*, main_application, prelude::*};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use tracing::info;

pub(crate) struct WindowGroup {
    pub monitor: Monitor,
    pub frame: Option<Controller<Frame>>,
    pub _wallpaper: Option<Controller<WallpaperModel>>,
    pub _popup_notifications: Option<Controller<PopupNotificationsModel>>,
    pub _volume_osd: Option<Controller<VolumeOsdModel>>,
    pub _brightness_osd: Option<Controller<BrightnessOsdModel>>,
}

pub(crate) struct Shell {
    window_groups: HashMap<String, WindowGroup>,
    _lock_screen_manager: Controller<LockScreenManagerModel>,
    _polkit: Controller<PolkitPromptModel>,
    _sound_alerts: Controller<SoundAlertsModel>,
    _style_manager: Controller<StyleManagerModel>,
    monitor_filter: Vec<String>,
}

pub(crate) struct ShellInit {}

#[derive(Debug)]
pub(crate) enum ShellInput {
    SyncMonitors,
    MonitorFilterUpdated(Vec<String>),
    AddWindowGroup(String, Monitor),
    RemoveWindowGroup(String),
    Quit,
    ToggleQuickSettings(Option<String>),
    ToggleAppLauncher(Option<String>),
    ToggleClipboard(Option<String>),
    ToggleClockMenu(Option<String>),
    ToggleNotifications(Option<String>),
    ToggleScreenshotMenu(Option<String>),
    ToggleWallpaperMenu(Option<String>),
    CloseAllMenus,
    ToggleScreenshareMenu(Option<String>, tokio::sync::oneshot::Sender<String>, String),
    QueueFrameRedraw,
    BarToggleTop(Option<String>),
    BarToggleBottom(Option<String>),
    BarToggleLeft(Option<String>),
    BarToggleRight(Option<String>),
    BarToggleAll(Option<String>, bool),
    BarRevealAll(Option<String>, bool),
    BarHideAll(Option<String>, bool),
}

#[derive(Debug)]
pub enum ShellCommandOutput {}

#[relm4::component(pub(crate))]
impl Component for Shell {
    type CommandOutput = ShellCommandOutput;
    type Input = ShellInput;
    type Output = ();
    type Init = ShellInit;

    view! {
        #[root]
        gtk::Window {
            set_decorated: false,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let icon_theme = gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap());
        icon_theme.add_search_path(dirs::home_dir().unwrap().join(".config/okshell/icons"));

        root.init_layer_shell();
        root.set_layer(Layer::Background);
        root.set_default_size(1, 1);
        root.set_visible(true);
        root.set_namespace(Some("okshell-invisible-root"));

        root.connect_realize(|window| {
            let window = window.clone();
            relm4::spawn_local(async move {
                if let Err(e) = okshell_idle::wayland::IdleInhibitor::global()
                    .init(&window)
                    .await
                {
                    tracing::warn!("failed to init idle inhibitor: {e}");
                }
            });
        });

        let widgets = view_output!();

        let window_groups = HashMap::new();

        let lock_screen_manager = LockScreenManagerModel::builder()
            .launch(LockScreenManagerInit {})
            .detach();

        let polkit = PolkitPromptModel::builder().launch(()).detach();

        let sound_alerts = SoundAlertsModel::builder().launch(()).detach();

        let style_manager = StyleManagerModel::builder().launch(()).forward(
            sender.input_sender(),
            |msg| match msg {
                StyleManagerOutput::QueueFrameRedraw => ShellInput::QueueFrameRedraw,
            },
        );

        let sender_clone = sender.clone();
        Effect::new(move |_| {
            let monitors = config_manager()
                .config()
                .bars()
                .frame()
                .monitor_filter()
                .get();
            sender_clone.input(ShellInput::MonitorFilterUpdated(monitors));
        });

        let monitor_filter = config_manager()
            .config()
            .bars()
            .frame()
            .monitor_filter()
            .get_untracked();

        init_ipc_shell_service(&sender);

        let model = Shell {
            window_groups,
            _lock_screen_manager: lock_screen_manager,
            _polkit: polkit,
            _sound_alerts: sound_alerts,
            _style_manager: style_manager,
            monitor_filter,
        };

        monitors::setup_monitor_watcher(&sender);
        monitors::sync_monitors(&model.window_groups, &sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            ShellInput::SyncMonitors => {
                monitors::sync_monitors(&self.window_groups, &sender);
            }
            ShellInput::MonitorFilterUpdated(monitors) => {
                self.monitor_filter = monitors;

                for (name, group) in self.window_groups.iter_mut() {
                    let should_have_frame =
                        self.monitor_filter.is_empty() || self.monitor_filter.contains(name);

                    if should_have_frame && group.frame.is_none() {
                        group.frame = Some(
                            Frame::builder()
                                .launch(FrameInit {
                                    monitor: group.monitor.clone(),
                                })
                                .detach(),
                        );
                    } else if !should_have_frame && let Some(frame) = group.frame.take() {
                        frame.widget().close();
                    }
                }
            }
            ShellInput::AddWindowGroup(name, monitor) => {
                info!("Creating new window group");
                let wallpaper = Some(
                    WallpaperModel::builder()
                        .launch(WallpaperInit {
                            monitor: monitor.clone(),
                        })
                        .detach(),
                );

                let create_frame =
                    self.monitor_filter.is_empty() || self.monitor_filter.contains(&name);

                let frame = if create_frame {
                    Some(
                        Frame::builder()
                            .launch(FrameInit {
                                monitor: monitor.clone(),
                            })
                            .detach(),
                    )
                } else {
                    None
                };

                let popup_notifications = Some(
                    PopupNotificationsModel::builder()
                        .launch(PopupNotificationsInit {
                            monitor: monitor.clone(),
                        })
                        .detach(),
                );

                let volume_osd = Some(
                    VolumeOsdModel::builder()
                        .launch(VolumeOsdInit {
                            monitor: monitor.clone(),
                        })
                        .detach(),
                );

                let brightness_osd = Some(
                    BrightnessOsdModel::builder()
                        .launch(BrightnessOsdInit {
                            monitor: monitor.clone(),
                        })
                        .detach(),
                );

                let window_group = WindowGroup {
                    monitor: monitor.clone(),
                    frame,
                    _wallpaper: wallpaper,
                    _popup_notifications: popup_notifications,
                    _volume_osd: volume_osd,
                    _brightness_osd: brightness_osd,
                };

                self.window_groups.insert(name, window_group);
            }
            ShellInput::RemoveWindowGroup(name) => {
                if let Some(group) = self.window_groups.remove(&name) {
                    if let Some(frame) = &group.frame {
                        frame.widget().close();
                    }
                    if let Some(wallpaper) = &group._wallpaper {
                        wallpaper.widget().close();
                    }
                    if let Some(popup) = &group._popup_notifications {
                        popup.widget().close();
                    }
                    if let Some(vol) = &group._volume_osd {
                        vol.widget().close();
                    }
                    if let Some(bright) = &group._brightness_osd {
                        bright.widget().close();
                    }
                }
            }
            ShellInput::Quit => {
                main_application().quit();
            }
            ShellInput::ToggleAppLauncher(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleAppLauncherMenu);
                }
            }
            ShellInput::ToggleScreenshotMenu(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleScreenshotMenu);
                }
            }
            ShellInput::ToggleNotifications(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleNotificationMenu);
                }
            }
            ShellInput::ToggleClipboard(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleClipboardMenu);
                }
            }
            ShellInput::ToggleClockMenu(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleClockMenu);
                }
            }
            ShellInput::ToggleQuickSettings(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleQuickSettingsMenu);
                }
            }
            ShellInput::ToggleWallpaperMenu(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleWallpaperMenu);
                }
            }
            ShellInput::CloseAllMenus => {
                self.window_groups.iter().for_each(|(_, wg)| {
                    if let Some(frame) = &wg.frame {
                        frame.emit(FrameInput::CloseMenus);
                    }
                });
            }
            ShellInput::ToggleScreenshareMenu(monitor_name, reply, payload) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::ToggleScreenshareMenu(reply, payload));
                }
            }
            ShellInput::QueueFrameRedraw => {
                self.window_groups.iter().for_each(|(_, wg)| {
                    if let Some(frame) = &wg.frame {
                        frame.emit(FrameInput::QueueFrameRedraw);
                    }
                });
            }
            ShellInput::BarToggleTop(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::BarToggleTop);
                }
            }
            ShellInput::BarToggleBottom(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::BarToggleBottom);
                }
            }
            ShellInput::BarToggleLeft(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::BarToggleLeft);
                }
            }
            ShellInput::BarToggleRight(monitor_name) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::BarToggleRight);
                }
            }
            ShellInput::BarToggleAll(monitor_name, exclude_hidden_by_default) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::BarToggleAll(exclude_hidden_by_default));
                }
            }
            ShellInput::BarRevealAll(monitor_name, exclude_hidden_by_default) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::BarRevealAll(exclude_hidden_by_default));
                }
            }
            ShellInput::BarHideAll(monitor_name, exclude_hidden_by_default) => {
                if let Some(frame) = resolve_frame(&self.window_groups, &monitor_name) {
                    frame.emit(FrameInput::BarHideAll(exclude_hidden_by_default));
                }
            }
        }
    }
}

/// Get the frame for the monitor name.  If it doesn't exist, get the first frame available
fn resolve_frame<'a>(
    window_groups: &'a HashMap<String, WindowGroup>,
    monitor_name: &Option<String>,
) -> Option<&'a Controller<Frame>> {
    if let Some(name) = monitor_name
        && let Some(frame) = window_groups.get(name).and_then(|g| g.frame.as_ref())
    {
        return Some(frame);
    }
    window_groups.values().find_map(|g| g.frame.as_ref())
}

impl Debug for WindowGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowGroup").finish()
    }
}

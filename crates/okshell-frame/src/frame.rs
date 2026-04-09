use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;
use relm4::gtk::{self, gdk, prelude::*, glib, Widget, Stack};
use reactive_graph::traits::*;
use relm4::RelmRemoveAllExt;
use tracing::info;
use okshell_common::box_with_resize::BoxWithResize;
use okshell_common::scoped_effects::EffectScope;
use okshell_config::{
    schema::config::*
};
use okshell_config::schema::position::Position;
use crate::bars::bar::{BarInit, BarModel, BarOutput, BarType};

use crate::frame_draw_widget::FrameDrawWidget;
use crate::frame_spacer::{FrameSpacerInit, FrameSpacerInput, FrameSpacerModel};
use crate::menus::menu::{MenuInit, MenuInput, MenuModel, MenuOutput, MenuType};
use crate::menus::menu::MenuInput::ForwardHyprlandScreenshareReply;

const CLOCK_MENU: &str = "clock";
const CLIPBOARD_MENU: &str = "clipboard";
const QUICK_SETTINGS_MENU: &str = "quick_settings";
const APP_LAUNCHER_MENU: &str = "app_launcher";
const SCREENSHOT_MENU: &str = "screenshot";
const NOTIFICATION_MENU: &str = "notification";
const WALLPAPER_MENU: &str = "wallpaper";
const SCREENSHARE_MENU: &str = "screenshare";

pub struct Frame {
    top_bar: Controller<BarModel>,
    bottom_bar: Controller<BarModel>,
    left_bar: Controller<BarModel>,
    right_bar: Controller<BarModel>,
    left_menu_expansion_type: VerticalMenuExpansion,
    right_menu_expansion_type: VerticalMenuExpansion,
    left_revealed: bool,
    right_revealed: bool,
    top_revealed: bool,
    top_left_revealed: bool,
    top_right_revealed: bool,
    bottom_revealed: bool,
    bottom_left_revealed: bool,
    bottom_right_revealed: bool,
    top_spacer: Controller<FrameSpacerModel>,
    bottom_spacer: Controller<FrameSpacerModel>,
    left_spacer: Controller<FrameSpacerModel>,
    right_spacer: Controller<FrameSpacerModel>,
    clock_menu: Controller<MenuModel>,
    clipboard_menu: Controller<MenuModel>,
    quick_settings_menu: Controller<MenuModel>,
    notification_menu: Controller<MenuModel>,
    screenshot_menu: Controller<MenuModel>,
    app_launcher_menu: Controller<MenuModel>,
    wallpaper_menu: Controller<MenuModel>,
    screenshare_menu: Controller<MenuModel>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub enum FrameInput {
    SetDrawFrame(bool),
    QueueFrameRedraw,
    SetLeftMenuExpansionType(VerticalMenuExpansion),
    SetRightMenuExpansionType(VerticalMenuExpansion),
    RepositionMenus(Position, Position, Position, Position, Position, Position, Position, Position),
    ToggleClockMenu,
    ToggleClipboardMenu,
    ToggleQuickSettingsMenu,
    ToggleNotificationMenu,
    ToggleScreenshotMenu,
    ToggleAppLauncherMenu,
    ToggleWallpaperMenu,
    CloseMenus,
    ToggleScreenshareMenu(tokio::sync::oneshot::Sender<String>, String),
}

#[derive(Debug)]
pub enum FrameOutput {}

pub struct FrameInit {
    pub monitor: gdk::Monitor,
}

#[relm4::component(pub)]
impl Component for Frame {
    type CommandOutput = ();
    type Input = FrameInput;
    type Output = FrameOutput;
    type Init = FrameInit;

    view! {
        gtk::Window {
            set_css_classes: &["frame-window", "window-opacity"],

            // Fill the window; draw happens in the DrawingArea
            // OkPanel required a hacky fix for fractional scaling.  Might need to do that here at some point.
            #[name = "overlay"]
            gtk::Overlay {
                set_hexpand: true,
                set_vexpand: true,

                add_overlay = &gtk::Box {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "top_bar_container"]
                    BoxWithResize::new() -> BoxWithResize {
                        set_hexpand: true,
                        append = &model.top_bar.widget().clone() {},
                    },

                    gtk::Box {
                        set_vexpand: true,
                        set_hexpand: true,
                        set_orientation: gtk::Orientation::Horizontal,

                        #[name = "left_bar_and_menu_container"]
                        BoxWithResize::new() -> BoxWithResize {

                            #[name = "left_bar_container"]
                            append = &BoxWithResize::new() -> BoxWithResize {
                                append = &model.left_bar.widget().clone() {},
                            },

                            #[name = "left_revealer_container"]
                            append = &BoxWithResize::new() -> BoxWithResize {

                                #[name = "left_revealer"]
                                append = &gtk::Revealer {
                                    set_transition_type: gtk::RevealerTransitionType::SlideRight,
                                    #[watch]
                                    set_reveal_child: model.left_revealed,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_vexpand: true,
                                        set_hexpand: false,

                                        #[name = "top_left_expander"]
                                        BoxWithResize::new() -> BoxWithResize {
                                            set_hexpand: true,
                                            #[watch]
                                            set_vexpand: match model.left_menu_expansion_type {
                                                VerticalMenuExpansion::AlwaysExpanded => {false}
                                                VerticalMenuExpansion::ExpandBothWays => {true}
                                                VerticalMenuExpansion::ExpandDown => {false}
                                                VerticalMenuExpansion::ExpandUp => {true}
                                            },
                                        },

                                        #[name = "left_stack"]
                                        gtk::Stack {
                                            set_transition_type: gtk::StackTransitionType::Crossfade,
                                            set_transition_duration: 200,
                                            set_vhomogeneous: false,
                                            set_hhomogeneous: false,
                                        },

                                        #[name = "bottom_left_expander"]
                                        BoxWithResize::new() -> BoxWithResize {
                                            set_hexpand: true,
                                            #[watch]
                                            set_vexpand: match model.left_menu_expansion_type {
                                                VerticalMenuExpansion::AlwaysExpanded => {false}
                                                VerticalMenuExpansion::ExpandBothWays => {true}
                                                VerticalMenuExpansion::ExpandDown => {true}
                                                VerticalMenuExpansion::ExpandUp => {false}
                                            },
                                        },
                                    },
                                },
                            },
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_hexpand: true,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_hexpand: false,

                                // This box is required to prevent the top and bottom menus
                                // from matching each other's width if one is larger than the other
                                gtk::Box {

                                    gtk::Box {
                                        set_hexpand: true,
                                    },

                                    #[name = "top_left_revealer_container"]
                                    BoxWithResize::new() -> BoxWithResize {

                                        #[name = "top_left_revealer"]
                                        append = &gtk::Revealer {
                                            set_transition_type: gtk::RevealerTransitionType::SlideDown,
                                            #[watch]
                                            set_reveal_child: model.top_left_revealed,

                                            #[name = "top_left_stack"]
                                            gtk::Stack {
                                                set_transition_type: gtk::StackTransitionType::Crossfade,
                                                set_transition_duration: 200,
                                                set_vhomogeneous: false,
                                                set_hhomogeneous: false,
                                            },
                                        },
                                    },

                                    gtk::Box {
                                        set_hexpand: true,
                                    },
                                },

                                gtk::Box {
                                    set_height_request: 200,
                                    set_vexpand: true,
                                },

                                // This box is required to prevent the top and bottom menus
                                // from matching each other's width if one is larger than the other
                                gtk::Box {

                                    gtk::Box {
                                        set_hexpand: true,
                                    },

                                    #[name = "bottom_left_revealer_container"]
                                    BoxWithResize::new() -> BoxWithResize {

                                        #[name = "bottom_left_revealer"]
                                        append = &gtk::Revealer {
                                            set_transition_type: gtk::RevealerTransitionType::SlideDown,
                                            #[watch]
                                            set_reveal_child: model.bottom_left_revealed,

                                            #[name = "bottom_left_stack"]
                                            gtk::Stack {
                                                set_transition_type: gtk::StackTransitionType::Crossfade,
                                                set_transition_duration: 200,
                                                set_vhomogeneous: false,
                                                set_hhomogeneous: false,
                                            },
                                        },
                                    },

                                    gtk::Box {
                                        set_hexpand: true,
                                    },
                                },
                            },

                            gtk::Box {
                                set_hexpand: true,
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_hexpand: false,

                                // This box is required to prevent the top and bottom menus
                                // from matching each other's width if one is larger than the other
                                gtk::Box {

                                    gtk::Box {
                                        set_hexpand: true,
                                    },

                                    #[name = "top_revealer_container"]
                                    BoxWithResize::new() -> BoxWithResize {

                                        #[name = "top_revealer"]
                                        append = &gtk::Revealer {
                                            set_transition_type: gtk::RevealerTransitionType::SlideDown,
                                            #[watch]
                                            set_reveal_child: model.top_revealed,

                                            #[name = "top_stack"]
                                            gtk::Stack {
                                                set_transition_type: gtk::StackTransitionType::Crossfade,
                                                set_transition_duration: 200,
                                                set_vhomogeneous: false,
                                                set_hhomogeneous: false,
                                            },
                                        },
                                    },

                                    gtk::Box {
                                        set_hexpand: true,
                                    },
                                },

                                gtk::Box {
                                    set_height_request: 200,
                                    set_vexpand: true,
                                },

                                // This box is required to prevent the top and bottom menus
                                // from matching each other's width if one is larger than the other
                                gtk::Box {

                                    gtk::Box {
                                        set_hexpand: true,
                                    },

                                    #[name = "bottom_revealer_container"]
                                    BoxWithResize::new() -> BoxWithResize {

                                        #[name = "bottom_revealer"]
                                        append = &gtk::Revealer {
                                            set_transition_type: gtk::RevealerTransitionType::SlideUp,
                                            #[watch]
                                            set_reveal_child: model.bottom_revealed,

                                            #[name = "bottom_stack"]
                                            gtk::Stack {
                                                set_transition_type: gtk::StackTransitionType::Crossfade,
                                                set_transition_duration: 200,
                                                set_vhomogeneous: false,
                                                set_hhomogeneous: false,
                                            },
                                        },
                                    },

                                    gtk::Box {
                                        set_hexpand: true,
                                    },
                                },
                            },

                            gtk::Box {
                                set_hexpand: true,
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_hexpand: false,

                                // This box is required to prevent the top and bottom menus
                                // from matching each other's width if one is larger than the other
                                gtk::Box {

                                    gtk::Box {
                                        set_hexpand: true,
                                    },

                                    #[name = "top_right_revealer_container"]
                                    BoxWithResize::new() -> BoxWithResize {

                                        #[name = "top_right_revealer"]
                                        append = &gtk::Revealer {
                                            set_transition_type: gtk::RevealerTransitionType::SlideDown,
                                            #[watch]
                                            set_reveal_child: model.top_right_revealed,

                                            #[name = "top_right_stack"]
                                            gtk::Stack {
                                                set_transition_type: gtk::StackTransitionType::Crossfade,
                                                set_transition_duration: 200,
                                                set_vhomogeneous: false,
                                                set_hhomogeneous: false,
                                            },
                                        },
                                    },

                                    gtk::Box {
                                        set_hexpand: true,
                                    },
                                },

                                gtk::Box {
                                    set_height_request: 200,
                                    set_vexpand: true,
                                },

                                // This box is required to prevent the top and bottom menus
                                // from matching each other's width if one is larger than the other
                                gtk::Box {

                                    gtk::Box {
                                        set_hexpand: true,
                                    },

                                    #[name = "bottom_right_revealer_container"]
                                    BoxWithResize::new() -> BoxWithResize {

                                        #[name = "bottom_right_revealer"]
                                        append = &gtk::Revealer {
                                            set_transition_type: gtk::RevealerTransitionType::SlideDown,
                                            #[watch]
                                            set_reveal_child: model.bottom_right_revealed,

                                            #[name = "bottom_right_stack"]
                                            gtk::Stack {
                                                set_transition_type: gtk::StackTransitionType::Crossfade,
                                                set_transition_duration: 200,
                                                set_vhomogeneous: false,
                                                set_hhomogeneous: false,
                                            },
                                        },
                                    },

                                    gtk::Box {
                                        set_hexpand: true,
                                    },
                                },
                            },
                        },

                        #[name = "right_bar_and_menu_container"]
                        BoxWithResize::new() -> BoxWithResize {

                            #[name = "right_revealer_container"]
                            append = &BoxWithResize::new() -> BoxWithResize {

                                #[name = "right_revealer"]
                                append = &gtk::Revealer {
                                    set_transition_type: gtk::RevealerTransitionType::SlideLeft,
                                    #[watch]
                                    set_reveal_child: model.right_revealed,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_vexpand: true,
                                        set_hexpand: false,

                                        #[name = "top_right_expander"]
                                        BoxWithResize::new() -> BoxWithResize {
                                            set_hexpand: true,
                                            #[watch]
                                            set_vexpand: match model.right_menu_expansion_type {
                                                VerticalMenuExpansion::AlwaysExpanded => {false}
                                                VerticalMenuExpansion::ExpandBothWays => {true}
                                                VerticalMenuExpansion::ExpandDown => {false}
                                                VerticalMenuExpansion::ExpandUp => {true}
                                            },
                                        },

                                        #[name = "right_stack"]
                                        gtk::Stack {
                                            set_transition_type: gtk::StackTransitionType::Crossfade,
                                            set_transition_duration: 200,
                                            set_vhomogeneous: false,
                                            set_hhomogeneous: false,
                                        },

                                        #[name = "bottom_right_expander"]
                                        BoxWithResize::new() -> BoxWithResize {
                                            set_hexpand: true,
                                            #[watch]
                                            set_vexpand: match model.right_menu_expansion_type {
                                                VerticalMenuExpansion::AlwaysExpanded => {false}
                                                VerticalMenuExpansion::ExpandBothWays => {true}
                                                VerticalMenuExpansion::ExpandDown => {true}
                                                VerticalMenuExpansion::ExpandUp => {false}
                                            },
                                        },
                                    },
                                },
                            },

                            #[name = "right_bar_container"]
                            append = &BoxWithResize::new() -> BoxWithResize {
                                append = &model.right_bar.widget().clone() {},
                            },
                        },
                    },

                    #[name = "bottom_bar_container"]
                    BoxWithResize::new() -> BoxWithResize {
                        set_hexpand: true,

                        append = &model.bottom_bar.widget().clone() {},
                    },
                },

                #[name = "frame_draw_widget"]
                FrameDrawWidget::new() -> FrameDrawWidget {
                    set_hexpand: true,
                    set_vexpand: true,
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        info!(
            monitor = params.monitor.connector().unwrap().to_string(),
            "Initializing frame"
        );

        root.init_layer_shell();
        root.set_monitor(Some(&params.monitor));
        root.set_namespace(Some("okshell-frame"));
        root.set_layer(Layer::Top);
        root.set_exclusive_zone(-1);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Bottom, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.set_decorated(false);
        root.set_visible(true);

        let base_config = okshell_config::config_manager::config_manager().config();
        let untracked_config = base_config.clone().get_untracked();

        let left_menu_expansion_type = untracked_config.menus.left_menu_expansion_type;
        let right_menu_expansion_type = untracked_config.menus.right_menu_expansion_type;

        let top_bar: Controller<BarModel> = Self::build_bar(&sender, BarType::Top);
        let bottom_bar: Controller<BarModel> = Self::build_bar(&sender, BarType::Bottom);
        let left_bar: Controller<BarModel> = Self::build_bar(&sender, BarType::Left);
        let right_bar: Controller<BarModel> = Self::build_bar(&sender, BarType::Right);

        let calendar_menu = Self::build_menu(&sender, MenuType::Clock);
        let clipboard_menu = Self::build_menu(&sender, MenuType::Clipboard);
        let main_menu = Self::build_menu(&sender, MenuType::QuickSettings);
        let notification_menu = Self::build_menu(&sender, MenuType::Notifications);
        let screenshot_menu = Self::build_menu(&sender, MenuType::Screenshot);
        let app_launcher_menu = Self::build_menu(&sender, MenuType::AppLauncher);
        let wallpaper_menu = Self::build_menu(&sender, MenuType::Wallpaper);
        let screenshare_menu = Self::build_menu(&sender, MenuType::HyprlandScreenshare);

        let mut effects = EffectScope::new();

        let config = base_config.clone();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config.clone();
            let enable_frame = config.bars().frame().enable_frame().get();
            sender_clone.input(FrameInput::SetDrawFrame(enable_frame));
        });

        let config = base_config.clone();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config.clone();
            let expansion_type = config.menus().left_menu_expansion_type().get();
            sender_clone.input(FrameInput::SetLeftMenuExpansionType(expansion_type));
        });

        let config = base_config.clone();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config.clone();
            let expansion_type = config.menus().right_menu_expansion_type().get();
            sender_clone.input(FrameInput::SetRightMenuExpansionType(expansion_type));
        });

        let menu_config = base_config.clone();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = menu_config.clone();
            let clock_menu_position = config.menus().clock_menu().position().get();
            let config = menu_config.clone();
            let clipboard_menu_position = config.menus().clipboard_menu().position().get();
            let config = menu_config.clone();
            let quick_settings_menu_position = config.menus().quick_settings_menu().position().get();
            let config = menu_config.clone();
            let notification_menu_position = config.menus().notification_menu().position().get();
            let config = menu_config.clone();
            let screenshot_menu_position = config.menus().screenshot_menu().position().get();
            let config = menu_config.clone();
            let app_launcher_menu_position = config.menus().app_launcher_menu().position().get();
            let config = menu_config.clone();
            let wallpaper_menu_position = config.menus().wallpaper_menu().position().get();
            let config = menu_config.clone();
            let screenshare_menu_position = config.menus().screenshare_menu().position().get();
            sender_clone.input(FrameInput::RepositionMenus(
                clock_menu_position,
                clipboard_menu_position,
                quick_settings_menu_position,
                notification_menu_position,
                screenshot_menu_position,
                app_launcher_menu_position,
                wallpaper_menu_position,
                screenshare_menu_position,
            ));
        });

        let monitor_clone = params.monitor.clone();
        let top_spacer = FrameSpacerModel::builder()
            .launch(FrameSpacerInit{
                bar_type: BarType::Top,
                monitor: monitor_clone,
            })
            .detach();
        let monitor_clone = params.monitor.clone();
        let bottom_spacer = FrameSpacerModel::builder()
            .launch(FrameSpacerInit{
                bar_type: BarType::Bottom,
                monitor: monitor_clone,
            })
            .detach();
        let monitor_clone = params.monitor.clone();
        let left_spacer = FrameSpacerModel::builder()
            .launch(FrameSpacerInit{
                bar_type: BarType::Left,
                monitor: monitor_clone,
            })
            .detach();
        let monitor_clone = params.monitor.clone();
        let right_spacer = FrameSpacerModel::builder()
            .launch(FrameSpacerInit{
                bar_type: BarType::Right,
                monitor: monitor_clone,
            })
            .detach();

        let model = Frame {
            top_bar,
            bottom_bar,
            left_bar,
            right_bar,
            left_menu_expansion_type,
            right_menu_expansion_type,
            left_revealed: false,
            right_revealed: false,
            top_revealed: false,
            top_left_revealed: false,
            top_right_revealed: false,
            bottom_revealed: false,
            bottom_left_revealed: false,
            bottom_right_revealed: false,
            top_spacer,
            bottom_spacer,
            left_spacer,
            right_spacer,
            clock_menu: calendar_menu,
            clipboard_menu,
            quick_settings_menu: main_menu,
            notification_menu,
            screenshot_menu,
            app_launcher_menu,
            wallpaper_menu,
            screenshare_menu,
            _effects: effects,
        };

        let widgets = view_output!();

        model.attach_resize_listeners(&widgets);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            FrameInput::SetDrawFrame(draw_frame) => {
                widgets.frame_draw_widget.update_style(|s| s.draw_frame = draw_frame);
            }
            FrameInput::QueueFrameRedraw => {
                widgets.frame_draw_widget.queue_draw();
            }
            FrameInput::SetLeftMenuExpansionType(expansion_type) => {
                self.left_menu_expansion_type = expansion_type;
            }
            FrameInput::SetRightMenuExpansionType(expansion_type) => {
                self.right_menu_expansion_type = expansion_type;
            }
            FrameInput::RepositionMenus(
                clock_menu_position,
                clipboard_menu_position,
                quick_settings_menu_position,
                notification_menu_position,
                screenshot_menu_position,
                app_launcher_menu_position,
                wallpaper_menu_position,
                screenshare_menu_position,
            ) => {
                sender.input(FrameInput::CloseMenus);
                self.apply_left_and_right_side_children(
                    &widgets,
                    clock_menu_position,
                    clipboard_menu_position,
                    quick_settings_menu_position,
                    notification_menu_position,
                    screenshot_menu_position,
                    app_launcher_menu_position,
                    wallpaper_menu_position,
                    screenshare_menu_position,
                );
            }
            FrameInput::ToggleClockMenu => {
                self.toggle_menu(
                    CLOCK_MENU,
                    widgets,
                );
            }
            FrameInput::ToggleClipboardMenu => {
                self.toggle_menu(
                    CLIPBOARD_MENU,
                    widgets,
                );
            }
            FrameInput::ToggleQuickSettingsMenu => {
                self.toggle_menu(
                    QUICK_SETTINGS_MENU,
                    widgets,
                );
            }
            FrameInput::ToggleNotificationMenu => {
                self.toggle_menu(
                    NOTIFICATION_MENU,
                    widgets,
                );
            }
            FrameInput::ToggleScreenshotMenu => {
                self.toggle_menu(
                    SCREENSHOT_MENU,
                    widgets,
                );
            }
            FrameInput::ToggleAppLauncherMenu => {
                self.toggle_menu(
                    APP_LAUNCHER_MENU,
                    widgets,
                );
            }
            FrameInput::ToggleWallpaperMenu => {
                self.toggle_menu(
                    WALLPAPER_MENU,
                    widgets,
                );
            }
            FrameInput::ToggleScreenshareMenu(reply, payload) => {
                self.screenshare_menu.emit(ForwardHyprlandScreenshareReply(reply, payload));
                self.toggle_menu(
                    SCREENSHARE_MENU,
                    widgets,
                );
            }
            FrameInput::CloseMenus => {
                self.right_revealed = false;
                self.left_revealed = false;
                self.top_revealed = false;
                self.top_left_revealed = false;
                self.top_right_revealed = false;
                self.bottom_revealed = false;
                self.bottom_left_revealed = false;
                self.bottom_right_revealed = false;

                self.clock_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();

                self.clipboard_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();

                self.quick_settings_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();

                self.notification_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();

                self.screenshot_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();

                self.app_launcher_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();

                self.wallpaper_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();

                self.screenshare_menu
                    .sender()
                    .send(MenuInput::RevealChanged(false))
                    .unwrap_or_default();
            }
        }
        self.update_view(widgets, sender);
    }
}

impl Frame {
    fn toggle_menu(
        &mut self,
        name: &str,
        widgets: &mut FrameWidgets,
    ) {
        let mut now_visible = true;
        let in_left = widgets.left_stack.child_by_name(name).is_some();
        let in_right = widgets.right_stack.child_by_name(name).is_some();
        let in_top = widgets.top_stack.child_by_name(name).is_some();
        let in_top_left = widgets.top_left_stack.child_by_name(name).is_some();
        let in_top_right = widgets.top_right_stack.child_by_name(name).is_some();
        let in_bottom = widgets.bottom_stack.child_by_name(name).is_some();
        let in_bottom_left = widgets.bottom_left_stack.child_by_name(name).is_some();
        let in_bottom_right = widgets.bottom_right_stack.child_by_name(name).is_some();

        let left_revealed = self.left_revealed;
        let right_revealed = self.right_revealed;
        let top_revealed = self.top_revealed;
        let top_left_revealed = self.top_left_revealed;
        let top_right_revealed = self.top_right_revealed;
        let bottom_revealed = self.bottom_revealed;
        let bottom_left_revealed = self.bottom_left_revealed;
        let bottom_right_revealed = self.bottom_right_revealed;

        self.left_revealed = false;
        self.right_revealed = false;
        self.top_revealed = false;
        self.top_left_revealed = false;
        self.top_right_revealed = false;
        self.bottom_revealed = false;
        self.bottom_left_revealed = false;
        self.bottom_right_revealed = false;

        if in_left {
            if let Some(visible) = widgets.left_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.left_revealed = !left_revealed;
                    now_visible = self.left_revealed;
                } else {
                    widgets.left_stack.set_visible_child_name(name);
                    self.left_revealed = true;
                }
            }
        } else if in_right {
            if let Some(visible) = widgets.right_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.right_revealed = !right_revealed;
                    now_visible = self.right_revealed;
                } else {
                    widgets.right_stack.set_visible_child_name(name);
                    self.right_revealed = true;
                }
            }
        } else if in_top {
            if let Some(visible) = widgets.top_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.top_revealed = !top_revealed;
                    now_visible = self.top_revealed;
                } else {
                    widgets.top_stack.set_visible_child_name(name);
                    self.top_revealed = true;
                }
            }
        } else if in_top_left {
            if let Some(visible) = widgets.top_left_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.top_left_revealed = !top_left_revealed;
                    now_visible = self.top_left_revealed;
                } else {
                    widgets.top_left_stack.set_visible_child_name(name);
                    self.top_left_revealed = true;
                }
            }
        } else if in_top_right {
            if let Some(visible) = widgets.top_right_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.top_right_revealed = !top_right_revealed;
                    now_visible = self.top_right_revealed;
                } else {
                    widgets.top_right_stack.set_visible_child_name(name);
                    self.top_right_revealed = true;
                }
            }
        } else if in_bottom {
            if let Some(visible) = widgets.bottom_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.bottom_revealed = !bottom_revealed;
                    now_visible = self.bottom_revealed;
                } else {
                    widgets.bottom_stack.set_visible_child_name(name);
                    self.bottom_revealed = true;
                }
            }
        } else if in_bottom_left {
            if let Some(visible) = widgets.bottom_left_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.bottom_left_revealed = !bottom_left_revealed;
                    now_visible = self.bottom_left_revealed;
                } else {
                    widgets.bottom_left_stack.set_visible_child_name(name);
                    self.bottom_left_revealed = true;
                }
            }
        } else if in_bottom_right {
            if let Some(visible) = widgets.bottom_right_stack.visible_child_name() {
                if visible.as_str() == name {
                    self.bottom_right_revealed = !bottom_right_revealed;
                    now_visible = self.bottom_right_revealed;
                } else {
                    widgets.bottom_right_stack.set_visible_child_name(name);
                    self.bottom_right_revealed = true;
                }
            }
        }

        self.clock_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == CLOCK_MENU && now_visible,
            ))
            .unwrap_or_default();

        self.clipboard_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == CLIPBOARD_MENU && now_visible,
            ))
            .unwrap_or_default();

        self.quick_settings_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == QUICK_SETTINGS_MENU && now_visible,
            ))
            .unwrap_or_default();

        self.notification_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == NOTIFICATION_MENU && now_visible,
            ))
            .unwrap_or_default();

        self.screenshot_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == SCREENSHOT_MENU && now_visible,
            ))
            .unwrap_or_default();

        self.app_launcher_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == APP_LAUNCHER_MENU && now_visible,
            ))
            .unwrap_or_default();

        self.wallpaper_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == WALLPAPER_MENU && now_visible,
            ))
            .unwrap_or_default();

        self.screenshare_menu
            .sender()
            .send(MenuInput::RevealChanged(
                name == SCREENSHARE_MENU && now_visible,
            ))
            .unwrap_or_default();
    }

    // Can't use sender for this.  Must queue redraw in the callback.  Otherwise, there is a slight
    // delay and the frame isn't draw immediately.
    fn attach_resize_listeners(
        &self,
        widgets: &FrameWidgets,
    ) {
        let frame_widget = widgets.frame_draw_widget.clone();
        let top_sender = self.top_spacer.sender().clone();
        widgets.top_bar_container.connect_local(
            "resized",
            false,
            move |values| {
                let height = values[2].get::<i32>().expect("height i32");
                let border = frame_widget.border_width() as i32;
                frame_widget.update_style(|s| s.top_thickness = (height + border) as f64);
                let _ = top_sender.send(FrameSpacerInput::HeightUpdated(height + border));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        let bottom_sender = self.bottom_spacer.sender().clone();
        widgets.bottom_bar_container.connect_local(
            "resized",
            false,
            move |values| {
                let height = values[2].get::<i32>().expect("height i32");
                let border = frame_widget.border_width() as i32;
                frame_widget.update_style(|s| s.bottom_thickness = (height + border) as f64);
                let _ = bottom_sender.send(FrameSpacerInput::HeightUpdated(height + border));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.left_bar_and_menu_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                frame_widget.update_style(|s| s.left_thickness = width as f64);
                None
            },
        );
        let frame_widget = widgets.frame_draw_widget.clone();
        let left_sender = self.left_spacer.sender().clone();
        widgets.left_bar_container.connect_local(
            "resized",
            false,
            move |values| {
                let width = values[1].get::<i32>().expect("width i32");
                let border = frame_widget.border_width() as i32;
                let _ = left_sender.send(FrameSpacerInput::WidthUpdated(width + border));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.right_bar_and_menu_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                frame_widget.update_style(|s| s.right_thickness = width as f64);
                None
            },
        );
        let frame_widget = widgets.frame_draw_widget.clone();
        let right_sender = self.right_spacer.sender().clone();
        widgets.right_bar_container.connect_local(
            "resized",
            false,
            move |values| {
                let width = values[1].get::<i32>().expect("width i32");
                let border = frame_widget.border_width() as i32;
                let _ = right_sender.send(FrameSpacerInput::WidthUpdated(width + border));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.top_left_expander.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.left_top_expander_height = height as f64);
                None
            },
        );
        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.top_right_expander.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.right_top_expander_height = height as f64);
                None
            },
        );
        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.bottom_left_expander.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.left_bottom_expander_height = height as f64);
                None
            },
        );
        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.bottom_right_expander.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.right_bottom_expander_height = height as f64);
                None
            },
        );
        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.left_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                frame_widget.update_style(|s| s.left_expander_width = width as f64);
                None
            },
        );
        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.right_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                frame_widget.update_style(|s| s.right_expander_width = width as f64);
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.top_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.top_revealer_size = (width as f64, height as f64));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.top_left_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.top_left_revealer_size = (width as f64, height as f64));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.top_right_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.top_right_revealer_size = (width as f64, height as f64));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.bottom_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.bottom_revealer_size = (width as f64, height as f64));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.bottom_left_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.bottom_left_revealer_size = (width as f64, height as f64));
                None
            },
        );

        let frame_widget = widgets.frame_draw_widget.clone();
        widgets.bottom_right_revealer_container.connect_local(
            "resized",
            false,
            move |values: &[glib::Value]| {
                let width = values[1].get::<i32>().expect("width i32");
                let height = values[2].get::<i32>().expect("height i32");
                frame_widget.update_style(|s| s.bottom_right_revealer_size = (width as f64, height as f64));
                None
            },
        );
    }

    fn apply_left_and_right_side_children(
        &self,
        widgets: &FrameWidgets,
        clock_menu_position: Position,
        clipboard_menu_position: Position,
        quick_settings_position: Position,
        notification_menu_position: Position,
        screenshot_menu_position: Position,
        app_launcher_menu_position: Position,
        wallpaper_menu_position: Position,
        screenshare_menu_position: Position,
    ) {
        let clock_widget: Widget = self.clock_menu.widget().clone().upcast();
        let clipboard_widget: Widget = self.clipboard_menu.widget().clone().upcast();
        let quick_settings_widget: Widget = self.quick_settings_menu.widget().clone().upcast();
        let notification_menu_widget: Widget = self.notification_menu.widget().clone().upcast();
        let screenshot_menu_widget: Widget = self.screenshot_menu.widget().clone().upcast();
        let app_launcher_menu_widget: Widget = self.app_launcher_menu.widget().clone().upcast();
        let wallpaper_menu_widget: Widget = self.wallpaper_menu.widget().clone().upcast();
        let screenshare_menu_widget: Widget = self.screenshare_menu.widget().clone().upcast();

        widgets.left_stack.remove_all();
        widgets.right_stack.remove_all();
        widgets.top_stack.remove_all();
        widgets.top_left_stack.remove_all();
        widgets.top_right_stack.remove_all();
        widgets.bottom_stack.remove_all();
        widgets.bottom_left_stack.remove_all();
        widgets.bottom_right_stack.remove_all();

        Self::add_to_stack(widgets, &clock_widget, CLOCK_MENU, &clock_menu_position);
        Self::add_to_stack(widgets, &clipboard_widget, CLIPBOARD_MENU, &clipboard_menu_position);
        Self::add_to_stack(widgets, &quick_settings_widget, QUICK_SETTINGS_MENU, &quick_settings_position);
        Self::add_to_stack(widgets, &notification_menu_widget, NOTIFICATION_MENU, &notification_menu_position);
        Self::add_to_stack(widgets, &screenshot_menu_widget, SCREENSHOT_MENU, &screenshot_menu_position);
        Self::add_to_stack(widgets, &app_launcher_menu_widget, APP_LAUNCHER_MENU, &app_launcher_menu_position);
        Self::add_to_stack(widgets, &wallpaper_menu_widget, WALLPAPER_MENU, &wallpaper_menu_position);
        Self::add_to_stack(widgets, &screenshare_menu_widget, SCREENSHARE_MENU, &screenshare_menu_position);
    }

    fn add_to_stack(
        widgets: &FrameWidgets,
        widget: &Widget,
        name: &str,
        position: &Position,
    ) {
        match position {
            Position::Top => {
                widgets.top_stack.add_named(widget, Some(name));
            }
            Position::Bottom => {
                widgets.bottom_stack.add_named(widget, Some(name));
            }
            Position::Left => {
                widgets.left_stack.add_named(widget, Some(name));
            }
            Position::Right => {
                widgets.right_stack.add_named(widget, Some(name));
            }
            Position::TopLeft => {
                widgets.top_left_stack.add_named(widget, Some(name));
            }
            Position::TopRight => {
                widgets.top_right_stack.add_named(widget, Some(name));
            }
            Position::BottomLeft => {
                widgets.bottom_left_stack.add_named(widget, Some(name));
            }
            Position::BottomRight => {
                widgets.bottom_right_stack.add_named(widget, Some(name));
            }
        }
    }

    fn build_bar(
        sender: &ComponentSender<Self>,
        bar_type: BarType,
    ) -> Controller<BarModel> {
        BarModel::builder()
            .launch(BarInit {
                bar_type,
            })
            .forward(sender.input_sender(), |msg| {
                match msg {
                    BarOutput::ClockClicked => {
                        FrameInput::ToggleClockMenu
                    }
                    BarOutput::ClipboardClicked => {
                        FrameInput::ToggleClipboardMenu
                    }
                    BarOutput::MainMenuClicked => {
                        FrameInput::ToggleQuickSettingsMenu
                    }
                    BarOutput::NotificationsClicked => {
                        FrameInput::ToggleNotificationMenu
                    }
                    BarOutput::ScreenshotClicked => {
                        FrameInput::ToggleScreenshotMenu
                    }
                    BarOutput::AppLauncherClicked => {
                        FrameInput::ToggleAppLauncherMenu
                    }
                    BarOutput::WallpaperClicked => {
                        FrameInput::ToggleWallpaperMenu
                    }
                    BarOutput::CloseMenu => {
                        FrameInput::CloseMenus
                    }
                }
            })
    }

    fn build_menu(
        sender: &ComponentSender<Self>,
        menu_type: MenuType,
    ) -> Controller<MenuModel> {
        MenuModel::builder()
            .launch(MenuInit {
                menu_type,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { 
                    MenuOutput::CloseMenu => {
                        FrameInput::CloseMenus
                    }
                }
            })
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        self.top_spacer.widget().destroy();
        self.bottom_spacer.widget().destroy();
        self.left_spacer.widget().destroy();
        self.right_spacer.widget().destroy();
    }
}

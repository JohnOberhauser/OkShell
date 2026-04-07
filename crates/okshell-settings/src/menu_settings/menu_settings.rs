use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{BarsStoreFields, ConfigStoreFields, MenuStoreFields, MenusStoreFields, ScreenshareMenuStoreFields, VerticalBarStoreFields, VerticalMenuExpansion};
use okshell_config::schema::menu_widgets::MenuWidget;
use okshell_config::schema::position::Position;
use crate::bar_settings::bar_settings::BarSettingsInput;
use crate::menu_settings::menu_widget_list::{MenuWidgetListInit, MenuWidgetListInput, MenuWidgetListModel, MenuWidgetListOutput};

#[derive(Debug)]
pub(crate) struct MenuSettingsModel {
    quick_settings_widget_list_controller: Controller<MenuWidgetListModel>,
    quick_settings_position: Position,
    quick_settings_min_width: i32,
    clock_widget_list_controller: Controller<MenuWidgetListModel>,
    clock_position: Position,
    clock_min_width: i32,
    clipboard_widget_list_controller: Controller<MenuWidgetListModel>,
    clipboard_position: Position,
    clipboard_min_width: i32,
    screenshot_widget_list_controller: Controller<MenuWidgetListModel>,
    screenshot_position: Position,
    screenshot_min_width: i32,
    notifications_widget_list_controller: Controller<MenuWidgetListModel>,
    notifications_position: Position,
    notifications_min_width: i32,
    app_launcher_widget_list_controller: Controller<MenuWidgetListModel>,
    app_launcher_position: Position,
    app_launcher_min_width: i32,
    wallpaper_widget_list_controller: Controller<MenuWidgetListModel>,
    wallpaper_position: Position,
    wallpaper_min_width: i32,
    screenshare_position: Position,
    left_menu_expansion_type: VerticalMenuExpansion,
    right_menu_expansion_type: VerticalMenuExpansion,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum MenuSettingsInput {
    QuickSettingsWidgetListChanged(Vec<MenuWidget>),
    QuickSettingsPositionChanged(Position),
    QuickSettingsMinWidthChanged(i32),
    ClockWidgetListChanged(Vec<MenuWidget>),
    ClockPositionChanged(Position),
    ClockMinWidthChanged(i32),
    ClipboardWidgetListChanged(Vec<MenuWidget>),
    ClipboardPositionChanged(Position),
    ClipboardMinWidthChanged(i32),
    ScreenshotWidgetListChanged(Vec<MenuWidget>),
    ScreenshotPositionChanged(Position),
    ScreenshotMinWidthChanged(i32),
    NotificationsWidgetListChanged(Vec<MenuWidget>),
    NotificationsPositionChanged(Position),
    NotificationsMinWidthChanged(i32),
    AppLauncherWidgetListChanged(Vec<MenuWidget>),
    AppLauncherPositionChanged(Position),
    AppLauncherMinWidthChanged(i32),
    WallpaperWidgetListChanged(Vec<MenuWidget>),
    WallpaperPositionChanged(Position),
    WallpaperMinWidthChanged(i32),
    ScreensharePositionChanged(Position),
    LeftMenuExpansionChanged(VerticalMenuExpansion),
    RightMenuExpansionChanged(VerticalMenuExpansion),

    QuickSettingsWidgetListEffect(Vec<MenuWidget>),
    QuickSettingsPositionEffect(Position),
    QuickSettingsMinWidthEffect(i32),
    ClockWidgetListEffect(Vec<MenuWidget>),
    ClockPositionEffect(Position),
    ClockMinWidthEffect(i32),
    ClipboardWidgetListEffect(Vec<MenuWidget>),
    ClipboardPositionEffect(Position),
    ClipboardMinWidthEffect(i32),
    ScreenshotWidgetListEffect(Vec<MenuWidget>),
    ScreenshotPositionEffect(Position),
    ScreenshotMinWidthEffect(i32),
    NotificationsWidgetListEffect(Vec<MenuWidget>),
    NotificationsPositionEffect(Position),
    NotificationsMinWidthEffect(i32),
    AppLauncherWidgetListEffect(Vec<MenuWidget>),
    AppLauncherPositionEffect(Position),
    AppLauncherMinWidthEffect(i32),
    WallpaperWidgetListEffect(Vec<MenuWidget>),
    WallpaperPositionEffect(Position),
    WallpaperMinWidthEffect(i32),
    ScreensharePositionEffect(Position),
    LeftMenuExpansionEffect(VerticalMenuExpansion),
    RightMenuExpansionEffect(VerticalMenuExpansion),
}

#[derive(Debug)]
pub(crate) enum MenuSettingsOutput {}

pub(crate) struct MenuSettingsInit {}

#[derive(Debug)]
pub(crate) enum MenuSettingsCommandOutput {}

#[relm4::component(pub)]
impl Component for MenuSettingsModel {
    type CommandOutput = MenuSettingsCommandOutput;
    type Input = MenuSettingsInput;
    type Output = MenuSettingsOutput;
    type Init = MenuSettingsInit;

    view! {
        #[root]
        gtk::ScrolledWindow {
            set_vscrollbar_policy: gtk::PolicyType::Automatic,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_height: false,
            set_propagate_natural_width: false,
            set_hexpand: true,
            set_vexpand: true,

            gtk::Box {
                add_css_class: "settings-page",
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 16,
                
                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Menu Expansion",
                    set_halign: gtk::Align::Start,
                },
                
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Left Menu Expansion",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "How the left menu expands vertically.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&VerticalMenuExpansion::display_names())),
                        #[watch]
                        #[block_signal(left_expansion_handler)]
                        set_selected: model.left_menu_expansion_type.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::LeftMenuExpansionChanged(
                                VerticalMenuExpansion::from_index(dd.selected())
                            ));
                        } @left_expansion_handler,
                    },
                },
                
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Right Menu Expansion",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "How the right menu expands vertically.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&VerticalMenuExpansion::display_names())),
                        #[watch]
                        #[block_signal(right_expansion_handler)]
                        set_selected: model.right_menu_expansion_type.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::RightMenuExpansionChanged(
                                VerticalMenuExpansion::from_index(dd.selected())
                            ));
                        } @right_expansion_handler,
                    },
                },
                
                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Quick Settings Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(qs_pos_handler)]
                        set_selected: model.quick_settings_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::QuickSettingsPositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @qs_pos_handler,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Minimum Width",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The minimum width of the menu.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 10000.0),
                        set_increments: (10.0, 50.0),
                        #[watch]
                        #[block_signal(qs_min_width_handler)]
                        set_value: model.quick_settings_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(MenuSettingsInput::QuickSettingsMinWidthChanged(s.value() as i32));
                        } @qs_min_width_handler,
                    },
                },

                model.quick_settings_widget_list_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Clock Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(clock_pos_handler)]
                        set_selected: model.clock_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::ClockPositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @clock_pos_handler,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Minimum Width",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The minimum width of the menu.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 10000.0),
                        set_increments: (10.0, 50.0),
                        #[watch]
                        #[block_signal(clock_min_width_handler)]
                        set_value: model.clock_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(MenuSettingsInput::ClockMinWidthChanged(s.value() as i32));
                        } @clock_min_width_handler,
                    },
                },

                model.clock_widget_list_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Clipboard Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(clip_pos_handler)]
                        set_selected: model.clipboard_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::ClipboardPositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @clip_pos_handler,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Minimum Width",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The minimum width of the menu.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 10000.0),
                        set_increments: (10.0, 50.0),
                        #[watch]
                        #[block_signal(clip_min_width_handler)]
                        set_value: model.clipboard_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(MenuSettingsInput::ClipboardMinWidthChanged(s.value() as i32));
                        } @clip_min_width_handler,
                    },
                },

                model.clipboard_widget_list_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Screenshot Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(ss_pos_handler)]
                        set_selected: model.screenshot_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::ScreenshotPositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @ss_pos_handler,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Minimum Width",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The minimum width of the menu.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 10000.0),
                        set_increments: (10.0, 50.0),
                        #[watch]
                        #[block_signal(ss_min_width_handler)]
                        set_value: model.screenshot_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(MenuSettingsInput::ScreenshotMinWidthChanged(s.value() as i32));
                        } @ss_min_width_handler,
                    },
                },

                model.screenshot_widget_list_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Notifications Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(not_pos_handler)]
                        set_selected: model.notifications_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::NotificationsPositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @not_pos_handler,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Minimum Width",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The minimum width of the menu.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 10000.0),
                        set_increments: (10.0, 50.0),
                        #[watch]
                        #[block_signal(not_min_width_handler)]
                        set_value: model.notifications_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(MenuSettingsInput::NotificationsMinWidthChanged(s.value() as i32));
                        } @not_min_width_handler,
                    },
                },

                model.notifications_widget_list_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "App Launcher Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(al_pos_handler)]
                        set_selected: model.app_launcher_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::AppLauncherPositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @al_pos_handler,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Minimum Width",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The minimum width of the menu.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 10000.0),
                        set_increments: (10.0, 50.0),
                        #[watch]
                        #[block_signal(al_min_width_handler)]
                        set_value: model.app_launcher_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(MenuSettingsInput::AppLauncherMinWidthChanged(s.value() as i32));
                        } @al_min_width_handler,
                    },
                },

                model.app_launcher_widget_list_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Wallpaper Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(wall_pos_handler)]
                        set_selected: model.wallpaper_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::WallpaperPositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @wall_pos_handler,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Minimum Width",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The minimum width of the menu.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 10000.0),
                        set_increments: (10.0, 50.0),
                        #[watch]
                        #[block_signal(wall_min_width_handler)]
                        set_value: model.wallpaper_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(MenuSettingsInput::WallpaperMinWidthChanged(s.value() as i32));
                        } @wall_min_width_handler,
                    },
                },

                model.wallpaper_widget_list_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Screen Share Menu",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where this menu should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&Position::display_names())),
                        #[watch]
                        #[block_signal(sh_pos_handler)]
                        set_selected: model.screenshare_position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(MenuSettingsInput::ScreensharePositionChanged(
                                Position::from_index(dd.selected())
                            ));
                        } @sh_pos_handler,
                    },
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().left_menu_expansion_type().get();
            sender_clone.input(MenuSettingsInput::LeftMenuExpansionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().right_menu_expansion_type().get();
            sender_clone.input(MenuSettingsInput::RightMenuExpansionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().quick_settings_menu().position().get();
            sender_clone.input(MenuSettingsInput::QuickSettingsPositionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().quick_settings_menu().minimum_width().get();
            sender_clone.input(MenuSettingsInput::QuickSettingsMinWidthEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().quick_settings_menu().widgets().get();
            sender_clone.input(MenuSettingsInput::QuickSettingsWidgetListEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().clock_menu().position().get();
            sender_clone.input(MenuSettingsInput::ClockPositionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().clock_menu().minimum_width().get();
            sender_clone.input(MenuSettingsInput::ClockMinWidthEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().clock_menu().widgets().get();
            sender_clone.input(MenuSettingsInput::ClockWidgetListEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().clipboard_menu().position().get();
            sender_clone.input(MenuSettingsInput::ClipboardPositionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().clipboard_menu().minimum_width().get();
            sender_clone.input(MenuSettingsInput::ClipboardMinWidthEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().clipboard_menu().widgets().get();
            sender_clone.input(MenuSettingsInput::ClipboardWidgetListEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().screenshot_menu().position().get();
            sender_clone.input(MenuSettingsInput::ScreenshotPositionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().screenshot_menu().minimum_width().get();
            sender_clone.input(MenuSettingsInput::ScreenshotMinWidthEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().screenshot_menu().widgets().get();
            sender_clone.input(MenuSettingsInput::ScreenshotWidgetListEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().notification_menu().position().get();
            sender_clone.input(MenuSettingsInput::NotificationsPositionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().notification_menu().minimum_width().get();
            sender_clone.input(MenuSettingsInput::NotificationsMinWidthEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().notification_menu().widgets().get();
            sender_clone.input(MenuSettingsInput::NotificationsWidgetListEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().app_launcher_menu().position().get();
            sender_clone.input(MenuSettingsInput::AppLauncherPositionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().app_launcher_menu().minimum_width().get();
            sender_clone.input(MenuSettingsInput::AppLauncherMinWidthEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().app_launcher_menu().widgets().get();
            sender_clone.input(MenuSettingsInput::AppLauncherWidgetListEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().wallpaper_menu().position().get();
            sender_clone.input(MenuSettingsInput::WallpaperPositionEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().wallpaper_menu().minimum_width().get();
            sender_clone.input(MenuSettingsInput::WallpaperMinWidthEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().wallpaper_menu().widgets().get();
            sender_clone.input(MenuSettingsInput::WallpaperWidgetListEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.menus().screenshare_menu().position().get();
            sender_clone.input(MenuSettingsInput::ScreensharePositionEffect(value));
        });

        let quick_settings_widget_list_controller = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config_manager().config().menus().quick_settings_menu().widgets().get_untracked(),
                draw_border: true,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { MenuWidgetListOutput::Changed(widgets) => {
                    MenuSettingsInput::QuickSettingsWidgetListChanged(widgets)
                } }
            });

        let clock_widget_list_controller = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config_manager().config().menus().clock_menu().widgets().get_untracked(),
                draw_border: true,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { MenuWidgetListOutput::Changed(widgets) => {
                    MenuSettingsInput::ClockWidgetListChanged(widgets)
                } }
            });

        let clipboard_widget_list_controller = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config_manager().config().menus().clipboard_menu().widgets().get_untracked(),
                draw_border: true,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { MenuWidgetListOutput::Changed(widgets) => {
                    MenuSettingsInput::ClipboardWidgetListChanged(widgets)
                } }
            });

        let screenshot_widget_list_controller = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config_manager().config().menus().screenshot_menu().widgets().get_untracked(),
                draw_border: true,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { MenuWidgetListOutput::Changed(widgets) => {
                    MenuSettingsInput::ScreenshotWidgetListChanged(widgets)
                } }
            });

        let notifications_widget_list_controller = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config_manager().config().menus().notification_menu().widgets().get_untracked(),
                draw_border: true,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { MenuWidgetListOutput::Changed(widgets) => {
                    MenuSettingsInput::NotificationsWidgetListChanged(widgets)
                } }
            });

        let app_launcher_widget_list_controller = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config_manager().config().menus().app_launcher_menu().widgets().get_untracked(),
                draw_border: true,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { MenuWidgetListOutput::Changed(widgets) => {
                    MenuSettingsInput::AppLauncherWidgetListChanged(widgets)
                } }
            });

        let wallpaper_widget_list_controller = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config_manager().config().menus().wallpaper_menu().widgets().get_untracked(),
                draw_border: true,
            })
            .forward(sender.input_sender(), |msg| {
                match msg { MenuWidgetListOutput::Changed(widgets) => {
                    MenuSettingsInput::WallpaperWidgetListChanged(widgets)
                } }
            });

        let model = MenuSettingsModel {
            quick_settings_widget_list_controller,
            quick_settings_position: config_manager().config().menus().quick_settings_menu().position().get_untracked(),
            quick_settings_min_width: config_manager().config().menus().quick_settings_menu().minimum_width().get_untracked(),
            clock_widget_list_controller,
            clock_position: config_manager().config().menus().clock_menu().position().get_untracked(),
            clock_min_width: config_manager().config().menus().clock_menu().minimum_width().get_untracked(),
            clipboard_widget_list_controller,
            clipboard_position: config_manager().config().menus().clipboard_menu().position().get_untracked(),
            clipboard_min_width: config_manager().config().menus().clipboard_menu().minimum_width().get_untracked(),
            screenshot_widget_list_controller,
            screenshot_position: config_manager().config().menus().screenshot_menu().position().get_untracked(),
            screenshot_min_width: config_manager().config().menus().screenshot_menu().minimum_width().get_untracked(),
            notifications_widget_list_controller,
            notifications_position: config_manager().config().menus().notification_menu().position().get_untracked(),
            notifications_min_width: config_manager().config().menus().notification_menu().minimum_width().get_untracked(),
            app_launcher_widget_list_controller,
            app_launcher_position: config_manager().config().menus().app_launcher_menu().position().get_untracked(),
            app_launcher_min_width: config_manager().config().menus().app_launcher_menu().minimum_width().get_untracked(),
            wallpaper_widget_list_controller,
            wallpaper_position: config_manager().config().menus().wallpaper_menu().position().get_untracked(),
            wallpaper_min_width: config_manager().config().menus().wallpaper_menu().minimum_width().get_untracked(),
            screenshare_position: config_manager().config().menus().screenshare_menu().position().get_untracked(),
            left_menu_expansion_type: config_manager().config().menus().left_menu_expansion_type().get_untracked(),
            right_menu_expansion_type: config_manager().config().menus().right_menu_expansion_type().get_untracked(),
            _effects: effects,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MenuSettingsInput::QuickSettingsWidgetListChanged(widgets) => {
                config_manager().update_config(|config| {
                    config.menus.quick_settings_menu.widgets = widgets;
                });
            }
            MenuSettingsInput::QuickSettingsPositionChanged(position) => {
                self.quick_settings_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.quick_settings_menu.position = position;
                });
            }
            MenuSettingsInput::QuickSettingsMinWidthChanged(width) => {
                self.quick_settings_min_width = width;
                config_manager().update_config(|config| {
                    config.menus.quick_settings_menu.minimum_width = width;
                });
            }
            MenuSettingsInput::ClockWidgetListChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.menus.clock_menu.widgets = widgets;
                });
            }
            MenuSettingsInput::ClockPositionChanged(position) => {
                self.clock_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.clock_menu.position = position;
                });
            }
            MenuSettingsInput::ClockMinWidthChanged(width) => {
                self.clock_min_width = width;
                config_manager().update_config(|config| {
                    config.menus.clock_menu.minimum_width = width;
                });
            }
            MenuSettingsInput::WallpaperWidgetListChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.menus.wallpaper_menu.widgets = widgets;
                });
            }
            MenuSettingsInput::WallpaperPositionChanged(position) => {
                self.wallpaper_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.wallpaper_menu.position = position;
                });
            }
            MenuSettingsInput::WallpaperMinWidthChanged(width) => {
                self.wallpaper_min_width = width;
                config_manager().update_config(|config| {
                    config.menus.wallpaper_menu.minimum_width = width;
                });
            }
            MenuSettingsInput::NotificationsWidgetListChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.menus.notification_menu.widgets = widgets;
                });
            }
            MenuSettingsInput::NotificationsPositionChanged(position) => {
                self.notifications_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.notification_menu.position = position;
                });
            }
            MenuSettingsInput::NotificationsMinWidthChanged(width) => {
                self.notifications_min_width = width;
                config_manager().update_config(|config| {
                    config.menus.notification_menu.minimum_width = width;
                });
            }
            MenuSettingsInput::ClipboardWidgetListChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.menus.clipboard_menu.widgets = widgets;
                });
            }
            MenuSettingsInput::ClipboardPositionChanged(position) => {
                self.clipboard_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.clipboard_menu.position = position;
                });
            }
            MenuSettingsInput::ClipboardMinWidthChanged(width) => {
                self.clipboard_min_width = width;
                config_manager().update_config(|config| {
                    config.menus.clipboard_menu.minimum_width = width;
                });
            }
            MenuSettingsInput::ScreenshotWidgetListChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.menus.screenshot_menu.widgets = widgets;
                });
            }
            MenuSettingsInput::ScreenshotPositionChanged(position) => {
                self.screenshot_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.screenshot_menu.position = position;
                });
            }
            MenuSettingsInput::ScreenshotMinWidthChanged(width) => {
                self.screenshot_min_width = width;
                config_manager().update_config(|config| {
                    config.menus.screenshot_menu.minimum_width = width;
                });
            }
            MenuSettingsInput::AppLauncherWidgetListChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.menus.app_launcher_menu.widgets = widgets;
                });
            }
            MenuSettingsInput::AppLauncherPositionChanged(position) => {
                self.app_launcher_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.app_launcher_menu.position = position;
                });
            }
            MenuSettingsInput::AppLauncherMinWidthChanged(width) => {
                self.app_launcher_min_width = width;
                config_manager().update_config(|config| {
                    config.menus.app_launcher_menu.minimum_width = width;
                });
            }
            MenuSettingsInput::ScreensharePositionChanged(position) => {
                self.screenshare_position = position.clone();
                config_manager().update_config(|config| {
                    config.menus.screenshare_menu.position = position;
                });
            }
            MenuSettingsInput::LeftMenuExpansionChanged(expansion_type) => {
                self.left_menu_expansion_type = expansion_type.clone();
                config_manager().update_config(|config| {
                    config.menus.left_menu_expansion_type = expansion_type;
                });
            }
            MenuSettingsInput::RightMenuExpansionChanged(expansion_type) => {
                self.right_menu_expansion_type = expansion_type.clone();
                config_manager().update_config(|config| {
                    config.menus.right_menu_expansion_type = expansion_type;
                });
            }
            MenuSettingsInput::QuickSettingsWidgetListEffect(widgets) => {
                self.quick_settings_widget_list_controller.emit(MenuWidgetListInput::SetWidgetsEffect(widgets));
            }
            MenuSettingsInput::QuickSettingsPositionEffect(position) => {
                self.quick_settings_position = position;
            }
            MenuSettingsInput::QuickSettingsMinWidthEffect(width) => {
                self.quick_settings_min_width = width;
            }
            MenuSettingsInput::ClockWidgetListEffect(widgets) => {
                self.clock_widget_list_controller.emit(MenuWidgetListInput::SetWidgetsEffect(widgets));
            }
            MenuSettingsInput::ClockPositionEffect(position) => {
                self.clock_position = position;
            }
            MenuSettingsInput::ClockMinWidthEffect(width) => {
                self.clock_min_width = width;
            }
            MenuSettingsInput::ClipboardWidgetListEffect(widgets) => {
                self.clipboard_widget_list_controller.emit(MenuWidgetListInput::SetWidgetsEffect(widgets));
            }
            MenuSettingsInput::ClipboardPositionEffect(position) => {
                self.clipboard_position = position;
            }
            MenuSettingsInput::ClipboardMinWidthEffect(width) => {
                self.clipboard_min_width = width;
            }
            MenuSettingsInput::ScreenshotWidgetListEffect(widgets) => {
                self.screenshot_widget_list_controller.emit(MenuWidgetListInput::SetWidgetsEffect(widgets));
            }
            MenuSettingsInput::ScreenshotPositionEffect(position) => {
                self.screenshot_position = position;
            }
            MenuSettingsInput::ScreenshotMinWidthEffect(width) => {
                self.screenshot_min_width = width;
            }
            MenuSettingsInput::NotificationsWidgetListEffect(widgets) => {
                self.notifications_widget_list_controller.emit(MenuWidgetListInput::SetWidgetsEffect(widgets));
            }
            MenuSettingsInput::NotificationsPositionEffect(position) => {
                self.notifications_position = position;
            }
            MenuSettingsInput::NotificationsMinWidthEffect(width) => {
                self.notifications_min_width = width;
            }
            MenuSettingsInput::AppLauncherWidgetListEffect(widgets) => {
                self.app_launcher_widget_list_controller.emit(MenuWidgetListInput::SetWidgetsEffect(widgets));
            }
            MenuSettingsInput::AppLauncherPositionEffect(position) => {
                self.app_launcher_position = position;
            }
            MenuSettingsInput::AppLauncherMinWidthEffect(width) => {
                self.app_launcher_min_width = width;
            }
            MenuSettingsInput::WallpaperWidgetListEffect(widgets) => {
                self.wallpaper_widget_list_controller.emit(MenuWidgetListInput::SetWidgetsEffect(widgets));
            }
            MenuSettingsInput::WallpaperPositionEffect(position) => {
                self.wallpaper_position = position;
            }
            MenuSettingsInput::WallpaperMinWidthEffect(width) => {
                self.wallpaper_min_width = width;
            }
            MenuSettingsInput::ScreensharePositionEffect(position) => {
                self.screenshare_position = position;
            }
            MenuSettingsInput::LeftMenuExpansionEffect(expansion) => {
                self.left_menu_expansion_type = expansion;
            }
            MenuSettingsInput::RightMenuExpansionEffect(expansion) => {
                self.right_menu_expansion_type = expansion;
            }
        }

        self.update_view(widgets, sender);
    }
}
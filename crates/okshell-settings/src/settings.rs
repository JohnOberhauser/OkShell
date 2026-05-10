use crate::bar_settings::bar_settings::{BarSettingsInit, BarSettingsModel};
use crate::general_settings::{GeneralSettingsInit, GeneralSettingsModel};
use crate::menu_settings::menu_settings::{MenuSettingsInit, MenuSettingsModel};
use crate::notification_settings::{NotificationSettingsInit, NotificationSettingsModel};
use crate::theme_settings::theme_settings::{ThemeSettingsInit, ThemeSettingsModel};
use crate::wallpaper_settings::{WallpaperSettingsInit, WallpaperSettingsModel};
use relm4::gtk::prelude::{BoxExt, GtkWindowExt, OrientableExt, ToggleButtonExt, WidgetExt};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};

pub(crate) struct SettingsWindowModel {
    general_settings_controller: Controller<GeneralSettingsModel>,
    wallpaper_settings_controller: Controller<WallpaperSettingsModel>,
    theme_settings_controller: Controller<ThemeSettingsModel>,
    bar_settings_controller: Controller<BarSettingsModel>,
    menu_settings_controller: Controller<MenuSettingsModel>,
    notification_settings_controller: Controller<NotificationSettingsModel>,
}

#[derive(Debug)]
pub(crate) enum SettingsWindowInput {}

#[derive(Debug)]
pub(crate) enum SettingsWindowOutput {}

pub(crate) struct SettingsWindowInit {}

#[derive(Debug)]
pub(crate) enum SettingsWindowCommandOutput {}

#[relm4::component(pub)]
impl Component for SettingsWindowModel {
    type CommandOutput = SettingsWindowCommandOutput;
    type Input = SettingsWindowInput;
    type Output = SettingsWindowOutput;
    type Init = SettingsWindowInit;

    view! {
        #[root]
        gtk::Window {
            add_css_class: "settings-window",
            set_decorated: true,
            set_resizable: true,
            set_visible: true,
            set_default_size: (800, 700),

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    add_css_class: "settings-sidebar",
                    set_orientation: gtk::Orientation::Vertical,
                    set_width_request: 180,
                    set_spacing: 4,
                    set_hexpand: false,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,
                        gtk::Label {
                            add_css_class: "label-large-bold",
                            set_margin_start: 8,
                            set_margin_bottom: 8,
                            set_margin_end: 8,
                            set_margin_top: 8,
                            set_label: "Settings",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                        },
                    },

                    gtk::Separator {},

                    #[name = "general_btn"]
                    gtk::ToggleButton {
                        add_css_class: "sidebar-button",
                        set_active: true,
                        connect_toggled[stack] => move |b| {
                            if b.is_active() { stack.set_visible_child_name("general"); }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            gtk::Image { set_icon_name: Some("settings-symbolic") },
                            gtk::Label {
                                add_css_class: "label-medium",
                                set_label: "General",
                                set_halign: gtk::Align::Start,
                                set_hexpand: true,
                            },
                        },
                    },

                    gtk::ToggleButton {
                        add_css_class: "sidebar-button",
                        set_group: Some(&general_btn),
                        connect_toggled[stack] => move |b| {
                            if b.is_active() { stack.set_visible_child_name("theme"); }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            gtk::Image { set_icon_name: Some("palette-symbolic") },
                            gtk::Label {
                                add_css_class: "label-medium",
                                set_label: "Theme",
                                set_halign: gtk::Align::Start,
                                set_hexpand: true,
                            },
                        },
                    },

                    gtk::ToggleButton {
                        add_css_class: "sidebar-button",
                        set_group: Some(&general_btn),
                        connect_toggled[stack] => move |b| {
                            if b.is_active() { stack.set_visible_child_name("wallpaper"); }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            gtk::Image { set_icon_name: Some("wallpaper-symbolic") },
                            gtk::Label {
                                add_css_class: "label-medium",
                                set_label: "Wallpaper",
                                set_halign: gtk::Align::Start,
                                set_hexpand: true,
                            },
                        },
                    },

                    gtk::ToggleButton {
                        add_css_class: "sidebar-button",
                        set_group: Some(&general_btn),
                        connect_toggled[stack] => move |b| {
                            if b.is_active() { stack.set_visible_child_name("bar"); }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            gtk::Image { set_icon_name: Some("sidebar-symbolic") },
                            gtk::Label {
                                add_css_class: "label-medium",
                                set_label: "Bar",
                                set_halign: gtk::Align::Start,
                                set_hexpand: true,
                            },
                        },
                    },

                    gtk::ToggleButton {
                        add_css_class: "sidebar-button",
                        set_group: Some(&general_btn),
                        connect_toggled[stack] => move |b| {
                            if b.is_active() { stack.set_visible_child_name("menus"); }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            gtk::Image { set_icon_name: Some("square-symbolic") },
                            gtk::Label {
                                add_css_class: "label-medium",
                                set_label: "Menus",
                                set_halign: gtk::Align::Start,
                                set_hexpand: true,
                            },
                        },
                    },

                    gtk::ToggleButton {
                        add_css_class: "sidebar-button",
                        set_group: Some(&general_btn),
                        connect_toggled[stack] => move |b| {
                            if b.is_active() { stack.set_visible_child_name("notifications"); }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            gtk::Image { set_icon_name: Some("notification-symbolic") },
                            gtk::Label {
                                add_css_class: "label-medium",
                                set_label: "Notifications",
                                set_halign: gtk::Align::Start,
                                set_hexpand: true,
                            },
                        },
                    },
                },

                #[name = "stack"]
                gtk::Stack {
                    add_css_class: "settings-stack",
                    set_transition_type: gtk::StackTransitionType::Crossfade,
                    set_transition_duration: 50,
                    set_hexpand: true,
                    set_vexpand: true,
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let general_settings_controller = GeneralSettingsModel::builder()
            .launch(GeneralSettingsInit {})
            .detach();

        let wallpaper_settings_controller = WallpaperSettingsModel::builder()
            .launch(WallpaperSettingsInit {})
            .detach();

        let theme_settings_controller = ThemeSettingsModel::builder()
            .launch(ThemeSettingsInit {})
            .detach();

        let bar_settings_controller = BarSettingsModel::builder()
            .launch(BarSettingsInit {})
            .detach();

        let menu_settings_controller = MenuSettingsModel::builder()
            .launch(MenuSettingsInit {})
            .detach();

        let notification_settings_controller = NotificationSettingsModel::builder()
            .launch(NotificationSettingsInit {})
            .detach();

        let model = SettingsWindowModel {
            general_settings_controller,
            wallpaper_settings_controller,
            theme_settings_controller,
            bar_settings_controller,
            menu_settings_controller,
            notification_settings_controller,
        };

        let widgets = view_output!();

        // widgets.sidebar.set_stack(&widgets.stack);

        widgets.stack.add_titled(
            model.general_settings_controller.widget(),
            Some("general"),
            "General",
        );

        widgets.stack.add_titled(
            model.theme_settings_controller.widget(),
            Some("theme"),
            "Theme",
        );

        widgets.stack.add_titled(
            model.wallpaper_settings_controller.widget(),
            Some("wallpaper"),
            "Wallpaper",
        );

        widgets
            .stack
            .add_titled(model.bar_settings_controller.widget(), Some("bar"), "Bar");

        widgets.stack.add_titled(
            model.menu_settings_controller.widget(),
            Some("menus"),
            "Menus",
        );

        widgets.stack.add_titled(
            model.notification_settings_controller.widget(),
            Some("notifications"),
            "Notifications",
        );

        ComponentParts { model, widgets }
    }
}

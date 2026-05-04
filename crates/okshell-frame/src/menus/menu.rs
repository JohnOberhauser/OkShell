use crate::menus::builder::build_widget;
use crate::menus::menu_widgets::app_launcher::app_launcher::{AppLauncherInput, AppLauncherModel};
use crate::menus::menu_widgets::audio_in::audio_in_menu_widget::{
    AudioInMenuWidgetInput, AudioInMenuWidgetModel,
};
use crate::menus::menu_widgets::audio_out::audio_out_menu_widget::{
    AudioOutMenuWidgetInput, AudioOutMenuWidgetModel,
};
use crate::menus::menu_widgets::bluetooth::bluetooth_menu_widget::{
    BluetoothMenuWidgetInput, BluetoothMenuWidgetModel,
};
use crate::menus::menu_widgets::network::network_menu_widget::{
    NetworkMenuWidgetInput, NetworkMenuWidgetModel,
};
use crate::menus::menu_widgets::power_profile::power_profile_menu_widget::{
    PowerProfileMenuWidgetInput, PowerProfileMenuWidgetModel,
};
use crate::menus::menu_widgets::screenshare::screenshare_menu_widget::{
    ScreenshareMenuWidgetInit, ScreenshareMenuWidgetInput, ScreenshareMenuWidgetModel,
    ScreenshareMenuWidgetOutput,
};
use crate::menus::menu_widgets::wallpaper::wallpaper_dropdown_menu_widget::{
    WallpaperDropdownMenuWidgetInput, WallpaperDropdownMenuWidgetModel,
};
use crate::menus::menu_widgets::wallpaper::wallpaper_menu_widget::{
    WallpaperMenuWidgetInput, WallpaperMenuWidgetModel,
};
use okshell_common::dynamic_box::generic_widget_controller::{
    GenericWidgetController, GenericWidgetControllerExtSafe,
};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::schema::config::{ConfigStoreFields, MenuStoreFields, MenusStoreFields};
use okshell_config::schema::menu_widgets::MenuWidget;
use okshell_utils::clear_box::clear_box;
use reactive_graph::traits::Get;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    gtk, gtk::prelude::*,
};
use std::fmt::Debug;

pub(crate) enum MenuType {
    Clipboard,
    Clock,
    Notifications,
    QuickSettings,
    Screenshot,
    AppLauncher,
    Wallpaper,
    HyprlandScreenshare,
}

pub(crate) struct MenuModel {
    widget_controllers: Vec<Box<dyn GenericWidgetController>>,
    minimum_width: i32,
    css_class: String,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum MenuInput {
    RevealChanged(bool),
    SetWidget(Vec<MenuWidget>),
    SetMinimumWidth(i32),
    AddHyprlandScreenshareWidget,
    ForwardHyprlandScreenshareReply(tokio::sync::oneshot::Sender<String>, String),
}

#[derive(Debug)]
pub(crate) enum MenuOutput {
    CloseMenu,
}

pub(crate) struct MenuInit {
    pub(crate) menu_type: MenuType,
}

#[relm4::component(pub)]
impl Component for MenuModel {
    type CommandOutput = ();
    type Input = MenuInput;
    type Output = MenuOutput;
    type Init = MenuInit;

    view! {
        #[root]
        #[name = "scrolled_window"]
        gtk::ScrolledWindow {
            set_css_classes: &["menu-scroll-window", model.css_class.as_str()],
            set_vscrollbar_policy: gtk::PolicyType::Automatic,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_height: true,
            #[watch]
            set_width_request: model.minimum_width,
            set_propagate_natural_width: false,

            #[name = "widget_container"]
            gtk::Box {
                set_margin_all: 20,
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: false,
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let base_config = okshell_config::config_manager::config_manager().config();

        let mut effects = EffectScope::new();

        let css_class: String;

        match params.menu_type {
            MenuType::Clock => {
                css_class = "clock-menu".to_string();
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let widgets = config.menus().clock_menu().widgets().get();
                    sender_clone.input(MenuInput::SetWidget(widgets));
                });
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let minimum_width = config.menus().clock_menu().minimum_width().get();
                    sender_clone.input(MenuInput::SetMinimumWidth(minimum_width));
                });
            }
            MenuType::Clipboard => {
                css_class = "clipboard-menu".to_string();
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let widgets = config.menus().clipboard_menu().widgets().get();
                    sender_clone.input(MenuInput::SetWidget(widgets));
                });
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let minimum_width = config.menus().clipboard_menu().minimum_width().get();
                    sender_clone.input(MenuInput::SetMinimumWidth(minimum_width));
                });
            }
            MenuType::QuickSettings => {
                css_class = "quick-settings-menu".to_string();
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let widgets = config.menus().quick_settings_menu().widgets().get();
                    sender_clone.input(MenuInput::SetWidget(widgets));
                });
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let minimum_width = config.menus().quick_settings_menu().minimum_width().get();
                    sender_clone.input(MenuInput::SetMinimumWidth(minimum_width));
                });
            }
            MenuType::Notifications => {
                css_class = "notifications-menu".to_string();
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let widgets = config.menus().notification_menu().widgets().get();
                    sender_clone.input(MenuInput::SetWidget(widgets));
                });
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let minimum_width = config.menus().notification_menu().minimum_width().get();
                    sender_clone.input(MenuInput::SetMinimumWidth(minimum_width));
                });
            }
            MenuType::Screenshot => {
                css_class = "screenshot-menu".to_string();
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let widgets = config.menus().screenshot_menu().widgets().get();
                    sender_clone.input(MenuInput::SetWidget(widgets));
                });
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let minimum_width = config.menus().screenshot_menu().minimum_width().get();
                    sender_clone.input(MenuInput::SetMinimumWidth(minimum_width));
                });
            }
            MenuType::AppLauncher => {
                css_class = "app-launcher-menu".to_string();
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let widgets = config.menus().app_launcher_menu().widgets().get();
                    sender_clone.input(MenuInput::SetWidget(widgets));
                });
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let minimum_width = config.menus().app_launcher_menu().minimum_width().get();
                    sender_clone.input(MenuInput::SetMinimumWidth(minimum_width));
                });
            }
            MenuType::Wallpaper => {
                css_class = "wallpaper-menu".to_string();
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let widgets = config.menus().wallpaper_menu().widgets().get();
                    sender_clone.input(MenuInput::SetWidget(widgets));
                });
                let config = base_config.clone();
                let sender_clone = sender.clone();
                effects.push(move |_| {
                    let config = config.clone();
                    let minimum_width = config.menus().wallpaper_menu().minimum_width().get();
                    sender_clone.input(MenuInput::SetMinimumWidth(minimum_width));
                });
            }
            MenuType::HyprlandScreenshare => {
                css_class = "hyprland-screenshare-menu".to_string();
                sender.input(MenuInput::AddHyprlandScreenshareWidget);
            }
        }

        let model = MenuModel {
            widget_controllers: Vec::new(),
            minimum_width: 410,
            css_class,
            _effects: effects,
        };

        let widgets = view_output!();

        if let MenuType::Wallpaper = params.menu_type {
            widgets.widget_container.set_margin_all(8);
        }

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
            MenuInput::RevealChanged(visible) => {
                // Let widgets that care know they are being revealed
                for controller in &self.widget_controllers {
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<AppLauncherModel>>()
                    {
                        controller
                            .sender()
                            .send(AppLauncherInput::ParentRevealChanged(visible))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<NetworkMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(NetworkMenuWidgetInput::ParentRevealChanged(visible))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<BluetoothMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(BluetoothMenuWidgetInput::ParentRevealChanged(visible))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<AudioOutMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(AudioOutMenuWidgetInput::ParentRevealChanged(visible))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<AudioInMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(AudioInMenuWidgetInput::ParentRevealChanged(visible))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<PowerProfileMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(PowerProfileMenuWidgetInput::ParentRevealChanged(visible))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<ScreenshareMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(ScreenshareMenuWidgetInput::ParentRevealChanged(visible))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<WallpaperDropdownMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(WallpaperDropdownMenuWidgetInput::ParentRevealChanged(
                                visible,
                            ))
                            .ok();
                    }
                    if let Some(controller) =
                        controller.downcast_ref::<Controller<WallpaperMenuWidgetModel>>()
                    {
                        controller
                            .sender()
                            .send(WallpaperMenuWidgetInput::ParentRevealChanged(visible))
                            .ok();
                    }
                }
            }
            MenuInput::SetWidget(menu_widgets) => {
                clear_box(&widgets.widget_container);
                self.widget_controllers.clear();
                for item in menu_widgets {
                    let controller = build_widget(&item, gtk::Orientation::Vertical, &sender);
                    widgets.widget_container.append(&controller.root_widget());
                    self.widget_controllers.push(controller);
                }
            }
            MenuInput::SetMinimumWidth(width) => {
                self.minimum_width = width;
            }
            MenuInput::AddHyprlandScreenshareWidget => {
                let controller = Box::new(
                    ScreenshareMenuWidgetModel::builder()
                        .launch(ScreenshareMenuWidgetInit {})
                        .forward(sender.output_sender(), |msg| match msg {
                            ScreenshareMenuWidgetOutput::CloseMenu => MenuOutput::CloseMenu,
                        }),
                );
                widgets.widget_container.append(&controller.root_widget());
                self.widget_controllers.push(controller);
            }
            MenuInput::ForwardHyprlandScreenshareReply(reply, payload) => {
                if let Some(first_controller) = self.widget_controllers.first()
                    && let Some(controller) =
                        first_controller.downcast_ref::<Controller<ScreenshareMenuWidgetModel>>()
                {
                    controller
                        .sender()
                        .send(ScreenshareMenuWidgetInput::SetReply(reply, payload))
                        .ok();
                }
            }
        }
        self.update_view(widgets, sender);
    }
}

impl Debug for MenuModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuModel").finish()
    }
}

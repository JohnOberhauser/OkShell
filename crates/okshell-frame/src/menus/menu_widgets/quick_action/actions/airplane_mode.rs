use okshell_common::WatcherToken;
use okshell_services::network_service;
use okshell_utils::network::{spawn_wifi_available_watcher, spawn_wifi_enabled_watcher};
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

pub(crate) struct AirplaneModeModel {
    enabled: bool,
    wifi_available: bool,
    wifi_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum AirplaneModeInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum AirplaneModeOutput {}

pub(crate) struct AirplaneModeInit {}

#[derive(Debug)]
pub(crate) enum AirplaneModeCommandOutput {
    WifiChanged,
    WifiEnabledChanged,
}

#[relm4::component(pub)]
impl Component for AirplaneModeModel {
    type Input = AirplaneModeInput;
    type Output = AirplaneModeOutput;
    type Init = AirplaneModeInit;
    type CommandOutput = AirplaneModeCommandOutput;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                #[watch]
                set_css_classes: if model.enabled {
                    &["ok-button-surface", "ok-button-medium",]
                } else {
                    &["ok-button-surface", "ok-button-medium", "selected"]
                },
                #[watch]
                set_sensitive: model.wifi_available,
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(AirplaneModeInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    #[watch]
                    set_css_classes: if model.enabled {
                        &["selected"]
                    } else {
                        &[]
                    },
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("airplane-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_wifi_available_watcher(&sender, || AirplaneModeCommandOutput::WifiChanged);

        let model = AirplaneModeModel {
            enabled: false,
            wifi_available: false,
            wifi_watcher_token: WatcherToken::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AirplaneModeInput::Clicked => {
                if let Some(wifi) = network_service().wifi.get() {
                    tokio::spawn(async move {
                        let _ = wifi.set_enabled(!wifi.enabled.get()).await;
                    });
                }
            }
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AirplaneModeCommandOutput::WifiChanged => {
                if let Some(wifi) = network_service().wifi.get() {
                    self.enabled = wifi.enabled.get();
                    self.wifi_available = true;
                } else {
                    self.enabled = false;
                    self.wifi_available = false;
                }

                let token = self.wifi_watcher_token.reset();
                spawn_wifi_enabled_watcher(&sender, token, || {
                    AirplaneModeCommandOutput::WifiEnabledChanged
                });
            }
            AirplaneModeCommandOutput::WifiEnabledChanged => {
                if let Some(wifi) = network_service().wifi.get() {
                    self.enabled = wifi.enabled.get();
                    self.wifi_available = true;
                } else {
                    self.enabled = false;
                    self.wifi_available = false;
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

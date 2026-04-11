use std::sync::Arc;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::{Justification, RevealerTransitionType};
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use okshell_common::WatcherToken;
use wayle_network::core::access_point::{AccessPoint, Ssid};
use okshell_services::network_service;
use crate::common_widgets::revealer_button::revealer_button::{RevealerButtonInit, RevealerButtonInput, RevealerButtonModel};
use crate::common_widgets::revealer_button::revealer_button_icon_label::{RevealerButtonIconLabelInit, RevealerButtonIconLabelModel};
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::{GenericWidgetController, GenericWidgetControllerExtSafe};
use crate::menus::menu_widgets::network::disconnect_button::DisconnectButtonModel;
use crate::menus::menu_widgets::network::available_network_revealed_content::{AvailableNetworkRevealedContentInit, AvailableNetworkRevealedContentInput, AvailableNetworkRevealedContentModel};
use okshell_utils::network::{get_wifi_icon_for_strength, set_network_icon, set_network_label, spawn_available_wifi_networks_watcher, spawn_network_watcher, spawn_wifi_watcher, spawn_wired_watcher};

pub(crate) struct NetworkRevealedContentModel {
    active_network_button: Controller<RevealerButtonModel<RevealerButtonIconLabelModel, DisconnectButtonModel>>,
    available_networks_dynamic_box_controller: Controller<DynamicBoxModel<Arc<AccessPoint>, Ssid>>,
    wifi_watcher_token: WatcherToken,
    wired_watcher_token: WatcherToken,
    available_network_count: i16,
    scanning: bool,
}

#[derive(Debug)]
pub(crate) enum NetworkRevealedContentInput {
    UpdateState,
    UpdateAvailableNetworks,
    SetScanning(bool),
    Reset,
}

#[derive(Debug)]
pub(crate) enum NetworkRevealedContentOutput {}

pub(crate) struct NetworkRevealedContentInit {}

#[derive(Debug)]
pub(crate) enum NetworkRevealedContentCommandOutput {
    StateChanged,
    WifiChanged,
    WiredChanged,
    AvailableNetworksChanged,
}

#[relm4::component(pub)]
impl Component for NetworkRevealedContentModel {
    type CommandOutput = NetworkRevealedContentCommandOutput;
    type Input = NetworkRevealedContentInput;
    type Output = NetworkRevealedContentOutput;
    type Init = NetworkRevealedContentInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            #[name = "active_network_container"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Label {
                    add_css_class: "label-large-bold-variant",
                    set_label: "Active Network",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                },

                model.active_network_button.widget().clone() {}
            },

            #[name = "available_networks_container"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Label {
                    add_css_class: "label-large-bold-variant",
                    set_label: "Available Networks",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                },

                gtk::Label {
                    add_css_class: "label-medium",
                    set_label: "No Available Networks",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                    #[watch]
                    set_visible: model.available_network_count == 0 && !model.scanning,
                },

                model.available_networks_dynamic_box_controller.widget().clone() {},

                gtk::Label {
                    add_css_class: "label-medium",
                    set_label: "Scanning…",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                    #[watch]
                    set_visible: model.scanning,
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        spawn_network_watcher(
            &sender,
            || NetworkRevealedContentCommandOutput::StateChanged,
            || NetworkRevealedContentCommandOutput::WifiChanged,
            || NetworkRevealedContentCommandOutput::WiredChanged,
        );

        let active_network_content = RevealerButtonIconLabelModel::builder()
            .launch(RevealerButtonIconLabelInit {
                label: "Not Connected".to_string(),
                icon_name: "network-wireless-disabled-symbolic".to_string(),
                secondary_icon_name: "".to_string(),
            })
            .detach();

        let active_network_revealed_content = DisconnectButtonModel::builder()
            .launch(())
            .detach();

        let active_network_button = RevealerButtonModel::builder()
            .launch(RevealerButtonInit {
                content: active_network_content,
                revealed_content: active_network_revealed_content,
            })
            .detach();

        let available_networks_dynamic_box_factory = DynamicBoxFactory::<Arc<AccessPoint>, Ssid> {
            id: Box::new(|item| item.ssid.get()),
            create: Box::new(move |item| {
                let available_network_content = RevealerButtonIconLabelModel::builder()
                    .launch(RevealerButtonIconLabelInit {
                        label: item.ssid.get().to_string(),
                        icon_name: get_wifi_icon_for_strength(item.strength.get()).to_string(),
                        secondary_icon_name: "".to_string(),
                    })
                    .detach();

                let access_point = item.clone();
                let available_network_revealed_content = AvailableNetworkRevealedContentModel::builder()
                    .launch(AvailableNetworkRevealedContentInit {
                        access_point,
                    })
                    .detach();

                let available_network_button = RevealerButtonModel::builder()
                    .launch(RevealerButtonInit {
                        content: available_network_content,
                        revealed_content: available_network_revealed_content,
                    })
                    .detach();

                Box::new(available_network_button) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let available_networks_dynamic_box_controller: Controller<DynamicBoxModel<Arc<AccessPoint>, Ssid>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory: available_networks_dynamic_box_factory,
                    orientation: gtk::Orientation::Vertical,
                    spacing: 0,
                    transition_type: RevealerTransitionType::SlideDown,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                    allow_drag_and_drop: false,
                })
                .detach();

        let model = NetworkRevealedContentModel {
            active_network_button,
            available_networks_dynamic_box_controller,
            wifi_watcher_token: WatcherToken::new(),
            wired_watcher_token: WatcherToken::new(),
            available_network_count: 0,
            scanning: false,
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
            NetworkRevealedContentInput::UpdateState => {
                let network = network_service();
                let wifi = network.wifi.get();
                let wifi_exists = wifi.is_some();
                let has_ssid = wifi.map(|w| w.ssid.get().is_some()).unwrap_or(false);

                widgets.active_network_container.set_visible(wifi_exists && has_ssid);
                set_network_label(&self.active_network_button.model().content.widgets().label);
                set_network_icon(&self.active_network_button.model().content.widgets().image);

                widgets.available_networks_container.set_visible(wifi_exists);
            }
            NetworkRevealedContentInput::UpdateAvailableNetworks => {
                let network = network_service();

                if let Some(wifi) = network.wifi.get() {
                    let access_points: Vec<Arc<AccessPoint>> = wifi.access_points.get()
                        .iter()
                        .filter(|a| a.ssid.get().to_string() != wifi.ssid.get().unwrap_or("".to_string()))
                        .cloned()
                        .collect();

                    self.available_network_count = access_points.len() as i16;
                    self.available_networks_dynamic_box_controller.emit(DynamicBoxInput::SetItems(access_points))
                }
            }
            NetworkRevealedContentInput::SetScanning(scanning) => {
                self.scanning = scanning;
            }
            NetworkRevealedContentInput::Reset => {
                self.active_network_button.emit(RevealerButtonInput::SetRevealed(false));
                self.available_networks_dynamic_box_controller.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<RevealerButtonModel<RevealerButtonIconLabelModel, AvailableNetworkRevealedContentModel>>>()
                    {
                        ctrl.emit(RevealerButtonInput::SetRevealed(false));
                        ctrl.model().revealed_content.emit(AvailableNetworkRevealedContentInput::Reset);
                    }
                })
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            NetworkRevealedContentCommandOutput::StateChanged => {
                sender.input(NetworkRevealedContentInput::UpdateState);
            }
            NetworkRevealedContentCommandOutput::AvailableNetworksChanged => {
                sender.input(NetworkRevealedContentInput::UpdateAvailableNetworks);
                sender.input(NetworkRevealedContentInput::SetScanning(false));
            }
            NetworkRevealedContentCommandOutput::WifiChanged => {
                let token = self.wifi_watcher_token.reset();
                let token_clone = token.clone();
                spawn_wifi_watcher(
                    &sender,
                    token_clone,
                    || NetworkRevealedContentCommandOutput::StateChanged
                );
                let token_clone = token.clone();
                spawn_available_wifi_networks_watcher(
                    &sender,
                    token_clone,
                    || NetworkRevealedContentCommandOutput::AvailableNetworksChanged
                );
            }
            NetworkRevealedContentCommandOutput::WiredChanged => {
                let token = self.wired_watcher_token.reset();
                spawn_wired_watcher(
                    &sender,
                    token,
                    || NetworkRevealedContentCommandOutput::StateChanged
                );
            }
        }
    }
}
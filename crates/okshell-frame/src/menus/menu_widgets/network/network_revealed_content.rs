use crate::common_widgets::revealer_button::revealer_button::{
    RevealerButtonInit, RevealerButtonInput, RevealerButtonModel,
};
use crate::common_widgets::revealer_button::revealer_button_icon_label::{
    RevealerButtonIconLabelInit, RevealerButtonIconLabelInput, RevealerButtonIconLabelModel,
};
use crate::menus::menu_widgets::network::available_network_revealed_content::{
    AvailableNetworkRevealedContentInit, AvailableNetworkRevealedContentInput,
    AvailableNetworkRevealedContentModel,
};
use crate::menus::menu_widgets::network::disconnect_button::DisconnectButtonModel;
use crate::menus::menu_widgets::network::wireguard_revealed_content::{
    WireguardRevealedContentInit, WireguardRevealedContentModel,
};
use okshell_common::WatcherToken;
use okshell_common::dynamic_box::dynamic_box::{
    DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel,
};
use okshell_common::dynamic_box::generic_widget_controller::{
    GenericWidgetController, GenericWidgetControllerExtSafe,
};
use okshell_services::network_service;
use okshell_utils::network::{
    get_wifi_icon_for_strength, set_network_icon, set_network_label,
    spawn_available_wifi_networks_watcher, spawn_network_watcher, spawn_wifi_watcher,
    spawn_wired_watcher, spawn_wireguard_tunnels_watcher, spawn_wireguard_watcher,
};
use relm4::gtk::gio::prelude::FileExt;
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::gtk::{Justification, RevealerTransitionType};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use wayle_network::core::access_point::{AccessPoint, Ssid};
use wayle_network::wireguard::WireGuardTunnel;

pub(crate) struct NetworkRevealedContentModel {
    active_network_button:
        Controller<RevealerButtonModel<RevealerButtonIconLabelModel, DisconnectButtonModel>>,
    available_networks_dynamic_box_controller: Controller<DynamicBoxModel<Arc<AccessPoint>, Ssid>>,
    wireguard_dynamic_box_controller: Controller<DynamicBoxModel<Arc<WireGuardTunnel>, String>>,
    wifi_watcher_token: WatcherToken,
    wired_watcher_token: WatcherToken,
    wireguard_watcher_token: WatcherToken,
    available_network_count: i16,
    available_wg_count: i16,
    scanning: bool,
}

#[derive(Debug)]
pub(crate) enum NetworkRevealedContentInput {
    UpdateState,
    UpdateAvailableNetworks,
    SetScanning(bool),
    Reset,
    ImportClicked,
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
    WireguardChanged,
    WireguardTunnelsChanged,
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

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_hexpand: true,
                    set_halign: gtk::Align::Center,

                    gtk::Box {
                        set_width_request: 30,
                    },

                    gtk::Label {
                        add_css_class: "label-large-bold-variant",
                        set_label: "Wireguard Connections",
                        set_justify: Justification::Center,
                    },

                    gtk::Button {
                        add_css_class: "ok-button-surface",
                        set_tooltip_text: Some("Import from file"),
                        connect_clicked[sender] => move |_| {
                            sender.input(NetworkRevealedContentInput::ImportClicked);
                        },

                        #[name="image"]
                        gtk::Image {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_icon_name: Some("plus-symbolic"),
                        }
                    },
                },

                gtk::Label {
                    add_css_class: "label-medium",
                    set_label: "No Available WG Connections",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                    #[watch]
                    set_visible: model.available_wg_count == 0,
                },

                model.wireguard_dynamic_box_controller.widget().clone() {},
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

        spawn_wireguard_watcher(&sender, || {
            NetworkRevealedContentCommandOutput::WireguardChanged
        });

        let active_network_content = RevealerButtonIconLabelModel::builder()
            .launch(RevealerButtonIconLabelInit {
                label: "Not Connected".to_string(),
                icon_name: "network-wireless-disabled-symbolic".to_string(),
                secondary_icon_name: "".to_string(),
            })
            .detach();

        let active_network_revealed_content = DisconnectButtonModel::builder().launch(()).detach();

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
                let available_network_revealed_content =
                    AvailableNetworkRevealedContentModel::builder()
                        .launch(AvailableNetworkRevealedContentInit { access_point })
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

        let available_networks_dynamic_box_controller: Controller<
            DynamicBoxModel<Arc<AccessPoint>, Ssid>,
        > = DynamicBoxModel::builder()
            .launch(DynamicBoxInit {
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

        let wireguard_dynamic_box_factory = DynamicBoxFactory::<Arc<WireGuardTunnel>, String> {
            id: Box::new(|item| item.profile.uuid.get()),
            create: Box::new(move |item| {
                let icon_name;
                if item.active.get() {
                    icon_name = "shield-check-symbolic";
                } else {
                    icon_name = "";
                }
                let content = RevealerButtonIconLabelModel::builder()
                    .launch(RevealerButtonIconLabelInit {
                        label: item.profile.id.get().to_string(),
                        icon_name: icon_name.to_string(),
                        secondary_icon_name: "".to_string(),
                    })
                    .detach();

                let wg = item.clone();
                let revealed_content = WireguardRevealedContentModel::builder()
                    .launch(WireguardRevealedContentInit { wg })
                    .detach();

                let button = RevealerButtonModel::builder()
                    .launch(RevealerButtonInit {
                        content: content,
                        revealed_content: revealed_content,
                    })
                    .detach();

                Box::new(button) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let wireguard_dynamic_box_controller: Controller<
            DynamicBoxModel<Arc<WireGuardTunnel>, String>,
        > = DynamicBoxModel::builder()
            .launch(DynamicBoxInit {
                factory: wireguard_dynamic_box_factory,
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
            wireguard_dynamic_box_controller,
            wifi_watcher_token: WatcherToken::new(),
            wired_watcher_token: WatcherToken::new(),
            wireguard_watcher_token: WatcherToken::new(),
            available_network_count: 0,
            available_wg_count: 0,
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

                widgets
                    .active_network_container
                    .set_visible(wifi_exists && has_ssid);
                set_network_label(&self.active_network_button.model().content.widgets().label);
                set_network_icon(&self.active_network_button.model().content.widgets().image);

                widgets
                    .available_networks_container
                    .set_visible(wifi_exists);
            }
            NetworkRevealedContentInput::UpdateAvailableNetworks => {
                let network = network_service();

                if let Some(wifi) = network.wifi.get() {
                    let access_points: Vec<Arc<AccessPoint>> = wifi
                        .access_points
                        .get()
                        .iter()
                        .filter(|a| {
                            a.ssid.get().to_string() != wifi.ssid.get().unwrap_or("".to_string())
                        })
                        .cloned()
                        .collect();

                    self.available_network_count = access_points.len() as i16;
                    self.available_networks_dynamic_box_controller
                        .emit(DynamicBoxInput::SetItems(access_points))
                }
            }
            NetworkRevealedContentInput::SetScanning(scanning) => {
                self.scanning = scanning;
            }
            NetworkRevealedContentInput::Reset => {
                self.active_network_button
                    .emit(RevealerButtonInput::SetRevealed(false));
                self.available_networks_dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry.controller.as_ref().downcast_ref::<Controller<
                            RevealerButtonModel<
                                RevealerButtonIconLabelModel,
                                AvailableNetworkRevealedContentModel,
                            >,
                        >>() {
                            ctrl.emit(RevealerButtonInput::SetRevealed(false));
                            ctrl.model()
                                .revealed_content
                                .emit(AvailableNetworkRevealedContentInput::Reset);
                        }
                    });
                self.wireguard_dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry.controller.as_ref().downcast_ref::<Controller<
                            RevealerButtonModel<
                                RevealerButtonIconLabelModel,
                                WireguardRevealedContentModel,
                            >,
                        >>() {
                            ctrl.emit(RevealerButtonInput::SetRevealed(false));
                        }
                    })
            }
            NetworkRevealedContentInput::ImportClicked => {
                let filter = gtk::FileFilter::new();
                filter.set_name(Some("WireGuard Config (*.conf)"));
                filter.add_pattern("*.conf");

                let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);

                let dialog = gtk::FileDialog::builder()
                    .title("Select WireGuard Config")
                    .accept_label("Open")
                    .filters(&filters)
                    .default_filter(&filter)
                    .modal(true)
                    .build();

                dialog.open(
                    None::<&gtk::Window>,
                    None::<&gtk::gio::Cancellable>,
                    move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                tokio::spawn(async move {
                                    // The commit for wireguard in wayle-services doesn't work
                                    // for importing yet.  When it does, switch to using that instead
                                    // of nmcli.  Or maybe just keep this since we know it works?
                                    let _ = tokio::process::Command::new("nmcli")
                                        .args([
                                            "connection",
                                            "import",
                                            "type",
                                            "wireguard",
                                            "file",
                                            &path.to_string_lossy(),
                                        ])
                                        .output()
                                        .await;

                                    // let Some(wg) = network_service().wireguard.get() else {
                                    //     return;
                                    // };

                                    // let name = path
                                    //     .file_stem()
                                    //     .and_then(|s| s.to_str())
                                    //     .unwrap_or("wireguard");

                                    // let content = match tokio::fs::read_to_string(&path).await {
                                    //     Ok(c) => c,
                                    //     Err(e) => {
                                    //         tracing::error!(
                                    //             "Failed to read {}: {e}",
                                    //             path.display()
                                    //         );
                                    //         return;
                                    //     }
                                    // };

                                    // let result = wg.import(name, content.as_str()).await;
                                });
                            }
                        }
                    },
                );
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
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
                spawn_wifi_watcher(&sender, token_clone, || {
                    NetworkRevealedContentCommandOutput::StateChanged
                });
                let token_clone = token.clone();
                spawn_available_wifi_networks_watcher(&sender, token_clone, || {
                    NetworkRevealedContentCommandOutput::AvailableNetworksChanged
                });
            }
            NetworkRevealedContentCommandOutput::WiredChanged => {
                let token = self.wired_watcher_token.reset();
                spawn_wired_watcher(&sender, token, || {
                    NetworkRevealedContentCommandOutput::StateChanged
                });
            }
            NetworkRevealedContentCommandOutput::WireguardChanged => {
                let token = self.wireguard_watcher_token.reset();
                spawn_wireguard_tunnels_watcher(&sender, token, || {
                    NetworkRevealedContentCommandOutput::WireguardTunnelsChanged
                });
            }
            NetworkRevealedContentCommandOutput::WireguardTunnelsChanged => {
                let Some(wg) = network_service().wireguard.get() else {
                    self.available_wg_count = 0;
                    return;
                };

                let tunnels = wg.tunnels.get();

                self.available_wg_count = tunnels.len() as i16;
                self.wireguard_dynamic_box_controller
                    .emit(DynamicBoxInput::SetItems(tunnels));

                self.wireguard_dynamic_box_controller
                    .model()
                    .for_each_entry(|id, entry| {
                        if let Some(ctrl) = entry.controller.as_ref().downcast_ref::<Controller<
                            RevealerButtonModel<
                                RevealerButtonIconLabelModel,
                                WireguardRevealedContentModel,
                            >,
                        >>() {
                            let tunnels = wg.tunnels.get();
                            let wg = tunnels.iter().find(|tun| tun.profile.uuid.get() == *id);

                            let Some(wg) = wg else {
                                return;
                            };

                            if wg.active.get() {
                                ctrl.model().content.emit(
                                    RevealerButtonIconLabelInput::SetPrimaryIconName(
                                        "shield-check-symbolic".to_string(),
                                    ),
                                );
                            } else {
                                ctrl.model().content.emit(
                                    RevealerButtonIconLabelInput::SetPrimaryIconName(
                                        "".to_string(),
                                    ),
                                );
                            }
                        }
                    })
            }
        }

        self.update_view(widgets, sender);
    }
}

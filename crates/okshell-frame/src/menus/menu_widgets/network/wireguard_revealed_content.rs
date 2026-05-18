use okshell_common::WatcherToken;
use okshell_services::network_service;
use okshell_utils::network::spawn_wireguard_tunnels_watcher;
use okshell_utils::network::spawn_wireguard_watcher;
use relm4::gtk::Justification;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use std::sync::Arc;
use wayle_network::wireguard::WireGuardTunnel;

#[derive(Debug)]
pub(crate) struct WireguardRevealedContentModel {
    uuid: String,
    active: bool,
    wireguard_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum WireguardRevealedContentInput {
    Connect,
    Disconnect,
}

#[derive(Debug)]
pub(crate) enum WireguardRevealedContentOutput {}

pub(crate) struct WireguardRevealedContentInit {
    pub wg: Arc<WireGuardTunnel>,
}

#[derive(Debug)]
pub(crate) enum WireguardRevealedContentCommandOutput {
    WireguardChanged,
    ConnectivityChanged,
}

#[relm4::component(pub)]
impl Component for WireguardRevealedContentModel {
    type CommandOutput = WireguardRevealedContentCommandOutput;
    type Input = WireguardRevealedContentInput;
    type Output = WireguardRevealedContentOutput;
    type Init = WireguardRevealedContentInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: !model.active,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(WireguardRevealedContentInput::Connect);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    set_label: "Connect",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: model.active,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(WireguardRevealedContentInput::Disconnect);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    set_label: "Disconnect",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_wireguard_watcher(&sender, || {
            WireguardRevealedContentCommandOutput::WireguardChanged
        });

        let model = WireguardRevealedContentModel {
            uuid: params.wg.profile.uuid.get(),
            active: params.wg.active.get(),
            wireguard_watcher_token: WatcherToken::new(),
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
            WireguardRevealedContentInput::Connect => {
                let uuid = self.uuid.clone();
                tokio::spawn(async move {
                    let Some(wg) = network_service().wireguard.get() else {
                        return;
                    };

                    let tunnels = wg.tunnels.get();

                    let Some(connection) =
                        tunnels.iter().find(|tun| tun.profile.uuid.get() == uuid)
                    else {
                        return;
                    };

                    let _ = wg.activate(&connection.profile.object_path).await;
                });
            }
            WireguardRevealedContentInput::Disconnect => {
                let uuid = self.uuid.clone();
                tokio::spawn(async move {
                    let Some(wg) = network_service().wireguard.get() else {
                        return;
                    };

                    let tunnels = wg.tunnels.get();

                    let Some(connection) =
                        tunnels.iter().find(|tun| tun.profile.uuid.get() == uuid)
                    else {
                        return;
                    };

                    let _ = wg.deactivate(&connection).await;
                });
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
            WireguardRevealedContentCommandOutput::WireguardChanged => {
                let token = self.wireguard_watcher_token.reset();
                spawn_wireguard_tunnels_watcher(&sender, token, || {
                    WireguardRevealedContentCommandOutput::ConnectivityChanged
                });
            }
            WireguardRevealedContentCommandOutput::ConnectivityChanged => {
                let Some(wg) = network_service().wireguard.get() else {
                    return;
                };

                let tunnels = wg.tunnels.get();

                let Some(connection) = tunnels
                    .iter()
                    .find(|tun| tun.profile.uuid.get() == self.uuid)
                else {
                    return;
                };

                self.active = connection.active.get();
            }
        }

        self.update_view(widgets, sender);
    }
}

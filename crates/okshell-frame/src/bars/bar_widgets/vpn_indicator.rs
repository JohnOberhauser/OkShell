use okshell_common::WatcherToken;
use okshell_utils::network::{
    is_wireguard_connected, spawn_wireguard_tunnels_watcher, spawn_wireguard_watcher,
};
use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use tracing::info;

#[derive(Debug)]
pub(crate) struct VpnIndicatorModel {
    visible: bool,
    wireguard_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum VpnIndicatorInput {}

#[derive(Debug)]
pub(crate) enum VpnIndicatorOutput {}

pub(crate) struct VpnIndicatorInit {}

#[derive(Debug)]
pub(crate) enum VpnIndicatorCommandOutput {
    StateChanged,
    WireguardChanged,
}

#[relm4::component(pub)]
impl Component for VpnIndicatorModel {
    type CommandOutput = VpnIndicatorCommandOutput;
    type Input = VpnIndicatorInput;
    type Output = VpnIndicatorOutput;
    type Init = VpnIndicatorInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["ok-button-surface", "ok-bar-widget", "vpn-indicator-bar-widget"],
            set_hexpand: false,
            set_vexpand: false,
            #[watch]
            set_visible: model.visible,

            #[name="image"]
            gtk::Image {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_icon_name: Some("shield-check-symbolic"),
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_wireguard_watcher(&sender, || VpnIndicatorCommandOutput::WireguardChanged);

        let model = VpnIndicatorModel {
            visible: false,
            wireguard_watcher_token: WatcherToken::new(),
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
        match message {}
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            VpnIndicatorCommandOutput::StateChanged => {
                info!("state changed");
                self.visible = is_wireguard_connected();
            }
            VpnIndicatorCommandOutput::WireguardChanged => {
                info!("wireguard changed");
                let token = self.wireguard_watcher_token.reset();
                spawn_wireguard_tunnels_watcher(&sender, token, || {
                    VpnIndicatorCommandOutput::StateChanged
                });
            }
        }

        self.update_view(widgets, sender);
    }
}

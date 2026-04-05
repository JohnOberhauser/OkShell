use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::prelude::WidgetExt;
use okshell_common::{WatcherToken};
use okshell_utils::network::{set_network_icon, spawn_network_watcher, spawn_wifi_watcher, spawn_wired_watcher};

#[derive(Debug)]
pub(crate) struct NetworkModel {
    wifi_watcher_token: WatcherToken,
    wired_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum NetworkInput {
    UpdateState,
}

#[derive(Debug)]
pub(crate) enum NetworkOutput {}

pub(crate) struct NetworkInit {}

#[derive(Debug)]
pub(crate) enum NetworkCommandOutput {
    StateChanged,
    WifiChanged,
    WiredChanged,
}

#[relm4::component(pub)]
impl Component for NetworkModel {
    type CommandOutput = NetworkCommandOutput;
    type Input = NetworkInput;
    type Output = NetworkOutput;
    type Init = NetworkInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["ok-button-surface", "ok-bar-widget", "network-bar-widget"],
            set_hexpand: false,
            set_vexpand: false,

            #[name="image"]
            gtk::Image {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_network_watcher(
            &sender,
            || NetworkCommandOutput::StateChanged,
            || NetworkCommandOutput::WifiChanged,
            || NetworkCommandOutput::WiredChanged,
        );

        let model = NetworkModel {
            wifi_watcher_token: WatcherToken::new(),
            wired_watcher_token: WatcherToken::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            NetworkInput::UpdateState => {
                set_network_icon(&widgets.image);
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            NetworkCommandOutput::StateChanged => {
                sender.input(NetworkInput::UpdateState)
            }
            NetworkCommandOutput::WifiChanged => {
                let token = self.wifi_watcher_token.reset();
                spawn_wifi_watcher(
                    &sender,
                    token,
                    || NetworkCommandOutput::StateChanged
                );
            }
            NetworkCommandOutput::WiredChanged => {
                let token = self.wired_watcher_token.reset();
                spawn_wired_watcher(
                    &sender,
                    token,
                    || NetworkCommandOutput::StateChanged
                );
            }
        }
    }
}

use relm4::{
    gtk,
    ComponentParts,
    ComponentSender,
    SimpleComponent,
};
use relm4::gtk::Justification;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use okshell_services::network_service;

#[derive(Debug, Clone)]
pub(crate) struct DisconnectButtonModel {}

#[derive(Debug)]
pub(crate) enum DisconnectButtonInput {
    DisconnectClicked,
}

#[derive(Debug)]
pub(crate) enum DisconnectButtonOutput {}

#[relm4::component(pub)]
impl SimpleComponent for DisconnectButtonModel {
    type Input = DisconnectButtonInput;
    type Output = DisconnectButtonOutput;
    type Init = ();

    view! {
        #[root]
        gtk::Button {
            add_css_class: "ok-button-primary",
            set_hexpand: true,
            connect_clicked[sender] => move |_| {
                sender.input(DisconnectButtonInput::DisconnectClicked);
            },
            
            gtk::Label {
                add_css_class: "label-medium-bold-primary",
                set_label: "Disconnect",
                set_hexpand: true,
                set_justify: Justification::Center,
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DisconnectButtonModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: ComponentSender<Self>,
    ) {
        match message {
            DisconnectButtonInput::DisconnectClicked => {
                if let Some(wifi) = network_service().wifi.get() {
                    tokio::spawn(async move {
                        let _ = wifi.disconnect().await;
                    });
                }
            }
        }
    }
}
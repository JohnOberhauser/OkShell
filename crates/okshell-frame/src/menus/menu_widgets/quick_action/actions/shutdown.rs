use relm4::{gtk, Component, ComponentParts, ComponentSender, Controller, SimpleComponent};
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use crate::common_widgets::confirmation_dialog::{ConfirmationDialogInit, ConfirmationDialogModel, ConfirmationDialogOutput};
use okshell_utils::shutdown::shutdown;

pub(crate) struct ShutdownModel {
    dialog: Option<Controller<ConfirmationDialogModel>>,
}

#[derive(Debug)]
pub(crate) enum ShutdownInput {
    Clicked,
    ConfirmClicked,
    CancelClicked,
}

#[derive(Debug)]
pub(crate) enum ShutdownOutput {}

pub(crate) struct ShutdownInit {}

#[relm4::component(pub)]
impl SimpleComponent for ShutdownModel {
    type Input = ShutdownInput;
    type Output = ShutdownOutput;
    type Init = ShutdownInit;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-button-medium"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(ShutdownInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("system-shutdown-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ShutdownModel {
            dialog: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>,
    ) {
        match message {
            ShutdownInput::Clicked => {
                let dialog = ConfirmationDialogModel::builder()
                    .launch(ConfirmationDialogInit {
                        message: "Are you sure you want to log out?".to_string(),
                        negative_label: "Cancel".to_string(),
                        positive_label: "Logout".to_string(),
                    })
                    .forward(sender.input_sender(), |msg| {
                        match msg {
                            ConfirmationDialogOutput::PositiveClicked => {
                                ShutdownInput::ConfirmClicked
                            }
                            ConfirmationDialogOutput::NegativeClicked => {
                                ShutdownInput::CancelClicked
                            }
                        }
                    });

                self.dialog = Some(dialog);
            }
            ShutdownInput::ConfirmClicked => {
                self.dialog = None;
                shutdown();
            }
            ShutdownInput::CancelClicked => {
                self.dialog = None;
            }
        }
    }
}
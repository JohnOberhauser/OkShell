use relm4::{gtk, Component, ComponentParts, ComponentSender, Controller, SimpleComponent};
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use crate::common_widgets::confirmation_dialog::{ConfirmationDialogInit, ConfirmationDialogModel, ConfirmationDialogOutput};
use okshell_utils::reboot::reboot;

pub(crate) struct RebootModel {
    dialog: Option<Controller<ConfirmationDialogModel>>,
}

#[derive(Debug)]
pub(crate) enum RebootInput {
    Clicked,
    ConfirmClicked,
    CancelClicked,
}

#[derive(Debug)]
pub(crate) enum RebootOutput {}

pub(crate) struct RebootInit {}

#[relm4::component(pub)]
impl SimpleComponent for RebootModel {
    type Input = RebootInput;
    type Output = RebootOutput;
    type Init = RebootInit;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-button-medium"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(RebootInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("system-reboot-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RebootModel {
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
            RebootInput::Clicked => {
                let dialog = ConfirmationDialogModel::builder()
                    .launch(ConfirmationDialogInit {
                        message: "Are you sure you want to reboot?".to_string(),
                        negative_label: "Cancel".to_string(),
                        positive_label: "Reboot".to_string(),
                    })
                    .forward(sender.input_sender(), |msg| {
                        match msg {
                            ConfirmationDialogOutput::PositiveClicked => {
                                RebootInput::ConfirmClicked
                            }
                            ConfirmationDialogOutput::NegativeClicked => {
                                RebootInput::CancelClicked
                            }
                        }
                    });

                self.dialog = Some(dialog);
            }
            RebootInput::ConfirmClicked => {
                self.dialog = None;
                reboot()
            }
            RebootInput::CancelClicked => {
                self.dialog = None;
            }
        }
    }
}
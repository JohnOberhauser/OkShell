use crate::common_widgets::confirmation_dialog::{
    ConfirmationDialogInit, ConfirmationDialogModel, ConfirmationDialogOutput,
};
use okshell_utils::logout::logout;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, Controller, SimpleComponent, gtk};

pub(crate) struct LogoutModel {
    dialog: Option<Controller<ConfirmationDialogModel>>,
}

#[derive(Debug)]
pub(crate) enum LogoutInput {
    Clicked,
    ConfirmClicked,
    CancelClicked,
}

#[derive(Debug)]
pub(crate) enum LogoutOutput {}

pub(crate) struct LogoutInit {}

#[relm4::component(pub)]
impl SimpleComponent for LogoutModel {
    type Input = LogoutInput;
    type Output = LogoutOutput;
    type Init = LogoutInit;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-button-medium"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(LogoutInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("system-log-out-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LogoutModel { dialog: None };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            LogoutInput::Clicked => {
                let dialog = ConfirmationDialogModel::builder()
                    .launch(ConfirmationDialogInit {
                        message: "Are you sure you want to log out?".to_string(),
                        negative_label: "Cancel".to_string(),
                        positive_label: "Logout".to_string(),
                    })
                    .forward(sender.input_sender(), |msg| match msg {
                        ConfirmationDialogOutput::PositiveClicked => LogoutInput::ConfirmClicked,
                        ConfirmationDialogOutput::NegativeClicked => LogoutInput::CancelClicked,
                    });

                self.dialog = Some(dialog);
            }
            LogoutInput::ConfirmClicked => {
                self.dialog = None;
                logout();
            }
            LogoutInput::CancelClicked => {
                self.dialog = None;
            }
        }
    }
}

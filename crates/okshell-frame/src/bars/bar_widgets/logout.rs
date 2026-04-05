use relm4::{gtk, Component, ComponentParts, ComponentSender, Controller};
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use crate::common_widgets::confirmation_dialog::{ConfirmationDialogInit, ConfirmationDialogModel, ConfirmationDialogOutput};
use okshell_utils::logout::logout;

pub(crate) struct LogoutModel {
    dialog: Option<Controller<ConfirmationDialogModel>>,
    orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum LogoutInput {
    Clicked,
    ConfirmClicked,
    CancelClicked,
}

#[derive(Debug)]
pub(crate) enum LogoutOutput {}

pub(crate) struct LogoutInit {
    pub(crate) orientation: Orientation,
}

#[relm4::component(pub)]
impl Component for LogoutModel {
    type CommandOutput = ();
    type Input = LogoutInput;
    type Output = LogoutOutput;
    type Init = LogoutInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            add_css_class: "logout-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(LogoutInput::Clicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("system-log-out-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LogoutModel {
            dialog: None,
            orientation: params.orientation,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            LogoutInput::Clicked => {
                let dialog = ConfirmationDialogModel::builder()
                    .launch(ConfirmationDialogInit {
                        message: "Are you sure you want to log out?".to_string(),
                        negative_label: "Cancel".to_string(),
                        positive_label: "Logout".to_string(),
                    })
                    .forward(sender.input_sender(), |msg| {
                        match msg {
                            ConfirmationDialogOutput::PositiveClicked => {
                                LogoutInput::ConfirmClicked
                            }
                            ConfirmationDialogOutput::NegativeClicked => {
                                LogoutInput::CancelClicked
                            }
                        }
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
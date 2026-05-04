use gtk::prelude::*;
use gtk4_layer_shell::LayerShell;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

pub struct ConfirmationDialogModel {
    message: String,
    negative_label: String,
    positive_label: String,
}

/// Init data to configure the dialog.
#[derive(Debug, Clone)]
pub struct ConfirmationDialogInit {
    pub message: String,
    pub negative_label: String,
    pub positive_label: String,
}

/// Outputs emitted when a button is clicked.
#[derive(Debug, Clone)]
pub enum ConfirmationDialogOutput {
    NegativeClicked,
    PositiveClicked,
}

/// Inputs you can send to the dialog.
#[derive(Debug, Clone)]
pub enum ConfirmationDialogInput {
    Close,
}

#[relm4::component(pub)]
impl Component for ConfirmationDialogModel {
    type CommandOutput = ();
    type Init = ConfirmationDialogInit;
    type Input = ConfirmationDialogInput;
    type Output = ConfirmationDialogOutput;

    view! {
        #[root]
        #[name = "root"]
        gtk::Window {
            set_css_classes: &["dialog-window", "window-opacity"],
            set_decorated: false,
            set_resizable: false,
            set_modal: true,
            set_visible: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                add_css_class: "confirmation-dialog",

                #[name = "message_label"]
                gtk::Label {
                    add_css_class: "label-large",
                    add_css_class: "confirmation-dialog-title",
                    set_label: &model.message,
                    set_wrap: true,
                    set_xalign: 0.5,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    #[name = "negative"]
                    gtk::Button {
                        set_css_classes: &["label-small", "ok-button-surface"],
                        set_hexpand: true,
                        set_label: &model.negative_label,
                        connect_clicked[sender] => move |_| {
                            sender.output(ConfirmationDialogOutput::NegativeClicked).unwrap();
                            sender.input(ConfirmationDialogInput::Close);
                        }
                    },

                    #[name = "positive"]
                    gtk::Button {
                        set_css_classes: &["label-small", "ok-button-surface"],
                        set_hexpand: true,
                        set_label: &model.positive_label,
                        connect_clicked[sender] => move |_| {
                            sender.output(ConfirmationDialogOutput::PositiveClicked).unwrap();
                            sender.input(ConfirmationDialogInput::Close);
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_namespace(Some("okshell-dialog"));

        let model = ConfirmationDialogModel {
            message: init.message,
            negative_label: init.negative_label,
            positive_label: init.positive_label,
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
            ConfirmationDialogInput::Close => {
                widgets.root.close();
            }
        }
    }
}

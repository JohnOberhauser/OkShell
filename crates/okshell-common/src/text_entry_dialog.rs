use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

pub struct TextEntryDialogModel {
    message: String,
    negative_label: String,
    positive_label: String,
    entry_placeholder: String,
    entry2_placeholder: String,
    show_second_entry: bool,
}

/// Init data to configure the dialog.
#[derive(Debug, Clone)]
pub struct TextEntryDialogInit {
    pub message: String,
    pub negative_label: String,
    pub positive_label: String,
    pub entry_placeholder: String,
    pub entry2_placeholder: String,
    pub show_second_entry: bool,
}

/// Outputs emitted when a button is clicked.
#[derive(Debug, Clone)]
pub enum TextEntryDialogOutput {
    NegativeSelected,
    PositiveSelected(String, String),
}

/// Inputs you can send to the dialog.
#[derive(Debug, Clone)]
pub enum TextEntryDialogInput {
    Close,
    PositiveClicked,
    EntryActivated,
    Entry2Activated,
}

#[relm4::component(pub)]
impl Component for TextEntryDialogModel {
    type CommandOutput = ();
    type Init = TextEntryDialogInit;
    type Input = TextEntryDialogInput;
    type Output = TextEntryDialogOutput;

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
                    add_css_class: "text-entry-dialog-title",
                    set_label: &model.message,
                    set_wrap: true,
                    set_xalign: 0.5,
                },

                #[name = "entry"]
                gtk::Entry {
                    set_hexpand: true,
                    add_css_class: "ok-entry",
                    connect_activate => TextEntryDialogInput::EntryActivated,
                    set_margin_start: 20,
                    set_margin_end: 20,
                    set_placeholder_text: Some(model.entry_placeholder.as_str()),
                },

                #[name = "entry2"]
                gtk::Entry {
                    set_visible: model.show_second_entry,
                    set_hexpand: true,
                    add_css_class: "ok-entry",
                    connect_activate => TextEntryDialogInput::Entry2Activated,
                    set_margin_start: 20,
                    set_margin_end: 20,
                    set_placeholder_text: Some(model.entry2_placeholder.as_str()),
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
                            sender.output(TextEntryDialogOutput::NegativeSelected).unwrap();
                            sender.input(TextEntryDialogInput::Close);
                        }
                    },

                    #[name = "positive"]
                    gtk::Button {
                        set_css_classes: &["label-small", "ok-button-surface"],
                        set_hexpand: true,
                        set_label: &model.positive_label,
                        connect_clicked[sender] => move |_| {
                            sender.input(TextEntryDialogInput::PositiveClicked);
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
        root.set_keyboard_mode(KeyboardMode::Exclusive);

        let model = TextEntryDialogModel {
            message: init.message,
            negative_label: init.negative_label,
            positive_label: init.positive_label,
            entry_placeholder: init.entry_placeholder,
            entry2_placeholder: init.entry2_placeholder,
            show_second_entry: init.show_second_entry,
        };

        let widgets = view_output!();

        widgets.entry.grab_focus();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            TextEntryDialogInput::PositiveClicked => {
                let _ = sender.output(TextEntryDialogOutput::PositiveSelected(
                    widgets.entry.text().to_string(),
                    widgets.entry2.text().to_string(),
                ));
                sender.input(TextEntryDialogInput::Close);
            }
            TextEntryDialogInput::Close => {
                root.set_keyboard_mode(KeyboardMode::None);
                widgets.root.close();
            }
            TextEntryDialogInput::EntryActivated => {
                if self.show_second_entry {
                    widgets.entry2.grab_focus();
                } else {
                    sender.input(TextEntryDialogInput::PositiveClicked);
                }
            }
            TextEntryDialogInput::Entry2Activated => {
                sender.input(TextEntryDialogInput::PositiveClicked);
            }
        }
    }
}

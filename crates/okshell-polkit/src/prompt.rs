use gtk4::glib;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use gtk::prelude::*;
use relm4::prelude::*;
use tracing::info;
use crate::agent::PasswordAction;
use crate::register_polkit_agent;

pub struct PolkitPromptModel {
    visible: bool,
    message: String,
    icon_name: String,
    prompt_text: String,
    info_text: String,
    error_text: String,
    password_visible: bool,
    password_tx: Option<tokio::sync::mpsc::Sender<PasswordAction>>,
    entry_buffer: gtk::EntryBuffer,
}

#[derive(Debug)]
pub enum PolkitPromptInput {
    Show {
        message: String,
        icon_name: String,
        password_tx: tokio::sync::mpsc::Sender<PasswordAction>,
    },
    PromptReady {
        prompt: String,
        echo: bool,
    },
    InfoMessage(String),
    ErrorMessage(String),
    ClearEntry,
    Hide,
    Submit,
    Cancel,
    ToggleVisibility,
}

#[derive(Debug)]
pub enum PolkitPromptCommand {}

#[relm4::component(pub)]
impl Component for PolkitPromptModel {
    type Init = ();
    type Input = PolkitPromptInput;
    type Output = ();
    type CommandOutput = PolkitPromptCommand;

    view! {
        #[root]
        gtk::Window {
            set_css_classes: &["polkit-window", "window-opacity"],
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 20,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Image {
                        add_css_class: "polkit-icon",
                        set_icon_name: Some("polkit-symbolic"),
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,
                        set_hexpand: true,

                        gtk::Label {
                            add_css_class: "label-large-bold",
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            #[watch]
                            set_label: &model.message,
                        },

                        gtk::Label {
                            add_css_class: "label-medium",
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            #[watch]
                            set_visible: !model.info_text.is_empty(),
                            #[watch]
                            set_label: &model.info_text,
                        },

                        gtk::Box {
                            #[watch]
                            set_visible: !model.error_text.is_empty(),
                            add_css_class: "surface-error",

                            gtk::Label {
                                add_css_class: "label-medium-error",
                                set_halign: gtk::Align::Start,
                                set_wrap: true,
                                #[watch]
                                set_label: &model.error_text,
                            },
                        },

                        gtk::Label {
                            set_css_classes: &["label-medium"],
                            set_halign: gtk::Align::Start,
                            #[watch]
                            set_visible: !model.prompt_text.is_empty(),
                            #[watch]
                            set_label: &model.prompt_text,
                        },

                        #[name = "password_entry"]
                        gtk::Entry {
                            set_hexpand: true,
                            add_css_class: "ok-entry",
                            #[watch]
                            set_visibility: model.password_visible,
                            set_buffer: &model.entry_buffer,
                            connect_activate => PolkitPromptInput::Submit,
                            #[watch]
                            set_icon_from_icon_name: (
                                gtk::EntryIconPosition::Secondary,
                                Some(
                                    if model.password_visible {
                                        "eye-symbolic"
                                    } else {
                                        "eye-off-symbolic"
                                    }
                                )
                            ),
                            set_icon_activatable: (gtk::EntryIconPosition::Secondary, true),
                            connect_icon_press[sender] => move |_, pos| {
                                if pos == gtk::EntryIconPosition::Secondary {
                                    sender.input(PolkitPromptInput::ToggleVisibility);
                                }
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_namespace(Some("okshell-dialog"));
        root.set_layer(Layer::Top);
        root.set_exclusive_zone(-1);
        root.set_keyboard_mode(KeyboardMode::Exclusive);
        root.set_default_size(600, -1);

        let esc_sender = sender.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                esc_sender.input(PolkitPromptInput::Cancel);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        root.add_controller(key_controller);

        let model = Self {
            visible: false,
            message: String::new(),
            icon_name: String::new(),
            prompt_text: String::new(),
            info_text: String::new(),
            error_text: String::new(),
            password_visible: false,
            password_tx: None,
            entry_buffer: gtk::EntryBuffer::new(None::<&str>),
        };

        let widgets = view_output!();

        let prompt_sender = sender.clone();
        tokio::spawn(async move {
            match register_polkit_agent(prompt_sender).await {
                Ok(_connection) => {
                    // Keep the future alive forever so the connection isn't dropped
                    std::future::pending::<()>().await;
                }
                Err(e) => tracing::error!("polkit agent registration failed: {e}"),
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        input: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match input {
            PolkitPromptInput::Show {
                message,
                icon_name,
                password_tx,
            } => {
                info!("show");
                self.message = message;
                self.icon_name = icon_name;
                self.info_text.clear();
                self.prompt_text.clear();
                self.entry_buffer.set_text("");
                self.password_tx = Some(password_tx);
                self.visible = true;
            }
            PolkitPromptInput::PromptReady { prompt, echo } => {
                self.prompt_text = prompt;
                self.password_visible = echo;
            }
            PolkitPromptInput::InfoMessage(text) => {
                info!("info message: {text}");
                if self.info_text.is_empty() {
                    self.info_text = text;
                } else {
                    self.info_text.push('\n');
                    self.info_text.push_str(&text);
                }
            }
            PolkitPromptInput::ErrorMessage(text) => {
                info!("error message: {text}");
                self.error_text = text;
            }
            PolkitPromptInput::ClearEntry => {
                info!("clear entry");
                self.entry_buffer.set_text("");
            }
            PolkitPromptInput::Hide => {
                self.visible = false;
                self.password_tx = None;
                self.entry_buffer.set_text("");
            }
            PolkitPromptInput::Submit => {
                let password = self.entry_buffer.text().to_string();
                if let Some(tx) = &self.password_tx {
                    let tx = tx.clone();
                    relm4::spawn_local(async move {
                        let _ = tx.send(PasswordAction::Submit(password)).await;
                    });
                }
            }
            PolkitPromptInput::Cancel => {
                if let Some(tx) = &self.password_tx {
                    let tx = tx.clone();
                    relm4::spawn_local(async move {
                        let _ = tx.send(PasswordAction::Cancel).await;
                    });
                }
            }
            PolkitPromptInput::ToggleVisibility => {
                self.password_visible = !self.password_visible;
            }
        }
    }
}
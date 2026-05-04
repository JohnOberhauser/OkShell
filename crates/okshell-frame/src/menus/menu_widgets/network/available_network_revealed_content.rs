use okshell_services::network_service;
use okshell_utils::key_mode::wire_entry_focus;
use relm4::gtk::Justification;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use std::ops::Not;
use std::sync::Arc;
use wayle_network::core::access_point::{AccessPoint, SecurityType};

#[derive(Debug, Clone)]
pub(crate) struct AvailableNetworkRevealedContentModel {
    access_point: Arc<AccessPoint>,
    connecting: bool,
    error: bool,
    show_password: bool,
    show_password_entry: bool,
}

#[derive(Debug)]
pub(crate) enum AvailableNetworkRevealedContentInput {
    Connect,
    ErrorConnecting,
    TogglePasswordVisibility,
    Reset,
}

#[derive(Debug)]
pub(crate) enum AvailableNetworkRevealedContentOutput {}

pub(crate) struct AvailableNetworkRevealedContentInit {
    pub access_point: Arc<AccessPoint>,
}

#[derive(Debug)]
pub(crate) enum AvailableNetworkRevealedContentCommandOutput {}

#[relm4::component(pub)]
impl Component for AvailableNetworkRevealedContentModel {
    type CommandOutput = AvailableNetworkRevealedContentCommandOutput;
    type Input = AvailableNetworkRevealedContentInput;
    type Output = AvailableNetworkRevealedContentOutput;
    type Init = AvailableNetworkRevealedContentInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            #[name = "password_entry"]
            gtk::Entry {
                add_css_class: "ok-entry-with-border",
                set_placeholder_text: Some("Password"),
                #[watch]
                set_visible: model.show_password_entry,
                #[watch]
                set_visibility: model.show_password,
                #[watch]
                set_icon_from_icon_name: (
                    gtk::EntryIconPosition::Secondary,
                    Some(
                        if model.show_password {
                            "eye-symbolic"
                        } else {
                            "eye-off-symbolic"
                        }
                    )
                ),
                set_icon_activatable: (gtk::EntryIconPosition::Secondary, true),
                connect_icon_press[sender] => move |_, pos| {
                    if pos == gtk::EntryIconPosition::Secondary {
                        sender.input(AvailableNetworkRevealedContentInput::TogglePasswordVisibility);
                    }
                },
                connect_activate[sender] => move |_| {
                    sender.input(AvailableNetworkRevealedContentInput::Connect);
                },
            },

            gtk::Label {
                add_css_class: "label-medium-bold-error",
                set_label: "Error Connecting",
                #[watch]
                set_visible: model.error,
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                set_sensitive: !model.connecting,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(AvailableNetworkRevealedContentInput::Connect);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    set_label: "Connect",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let network = network_service();
        let ssid = params.access_point.ssid.get().clone();

        let has_security = params.access_point.security.get() != SecurityType::None;

        let is_saved = network
            .settings
            .connections_for_ssid(&ssid)
            .is_empty()
            .not();

        let show_password_entry = !is_saved && has_security;

        let model = AvailableNetworkRevealedContentModel {
            access_point: params.access_point,
            connecting: false,
            error: false,
            show_password: false,
            show_password_entry,
        };

        let widgets = view_output!();

        wire_entry_focus(&widgets.password_entry);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AvailableNetworkRevealedContentInput::Connect => {
                if self.connecting {
                    return;
                }
                let network = network_service();
                if let Some(wifi) = network.wifi.get() {
                    let password = if !self.show_password_entry {
                        None
                    } else {
                        Some(widgets.password_entry.text().to_string())
                    };
                    let object_path = self.access_point.object_path().clone();
                    let sender_clone = sender.clone();
                    self.connecting = true;
                    self.error = false;
                    tokio::spawn(async move {
                        match wifi.connect(object_path, password).await {
                            Ok(_) => {
                                // this widget should be removed upon success, so do nothing here
                            }
                            Err(_) => {
                                glib::idle_add_once(move || {
                                    sender_clone.input(
                                        AvailableNetworkRevealedContentInput::ErrorConnecting,
                                    );
                                });
                            }
                        }
                    });
                }
            }
            AvailableNetworkRevealedContentInput::ErrorConnecting => {
                self.connecting = false;
                self.error = true;
                widgets.password_entry.set_text("");
            }
            AvailableNetworkRevealedContentInput::TogglePasswordVisibility => {
                self.show_password = !self.show_password;
            }
            AvailableNetworkRevealedContentInput::Reset => {
                self.connecting = false;
                self.error = false;
                self.show_password = false;
                widgets.password_entry.set_text("");
            }
        }

        self.update_view(widgets, sender);
    }
}

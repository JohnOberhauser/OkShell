use std::sync::Arc;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::Justification;
use relm4::gtk::prelude::*;
use relm4::gtk::glib;
use wayle_bluetooth::core::device::Device;
use okshell_common::WatcherToken;
use okshell_utils::bluetooth::spawn_bluetooth_device_watcher;

pub(crate) struct DeviceRevealedContentModel {
    device: Arc<Device>,
    paired: bool,
    connected: bool,
    trusted: bool,
    is_pairing: bool,
    is_forgetting: bool,
    is_trusting: bool,
    is_untrusting: bool,
    is_connecting: bool,
    is_disconnecting: bool,
    device_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum DeviceRevealedContentInput {
    PairClicked,
    DonePairing,
    ForgetClicked,
    DoneForgetting,
    TrustClicked,
    DoneTrusting,
    UntrustClicked,
    DoneUntrusting,
    ConnectClicked,
    DoneConnecting,
    DisconnectClicked,
    DoneDisconnecting,
    UpdateState,
    ParentRevealed(bool),
}

#[derive(Debug)]
pub(crate) enum DeviceRevealedContentOutput {}

pub(crate) struct DeviceRevealedContentInit {
    pub device: Arc<Device>,
}

#[derive(Debug)]
pub(crate) enum DeviceRevealedContentCommandOutput {
    DeviceUpdated,
}

#[relm4::component(pub)]
impl Component for DeviceRevealedContentModel {
    type CommandOutput = DeviceRevealedContentCommandOutput;
    type Input = DeviceRevealedContentInput;
    type Output = DeviceRevealedContentOutput;
    type Init = DeviceRevealedContentInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: model.paired && !model.connected,
                #[watch]
                set_sensitive: !model.is_connecting,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(DeviceRevealedContentInput::ConnectClicked);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    #[watch]
                    set_label: if model.is_connecting {
                        "Connecting…"
                    } else {
                        "Connect"
                    },
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: model.paired && model.connected,
                #[watch]
                set_sensitive: !model.is_disconnecting,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(DeviceRevealedContentInput::DisconnectClicked);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    #[watch]
                    set_label: if model.is_disconnecting {
                        "Disconnecting…"
                    } else {
                        "Disconnect"
                    },
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: model.paired && !model.trusted,
                #[watch]
                set_sensitive: !model.is_trusting,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(DeviceRevealedContentInput::TrustClicked);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    #[watch]
                    set_label: if model.is_trusting {
                        "Trusting…"
                    } else {
                        "Trust"
                    },
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: model.paired && model.trusted,
                #[watch]
                set_sensitive: !model.is_untrusting,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(DeviceRevealedContentInput::UntrustClicked);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    #[watch]
                    set_label: if model.is_untrusting {
                        "Untrusting…"
                    } else {
                        "Untrust"
                    },
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: !model.paired,
                #[watch]
                set_sensitive: !model.is_pairing,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(DeviceRevealedContentInput::PairClicked);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    #[watch]
                    set_label: if model.is_pairing {
                        "Pairing…"
                    } else {
                        "Pair"
                    },
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                #[watch]
                set_visible: model.paired,
                #[watch]
                set_sensitive: !model.is_forgetting,
                set_hexpand: true,
                connect_clicked[sender] => move |_| {
                    sender.input(DeviceRevealedContentInput::ForgetClicked);
                },

                gtk::Label {
                    add_css_class: "label-medium-bold-primary",
                    #[watch]
                    set_label: if model.is_forgetting {
                        "Forgetting…"
                    } else {
                        "Forget"
                    },
                    set_hexpand: true,
                    set_justify: Justification::Center,
                }
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut device_watcher_token = WatcherToken::new();

        let token = device_watcher_token.reset();

        spawn_bluetooth_device_watcher(
            &params.device,
            token,
            &sender,
            ||DeviceRevealedContentCommandOutput::DeviceUpdated,
        );

        let device = params.device.clone();
        let model = DeviceRevealedContentModel {
            device,
            paired: params.device.paired.get(),
            connected: params.device.connected.get(),
            trusted: params.device.trusted.get(),
            is_pairing: false,
            is_forgetting: false,
            is_trusting: false,
            is_untrusting: false,
            is_connecting: false,
            is_disconnecting: false,
            device_watcher_token,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let sender_clone = sender.clone();
        match message {
            DeviceRevealedContentInput::PairClicked => {
                let device = self.device.clone();
                self.is_pairing = true;
                tokio::spawn(async move {
                    let _ = device.pair().await;
                    glib::idle_add_once(move || {
                        sender.input(DeviceRevealedContentInput::DonePairing);
                    });
                });
            }
            DeviceRevealedContentInput::DonePairing => {
                self.is_pairing = false;
            }
            DeviceRevealedContentInput::ForgetClicked => {
                let device = self.device.clone();
                self.is_forgetting = true;
                tokio::spawn(async move {
                    let _ = device.forget().await;
                    glib::idle_add_once(move || {
                        sender.input(DeviceRevealedContentInput::DoneForgetting);
                    });
                });
            }
            DeviceRevealedContentInput::DoneForgetting => {
                self.is_forgetting = false;
            }
            DeviceRevealedContentInput::TrustClicked => {
                let device = self.device.clone();
                self.is_trusting = true;
                tokio::spawn(async move {
                    let _ = device.set_trusted(true).await;
                    glib::idle_add_once(move || {
                        sender.input(DeviceRevealedContentInput::DoneTrusting);
                    });
                });
            }
            DeviceRevealedContentInput::DoneTrusting => {
                self.is_trusting = false;
            }
            DeviceRevealedContentInput::UntrustClicked => {
                let device = self.device.clone();
                self.is_untrusting = true;
                tokio::spawn(async move {
                    let _ = device.set_trusted(false).await;
                    glib::idle_add_once(move || {
                        sender.input(DeviceRevealedContentInput::DoneUntrusting);
                    });
                });
            }
            DeviceRevealedContentInput::DoneUntrusting => {
                self.is_untrusting = false;
            }
            DeviceRevealedContentInput::ConnectClicked => {
                let device = self.device.clone();
                self.is_connecting = true;
                tokio::spawn(async move {
                    let _ = device.connect().await;
                    glib::idle_add_once(move || {
                        sender.input(DeviceRevealedContentInput::DoneConnecting);
                    });
                });
            }
            DeviceRevealedContentInput::DoneConnecting => {
                self.is_connecting = false;
            }
            DeviceRevealedContentInput::DisconnectClicked => {
                let device = self.device.clone();
                self.is_disconnecting = true;
                tokio::spawn(async move {
                    let _ = device.disconnect().await;
                    glib::idle_add_once(move || {
                        sender.input(DeviceRevealedContentInput::DoneDisconnecting);
                    });
                });
            }
            DeviceRevealedContentInput::DoneDisconnecting => {
                self.is_disconnecting = false;
            }
            DeviceRevealedContentInput::UpdateState => {
                self.paired = self.device.paired.get();
                self.connected = self.device.connected.get();
                self.trusted = self.device.trusted.get();
            }
            DeviceRevealedContentInput::ParentRevealed(revealed) => {
                let token = self.device_watcher_token.reset();

                if revealed {
                    spawn_bluetooth_device_watcher(
                        &self.device,
                        token,
                        &sender,
                        ||DeviceRevealedContentCommandOutput::DeviceUpdated,
                    );
                }
            }
        }

        self.update_view(widgets, sender_clone);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            DeviceRevealedContentCommandOutput::DeviceUpdated => {
                sender.input(DeviceRevealedContentInput::UpdateState);
            }
        }
    }
}
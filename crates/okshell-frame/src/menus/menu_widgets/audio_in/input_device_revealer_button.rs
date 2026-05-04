use crate::common_widgets::revealer_button::revealer_button_icon_label::{
    RevealerButtonIconLabelInit, RevealerButtonIconLabelInput, RevealerButtonIconLabelModel,
};
use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::spawn_default_input_watcher;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use wayle_audio::core::device::input::InputDevice;

pub(crate) struct InputDeviceRevealerButtonModel {
    input_device: Arc<InputDevice>,
    content: Controller<RevealerButtonIconLabelModel>,
    watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum InputDeviceRevealerButtonInput {
    Clicked,
    DefaultDeviceChanged,
    Revealed,
    Hidden,
}

#[derive(Debug)]
pub(crate) enum InputDeviceRevealerButtonOutput {}

pub(crate) struct InputDeviceRevealerButtonInit {
    pub input_device: Arc<InputDevice>,
}

#[derive(Debug)]
pub(crate) enum InputDeviceRevealerButtonCommandOutput {
    DefaultDeviceChanged,
}

#[relm4::component(pub)]
impl Component for InputDeviceRevealerButtonModel {
    type CommandOutput = InputDeviceRevealerButtonCommandOutput;
    type Input = InputDeviceRevealerButtonInput;
    type Output = InputDeviceRevealerButtonOutput;
    type Init = InputDeviceRevealerButtonInit;

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                add_css_class: "ok-button-surface",
                set_hexpand: true,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(InputDeviceRevealerButtonInput::Clicked);
                },

                model.content.widget().clone() {},
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut watcher_token = WatcherToken::new();

        let token = watcher_token.reset();

        spawn_default_input_watcher(&sender, Some(token), || {
            InputDeviceRevealerButtonCommandOutput::DefaultDeviceChanged
        });

        let button_content = RevealerButtonIconLabelModel::builder()
            .launch(RevealerButtonIconLabelInit {
                label: params.input_device.description.get(),
                icon_name: "".to_string(),
                secondary_icon_name: "".to_string(),
            })
            .detach();

        let model = InputDeviceRevealerButtonModel {
            input_device: params.input_device,
            content: button_content,
            watcher_token,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            InputDeviceRevealerButtonInput::Clicked => {
                let device = self.input_device.clone();
                tokio::spawn(async move {
                    let _ = device.set_as_default().await;
                });
            }
            InputDeviceRevealerButtonInput::DefaultDeviceChanged => {
                let default_device = audio_service().default_input.get();

                if let Some(default_device) = default_device {
                    if default_device.eq(&self.input_device) {
                        self.content
                            .emit(RevealerButtonIconLabelInput::SetPrimaryIconName(
                                "check-circle-symbolic".to_string(),
                            ))
                    } else {
                        self.content
                            .emit(RevealerButtonIconLabelInput::SetPrimaryIconName(
                                "".to_string(),
                            ))
                    }
                } else {
                    self.content
                        .emit(RevealerButtonIconLabelInput::SetPrimaryIconName(
                            "".to_string(),
                        ))
                }
            }
            InputDeviceRevealerButtonInput::Revealed => {
                let token = self.watcher_token.reset();

                spawn_default_input_watcher(&sender, Some(token), || {
                    InputDeviceRevealerButtonCommandOutput::DefaultDeviceChanged
                });
            }
            InputDeviceRevealerButtonInput::Hidden => {
                self.watcher_token.reset();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            InputDeviceRevealerButtonCommandOutput::DefaultDeviceChanged => {
                sender.input(InputDeviceRevealerButtonInput::DefaultDeviceChanged);
            }
        }
    }
}

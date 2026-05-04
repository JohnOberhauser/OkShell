use crate::common_widgets::revealer_button::revealer_button_icon_label::{
    RevealerButtonIconLabelInit, RevealerButtonIconLabelInput, RevealerButtonIconLabelModel,
};
use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::spawn_default_output_watcher;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use wayle_audio::core::device::output::OutputDevice;

pub(crate) struct OutputDeviceRevealerButtonModel {
    output_device: Arc<OutputDevice>,
    content: Controller<RevealerButtonIconLabelModel>,
    watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum OutputDeviceRevealerButtonInput {
    Clicked,
    DefaultDeviceChanged,
    Revealed,
    Hidden,
}

#[derive(Debug)]
pub(crate) enum OutputDeviceRevealerButtonOutput {}

pub(crate) struct OutputDeviceRevealerButtonInit {
    pub output_device: Arc<OutputDevice>,
}

#[derive(Debug)]
pub(crate) enum OutputDeviceRevealerButtonCommandOutput {
    DefaultDeviceChanged,
}

#[relm4::component(pub)]
impl Component for OutputDeviceRevealerButtonModel {
    type CommandOutput = OutputDeviceRevealerButtonCommandOutput;
    type Input = OutputDeviceRevealerButtonInput;
    type Output = OutputDeviceRevealerButtonOutput;
    type Init = OutputDeviceRevealerButtonInit;

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                add_css_class: "ok-button-surface",
                set_hexpand: true,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(OutputDeviceRevealerButtonInput::Clicked);
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

        spawn_default_output_watcher(&sender, Some(token), || {
            OutputDeviceRevealerButtonCommandOutput::DefaultDeviceChanged
        });

        let button_content = RevealerButtonIconLabelModel::builder()
            .launch(RevealerButtonIconLabelInit {
                label: params.output_device.description.get(),
                icon_name: "".to_string(),
                secondary_icon_name: "".to_string(),
            })
            .detach();

        let model = OutputDeviceRevealerButtonModel {
            output_device: params.output_device,
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
            OutputDeviceRevealerButtonInput::Clicked => {
                let device = self.output_device.clone();
                tokio::spawn(async move {
                    let _ = device.set_as_default().await;
                });
            }
            OutputDeviceRevealerButtonInput::DefaultDeviceChanged => {
                let default_device = audio_service().default_output.get();

                if let Some(default_device) = default_device {
                    if default_device.eq(&self.output_device) {
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
            OutputDeviceRevealerButtonInput::Revealed => {
                let token = self.watcher_token.reset();

                spawn_default_output_watcher(&sender, Some(token), || {
                    OutputDeviceRevealerButtonCommandOutput::DefaultDeviceChanged
                });
            }
            OutputDeviceRevealerButtonInput::Hidden => {
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
            OutputDeviceRevealerButtonCommandOutput::DefaultDeviceChanged => {
                sender.input(OutputDeviceRevealerButtonInput::DefaultDeviceChanged);
            }
        }
    }
}

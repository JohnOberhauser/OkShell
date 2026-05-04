use crate::common_widgets::revealer_row::revealer_row::{
    RevealerRowInit, RevealerRowInput, RevealerRowModel, RevealerRowOutput,
};
use crate::common_widgets::revealer_row::revealer_row_slider::{
    RevealerRowSliderInit, RevealerRowSliderInput, RevealerRowSliderModel, RevealerRowSliderOutput,
};
use crate::menus::menu_widgets::audio_out::audio_out_revealed_content::{
    AudioOutRevealedContentInit, AudioOutRevealedContentInput, AudioOutRevealedContentModel,
};
use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::{
    get_audio_out_icon, spawn_default_output_watcher, spawn_output_device_volume_mute_watcher,
};
use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use wayle_audio::core::device::output::OutputDevice;
use wayle_audio::volume::types::Volume;

pub(crate) struct AudioOutMenuWidgetModel {
    revealer_row:
        Controller<RevealerRowModel<RevealerRowSliderModel, AudioOutRevealedContentModel>>,
    active_device_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum AudioOutMenuWidgetInput {
    ActionButtonClicked,
    RevealerRowRevealed,
    RevealerRowHidden,
    ParentRevealChanged(bool),
    UpdateDevice(Arc<OutputDevice>),
    SetVolume(f64),
}

#[derive(Debug)]
pub(crate) enum AudioOutMenuWidgetOutput {}

pub(crate) struct AudioOutMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum AudioOutMenuWidgetCommandOutput {
    DeviceChanged,
    VolumeChanged,
}

#[relm4::component(pub)]
impl Component for AudioOutMenuWidgetModel {
    type CommandOutput = AudioOutMenuWidgetCommandOutput;
    type Input = AudioOutMenuWidgetInput;
    type Output = AudioOutMenuWidgetOutput;
    type Init = AudioOutMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "audio-out-menu-widget",

            model.revealer_row.widget().clone() {},
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_default_output_watcher(&sender, None, || {
            AudioOutMenuWidgetCommandOutput::DeviceChanged
        });

        let row_content = RevealerRowSliderModel::builder()
            .launch(RevealerRowSliderInit {})
            .forward(sender.input_sender(), |msg| match msg {
                RevealerRowSliderOutput::ValueChanged(value) => {
                    AudioOutMenuWidgetInput::SetVolume(value)
                }
            });

        let revealed_content = AudioOutRevealedContentModel::builder()
            .launch(AudioOutRevealedContentInit {})
            .detach();

        let revealer_row =
            RevealerRowModel::<RevealerRowSliderModel, AudioOutRevealedContentModel>::builder()
                .launch(RevealerRowInit {
                    icon_name: "audio-volume-medium-symbolic".into(),
                    action_button_sensitive: true,
                    content: row_content,
                    revealed_content,
                })
                .forward(sender.input_sender(), |msg| match msg {
                    RevealerRowOutput::ActionButtonClicked => {
                        AudioOutMenuWidgetInput::ActionButtonClicked
                    }
                    RevealerRowOutput::Revealed => AudioOutMenuWidgetInput::RevealerRowRevealed,
                    RevealerRowOutput::Hidden => AudioOutMenuWidgetInput::RevealerRowHidden,
                });

        let model = AudioOutMenuWidgetModel {
            revealer_row,
            active_device_watcher_token: WatcherToken::new(),
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
        match message {
            AudioOutMenuWidgetInput::ActionButtonClicked => {
                if let Some(default_output) = audio_service().default_output.get() {
                    tokio::spawn(async move {
                        let mute = !default_output.muted.get();
                        let _ = default_output.set_mute(mute).await;
                    });
                }
            }
            AudioOutMenuWidgetInput::RevealerRowRevealed => {
                self.revealer_row
                    .model()
                    .revealed_content
                    .emit(AudioOutRevealedContentInput::Revealed);
            }
            AudioOutMenuWidgetInput::RevealerRowHidden => {
                self.revealer_row
                    .model()
                    .revealed_content
                    .emit(AudioOutRevealedContentInput::Hidden);
            }
            AudioOutMenuWidgetInput::ParentRevealChanged(revealed) => {
                if !revealed {
                    self.revealer_row.emit(RevealerRowInput::SetRevealed(false));
                }
            }
            AudioOutMenuWidgetInput::UpdateDevice(device) => {
                self.revealer_row
                    .emit(RevealerRowInput::UpdateActionIconName(
                        get_audio_out_icon(&device).to_string(),
                    ));
                self.revealer_row
                    .model()
                    .content
                    .emit(RevealerRowSliderInput::SetValue(
                        device.volume.get().average(),
                    ))
            }
            AudioOutMenuWidgetInput::SetVolume(volume) => {
                if let Some(default_output) = audio_service().default_output.get() {
                    tokio::spawn(async move {
                        let _ = default_output
                            .set_volume(Volume::stereo(volume, volume))
                            .await;
                    });
                }
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AudioOutMenuWidgetCommandOutput::DeviceChanged => {
                let default_output = audio_service().default_output.get();
                if let Some(audio_device) = default_output {
                    sender.input(AudioOutMenuWidgetInput::UpdateDevice(audio_device.clone()));

                    let token = self.active_device_watcher_token.reset();

                    spawn_output_device_volume_mute_watcher(&audio_device, token, &sender, || {
                        AudioOutMenuWidgetCommandOutput::VolumeChanged
                    });
                }
            }
            AudioOutMenuWidgetCommandOutput::VolumeChanged => {
                if let Some(default_output) = audio_service().default_output.get() {
                    sender.input(AudioOutMenuWidgetInput::UpdateDevice(default_output));
                }
            }
        }
    }
}

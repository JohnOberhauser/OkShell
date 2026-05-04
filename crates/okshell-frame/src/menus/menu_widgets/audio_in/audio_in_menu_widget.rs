use crate::common_widgets::revealer_row::revealer_row::{
    RevealerRowInit, RevealerRowInput, RevealerRowModel, RevealerRowOutput,
};
use crate::common_widgets::revealer_row::revealer_row_slider::{
    RevealerRowSliderInit, RevealerRowSliderInput, RevealerRowSliderModel, RevealerRowSliderOutput,
};
use crate::menus::menu_widgets::audio_in::audio_in_revealed_content::{
    AudioInRevealedContentInit, AudioInRevealedContentInput, AudioInRevealedContentModel,
};
use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::{
    get_audio_in_icon, spawn_default_input_watcher, spawn_input_device_volume_mute_watcher,
};
use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use wayle_audio::core::device::input::InputDevice;
use wayle_audio::volume::types::Volume;

pub(crate) struct AudioInMenuWidgetModel {
    revealer_row: Controller<RevealerRowModel<RevealerRowSliderModel, AudioInRevealedContentModel>>,
    active_device_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum AudioInMenuWidgetInput {
    ActionButtonClicked,
    RevealerRowRevealed,
    RevealerRowHidden,
    ParentRevealChanged(bool),
    UpdateDevice(Arc<InputDevice>),
    SetVolume(f64),
}

#[derive(Debug)]
pub(crate) enum AudioInMenuWidgetOutput {}

pub(crate) struct AudioInMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum AudioInMenuWidgetCommandOutput {
    DeviceChanged,
    VolumeChanged,
}

#[relm4::component(pub)]
impl Component for AudioInMenuWidgetModel {
    type CommandOutput = AudioInMenuWidgetCommandOutput;
    type Input = AudioInMenuWidgetInput;
    type Output = AudioInMenuWidgetOutput;
    type Init = AudioInMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "audio-in-menu-widget",

            model.revealer_row.widget().clone() {},
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_default_input_watcher(&sender, None, || {
            AudioInMenuWidgetCommandOutput::DeviceChanged
        });

        let row_content = RevealerRowSliderModel::builder()
            .launch(RevealerRowSliderInit {})
            .forward(sender.input_sender(), |msg| match msg {
                RevealerRowSliderOutput::ValueChanged(value) => {
                    AudioInMenuWidgetInput::SetVolume(value)
                }
            });

        let revealed_content = AudioInRevealedContentModel::builder()
            .launch(AudioInRevealedContentInit {})
            .detach();

        let revealer_row =
            RevealerRowModel::<RevealerRowSliderModel, AudioInRevealedContentModel>::builder()
                .launch(RevealerRowInit {
                    icon_name: "microphone-sensitivity-medium-symbolic".into(),
                    action_button_sensitive: true,
                    content: row_content,
                    revealed_content,
                })
                .forward(sender.input_sender(), |msg| match msg {
                    RevealerRowOutput::ActionButtonClicked => {
                        AudioInMenuWidgetInput::ActionButtonClicked
                    }
                    RevealerRowOutput::Revealed => AudioInMenuWidgetInput::RevealerRowRevealed,
                    RevealerRowOutput::Hidden => AudioInMenuWidgetInput::RevealerRowHidden,
                });

        let model = AudioInMenuWidgetModel {
            revealer_row,
            active_device_watcher_token: WatcherToken::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AudioInMenuWidgetInput::ActionButtonClicked => {
                if let Some(default) = audio_service().default_input.get() {
                    tokio::spawn(async move {
                        let mute = !default.muted.get();
                        let _ = default.set_mute(mute).await;
                    });
                }
            }
            AudioInMenuWidgetInput::RevealerRowRevealed => {
                self.revealer_row
                    .model()
                    .revealed_content
                    .emit(AudioInRevealedContentInput::Revealed);
            }
            AudioInMenuWidgetInput::RevealerRowHidden => {
                self.revealer_row
                    .model()
                    .revealed_content
                    .emit(AudioInRevealedContentInput::Hidden);
            }
            AudioInMenuWidgetInput::ParentRevealChanged(revealed) => {
                if !revealed {
                    self.revealer_row.emit(RevealerRowInput::SetRevealed(false));
                }
            }
            AudioInMenuWidgetInput::UpdateDevice(device) => {
                self.revealer_row
                    .emit(RevealerRowInput::UpdateActionIconName(
                        get_audio_in_icon(&device).to_string(),
                    ));
                self.revealer_row
                    .model()
                    .content
                    .emit(RevealerRowSliderInput::SetValue(
                        device.volume.get().average(),
                    ))
            }
            AudioInMenuWidgetInput::SetVolume(volume) => {
                if let Some(default) = audio_service().default_input.get() {
                    tokio::spawn(async move {
                        let _ = default.set_volume(Volume::stereo(volume, volume)).await;
                    });
                }
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
            AudioInMenuWidgetCommandOutput::DeviceChanged => {
                let default = audio_service().default_input.get();
                if let Some(audio_device) = default {
                    sender.input(AudioInMenuWidgetInput::UpdateDevice(audio_device.clone()));

                    let token = self.active_device_watcher_token.reset();

                    spawn_input_device_volume_mute_watcher(&audio_device, token, &sender, || {
                        AudioInMenuWidgetCommandOutput::VolumeChanged
                    });
                }
            }
            AudioInMenuWidgetCommandOutput::VolumeChanged => {
                if let Some(default) = audio_service().default_input.get() {
                    sender.input(AudioInMenuWidgetInput::UpdateDevice(default));
                }
            }
        }
    }
}

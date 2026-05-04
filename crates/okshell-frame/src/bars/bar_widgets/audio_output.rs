use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::{
    get_audio_out_icon, spawn_default_output_watcher, spawn_output_device_volume_mute_watcher,
};
use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use std::sync::Arc;
use wayle_audio::core::device::output::OutputDevice;

#[derive(Debug)]
pub(crate) struct AudioOutputModel {
    active_device_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum AudioOutputInput {
    UpdateDevice(Arc<OutputDevice>),
}

#[derive(Debug)]
pub(crate) enum AudioOutputOutput {}

pub(crate) struct AudioOutputInit {}

#[derive(Debug)]
pub(crate) enum AudioOutputCommandOutput {
    DeviceChanged,
    VolumeChanged,
}

#[relm4::component(pub)]
impl Component for AudioOutputModel {
    type CommandOutput = AudioOutputCommandOutput;
    type Input = AudioOutputInput;
    type Output = AudioOutputOutput;
    type Init = AudioOutputInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["audio-output-bar-widget", "ok-button-surface", "ok-bar-widget"],
            set_hexpand: false,
            set_vexpand: false,

            #[name="image"]
            gtk::Image {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_default_output_watcher(&sender, None, || AudioOutputCommandOutput::DeviceChanged);

        let model = AudioOutputModel {
            active_device_watcher_token: WatcherToken::new(),
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
            AudioOutputInput::UpdateDevice(device) => {
                widgets
                    .image
                    .set_icon_name(Some(get_audio_out_icon(&device)));
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
            AudioOutputCommandOutput::DeviceChanged => {
                let default_output = audio_service().default_output.get();
                if let Some(audio_device) = default_output {
                    sender.input(AudioOutputInput::UpdateDevice(audio_device.clone()));

                    let token = self.active_device_watcher_token.reset();

                    spawn_output_device_volume_mute_watcher(&audio_device, token, &sender, || {
                        AudioOutputCommandOutput::VolumeChanged
                    });
                }
            }
            AudioOutputCommandOutput::VolumeChanged => {
                if let Some(default_output) = audio_service().default_output.get() {
                    sender.input(AudioOutputInput::UpdateDevice(default_output));
                }
            }
        }
    }
}

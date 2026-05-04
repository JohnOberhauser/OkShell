use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::{
    get_audio_in_icon, spawn_default_input_watcher, spawn_input_device_volume_mute_watcher,
};
use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use std::sync::Arc;
use wayle_audio::core::device::input::InputDevice;

#[derive(Debug)]
pub(crate) struct AudioInputModel {
    active_device_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum AudioInputInput {
    UpdateDevice(Arc<InputDevice>),
}

#[derive(Debug)]
pub(crate) enum AudioInputOutput {}

pub(crate) struct AudioInputInit {}

#[derive(Debug)]
pub(crate) enum AudioInputCommandOutput {
    DeviceChanged,
    VolumeChanged,
}

#[relm4::component(pub)]
impl Component for AudioInputModel {
    type CommandOutput = AudioInputCommandOutput;
    type Input = AudioInputInput;
    type Output = AudioInputOutput;
    type Init = AudioInputInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["audio-input-bar-widget", "ok-button-surface", "ok-bar-widget"],
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
        spawn_default_input_watcher(&sender, None, || AudioInputCommandOutput::DeviceChanged);

        let model = AudioInputModel {
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
            AudioInputInput::UpdateDevice(device) => {
                widgets
                    .image
                    .set_icon_name(Some(get_audio_in_icon(&device)));
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
            AudioInputCommandOutput::DeviceChanged => {
                let device = audio_service().default_input.get();
                if let Some(audio_device) = device.as_ref() {
                    sender.input(AudioInputInput::UpdateDevice(audio_device.clone()));

                    let token = self.active_device_watcher_token.reset();

                    spawn_input_device_volume_mute_watcher(audio_device, token, &sender, || {
                        AudioInputCommandOutput::VolumeChanged
                    });
                }
            }
            AudioInputCommandOutput::VolumeChanged => {
                if let Some(default_input) = audio_service().default_input.get() {
                    sender.input(AudioInputInput::UpdateDevice(default_input));
                }
            }
        }
    }
}

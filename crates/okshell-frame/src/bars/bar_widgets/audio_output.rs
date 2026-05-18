use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::{
    decrease_volume, get_audio_out_icon, increase_volume, spawn_default_output_watcher,
    spawn_output_device_volume_mute_watcher, toggle_mute,
};
use okshell_utils::hover_scroll::attach_hover_scroll;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
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
    Clicked,
}

#[derive(Debug)]
pub(crate) enum AudioOutputOutput {}

pub(crate) struct AudioOutputInit {
    pub orientation: gtk::Orientation,
}

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
            add_css_class: "audio-output-bar-widget",
            set_hexpand: params.orientation == gtk::Orientation::Vertical,
            set_vexpand: params.orientation == gtk::Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(AudioOutputInput::Clicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_default_output_watcher(&sender, None, || AudioOutputCommandOutput::DeviceChanged);

        let model = AudioOutputModel {
            active_device_watcher_token: WatcherToken::new(),
        };

        let widgets = view_output!();

        let _handles = attach_hover_scroll(&root, move |_dx, dy, _hovered, _| {
            if dy < 0.0 {
                tokio::spawn(async move {
                    increase_volume().await;
                });
            } else if dy > 0.0 {
                tokio::spawn(async move {
                    decrease_volume().await;
                });
            }
        });

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
            AudioOutputInput::Clicked => {
                tokio::spawn(async move {
                    toggle_mute().await;
                });
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

use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::{
    decrease_input_volume, get_audio_in_icon, increase_input_volume, spawn_default_input_watcher,
    spawn_input_device_volume_mute_watcher, toggle_input_mute,
};
use okshell_utils::hover_scroll::attach_hover_scroll;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
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
    Clicked,
}

#[derive(Debug)]
pub(crate) enum AudioInputOutput {}

pub(crate) struct AudioInputInit {
    pub orientation: gtk::Orientation,
}

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
            add_css_class: "audio-input-bar-widget",
            set_hexpand: params.orientation == gtk::Orientation::Vertical,
            set_vexpand: params.orientation == gtk::Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(AudioInputInput::Clicked);
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
        spawn_default_input_watcher(&sender, None, || AudioInputCommandOutput::DeviceChanged);

        let model = AudioInputModel {
            active_device_watcher_token: WatcherToken::new(),
        };

        let widgets = view_output!();

        let _handles = attach_hover_scroll(&root, move |_dx, dy, _hovered, _| {
            if dy < 0.0 {
                tokio::spawn(async move {
                    increase_input_volume().await;
                });
            } else if dy > 0.0 {
                tokio::spawn(async move {
                    decrease_input_volume().await;
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
            AudioInputInput::UpdateDevice(device) => {
                widgets
                    .image
                    .set_icon_name(Some(get_audio_in_icon(&device)));
            }
            AudioInputInput::Clicked => {
                tokio::spawn(async move {
                    toggle_input_mute().await;
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

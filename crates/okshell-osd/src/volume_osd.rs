use std::sync::Arc;
use gtk4::{gdk};
use gtk4::prelude::{BoxExt, GtkWindowExt, OrientableExt, RangeExt, WidgetExt};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use tracing::info;
use wayle_audio::core::device::output::OutputDevice;
use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_utils::audio::{get_audio_out_icon, spawn_default_output_watcher, spawn_output_device_volume_mute_watcher};

#[derive(Debug)]
pub struct VolumeOsdModel {
    active_device_watcher_token: WatcherToken,
    hide_token: WatcherToken,
    icon_name: String,
    slider_value: f64,
    shown_count: u16,
}

#[derive(Debug)]
pub enum VolumeOsdInput {
    UpdateDevice(Arc<OutputDevice>),
    Show,
    Hide,
}

#[derive(Debug)]
pub enum VolumeOsdOutput {}

pub struct VolumeOsdInit {
    pub monitor: gdk::Monitor,
}

#[derive(Debug)]
pub enum VolumeOsdCommandOutput {
    DeviceChanged,
    VolumeChanged,
    Hide,
}

#[relm4::component(pub)]
impl Component for VolumeOsdModel {
    type CommandOutput = VolumeOsdCommandOutput;
    type Input = VolumeOsdInput;
    type Output = VolumeOsdOutput;
    type Init = VolumeOsdInit;

    view! {
        #[root]
        gtk::Window {
            set_css_classes: &["osd-window", "window-opacity"],
            set_decorated: false,
            set_visible: false,
            set_default_height: 1,
            set_margin_bottom: 200,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_width_request: 300,
                set_spacing: 20,

                gtk::Image {
                    add_css_class: "osd-icon",
                    #[watch]
                    set_icon_name: Some(model.icon_name.as_str()),
                },

                gtk::Scale {
                    add_css_class: "ok-progress-bar",
                    set_hexpand: true,
                    set_can_focus: false,
                    set_focus_on_click: false,
                    set_range: (0.0, 1.0),
                    #[watch]
                    set_value: model.slider_value,
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        root.init_layer_shell();
        root.set_monitor(Some(&params.monitor));
        root.set_namespace(Some("okshell-osd"));
        root.set_layer(Layer::Overlay);
        root.set_exclusive_zone(0);
        root.set_anchor(Edge::Bottom, true);

        spawn_default_output_watcher(
            &sender,
            None,
            || VolumeOsdCommandOutput::DeviceChanged,
        );

        let model = VolumeOsdModel {
            active_device_watcher_token: WatcherToken::new(),
            hide_token: WatcherToken::new(),
            icon_name: "audio-volume-medium-symbolic".to_string(),
            slider_value: 0.0,
            shown_count: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            VolumeOsdInput::UpdateDevice(device) => {
                self.icon_name = get_audio_out_icon(&device).to_string();
                self.slider_value = device.volume.get().average();

                let token = self.hide_token.reset();
                sender.command(|out, shutdown| {
                    shutdown.register(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        if !token.is_cancelled() {
                            out.send(VolumeOsdCommandOutput::Hide).ok();
                        }
                    }).drop_on_shutdown()
                });
            }
            VolumeOsdInput::Show => {
                if self.shown_count > 1 {
                    root.set_visible(true);
                } else {
                    self.shown_count = self.shown_count + 1;
                }
            }
            VolumeOsdInput::Hide => {
                root.set_visible(false);
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            VolumeOsdCommandOutput::DeviceChanged => {
                let default_output = audio_service().default_output.get();
                if let Some(audio_device) = default_output {
                    sender.input(VolumeOsdInput::UpdateDevice(audio_device.clone()));

                    let token = self.active_device_watcher_token.reset();

                    spawn_output_device_volume_mute_watcher(
                        &audio_device,
                        token,
                        &sender,
                        ||VolumeOsdCommandOutput::VolumeChanged
                    );
                }
            }
            VolumeOsdCommandOutput::VolumeChanged => {
                if let Some(default_output) = audio_service().default_output.get() {
                    sender.input(VolumeOsdInput::UpdateDevice(default_output));
                    sender.input(VolumeOsdInput::Show);
                }
            }
            VolumeOsdCommandOutput::Hide => {
                sender.input(VolumeOsdInput::Hide);
            }
        }
    }
}
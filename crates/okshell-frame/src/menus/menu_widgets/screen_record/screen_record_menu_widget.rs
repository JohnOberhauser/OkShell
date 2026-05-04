use crate::common_widgets::big_button::BigButton;
use crate::common_widgets::option_list::{OptionsListInput, OptionsListOutput};
use crate::common_widgets::revealer_row::revealer_row::{
    RevealerRowInit, RevealerRowInput, RevealerRowModel,
};
use crate::common_widgets::revealer_row::revealer_row_label::RevealerRowLabelInput::SetLabel;
use crate::common_widgets::revealer_row::revealer_row_label::{
    RevealerRowLabelInit, RevealerRowLabelModel,
};
use crate::menus::menu_widgets::screen_record::audio_option::{
    AudioOption, AudioOptionsList, get_audio_option_icon_name, get_audio_option_label,
};
use crate::menus::menu_widgets::screen_record::recording_service::{
    RecordingState, RecordingStateStoreFields, recording_state,
};
use okshell_common::scoped_effects::EffectScope;
use okshell_common::watch;
use okshell_screenshot::record::{RecordHandle, RecordResult};
use okshell_screenshot::{CaptureArea, ScreenRecordRequest};
use okshell_services::audio_service;
use okshell_utils::notifications::show_file_saved_notification;
use reactive_graph::traits::{Get, GetUntracked};
use reactive_stores::Patch;
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use std::time::Duration;
use wayle_audio::core::device::input::InputDevice;

pub(crate) struct ScreenRecordMenuWidgetModel {
    audio_row: Controller<RevealerRowModel<RevealerRowLabelModel, AudioOptionsList>>,
    selected_audio_option: AudioOption,
    recording_handle: Option<RecordHandle>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum ScreenRecordMenuWidgetInput {
    AudioOptionSelected(AudioOption),
    AllClicked,
    MonitorClicked,
    WindowClicked,
    AreaClicked,
    StopClicked,
    RecordingHandleChanged(Option<RecordHandle>),
    RecordingStopped(RecordResult),
}

#[derive(Debug)]
pub(crate) enum ScreenRecordMenuWidgetOutput {
    CloseMenu,
}

pub(crate) struct ScreenRecordMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum ScreenRecordMenuWidgetCommandOutput {
    AudioDevicesChanges(Vec<Arc<InputDevice>>),
}

#[relm4::component(pub)]
impl Component for ScreenRecordMenuWidgetModel {
    type CommandOutput = ScreenRecordMenuWidgetCommandOutput;
    type Input = ScreenRecordMenuWidgetInput;
    type Output = ScreenRecordMenuWidgetOutput;
    type Init = ScreenRecordMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "screen-recording-menu-widget",
            set_hexpand: false,
            set_orientation: gtk::Orientation::Vertical,

            gtk::Label {
                add_css_class: "label-large-bold",
                set_label: "Screen Record",
                set_margin_bottom: 8,
            },

            model.audio_row.widget().clone() {},

            gtk::Box {
                set_margin_top: 8,
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_spacing: 16,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 32,

                    #[template]
                    BigButton {
                        #[watch]
                        set_sensitive: !model.recording_handle.is_some(),
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenRecordMenuWidgetInput::AllClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-all-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "All",
                        },
                    },

                    #[template]
                    BigButton {
                        #[watch]
                        set_sensitive: !model.recording_handle.is_some(),
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenRecordMenuWidgetInput::MonitorClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-monitor-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "Monitor",
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 32,

                    #[template]
                    BigButton {
                        #[watch]
                        set_sensitive: !model.recording_handle.is_some(),
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenRecordMenuWidgetInput::WindowClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-window-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "Window",
                        },
                    },

                    #[template]
                    BigButton {
                        #[watch]
                        set_sensitive: !model.recording_handle.is_some(),
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenRecordMenuWidgetInput::AreaClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-area-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "Area",
                        },
                    },
                },
            },

            gtk::Box {
                add_css_class: "screen-recording-menu-indicator-box",
                set_orientation: gtk::Orientation::Vertical,
                set_margin_top: 24,
                #[watch]
                set_visible: model.recording_handle.is_some(),

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Image {
                        add_css_class: "screen-recording-menu-indicator-image",
                        set_icon_name: Some("record-symbolic"),
                    },

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_margin_start: 12,
                        set_label: "Recording screen…",
                    },
                },

                #[name = "stop_button"]
                gtk::Button {
                    add_css_class: "ok-button-primary",
                    set_margin_top: 12,
                    set_valign: gtk::Align::Center,
                    connect_clicked[sender] => move |_| {
                        sender.input(ScreenRecordMenuWidgetInput::StopClicked);
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        set_label: "Stop",
                    },
                }
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        Self::spawn_default_output_watcher(&sender);

        let content = RevealerRowLabelModel::builder()
            .launch(RevealerRowLabelInit {
                label: "No Audio".to_string(),
            })
            .detach();

        let revealed_content = AudioOptionsList::builder()
            .launch(vec![AudioOption { value: None }])
            .forward(sender.input_sender(), |msg| match msg {
                OptionsListOutput::Selected(opt) => {
                    ScreenRecordMenuWidgetInput::AudioOptionSelected(opt)
                }
            });

        let audio_row = RevealerRowModel::<RevealerRowLabelModel, AudioOptionsList>::builder()
            .launch(RevealerRowInit {
                icon_name: "audio-volume-muted-symbolic".into(),
                action_button_sensitive: false,
                content,
                revealed_content,
            })
            .detach();

        let mut effects = EffectScope::new();

        let recording = recording_state();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let recording_state = recording.clone();
            let handle = recording_state.handle().get();
            sender_clone.input(ScreenRecordMenuWidgetInput::RecordingHandleChanged(handle));
        });

        let recording_handle = recording.clone().handle().get_untracked();

        let model = ScreenRecordMenuWidgetModel {
            audio_row,
            selected_audio_option: AudioOption { value: None },
            recording_handle,
            _effects: effects,
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
            ScreenRecordMenuWidgetInput::AudioOptionSelected(opt) => {
                self.selected_audio_option = opt.clone();
                self.audio_row
                    .model()
                    .content
                    .emit(SetLabel(get_audio_option_label(&opt)));
                self.audio_row.emit(RevealerRowInput::UpdateActionIconName(
                    get_audio_option_icon_name(&opt),
                ));
                self.audio_row.emit(RevealerRowInput::SetRevealed(false));
            }
            ScreenRecordMenuWidgetInput::AllClicked => self.start_record(CaptureArea::All, sender),
            ScreenRecordMenuWidgetInput::WindowClicked => {
                self.start_record(CaptureArea::SelectWindow, sender)
            }
            ScreenRecordMenuWidgetInput::MonitorClicked => {
                self.start_record(CaptureArea::SelectMonitor, sender)
            }
            ScreenRecordMenuWidgetInput::AreaClicked => {
                self.start_record(CaptureArea::SelectRegion, sender)
            }
            ScreenRecordMenuWidgetInput::StopClicked => {
                if let Some(handle) = &self.recording_handle {
                    handle.stop();
                }
            }
            ScreenRecordMenuWidgetInput::RecordingHandleChanged(handle) => {
                self.recording_handle = handle;
            }
            ScreenRecordMenuWidgetInput::RecordingStopped(result) => {
                recording_state().patch(RecordingState { handle: None });
                if let Some(path) = result.saved_path {
                    show_file_saved_notification("Screenshot saved & copied".to_string(), path);
                }
            }
        }

        self.update_view(widgets, sender_clone);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ScreenRecordMenuWidgetCommandOutput::AudioDevicesChanges(devices) => {
                let mut devices: Vec<AudioOption> = devices
                    .iter()
                    .map(|d| AudioOption {
                        value: Some(d.clone()),
                    })
                    .collect();

                devices.insert(0, AudioOption { value: None });

                self.audio_row
                    .model()
                    .revealed_content
                    .emit(OptionsListInput::SetOptions(devices))
            }
        }
    }
}

impl ScreenRecordMenuWidgetModel {
    fn spawn_default_output_watcher(sender: &ComponentSender<Self>) {
        let audio = audio_service();
        let input_devices = audio.input_devices.clone();

        watch!(sender, [input_devices.watch()], |out| {
            let _ = out.send(ScreenRecordMenuWidgetCommandOutput::AudioDevicesChanges(
                input_devices.get(),
            ));
        });
    }

    fn start_record(&self, area: CaptureArea, sender: ComponentSender<Self>) {
        if self.recording_handle.is_some() {
            return;
        }
        let _ = sender.output(ScreenRecordMenuWidgetOutput::CloseMenu);
        let audio: Option<String> = self
            .selected_audio_option
            .value
            .as_ref()
            .and_then(|v| Some(v.name.get()));
        let sender_clone = sender.clone();
        okshell_screenshot::record_screen(
            ScreenRecordRequest { area, audio },
            Duration::ZERO,
            move |handle_result| match handle_result {
                Ok(handle) => {
                    recording_state().patch(RecordingState {
                        handle: Some(handle),
                    });
                }
                Err(e) => eprintln!("Failed to start recording: {e}"),
            },
            move |done_result| match done_result {
                Ok(result) => {
                    sender_clone.input(ScreenRecordMenuWidgetInput::RecordingStopped(result))
                }
                Err(e) => eprintln!("Recording failed: {e}"),
            },
        );
    }
}

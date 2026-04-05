use std::sync::Arc;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::RevealerTransitionType;
use wayle_audio::core::device::output::OutputDevice;
use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::{GenericWidgetController, GenericWidgetControllerExtSafe};
use crate::menus::menu_widgets::audio_out::output_device_revealer_button::{OutputDeviceRevealerButtonInit, OutputDeviceRevealerButtonInput, OutputDeviceRevealerButtonModel};
use okshell_utils::audio::spawn_output_devices_watcher;

pub(crate) struct AudioOutRevealedContentModel {
    devices_dynamic_box_controller: Controller<DynamicBoxModel<Arc<OutputDevice>, String>>,
    watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum AudioOutRevealedContentInput {
    UpdateDevices,
    Revealed,
    Hidden,
}

#[derive(Debug)]
pub(crate) enum AudioOutRevealedContentOutput {}

pub(crate) struct AudioOutRevealedContentInit {}

#[derive(Debug)]
pub(crate) enum AudioOutRevealedContentCommandOutput {
    DevicesUpdated,
}

#[relm4::component(pub)]
impl Component for AudioOutRevealedContentModel {
    type CommandOutput = AudioOutRevealedContentCommandOutput;
    type Input = AudioOutRevealedContentInput;
    type Output = AudioOutRevealedContentOutput;
    type Init = AudioOutRevealedContentInit;

    view! {
        #[root]
        gtk::Box {
            model.devices_dynamic_box_controller.widget().clone() {},
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut watcher_token = WatcherToken::new();

        let token = watcher_token.reset();

        spawn_output_devices_watcher(
            &sender,
            token,
            ||AudioOutRevealedContentCommandOutput::DevicesUpdated,
        );

        let devices_dynamic_box_factory = DynamicBoxFactory::<Arc<OutputDevice>, String> {
            id: Box::new(|item| item.name.get()),
            create: Box::new(move |item| {
                let device = item.clone();
                let revealer_button = OutputDeviceRevealerButtonModel::builder()
                    .launch(OutputDeviceRevealerButtonInit {
                        output_device: device,
                    })
                    .detach();

                Box::new(revealer_button) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let devices_dynamic_box_controller: Controller<DynamicBoxModel<Arc<OutputDevice>, String>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory: devices_dynamic_box_factory,
                    orientation: gtk::Orientation::Vertical,
                    spacing: 0,
                    transition_type: RevealerTransitionType::SlideDown,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                })
                .detach();

        let model = AudioOutRevealedContentModel {
            devices_dynamic_box_controller,
            watcher_token: WatcherToken::new(),
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
            AudioOutRevealedContentInput::UpdateDevices => {
                let audio = audio_service();
                let devices = audio.output_devices.get();
                self.devices_dynamic_box_controller.emit(DynamicBoxInput::SetItems(devices))
            }
            AudioOutRevealedContentInput::Revealed => {
                let token = self.watcher_token.reset();

                spawn_output_devices_watcher(
                    &sender,
                    token,
                    ||AudioOutRevealedContentCommandOutput::DevicesUpdated,
                );

                self.devices_dynamic_box_controller.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<OutputDeviceRevealerButtonModel>>()
                    {
                        ctrl.emit(OutputDeviceRevealerButtonInput::Revealed);
                    }
                });
            }
            AudioOutRevealedContentInput::Hidden => {
                self.watcher_token.reset();

                self.devices_dynamic_box_controller.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<OutputDeviceRevealerButtonModel>>()
                    {
                        ctrl.emit(OutputDeviceRevealerButtonInput::Hidden);
                    }
                });
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
            AudioOutRevealedContentCommandOutput::DevicesUpdated => {
                sender.input(AudioOutRevealedContentInput::UpdateDevices);
            }
        }
    }
}
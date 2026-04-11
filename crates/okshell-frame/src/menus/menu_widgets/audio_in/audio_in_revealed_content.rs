use std::sync::Arc;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::RevealerTransitionType;
use wayle_audio::core::device::input::InputDevice;
use okshell_common::WatcherToken;
use okshell_services::audio_service;
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::{GenericWidgetController, GenericWidgetControllerExtSafe};
use crate::menus::menu_widgets::audio_in::input_device_revealer_button::{InputDeviceRevealerButtonInit, InputDeviceRevealerButtonInput, InputDeviceRevealerButtonModel};
use okshell_utils::audio::{spawn_input_devices_watcher};

pub(crate) struct AudioInRevealedContentModel {
    devices_dynamic_box_controller: Controller<DynamicBoxModel<Arc<InputDevice>, String>>,
    watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum AudioInRevealedContentInput {
    UpdateDevices,
    Revealed,
    Hidden,
}

#[derive(Debug)]
pub(crate) enum AudioInRevealedContentOutput {}

pub(crate) struct AudioInRevealedContentInit {}

#[derive(Debug)]
pub(crate) enum AudioInRevealedContentCommandOutput {
    DevicesUpdated,
}

#[relm4::component(pub)]
impl Component for AudioInRevealedContentModel {
    type CommandOutput = AudioInRevealedContentCommandOutput;
    type Input = AudioInRevealedContentInput;
    type Output = AudioInRevealedContentOutput;
    type Init = AudioInRevealedContentInit;

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

        spawn_input_devices_watcher(
            &sender,
            token,
            || AudioInRevealedContentCommandOutput::DevicesUpdated,
        );

        let devices_dynamic_box_factory = DynamicBoxFactory::<Arc<InputDevice>, String> {
            id: Box::new(|item| item.name.get()),
            create: Box::new(move |item| {
                let device = item.clone();
                let revealer_button = InputDeviceRevealerButtonModel::builder()
                    .launch(InputDeviceRevealerButtonInit {
                        input_device: device,
                    })
                    .detach();

                Box::new(revealer_button) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let devices_dynamic_box_controller: Controller<DynamicBoxModel<Arc<InputDevice>, String>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory: devices_dynamic_box_factory,
                    orientation: gtk::Orientation::Vertical,
                    spacing: 0,
                    transition_type: RevealerTransitionType::SlideDown,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                    allow_drag_and_drop: false,
                })
                .detach();

        let model = AudioInRevealedContentModel {
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
            AudioInRevealedContentInput::UpdateDevices => {
                let audio = audio_service();
                let devices = audio.input_devices.get();
                self.devices_dynamic_box_controller.emit(DynamicBoxInput::SetItems(devices))
            }
            AudioInRevealedContentInput::Revealed => {
                let token = self.watcher_token.reset();

                spawn_input_devices_watcher(
                    &sender,
                    token,
                    || AudioInRevealedContentCommandOutput::DevicesUpdated,
                );

                self.devices_dynamic_box_controller.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<InputDeviceRevealerButtonModel>>()
                    {
                        ctrl.emit(InputDeviceRevealerButtonInput::Revealed);
                    }
                });
            }
            AudioInRevealedContentInput::Hidden => {
                self.watcher_token.reset();

                self.devices_dynamic_box_controller.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<InputDeviceRevealerButtonModel>>()
                    {
                        ctrl.emit(InputDeviceRevealerButtonInput::Hidden);
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
            AudioInRevealedContentCommandOutput::DevicesUpdated => {
                sender.input(AudioInRevealedContentInput::UpdateDevices);
            }
        }
    }
}
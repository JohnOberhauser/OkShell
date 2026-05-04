use crate::menus::menu_widgets::bluetooth::device_revealed_content::DeviceRevealedContentInput;
use crate::menus::menu_widgets::bluetooth::device_revealer_button::{
    DeviceRevealerButtonInit, DeviceRevealerButtonInput, DeviceRevealerButtonModel,
};
use okshell_common::dynamic_box::dynamic_box::{
    DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel,
};
use okshell_common::dynamic_box::generic_widget_controller::{
    GenericWidgetController, GenericWidgetControllerExtSafe,
};
use okshell_services::bluetooth_service;
use okshell_utils::bluetooth::spawn_bluetooth_devices_watcher;
use relm4::gtk::prelude::*;
use relm4::gtk::{Justification, RevealerTransitionType};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use wayle_bluetooth::core::device::Device;

pub(crate) struct BluetoothRevealedContentModel {
    paired_devices_dynamic_box_controller: Controller<DynamicBoxModel<Arc<Device>, String>>,
    unpaired_devices_dynamic_box_controller: Controller<DynamicBoxModel<Arc<Device>, String>>,
    unpaired_devices_count: i16,
    paired_devices_count: i16,
}

#[derive(Debug)]
pub(crate) enum BluetoothRevealedContentInput {
    UpdateDevices,
    Revealed,
    Hidden,
}

#[derive(Debug)]
pub(crate) enum BluetoothRevealedContentOutput {}

pub(crate) struct BluetoothRevealedContentInit {}

#[derive(Debug)]
pub(crate) enum BluetoothRevealedContentCommandOutput {
    DevicesUpdate,
}

#[relm4::component(pub)]
impl Component for BluetoothRevealedContentModel {
    type CommandOutput = BluetoothRevealedContentCommandOutput;
    type Input = BluetoothRevealedContentInput;
    type Output = BluetoothRevealedContentOutput;
    type Init = BluetoothRevealedContentInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            #[name = "paired_devices_container"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Label {
                    add_css_class: "label-large-bold-variant",
                    set_label: "Paired Devices",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                },

                gtk::Label {
                    add_css_class: "label-medium",
                    set_label: "No Paired Devices",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                    #[watch]
                    set_visible: model.paired_devices_count == 0,
                },

                model.paired_devices_dynamic_box_controller.widget().clone() {},
            },

            #[name = "available_devices_container"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Label {
                    add_css_class: "label-large-bold-variant",
                    set_label: "Discovered Devices",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                },

                gtk::Label {
                    add_css_class: "label-medium",
                    set_label: "No Devices Found",
                    set_hexpand: true,
                    set_justify: Justification::Center,
                    #[watch]
                    set_visible: model.unpaired_devices_count == 0,
                },

                model.unpaired_devices_dynamic_box_controller.widget().clone() {},
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_bluetooth_devices_watcher(&sender, || {
            BluetoothRevealedContentCommandOutput::DevicesUpdate
        });

        let devices_dynamic_box_factory = Self::create_device_factory();

        let paired_devices_dynamic_box_controller: Controller<
            DynamicBoxModel<Arc<Device>, String>,
        > = DynamicBoxModel::builder()
            .launch(DynamicBoxInit {
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

        let devices_dynamic_box_factory = Self::create_device_factory();

        let unpaired_devices_dynamic_box_controller: Controller<
            DynamicBoxModel<Arc<Device>, String>,
        > = DynamicBoxModel::builder()
            .launch(DynamicBoxInit {
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

        let model = BluetoothRevealedContentModel {
            paired_devices_dynamic_box_controller,
            unpaired_devices_dynamic_box_controller,
            unpaired_devices_count: 0,
            paired_devices_count: 0,
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
            BluetoothRevealedContentInput::UpdateDevices => {
                let bluetooth = bluetooth_service();
                let devices = bluetooth.devices.get();
                let paired_devices: Vec<Arc<Device>> = devices
                    .clone()
                    .into_iter()
                    .filter(|device| device.paired.get())
                    .collect();

                let unpaired_devices: Vec<Arc<Device>> = devices
                    .into_iter()
                    .filter(|device| !device.paired.get())
                    .collect();

                self.paired_devices_count = paired_devices.len() as i16;
                self.paired_devices_dynamic_box_controller
                    .emit(DynamicBoxInput::SetItems(paired_devices));

                self.unpaired_devices_count = unpaired_devices.len() as i16;
                self.unpaired_devices_dynamic_box_controller
                    .emit(DynamicBoxInput::SetItems(unpaired_devices));
            }
            BluetoothRevealedContentInput::Revealed => {
                self.paired_devices_dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry
                            .controller
                            .as_ref()
                            .downcast_ref::<Controller<DeviceRevealerButtonModel>>()
                        {
                            ctrl.model()
                                .revealer_button_controller
                                .model()
                                .revealed_content
                                .emit(DeviceRevealedContentInput::ParentRevealed(true));
                            ctrl.emit(DeviceRevealerButtonInput::ParentRevealed(true));
                        }
                    });
                self.unpaired_devices_dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry
                            .controller
                            .as_ref()
                            .downcast_ref::<Controller<DeviceRevealerButtonModel>>()
                        {
                            ctrl.model()
                                .revealer_button_controller
                                .model()
                                .revealed_content
                                .emit(DeviceRevealedContentInput::ParentRevealed(true));
                            ctrl.emit(DeviceRevealerButtonInput::ParentRevealed(true));
                        }
                    });
            }
            BluetoothRevealedContentInput::Hidden => {
                self.paired_devices_dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry
                            .controller
                            .as_ref()
                            .downcast_ref::<Controller<DeviceRevealerButtonModel>>()
                        {
                            ctrl.model()
                                .revealer_button_controller
                                .model()
                                .revealed_content
                                .emit(DeviceRevealedContentInput::ParentRevealed(false));
                            ctrl.emit(DeviceRevealerButtonInput::ParentRevealed(false));
                        }
                    });
                self.unpaired_devices_dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry
                            .controller
                            .as_ref()
                            .downcast_ref::<Controller<DeviceRevealerButtonModel>>()
                        {
                            ctrl.model()
                                .revealer_button_controller
                                .model()
                                .revealed_content
                                .emit(DeviceRevealedContentInput::ParentRevealed(false));
                            ctrl.emit(DeviceRevealerButtonInput::ParentRevealed(false));
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
            BluetoothRevealedContentCommandOutput::DevicesUpdate => {
                sender.input(BluetoothRevealedContentInput::UpdateDevices);
            }
        }
    }
}

impl BluetoothRevealedContentModel {
    fn create_device_factory() -> DynamicBoxFactory<Arc<Device>, String> {
        DynamicBoxFactory::<Arc<Device>, String> {
            id: Box::new(|item| item.address.get()),
            create: Box::new(move |item| {
                let device = item.clone();
                let revealer_button = DeviceRevealerButtonModel::builder()
                    .launch(DeviceRevealerButtonInit { device })
                    .detach();

                Box::new(revealer_button) as Box<dyn GenericWidgetController>
            }),
            update: None,
        }
    }
}

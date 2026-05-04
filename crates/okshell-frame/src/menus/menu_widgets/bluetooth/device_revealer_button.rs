use crate::common_widgets::revealer_button::revealer_button::{
    RevealerButtonInit, RevealerButtonInput, RevealerButtonModel,
};
use crate::common_widgets::revealer_button::revealer_button_icon_label::{
    RevealerButtonIconLabelInit, RevealerButtonIconLabelInput, RevealerButtonIconLabelModel,
};
use crate::menus::menu_widgets::bluetooth::device_revealed_content::{
    DeviceRevealedContentInit, DeviceRevealedContentModel,
};
use okshell_common::WatcherToken;
use okshell_utils::battery::get_battery_icon;
use okshell_utils::bluetooth::{
    get_bluetooth_device_icon, spawn_bluetooth_device_battery_watcher,
    spawn_bluetooth_device_watcher,
};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::sync::Arc;
use wayle_bluetooth::core::device::Device;

pub(crate) struct DeviceRevealerButtonModel {
    device: Arc<Device>,
    pub revealer_button_controller:
        Controller<RevealerButtonModel<RevealerButtonIconLabelModel, DeviceRevealedContentModel>>,
    battery_watcher_token: WatcherToken,
    device_watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum DeviceRevealerButtonInput {
    ParentRevealed(bool),
    BatteryUpdated,
    DeviceUpdated,
}

#[derive(Debug)]
pub(crate) enum DeviceRevealerButtonOutput {}

pub(crate) struct DeviceRevealerButtonInit {
    pub device: Arc<Device>,
}

#[derive(Debug)]
pub(crate) enum DeviceRevealerButtonCommandOutput {
    BatteryUpdated,
    DeviceUpdated,
}

#[relm4::component(pub)]
impl Component for DeviceRevealerButtonModel {
    type CommandOutput = DeviceRevealerButtonCommandOutput;
    type Input = DeviceRevealerButtonInput;
    type Output = DeviceRevealerButtonOutput;
    type Init = DeviceRevealerButtonInit;

    view! {
        #[root]
        gtk::Box {
            model.revealer_button_controller.widget().clone() {},
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let device = params.device;

        let device_clone = device.clone();
        let device_content = RevealerButtonIconLabelModel::builder()
            .launch(RevealerButtonIconLabelInit {
                label: device_clone.alias.get().to_string(),
                icon_name: get_bluetooth_device_icon(device_clone),
                secondary_icon_name: "".to_string(),
            })
            .detach();

        let device_clone = device.clone();
        let device_revealed_content = DeviceRevealedContentModel::builder()
            .launch(DeviceRevealedContentInit {
                device: device_clone,
            })
            .detach();

        let revealer_button = RevealerButtonModel::builder()
            .launch(RevealerButtonInit {
                content: device_content,
                revealed_content: device_revealed_content,
            })
            .detach();

        let mut battery_watcher_token = WatcherToken::new();

        let token = battery_watcher_token.reset();

        spawn_bluetooth_device_battery_watcher(&device, token, &sender, || {
            DeviceRevealerButtonCommandOutput::BatteryUpdated
        });

        let mut device_watcher_token = WatcherToken::new();

        let token = device_watcher_token.reset();

        spawn_bluetooth_device_watcher(&device, token, &sender, || {
            DeviceRevealerButtonCommandOutput::DeviceUpdated
        });

        let model = DeviceRevealerButtonModel {
            device,
            revealer_button_controller: revealer_button,
            battery_watcher_token,
            device_watcher_token,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            DeviceRevealerButtonInput::ParentRevealed(revealed) => {
                let battery_token = self.battery_watcher_token.reset();
                let device_token = self.device_watcher_token.reset();

                if revealed {
                    spawn_bluetooth_device_battery_watcher(
                        &self.device,
                        battery_token,
                        &sender,
                        || DeviceRevealerButtonCommandOutput::BatteryUpdated,
                    );

                    spawn_bluetooth_device_watcher(&self.device, device_token, &sender, || {
                        DeviceRevealerButtonCommandOutput::DeviceUpdated
                    });
                } else {
                    self.revealer_button_controller
                        .emit(RevealerButtonInput::SetRevealed(false))
                }
            }
            DeviceRevealerButtonInput::BatteryUpdated => {
                if let Some(battery_percent) = self.device.battery_percentage.get() {
                    self.revealer_button_controller.model().content.emit(
                        RevealerButtonIconLabelInput::SetSecondaryIconName(
                            get_battery_icon(battery_percent as f64).to_string(),
                        ),
                    )
                }
            }
            DeviceRevealerButtonInput::DeviceUpdated => {
                if !self.device.connected.get() {
                    self.revealer_button_controller.model().content.emit(
                        RevealerButtonIconLabelInput::SetSecondaryIconName("".to_string()),
                    )
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
            DeviceRevealerButtonCommandOutput::BatteryUpdated => {
                sender.input(DeviceRevealerButtonInput::BatteryUpdated);
            }
            DeviceRevealerButtonCommandOutput::DeviceUpdated => {
                sender.input(DeviceRevealerButtonInput::DeviceUpdated);
            }
        }
    }
}

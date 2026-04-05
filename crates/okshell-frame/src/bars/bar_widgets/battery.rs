use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::prelude::WidgetExt;
use wayle_battery::types::DeviceState;
use okshell_services::battery_service;
use okshell_utils::battery::{get_battery_icon, get_charging_battery_icon, spawn_battery_watcher};

#[derive(Debug, Clone)]
pub(crate) struct BatteryModel {}

#[derive(Debug)]
pub(crate) enum BatteryInput {}

#[derive(Debug)]
pub(crate) enum BatteryOutput {}

pub(crate) struct BatteryInit {}

#[derive(Debug)]
pub(crate) enum BatteryCommandOutput {
    BatteryStateChanged,
}

#[relm4::component(pub)]
impl Component for BatteryModel {
    type CommandOutput = BatteryCommandOutput;
    type Input = BatteryInput;
    type Output = BatteryOutput;
    type Init = BatteryInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            set_css_classes: &["battery-bar-widget", "ok-button-surface", "ok-bar-widget"],
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
        spawn_battery_watcher(
            &sender,
            ||BatteryCommandOutput::BatteryStateChanged,
        );

        let model = BatteryModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            BatteryCommandOutput::BatteryStateChanged => {
                let battery = battery_service().device.clone();

                let exists = battery.is_present.get();

                widgets.root.set_visible(exists);

                if !exists {
                    return;
                }

                let percent = battery.percentage.get();
                let state = battery.state.get();

                if state == DeviceState::Charging || state == DeviceState::FullyCharged {
                    widgets.image.set_icon_name(Some(get_charging_battery_icon(percent)));
                } else {
                    widgets.image.set_icon_name(Some(get_battery_icon(percent)));
                }
            }
        }
    }
}

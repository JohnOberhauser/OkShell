use okshell_utils::bluetooth::{set_bluetooth_icon, spawn_bluetooth_enabled_watcher};
use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct BluetoothModel {}

#[derive(Debug)]
pub(crate) enum BluetoothInput {}

#[derive(Debug)]
pub(crate) enum BluetoothOutput {}

pub(crate) struct BluetoothInit {}

#[derive(Debug)]
pub(crate) enum BluetoothCommandOutput {
    StatusChanged,
}

#[relm4::component(pub)]
impl Component for BluetoothModel {
    type CommandOutput = BluetoothCommandOutput;
    type Input = BluetoothInput;
    type Output = BluetoothOutput;
    type Init = BluetoothInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["bluetooth-bar-widget", "ok-button-surface", "ok-bar-widget"],
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
        spawn_bluetooth_enabled_watcher(&sender, || BluetoothCommandOutput::StatusChanged);

        let model = BluetoothModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            BluetoothCommandOutput::StatusChanged => {
                set_bluetooth_icon(&widgets.image);
            }
        }
    }
}

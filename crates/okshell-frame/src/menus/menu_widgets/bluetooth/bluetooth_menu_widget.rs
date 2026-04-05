use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::prelude::WidgetExt;
use okshell_services::bluetooth_service;
use crate::common_widgets::revealer_row::revealer_row::{RevealerRowInit, RevealerRowInput, RevealerRowModel, RevealerRowOutput};
use crate::common_widgets::revealer_row::revealer_row_label::{RevealerRowLabelInit, RevealerRowLabelModel};
use crate::menus::menu_widgets::bluetooth::bluetooth_revealed_content::{BluetoothRevealedContentInit, BluetoothRevealedContentInput, BluetoothRevealedContentModel};
use okshell_utils::bluetooth::{set_bluetooth_icon, set_bluetooth_label, spawn_bluetooth_enabled_watcher};

pub(crate) struct BluetoothMenuWidgetModel {
    revealer_row: Controller<RevealerRowModel<RevealerRowLabelModel, BluetoothRevealedContentModel>>,
}

#[derive(Debug)]
pub(crate) enum BluetoothMenuWidgetInput {
    RevealerRowRevealed,
    RevealerRowHidden,
    ActionButtonClicked,
    ParentRevealChanged(bool),
}

#[derive(Debug)]
pub(crate) enum BluetoothMenuWidgetOutput {}

pub(crate) struct BluetoothMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum BluetoothMenuWidgetCommandOutput {
    BluetoothStateChanged,
}

#[relm4::component(pub)]
impl Component for BluetoothMenuWidgetModel {
    type CommandOutput = BluetoothMenuWidgetCommandOutput;
    type Input = BluetoothMenuWidgetInput;
    type Output = BluetoothMenuWidgetOutput;
    type Init = BluetoothMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "audio-out-menu-widget",
            
            model.revealer_row.widget().clone() {}
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        
        spawn_bluetooth_enabled_watcher(
            &sender,
            ||BluetoothMenuWidgetCommandOutput::BluetoothStateChanged,
        );

        let row_content = RevealerRowLabelModel::builder()
            .launch(RevealerRowLabelInit {
                label: "Bluetooth Disabled".to_string(),
            })
            .detach();

        let bluetooth_revealed_content = BluetoothRevealedContentModel::builder()
            .launch(BluetoothRevealedContentInit{})
            .detach();

        let revealer_row = RevealerRowModel::<RevealerRowLabelModel, BluetoothRevealedContentModel>::builder()
            .launch(RevealerRowInit {
                icon_name: "bluetooth-hardware-disabled-symbolic".into(),
                action_button_sensitive: false,
                content: row_content,
                revealed_content: bluetooth_revealed_content,
            })
            .forward(sender.input_sender(), |msg| {
                match msg {
                    RevealerRowOutput::ActionButtonClicked => {
                        BluetoothMenuWidgetInput::ActionButtonClicked
                    }
                    RevealerRowOutput::Revealed => {
                        BluetoothMenuWidgetInput::RevealerRowRevealed
                    }
                    RevealerRowOutput::Hidden => {
                        BluetoothMenuWidgetInput::RevealerRowHidden
                    }
                }
            });
        
        let model = BluetoothMenuWidgetModel {
            revealer_row,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            BluetoothMenuWidgetInput::RevealerRowRevealed => {
                let bluetooth = bluetooth_service();
                tokio::spawn(async move {
                    let _ = bluetooth.start_discovery().await;
                });
                self.revealer_row.model().revealed_content.emit(BluetoothRevealedContentInput::Revealed);
            }
            BluetoothMenuWidgetInput::RevealerRowHidden => {
                let bluetooth = bluetooth_service();
                tokio::spawn(async move {
                    let _ = bluetooth.stop_discovery().await;
                });
                self.revealer_row.model().revealed_content.emit(BluetoothRevealedContentInput::Hidden);
            }
            BluetoothMenuWidgetInput::ParentRevealChanged(revealed) => {
                if !revealed {
                    self.revealer_row.emit(RevealerRowInput::SetRevealed(false));
                }
            }
            BluetoothMenuWidgetInput::ActionButtonClicked => {
                
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            BluetoothMenuWidgetCommandOutput::BluetoothStateChanged => {
                set_bluetooth_icon(&self.revealer_row.widgets().action_icon_image);
                set_bluetooth_label(&self.revealer_row.model().content.widgets().label);
            }
        }
    }
}
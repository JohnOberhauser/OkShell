use relm4::{gtk, Component, ComponentParts, ComponentSender, Controller, ComponentController};
use relm4::gtk::prelude::*;
use relm4::gtk::RevealerTransitionType;
use relm4::gtk::glib;
use tokio::sync::broadcast;
use tracing::{error, warn};
use okshell_clipboard::{clipboard_service, ClipboardEntry, ClipboardHistory};
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;
use crate::menus::menu_widgets::clipboard::clipboard_item::{ClipboardItemModel};

pub(crate) struct ClipboardModel {
    dynamic_box: Controller<DynamicBoxModel<ClipboardEntry, u64>>,
    history: ClipboardHistory,
    delete_button_visible: bool,
}

#[derive(Debug)]
pub(crate) enum ClipboardInput {
    Refresh,
    DeleteAllClicked,
}

#[derive(Debug)]
pub(crate) enum ClipboardOutput {
    CloseMenu,
}

pub(crate) struct ClipboardInit {}

#[derive(Debug)]
pub(crate) enum ClipboardCommandOutput {}

#[relm4::component(pub)]
impl Component for ClipboardModel {
    type CommandOutput = ClipboardCommandOutput;
    type Input = ClipboardInput;
    type Output = ClipboardOutput;
    type Init = ClipboardInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "clipboard-menu-widget",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "label-medium-bold",
                    set_halign: gtk::Align::Start,
                    set_label: "Clipboard History",
                    set_hexpand: true,
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_valign: gtk::Align::Center,
                    connect_clicked[sender] => move |_| {
                        sender.input(ClipboardInput::DeleteAllClicked);
                    },

                    gtk::Label {
                        add_css_class: "label-small",
                        set_label: "Clear all",
                    },
                },
            },

            gtk::Label {
                add_css_class: "label-medium",
                #[watch]
                set_visible: !model.delete_button_visible,
                set_label: "Empty",
            },

            gtk::ScrolledWindow {
                set_vscrollbar_policy: gtk::PolicyType::Automatic,
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_propagate_natural_height: true,
                set_propagate_natural_width: false,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    model.dynamic_box.widget().clone() {}
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let service = clipboard_service();
        let history = service.history().clone();
        let mut rx = service.subscribe();

        let event_sender = sender.clone();
        glib::spawn_future_local(async move {
            loop {
                match rx.recv().await {
                    Ok(_) => {
                        event_sender.input(ClipboardInput::Refresh);
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Clipboard panel missed {n} events, refreshing");
                        event_sender.input(ClipboardInput::Refresh);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        error!("Clipboard broadcast channel closed");
                        break;
                    }
                }
            }
        });

        let factory = DynamicBoxFactory::<ClipboardEntry, u64> {
            id: Box::new(|item| item.id),
            create: Box::new(move |item| {
                let controller: Controller<ClipboardItemModel> =
                    ClipboardItemModel::builder()
                        .launch(item.clone().into())
                        .detach();
                Box::new(controller) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let dynamic: Controller<DynamicBoxModel<ClipboardEntry, u64>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory,
                    orientation: gtk::Orientation::Vertical,
                    spacing: 10,
                    transition_type: RevealerTransitionType::SlideDown,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                })
                .detach();

        let model = ClipboardModel {
            dynamic_box: dynamic,
            history,
            delete_button_visible: false,
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
            ClipboardInput::Refresh => {
                let items = self.history.entries();
                self.delete_button_visible = !items.is_empty();
                self.dynamic_box.sender().send(DynamicBoxInput::SetItems(items)).unwrap();
            }
            ClipboardInput::DeleteAllClicked => {
                clipboard_service().clear_history();
                let _ = sender.output(ClipboardOutput::CloseMenu);
            }
        }

        self.update_view(widgets, sender);
    }
}
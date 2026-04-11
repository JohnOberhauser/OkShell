use std::sync::Arc;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::{Orientation, RevealerTransitionType};
use relm4::gtk::prelude::*;
use okshell_common::watch;
use wayle_systray::core::item::TrayItem;
use okshell_services::sys_tray_service;
use crate::bars::bar_widgets::system_tray_item::SystemTrayItemModel;
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;

pub(crate) struct SystemTrayModel {
    dynamic_box: Controller<DynamicBoxModel<Arc<TrayItem>, String>>,
    revealed: bool,
    orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum SystemTrayInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum SystemTrayOutput {}

pub(crate) struct SystemTrayInit {
    pub(crate) orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum SystemTrayCommandOutput {
    ItemsChanged(Vec<Arc<TrayItem>>)
}

#[relm4::component(pub)]
impl Component for SystemTrayModel {
    type CommandOutput = SystemTrayCommandOutput;
    type Input = SystemTrayInput;
    type Output = SystemTrayOutput;
    type Init = SystemTrayInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            add_css_class: "system-tray-bar-widget",
            set_visible: false,
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Revealer {
                #[watch]
                set_reveal_child: model.revealed,

                #[name = "system_tray_box"]
                gtk::Box {},
            },

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(SystemTrayInput::Clicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("tray-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        Self::spawn_system_tray_watcher(&sender);

        let factory = DynamicBoxFactory::<Arc<TrayItem>, String> {
            id: Box::new(|item| item.id.get()),
            create: Box::new(move |item| {
                let controller: Controller<SystemTrayItemModel> =
                    SystemTrayItemModel::builder()
                        .launch(item.clone().into())
                        .detach();
                Box::new(controller) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let transition_type = if params.orientation == Orientation::Horizontal {
            RevealerTransitionType::SwingLeft
        } else {
            RevealerTransitionType::SwingUp
        };

        let dynamic: Controller<DynamicBoxModel<Arc<TrayItem>, String>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory,
                    orientation: params.orientation,
                    spacing: 0,
                    transition_type,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                    allow_drag_and_drop: false,
                })
                .detach();

        let model = SystemTrayModel {
            dynamic_box: dynamic,
            revealed: false,
            orientation: params.orientation,
        };

        let widgets = view_output!();

        widgets.root.set_orientation(params.orientation);

        widgets.system_tray_box.append(model.dynamic_box.widget());

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
            SystemTrayInput::Clicked => {
                self.revealed = !self.revealed;
            }
        }
        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            SystemTrayCommandOutput::ItemsChanged(items) => {
                widgets.root.set_visible(items.len() > 0);
                self.dynamic_box.sender().send(DynamicBoxInput::SetItems(items)).unwrap();
            }
        }
    }
}

impl SystemTrayModel {
    fn spawn_system_tray_watcher(
        sender: &ComponentSender<Self>,
    ) {
        let service = sys_tray_service();
        let items = service.items.clone();

        watch!(sender, [items.watch()], |out| {
            let _ = out.send(SystemTrayCommandOutput::ItemsChanged(items.get()));
        })
    }
}
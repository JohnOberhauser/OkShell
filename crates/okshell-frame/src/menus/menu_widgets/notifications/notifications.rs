use std::sync::Arc;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::gtk::RevealerTransitionType;
use wayle_notification::core::notification::Notification;
use okshell_services::notification_service;
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;
use okshell_common::notification::{NotificationInit, NotificationModel, NotificationOutput};
use okshell_utils::notifications::{spawn_dnd_watcher, spawn_notifications_watcher};

pub(crate) struct NotificationsModel {
    dynamic_box_controller: Controller<DynamicBoxModel<Arc<Notification>, u32>>,
    empty_label_visible: bool,
    dnd: bool,
}

#[derive(Debug)]
pub(crate) enum NotificationsInput {
    ClearAllClicked,
    DndClicked,
}

#[derive(Debug)]
pub(crate) enum NotificationsOutput {
    CloseMenu,
}

pub(crate) struct NotificationsInit {}

#[derive(Debug)]
pub(crate) enum NotificationsCommandOutput {
    NotificationsChanged,
    DndChanged,
}

#[relm4::component(pub)]
impl Component for NotificationsModel {
    type CommandOutput = NotificationsCommandOutput;
    type Input = NotificationsInput;
    type Output = NotificationsOutput;
    type Init = NotificationsInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "notifications-menu-widget",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_valign: gtk::Align::Center,
                    connect_clicked[sender] => move |_| {
                        sender.input(NotificationsInput::DndClicked);
                    },
                    set_margin_end: 4,

                    gtk::Image {
                        #[watch]
                        set_icon_name: if model.dnd {
                            Some("notification-disabled-symbolic")
                        } else {
                            Some("notification-symbolic")
                        },
                    },
                },

                gtk::Label {
                    add_css_class: "label-medium-bold",
                    set_halign: gtk::Align::Start,
                    set_label: "Notification History",
                    set_hexpand: true,
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_valign: gtk::Align::Center,
                    connect_clicked[sender] => move |_| {
                        sender.input(NotificationsInput::ClearAllClicked);
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
                set_visible: model.empty_label_visible,
                set_label: "Empty",
            },

            gtk::ScrolledWindow {
                set_vscrollbar_policy: gtk::PolicyType::Automatic,
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_propagate_natural_height: true,
                set_propagate_natural_width: false,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    model.dynamic_box_controller.widget().clone() {},
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        spawn_notifications_watcher(
            &sender,
            ||NotificationsCommandOutput::NotificationsChanged,
        );

        spawn_dnd_watcher(
            &sender,
            ||NotificationsCommandOutput::DndChanged,
        );

        let sender_clone = sender.clone();
        let notifications_dynamic_box_factory = DynamicBoxFactory::<Arc<Notification>, u32> {
            id: Box::new(|item| item.id),
            create: Box::new(move |item| {
                let notification = item.clone();
                let notifications_controller = NotificationModel::builder()
                    .launch(NotificationInit {
                        notification,
                    })
                    .forward(sender_clone.output_sender(), |msg| {
                        match msg { 
                            NotificationOutput::ActionActivated => {
                                NotificationsOutput::CloseMenu
                            } 
                        }
                    });

                Box::new(notifications_controller) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let notifications_dynamic_box_controller: Controller<DynamicBoxModel<Arc<Notification>, u32>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory: notifications_dynamic_box_factory,
                    orientation: gtk::Orientation::Vertical,
                    spacing: 10,
                    transition_type: RevealerTransitionType::SlideDown,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                    allow_drag_and_drop: false,
                })
                .detach();

        let model = NotificationsModel {
            dynamic_box_controller: notifications_dynamic_box_controller,
            empty_label_visible: true,
            dnd: false,
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
            NotificationsInput::ClearAllClicked => {
                tokio::spawn(async move {
                    let _ = notification_service().dismiss_all().await;
                });
                let _ = sender.output(NotificationsOutput::CloseMenu);
            }
            NotificationsInput::DndClicked => {
                let service = notification_service();
                let dnd = service.dnd.get();

                service.set_dnd(!dnd);
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            NotificationsCommandOutput::NotificationsChanged => {
                let notifications = notification_service().notifications.get();
                self.empty_label_visible = notifications.is_empty();
                self.dynamic_box_controller.emit(DynamicBoxInput::SetItems(notifications));
            }
            NotificationsCommandOutput::DndChanged => {
                let service = notification_service();
                self.dnd = service.dnd.get();
            }
        }

        self.update_view(widgets, sender);
    }
}
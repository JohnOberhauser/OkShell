use std::sync::Arc;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::{gdk, RevealerTransitionType};
use relm4::gtk::prelude::{GtkWindowExt, OrientableExt, WidgetExt};
use wayle_notification::core::notification::Notification;
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;
use okshell_common::notification::{NotificationInit, NotificationModel};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, NotificationsStoreFields};
use okshell_config::schema::position::NotificationPosition;
use okshell_services::notification_service;
use okshell_utils::notifications::spawn_notification_popups_watcher;

pub struct PopupNotificationsModel {
    dynamic_box_controller: Controller<DynamicBoxModel<Arc<Notification>, u32>>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub enum PopupNotificationsInput {
    PositionChanged(NotificationPosition),
}

#[derive(Debug)]
pub enum PopupNotificationsOutput {}

pub struct PopupNotificationsInit {
    pub monitor: gdk::Monitor,
}

#[derive(Debug)]
pub enum PopupNotificationsCommandOutput {
    NotificationsChanged,
}

#[relm4::component(pub)]
impl Component for PopupNotificationsModel {
    type CommandOutput = PopupNotificationsCommandOutput;
    type Input = PopupNotificationsInput;
    type Output = PopupNotificationsOutput;
    type Init = PopupNotificationsInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Window {
            set_css_classes: &["popup-notifications-window", "window-opacity"],
            set_decorated: false,
            set_visible: true,
            set_default_height: 1,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_width_request: 400,

                model.dynamic_box_controller.widget().clone() {},
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = config_manager().config();

        let position = config.notifications().notification_position().get_untracked();

        root.init_layer_shell();
        root.set_monitor(Some(&params.monitor));
        root.set_namespace(Some("okshell-notifications"));
        root.set_layer(Layer::Overlay);
        root.set_exclusive_zone(0);
        set_position(position, &root);

        spawn_notification_popups_watcher(
            &sender,
            ||PopupNotificationsCommandOutput::NotificationsChanged,
        );

        let notifications_dynamic_box_factory = DynamicBoxFactory::<Arc<Notification>, u32> {
            id: Box::new(|item| item.id),
            create: Box::new(move |item| {
                let notification = item.clone();
                let notification_controller = NotificationModel::builder()
                    .launch(NotificationInit {
                        notification,
                    })
                    .detach();

                Box::new(notification_controller) as Box<dyn GenericWidgetController>
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
                })
                .detach();

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let position = config.notifications().notification_position().get();
            sender_clone.input(PopupNotificationsInput::PositionChanged(position))
        });

        let model = PopupNotificationsModel {
            dynamic_box_controller: notifications_dynamic_box_controller,
            _effects: effects,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        root: &Self::Root
    ) {
        match message {
            PopupNotificationsInput::PositionChanged(pos) => {
                set_position(pos, root);
            }
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            PopupNotificationsCommandOutput::NotificationsChanged => {
                let notifications = notification_service().popups.get();
                self.dynamic_box_controller.emit(DynamicBoxInput::SetItems(notifications));
            }
        }

        self.update_view(widgets, sender);
    }
}

fn set_position(
    position: NotificationPosition,
    root: &gtk::Window,
) {
    match position {
        NotificationPosition::Left => {
            root.set_anchor(Edge::Top, true);
            root.set_anchor(Edge::Bottom, false);
            root.set_anchor(Edge::Left, true);
            root.set_anchor(Edge::Right, false);
        }
        NotificationPosition::Right => {
            root.set_anchor(Edge::Top, true);
            root.set_anchor(Edge::Bottom, false);
            root.set_anchor(Edge::Left, false);
            root.set_anchor(Edge::Right, true);
        }
        NotificationPosition::Center => {
            root.set_anchor(Edge::Top, true);
            root.set_anchor(Edge::Bottom, false);
            root.set_anchor(Edge::Left, false);
            root.set_anchor(Edge::Right, false);
        }
    }
}
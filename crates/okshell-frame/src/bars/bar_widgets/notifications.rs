use okshell_services::notification_service;
use okshell_utils::notifications::spawn_notifications_watcher;
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct NotificationsModel {
    orientation: Orientation,
    icon_name: String,
}

#[derive(Debug)]
pub(crate) enum NotificationsInput {}

#[derive(Debug)]
pub(crate) enum NotificationsOutput {
    Clicked,
}

pub(crate) struct NotificationsInit {
    pub(crate) orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum NotificationsCommandOutput {
    NotificationsChanged,
}

#[relm4::component(pub)]
impl Component for NotificationsModel {
    type Input = NotificationsInput;
    type Output = NotificationsOutput;
    type Init = NotificationsInit;
    type CommandOutput = NotificationsCommandOutput;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "notifications-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.output(NotificationsOutput::Clicked).unwrap_or_default();
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_icon_name: Some(model.icon_name.as_str()),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_notifications_watcher(&sender, || NotificationsCommandOutput::NotificationsChanged);

        let model = NotificationsModel {
            orientation: params.orientation,
            icon_name: "notification-symbolic".into(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            NotificationsCommandOutput::NotificationsChanged => {
                let service = notification_service();
                if service.notifications.get().is_empty() {
                    self.icon_name = "notification-symbolic".into();
                } else {
                    self.icon_name = "notification-alert-symbolic".into();
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

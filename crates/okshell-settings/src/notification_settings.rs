use reactive_graph::prelude::GetUntracked;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, NotificationsStoreFields};
use okshell_config::schema::position::{NotificationPosition};

#[derive(Debug, Clone)]
pub(crate) struct NotificationSettingsModel {
    position: NotificationPosition,
}

#[derive(Debug)]
pub(crate) enum NotificationSettingsInput {
    PositionChanged(NotificationPosition),
}

#[derive(Debug)]
pub(crate) enum NotificationSettingsOutput {}

pub(crate) struct NotificationSettingsInit {}

#[derive(Debug)]
pub(crate) enum NotificationSettingsCommandOutput {}

#[relm4::component(pub)]
impl Component for NotificationSettingsModel {
    type CommandOutput = NotificationSettingsCommandOutput;
    type Input = NotificationSettingsInput;
    type Output = NotificationSettingsOutput;
    type Init = NotificationSettingsInit;

    view! {
        #[root]
        gtk::ScrolledWindow {
            set_vscrollbar_policy: gtk::PolicyType::Automatic,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_height: false,
            set_propagate_natural_width: false,
            set_hexpand: true,
            set_vexpand: true,

            gtk::Box {
                add_css_class: "settings-page",
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 16,

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Notifications",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Position",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Where popup notifications should be positioned.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&NotificationPosition::display_names())),
                        #[watch]
                        set_selected: model.position.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(NotificationSettingsInput::PositionChanged(
                                NotificationPosition::from_index(dd.selected())
                            ));
                        },
                    },
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let model = NotificationSettingsModel {
            position: config_manager().config().notifications().notification_position().get_untracked(),
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
            NotificationSettingsInput::PositionChanged(position) => {
                self.position = position.clone();
                config_manager().update_config(|config| {
                    config.notifications.notification_position = position;
                });
            }
        }

        self.update_view(widgets, sender);
    }
}
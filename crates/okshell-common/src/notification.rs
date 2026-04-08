use std::sync::Arc;
use reactive_graph::traits::{
    GetUntracked,
    Get,
};
use relm4::{gtk, once_cell, Component, ComponentParts, ComponentSender};
use relm4::gtk::pango;
use relm4::gtk::prelude::*;
use time::format_description::parse;
use time::OffsetDateTime;
use wayle_notification::core::notification::Notification;
use crate::scoped_effects::EffectScope;
use okshell_config::schema::config::{ConfigStoreFields, GeneralStoreFields};

static TIME_FORMAT_24: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:24 padding:zero]:[minute padding:zero]").unwrap()
    });

static TIME_FORMAT_12: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:12 padding:zero]:[minute padding:zero] [period case:lower]").unwrap()
    });

#[derive(Debug, Clone)]
pub struct NotificationModel {
    notification: Arc<Notification>,
    time: String,
    _effects: EffectScope,
}

#[derive(Debug)]
pub enum NotificationInput {
    CloseClicked,
    ChangeTimeFormat(bool),
}

#[derive(Debug)]
pub enum NotificationOutput {
    ActionActivated,
}

pub struct NotificationInit {
    pub notification: Arc<Notification>,
}

#[derive(Debug)]
pub enum NotificationCommandOutput {}

#[relm4::component(pub)]
impl Component for NotificationModel {
    type CommandOutput = NotificationCommandOutput;
    type Input = NotificationInput;
    type Output = NotificationOutput;
    type Init = NotificationInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "notification",
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_spacing: 8,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "label-small-bold-variant",
                    set_label: model.notification.app_name.get().unwrap_or("".to_string()).as_str(),
                    set_hexpand: true,
                    set_xalign: 0.0,
                },

                gtk::Label {
                    add_css_class: "label-small",
                    #[watch]
                    set_label: model.time.as_str(),
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_margin_start: 4,
                    set_hexpand: false,
                    set_vexpand: false,
                    connect_clicked[sender] => move |_| {
                        sender.input(NotificationInput::CloseClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: Some("close-symbolic"),
                    },
                },
            },

            gtk::Label {
                add_css_class: "label-medium-bold",
                set_label: model.notification.summary.get().as_str(),
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: pango::WrapMode::WordChar,
                set_width_chars: 20,
                set_max_width_chars: 40,
            },

            gtk::Label {
                add_css_class: "label-small",
                set_label: model.notification.body.get().unwrap_or("".to_string()).as_str(),
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: pango::WrapMode::WordChar,
                set_width_chars: 20,
                set_max_width_chars: 40,
            },

            #[name = "actions_container"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 4,
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let base_config = okshell_config::config_manager::config_manager().config();

        let format_24_h = base_config.clone().general().clock_format_24_h().get_untracked();

        let time: String;

        let timestamp = params.notification.timestamp.get();

        let odt = OffsetDateTime::from_unix_timestamp(timestamp.timestamp())
            .unwrap()
            .replace_nanosecond(timestamp.timestamp_subsec_nanos())
            .unwrap();

        if format_24_h {
            time = odt.format(&TIME_FORMAT_24).unwrap();
        } else {
            time = odt.format(&TIME_FORMAT_12).unwrap();
        }

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = base_config.clone();
            let format_24_h = config.general().clock_format_24_h().get();
            sender_clone.input(NotificationInput::ChangeTimeFormat(format_24_h));
        });

        let model = NotificationModel {
            notification: params.notification,
            time,
            _effects: effects,
        };

        let widgets = view_output!();

        let actions = &model.notification.actions.get();
        if !actions.is_empty() {
            for action in actions {
                let btn = gtk::Button::with_label(&action.label);
                btn.add_css_class("ok-button-primary");

                let notification = model.notification.clone();
                let key = action.id.clone();
                let sender_clone = sender.clone();
                btn.connect_clicked(move |_| {
                    let notification = notification.clone();
                    let key = key.clone();
                    let sender_clone = sender_clone.clone();
                    tokio::spawn(async move {
                        let _ = notification.invoke(&key).await;
                        let _ = sender_clone.output(NotificationOutput::ActionActivated);
                    });
                });

                widgets.actions_container.append(&btn);
            }
        }

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
            NotificationInput::CloseClicked => {
                let notification = self.notification.clone();
                notification.dismiss();
            }
            NotificationInput::ChangeTimeFormat(format_24_h) => {
                let timestamp = self.notification.timestamp.get();

                let odt = OffsetDateTime::from_unix_timestamp(timestamp.timestamp())
                    .unwrap()
                    .replace_nanosecond(timestamp.timestamp_subsec_nanos())
                    .unwrap();

                if format_24_h {
                    self.time = odt.format(&TIME_FORMAT_24).unwrap();
                } else {
                    self.time = odt.format(&TIME_FORMAT_12).unwrap();
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
use relm4::{
    gtk::{
        self,
        glib::{
            self,
            SourceId,
        },
        Orientation,
        prelude::*,
    }, 
    once_cell, 
    ComponentParts, 
    ComponentSender, 
    SimpleComponent,
};
use time::format_description::parse;
use time::OffsetDateTime;
use okshell_config::{
    schema::config::*,
};
use reactive_graph::traits::{
    Get,
    GetUntracked,
};
use relm4::gtk::prelude::OrientableExt;
use okshell_common::scoped_effects::EffectScope;

static TIME_FORMAT_24: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:24 padding:zero]:[minute padding:zero]").unwrap()
    });

static TIME_FORMAT_12: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:12 padding:zero]:[minute padding:zero] [period case:lower]").unwrap()
    });

static DAY_FORMAT: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[weekday repr:long]").unwrap()
    });

static DATE_FORMAT: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[month repr:numerical]/[day padding:zero]/[year repr:full base:calendar]").unwrap()
    });

#[derive(Debug)]
pub(crate) struct ClockModel {
    format_24_h: bool,
    time_label: String,
    day_label: String,
    date_label: String,
    timer_id: Option<SourceId>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum ClockInput {
    UpdateTime,
    ChangeFormat(bool),
}

#[derive(Debug)]
pub(crate) enum ClockOutput {}

pub(crate) struct ClockInit {}

#[relm4::component(pub)]
impl SimpleComponent for ClockModel {
    type Input = ClockInput;
    type Output = ClockOutput;
    type Init = ClockInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "clock-menu-widget",
            set_orientation: Orientation::Horizontal,
            set_hexpand: true,
            set_spacing: 20,
            set_valign: gtk::Align::Center,
            
            gtk::Label {
                add_css_class: "label-xxl-bold",
                add_css_class: "clock-menu-widget-day-label",
                #[watch]
                set_label: model.day_label.as_str(),
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Center,
            },

            gtk::Box {
                set_orientation: Orientation::Vertical,
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    add_css_class: "label-medium-bold",
                    add_css_class: "clock-menu-widget-date-label",
                    #[watch]
                    set_label: model.date_label.as_str(),
                    set_halign: gtk::Align::Start,
                },

                gtk::Label {
                    add_css_class: "label-small",
                    add_css_class: "clock-menu-widget-time-label",
                    #[watch]
                    set_label: model.time_label.as_str(),
                    set_halign: gtk::Align::Start,
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let base_config = okshell_config::config_manager::config_manager().config();

        let sender_clone = sender.clone();
        let id = glib::timeout_add_local(
            std::time::Duration::from_secs(1),
            move || {
                sender_clone.input(ClockInput::UpdateTime);
                glib::ControlFlow::Continue
            },
        );

        let format_24_h = base_config.clone().general().clock_format_24_h().get_untracked();

        let now = OffsetDateTime::now_local()
            .unwrap_or_else(|_| OffsetDateTime::now_utc());

        let time: String;

        if format_24_h {
            time = now.format(&TIME_FORMAT_24).unwrap();
        } else {
            time = now.format(&TIME_FORMAT_12).unwrap();
        }

        let day = now.format(&DAY_FORMAT).unwrap();
        let date = now.format(&DATE_FORMAT).unwrap();

        let mut effects = EffectScope::new();
        
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = base_config.clone();
            let format_24_h = config.general().clock_format_24_h().get();
            sender_clone.input(ClockInput::ChangeFormat(format_24_h));
        });

        let model = ClockModel {
            format_24_h,
            time_label: time,
            date_label: date,
            day_label: day,
            timer_id: Some(id),
            _effects: effects,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self, 
        message: Self::Input,
        _sender: ComponentSender<Self>,
    ) {
        match message {
            ClockInput::UpdateTime => {
                let now = OffsetDateTime::now_local()
                    .unwrap_or_else(|_| OffsetDateTime::now_utc());

                let time: String;

                if self.format_24_h {
                    time = now.format(&TIME_FORMAT_24).unwrap();
                } else {
                    time = now.format(&TIME_FORMAT_12).unwrap();
                }

                let day = now.format(&DAY_FORMAT).unwrap();
                let date = now.format(&DATE_FORMAT).unwrap();

                self.day_label = day;
                self.date_label = date;
                self.time_label = time;
            },
            ClockInput::ChangeFormat(format_24_h) => {
                self.format_24_h = format_24_h;
            }
        }
    }
}

impl Drop for ClockModel {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.take() {
            id.remove();
        }
    }
}
use relm4::{gtk::{
    self,
    glib::{
        self,
        SourceId
    },
    prelude::*,
    Align,
    Justification,
}, ComponentParts, ComponentSender, once_cell, Component};
use time::format_description::parse;
use time::{Date, OffsetDateTime};

static DATE_FORMAT: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[weekday repr:long]\n[month repr:long] [day padding:none], [year]").unwrap()
    });

#[derive(Debug)]
pub(crate) struct CalendarModel {
    timer_id: Option<SourceId>,
    time: String,
    current_date: Date,
}

#[derive(Debug)]
pub(crate) enum CalendarInput {
    UpdateTime,
}

#[derive(Debug)]
pub(crate) enum CalendarOutput {}

pub(crate) struct CalendarInit {}

#[relm4::component(pub)]
impl Component for CalendarModel {
    type Input = CalendarInput;
    type Output = CalendarOutput;
    type Init = CalendarInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "calendar-menu-widget",
            set_hexpand: false,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            gtk::Label {
                add_css_class: "label-xl",
                #[watch]
                set_label: model.time.as_str(),
                set_xalign: 0.5,
                set_halign: Align::Center,
                set_justify: Justification::Center,
            },

            #[name = "calendar"]
            gtk::Calendar {
                set_can_focus: false,
                set_focus_on_click: false,
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let sender_clone = sender.clone();
        let id = glib::timeout_add_local(
            std::time::Duration::from_secs(1),
            move || {
                sender_clone.input(CalendarInput::UpdateTime);
                glib::ControlFlow::Continue
            },
        );

        let now = OffsetDateTime::now_local()
            .unwrap_or_else(|_| OffsetDateTime::now_utc());

        let formatted = now.format(&DATE_FORMAT).unwrap();

        let model = CalendarModel {
            timer_id: Some(id),
            time: formatted,
            current_date: now.date(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            CalendarInput::UpdateTime => {
                let now = OffsetDateTime::now_local()
                    .unwrap_or_else(|_| OffsetDateTime::now_utc());

                let formatted = now.format(&DATE_FORMAT).unwrap();

                self.time = formatted;

                if now.date() != self.current_date {
                    self.current_date = now.date();
                    widgets.calendar.set_year(now.year());
                    // GTK's months are 0-indexed, but time's is 1.
                    widgets.calendar.set_month(now.month() as i32 - 1);
                    widgets.calendar.set_day(now.day() as i32);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl Drop for CalendarModel {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.take() {
            id.remove();
        }
    }
}
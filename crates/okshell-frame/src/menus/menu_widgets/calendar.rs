use relm4::{
    gtk::{
        self,
        glib::{
            self,
            SourceId
        },
        prelude::*,
        Align,
        Justification,
    },
    ComponentParts,
    ComponentSender,
    SimpleComponent,
    once_cell,
};
use time::format_description::parse;
use time::OffsetDateTime;

static DATE_FORMAT: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[weekday repr:long]\n[month repr:long] [day padding:none], [year]").unwrap()
    });

#[derive(Debug)]
pub(crate) struct CalendarModel {
    timer_id: Option<SourceId>,
    time: String,
}

#[derive(Debug)]
pub(crate) enum CalendarInput {
    UpdateTime,
}

#[derive(Debug)]
pub(crate) enum CalendarOutput {}

pub(crate) struct CalendarInit {}

#[relm4::component(pub)]
impl SimpleComponent for CalendarModel {
    type Input = CalendarInput;
    type Output = CalendarOutput;
    type Init = CalendarInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "calendar-menu-widget",
            set_hexpand: false,
            set_orientation: gtk::Orientation::Vertical,

            gtk::Label {
                add_css_class: "label-xl",
                #[watch]
                set_label: model.time.as_str(),
                set_xalign: 0.5,
                set_halign: Align::Center,
                set_justify: Justification::Center,
            },

            gtk::Calendar {
                add_css_class: "ok-calendar",
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
            CalendarInput::UpdateTime => {
                let now = OffsetDateTime::now_local()
                    .unwrap_or_else(|_| OffsetDateTime::now_utc());

                let formatted = now.format(&DATE_FORMAT).unwrap();

                self.time = formatted;
            }
        }
    }
}

impl Drop for CalendarModel {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.take() {
            id.remove();
        }
    }
}
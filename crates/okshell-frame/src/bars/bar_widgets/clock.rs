use okshell_common::scoped_effects::EffectScope;
use okshell_config::schema::config::*;
use reactive_graph::traits::{Get, GetUntracked};
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
    gtk::{
        self, Orientation,
        glib::{self, SourceId},
        prelude::{ButtonExt, WidgetExt},
    },
    once_cell,
};
use time::OffsetDateTime;
use time::format_description::parse;

static TIME_FORMAT_24: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:24 padding:zero]:[minute padding:zero]").unwrap()
    });

static TIME_FORMAT_24_VERTICAL: once_cell::sync::Lazy<
    Vec<time::format_description::FormatItem<'static>>,
> = once_cell::sync::Lazy::new(|| {
    parse("[hour repr:24 padding:zero]\n[minute padding:zero]").unwrap()
});

static TIME_FORMAT_12: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:12 padding:zero]:[minute padding:zero]").unwrap()
    });

static TIME_FORMAT_12_VERTICAL: once_cell::sync::Lazy<
    Vec<time::format_description::FormatItem<'static>>,
> = once_cell::sync::Lazy::new(|| {
    parse("[hour repr:12 padding:zero]\n[minute padding:zero]").unwrap()
});

#[derive(Debug)]
pub(crate) struct ClockModel {
    orientation: Orientation,
    format_24_h: bool,
    time_label: String,
    timer_id: Option<SourceId>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum ClockInput {
    UpdateTime,
    ChangeFormat(bool),
}

#[derive(Debug)]
pub(crate) enum ClockOutput {
    Clicked,
}

pub(crate) struct ClockInit {
    pub(crate) orientation: Orientation,
}

#[relm4::component(pub)]
impl SimpleComponent for ClockModel {
    type Input = ClockInput;
    type Output = ClockOutput;
    type Init = ClockInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "clock-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                connect_clicked[sender] => move |_| {
                    sender.output(ClockOutput::Clicked).unwrap_or_default();
                },

                gtk::Label {
                    #[watch]
                    set_label: model.time_label.as_str(),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let base_config = okshell_config::config_manager::config_manager().config();

        let sender_clone = sender.clone();
        let id = glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            sender_clone.input(ClockInput::UpdateTime);
            glib::ControlFlow::Continue
        });

        let format_24_h = base_config
            .clone()
            .general()
            .clock_format_24_h()
            .get_untracked();

        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

        let formatted: String;

        if params.orientation == Orientation::Vertical {
            if format_24_h {
                formatted = now.format(&TIME_FORMAT_24_VERTICAL).unwrap();
            } else {
                formatted = now.format(&TIME_FORMAT_12_VERTICAL).unwrap();
            }
        } else {
            if format_24_h {
                formatted = now.format(&TIME_FORMAT_24).unwrap();
            } else {
                formatted = now.format(&TIME_FORMAT_12).unwrap();
            }
        }

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = base_config.clone();
            let format_24_h = config.general().clock_format_24_h().get();
            sender_clone.input(ClockInput::ChangeFormat(format_24_h));
        });

        let model = ClockModel {
            orientation: params.orientation,
            format_24_h,
            time_label: formatted,
            timer_id: Some(id),
            _effects: effects,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ClockInput::UpdateTime => {
                let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

                let formatted: String;

                if self.orientation == Orientation::Vertical {
                    if self.format_24_h {
                        formatted = now.format(&TIME_FORMAT_24_VERTICAL).unwrap();
                    } else {
                        formatted = now.format(&TIME_FORMAT_12_VERTICAL).unwrap();
                    }
                } else {
                    if self.format_24_h {
                        formatted = now.format(&TIME_FORMAT_24).unwrap();
                    } else {
                        formatted = now.format(&TIME_FORMAT_12).unwrap();
                    }
                }

                self.time_label = formatted;
            }
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

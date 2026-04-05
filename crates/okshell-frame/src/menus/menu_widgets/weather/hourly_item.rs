use reactive_graph::traits::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use wayle_weather::{HourlyForecast, TemperatureUnit};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, GeneralStoreFields};
use okshell_utils::weather::{get_temperature_string, get_weather_icon_name};

#[derive(Debug, Clone)]
pub(crate) struct HourlyItemModel {
    hourly: HourlyForecast,
    temperature_unit: TemperatureUnit,
    time_label: String,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum HourlyItemInput {
    UpdateTemperatureUnit(TemperatureUnit),
    ChangeFormat(bool),
}

#[derive(Debug)]
pub(crate) enum HourlyItemOutput {}

pub(crate) struct HourlyItemInit {
    pub hourly: HourlyForecast,
}

#[derive(Debug)]
pub(crate) enum HourlyItemCommandOutput {}

#[relm4::component(pub)]
impl Component for HourlyItemModel {
    type CommandOutput = HourlyItemCommandOutput;
    type Input = HourlyItemInput;
    type Output = HourlyItemOutput;
    type Init = HourlyItemInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            gtk::Label {
                add_css_class: "label-small-bold",
                set_label: model.time_label.as_str(),
            },

            gtk::Image {
                add_css_class: "hourly-weather-icon",
                #[watch]
                set_icon_name: Some(get_weather_icon_name(
                    &model.hourly.condition,
                    model.hourly.is_day,
                )),
            },

            gtk::Label {
                add_css_class: "label-small-bold",
                #[watch]
                set_label: get_temperature_string(
                    &model.hourly.temperature,
                    &model.temperature_unit
                ).as_str(),
            },

            gtk::Label {
                add_css_class: "label-small-bold",
                #[watch]
                set_label: format!("{} UV", model.hourly.uv_index).as_str(),
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let base_config = config_manager().config();

        let mut effects = EffectScope::new();

        let config = base_config.clone();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config.clone();
            let temperature_unit = config.general().temperature_unit().get();
            sender_clone.input(HourlyItemInput::UpdateTemperatureUnit(TemperatureUnit::from(temperature_unit)));
        });

        let sender_clone = sender.clone();
        let config = base_config.clone();
        effects.push(move |_| {
            let config = config.clone();
            let format_24_h = config.general().clock_format_24_h().get();
            sender_clone.input(HourlyItemInput::ChangeFormat(format_24_h));
        });

        let config = base_config.clone();
        let format_24_h = config.general().clock_format_24_h().get_untracked();

        let time_label: String;

        if format_24_h {
            time_label = params.hourly.time.format("%H").to_string();
        } else {
            time_label = params.hourly.time.format("%I %p").to_string();
        }

        let model = HourlyItemModel {
            hourly: params.hourly,
            temperature_unit: TemperatureUnit::from(base_config.general().temperature_unit().get_untracked()),
            time_label,
            _effects: effects,
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
            HourlyItemInput::UpdateTemperatureUnit(temperature_unit) => {
                self.temperature_unit = temperature_unit;
            }
            HourlyItemInput::ChangeFormat(format_24_h) => {
                let time_label: String;

                if format_24_h {
                    time_label = self.hourly.time.format("%H").to_string();
                } else {
                    time_label = self.hourly.time.format("%I %p").to_string();
                }

                self.time_label = time_label;
            }
        }

        self.update_view(widgets, sender);
    }
}
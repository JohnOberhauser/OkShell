use reactive_graph::traits::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use wayle_weather::{DailyForecast, TemperatureUnit};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, GeneralStoreFields};
use okshell_utils::weather::{get_temperature_string, get_weather_icon_name};

#[derive(Debug, Clone)]
pub(crate) struct DailyItemModel {
    daily: DailyForecast,
    temperature_unit: TemperatureUnit,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum DailyItemInput {
    UpdateTemperatureUnit(TemperatureUnit),
}

#[derive(Debug)]
pub(crate) enum DailyItemOutput {}

pub(crate) struct DailyItemInit {
    pub daily: DailyForecast,
}

#[derive(Debug)]
pub(crate) enum DailyItemCommandOutput {}

#[relm4::component(pub)]
impl Component for DailyItemModel {
    type CommandOutput = DailyItemCommandOutput;
    type Input = DailyItemInput;
    type Output = DailyItemOutput;
    type Init = DailyItemInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            gtk::Label {
                add_css_class: "label-small-bold",
                set_label: model.daily.date.format("%a").to_string().as_str(),
            },

            gtk::Image {
                add_css_class: "hourly-weather-icon",
                #[watch]
                set_icon_name: Some(get_weather_icon_name(
                    &model.daily.condition,
                    true,
                )),
            },

            gtk::Label {
                add_css_class: "label-small-bold",
                #[watch]
                set_label: get_temperature_string(
                    &model.daily.temp_high,
                    &model.temperature_unit
                ).as_str(),
            },

            gtk::Label {
                add_css_class: "label-small-bold",
                #[watch]
                set_label: get_temperature_string(
                    &model.daily.temp_low,
                    &model.temperature_unit
                ).as_str(),
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
            sender_clone.input(DailyItemInput::UpdateTemperatureUnit(TemperatureUnit::from(temperature_unit)));
        });

        let model = DailyItemModel {
            daily: params.daily,
            temperature_unit: TemperatureUnit::from(base_config.general().temperature_unit().get_untracked()),
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
            DailyItemInput::UpdateTemperatureUnit(temperature_unit) => {
                self.temperature_unit = temperature_unit;
            }
        }

        self.update_view(widgets, sender);
    }
}
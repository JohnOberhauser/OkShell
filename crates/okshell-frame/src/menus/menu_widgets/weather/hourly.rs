use crate::menus::menu_widgets::weather::hourly_item::{HourlyItemInit, HourlyItemModel};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, GeneralStoreFields};
use okshell_utils::scroll_extensions::wire_vertical_to_horizontal;
use reactive_graph::traits::{Get, GetUntracked};
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmRemoveAllExt,
    RelmWidgetExt, gtk,
};
use wayle_weather::{HourlyForecast, TemperatureUnit};

pub(crate) struct HourlyModel {
    controllers: Vec<Controller<HourlyItemModel>>,
    temperature_unit: TemperatureUnit,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum HourlyInput {
    Update(Vec<HourlyForecast>),
    UpdateTemperatureUnit(TemperatureUnit),
}

#[derive(Debug)]
pub(crate) enum HourlyOutput {}

pub(crate) struct HourlyInit {
    pub hourly: Vec<HourlyForecast>,
}

#[derive(Debug)]
pub(crate) enum HourlyCommandOutput {}

#[relm4::component(pub)]
impl Component for HourlyModel {
    type CommandOutput = HourlyCommandOutput;
    type Input = HourlyInput;
    type Output = HourlyOutput;
    type Init = HourlyInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "weather-container",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            gtk::Label {
                add_css_class: "label-large-bold",
                set_label: "Hourly Conditions",
            },

            #[name = "scroll_window"]
            gtk::ScrolledWindow {
                set_vscrollbar_policy: gtk::PolicyType::Never,
                set_hscrollbar_policy: gtk::PolicyType::External,
                set_hexpand: true,
                set_vexpand: true,

                #[name = "widget_container"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 32,
                    set_align: gtk::Align::Center,
                }
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
            sender_clone.input(HourlyInput::UpdateTemperatureUnit(TemperatureUnit::from(
                temperature_unit,
            )));
        });

        let mut model = HourlyModel {
            controllers: Vec::new(),
            temperature_unit: TemperatureUnit::from(
                base_config.general().temperature_unit().get_untracked(),
            ),
            _effects: effects,
        };

        let widgets = view_output!();

        let container = widgets.widget_container.clone();
        let mut controllers: Vec<Controller<HourlyItemModel>> = Vec::new();

        params.hourly.iter().for_each(|forecast| {
            let controller = HourlyItemModel::builder()
                .launch(HourlyItemInit {
                    hourly: forecast.clone(),
                })
                .detach();

            container.append(controller.widget());
            controllers.push(controller);
        });

        model.controllers = controllers;

        wire_vertical_to_horizontal(&widgets.scroll_window, 32.0);

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
            HourlyInput::Update(hourly) => {
                widgets.widget_container.remove_all();
                let container = widgets.widget_container.clone();
                let mut controllers: Vec<Controller<HourlyItemModel>> = Vec::new();

                hourly.iter().for_each(|forecast| {
                    let controller = HourlyItemModel::builder()
                        .launch(HourlyItemInit {
                            hourly: forecast.clone(),
                        })
                        .detach();

                    container.append(controller.widget());
                    controllers.push(controller);
                });

                self.controllers = controllers;
            }
            HourlyInput::UpdateTemperatureUnit(temperature_unit) => {
                self.temperature_unit = temperature_unit;
            }
        }

        self.update_view(widgets, sender);
    }
}

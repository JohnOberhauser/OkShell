use crate::menus::menu_widgets::weather::current::{CurrentInit, CurrentInput, CurrentModel};
use crate::menus::menu_widgets::weather::daily_graph::{
    DailyGraphInit, DailyGraphInput, DailyGraphModel,
};
use crate::menus::menu_widgets::weather::hourly_graph::{
    HourlyGraphInit, HourlyGraphInput, HourlyGraphModel,
};
use okshell_services::weather_service;
use okshell_utils::weather::spawn_weather_watcher;
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::ops::Not;
use std::time::Duration;
use wayle_weather::{WeatherErrorKind, WeatherStatus};

enum LoadingState {
    Loading,
    Loaded,
    Error,
}

/// Weather widget showing current conditions, the hourly graph, and the
/// daily graph stacked in one vertical column (no paging).
pub(crate) struct WeatherColumnModel {
    current_weather_controller: Option<Controller<CurrentModel>>,
    hourly_graph_controller: Option<Controller<HourlyGraphModel>>,
    daily_graph_controller: Option<Controller<DailyGraphModel>>,
    location: String,
    loading_state: LoadingState,
    error_msg: String,
}

#[derive(Debug)]
pub(crate) enum WeatherColumnInput {
    RetryClicked,
}

#[derive(Debug)]
pub(crate) enum WeatherColumnOutput {}

pub(crate) struct WeatherColumnInit {}

#[derive(Debug)]
pub(crate) enum WeatherColumnCommandOutput {
    WeatherChanged,
}

#[relm4::component(pub)]
impl Component for WeatherColumnModel {
    type CommandOutput = WeatherColumnCommandOutput;
    type Input = WeatherColumnInput;
    type Output = WeatherColumnOutput;
    type Init = WeatherColumnInit;

    view! {
        #[root]
        gtk::Stack {
            add_css_class: "weather-menu-widget",
            set_transition_type: gtk::StackTransitionType::Crossfade,
            set_transition_duration: 250,
            set_vhomogeneous: false,
            #[watch]
            set_visible_child_name: match model.loading_state {
                LoadingState::Loading => "loading",
                LoadingState::Loaded => "loaded",
                LoadingState::Error => "error",
            },

            add_named[Some("loading")] = &gtk::Box {
                set_hexpand: true,

                gtk::Label {
                    add_css_class: "label-medium-bold",
                    set_label: "Weather loading…",
                    set_hexpand: true,
                    set_xalign: 0.5,
                }
            },

            add_named[Some("error")] = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-medium-bold-error",
                    #[watch]
                    set_label: model.error_msg.as_str(),
                    set_hexpand: true,
                    set_xalign: 0.5,
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_halign: gtk::Align::Center,
                    connect_clicked[sender] => move |_| {
                        sender.input(WeatherColumnInput::RetryClicked);
                    },

                    gtk::Label {
                        add_css_class: "label-small",
                        set_label: "Retry",
                    },
                },
            },

            add_named[Some("loaded")] = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small-bold-variant",
                    #[watch]
                    set_label: model.location.as_str(),
                    set_hexpand: true,
                    set_xalign: 0.0
                },

                // Current, hourly graph, and daily graph are appended here
                // once the first weather data arrives.
                #[name = "column"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_hexpand: true,
                }
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_weather_watcher(&sender, || WeatherColumnCommandOutput::WeatherChanged);

        let model = WeatherColumnModel {
            current_weather_controller: None,
            hourly_graph_controller: None,
            daily_graph_controller: None,
            location: "".to_string(),
            loading_state: LoadingState::Loading,
            error_msg: "Error loading weather".to_string(),
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
            WeatherColumnInput::RetryClicked => {
                // there is no retry, so set the poll interval to the same normal value to
                // kick off restarting the polling service.
                weather_service().set_poll_interval(Duration::from_mins(15));
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            WeatherColumnCommandOutput::WeatherChanged => {
                let service = weather_service();

                match service.status.get() {
                    WeatherStatus::Loading => self.loading_state = LoadingState::Loading,
                    WeatherStatus::Loaded => {
                        if let Some(weather) = service.weather.get() {
                            if let Some(controller) = &self.current_weather_controller {
                                // Already built: push the new data to each widget.
                                controller.emit(CurrentInput::Update(
                                    weather.current.clone(),
                                    weather.astronomy.clone(),
                                ));
                                if let Some(controller) = &self.hourly_graph_controller {
                                    controller
                                        .emit(HourlyGraphInput::Update(weather.hourly.clone()));
                                }
                                if let Some(controller) = &self.daily_graph_controller {
                                    controller.emit(DailyGraphInput::Update(weather.daily.clone()));
                                }
                            } else {
                                // First load: build the widgets and add them in order.
                                let current_controller = CurrentModel::builder()
                                    .launch(CurrentInit {
                                        current_weather: weather.current.clone(),
                                        astronomy: weather.astronomy.clone(),
                                    })
                                    .detach();
                                widgets.column.append(current_controller.widget());

                                let hourly_graph_controller = HourlyGraphModel::builder()
                                    .launch(HourlyGraphInit {
                                        hourly: weather.hourly.clone(),
                                    })
                                    .detach();
                                widgets.column.append(hourly_graph_controller.widget());

                                let daily_graph_controller = DailyGraphModel::builder()
                                    .launch(DailyGraphInit {
                                        daily: weather.daily.clone(),
                                    })
                                    .detach();
                                widgets.column.append(daily_graph_controller.widget());

                                self.current_weather_controller = Some(current_controller);
                                self.hourly_graph_controller = Some(hourly_graph_controller);
                                self.daily_graph_controller = Some(daily_graph_controller);
                            }

                            if weather.location.city.is_empty().not() {
                                self.location = format!(
                                    "{}, {}",
                                    weather.location.city.clone(),
                                    if let Some(region) = &weather.location.region {
                                        region
                                    } else {
                                        &weather.location.country
                                    }
                                );
                            } else {
                                self.location = format!(
                                    "{}, {}",
                                    weather.location.lat.clone(),
                                    weather.location.lon.clone(),
                                );
                            }
                        }

                        self.loading_state = LoadingState::Loaded
                    }
                    WeatherStatus::Error(error) => {
                        self.loading_state = LoadingState::Error;
                        self.error_msg = error_message(&error).to_string();
                    }
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

fn error_message(error: &WeatherErrorKind) -> &'static str {
    match error {
        WeatherErrorKind::Network => "Error loading weather. Check network.",
        WeatherErrorKind::ApiKeyMissing { provider: _ } => {
            "Error loading weather. Api key missing."
        }
        WeatherErrorKind::LocationNotFound { query: _ } => {
            "Error loading weather. Location not found."
        }
        WeatherErrorKind::RateLimited => "Error loading weather. Too many requests.",
        WeatherErrorKind::Other => "Error loading weather.",
    }
}

use crate::menus::menu_widgets::weather::daily_graph::{
    DailyGraphInit, DailyGraphInput, DailyGraphModel,
};
use okshell_services::weather_service;
use okshell_utils::weather::spawn_weather_watcher;
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::time::Duration;
use wayle_weather::{WeatherErrorKind, WeatherStatus};

enum LoadingState {
    Loading,
    Loaded,
    Error,
}

/// Weather widget showing current conditions, the hourly graph, and the
/// daily graph stacked in one vertical column (no paging).
pub(crate) struct WeatherDailyGraphModel {
    daily_graph_controller: Option<Controller<DailyGraphModel>>,
    loading_state: LoadingState,
    error_msg: String,
}

#[derive(Debug)]
pub(crate) enum WeatherDailyGraphInput {
    RetryClicked,
}

#[derive(Debug)]
pub(crate) enum WeatherDailyGraphOutput {}

pub(crate) struct WeatherDailyGraphInit {}

#[derive(Debug)]
pub(crate) enum WeatherDailyGraphCommandOutput {
    WeatherChanged,
}

#[relm4::component(pub)]
impl Component for WeatherDailyGraphModel {
    type CommandOutput = WeatherDailyGraphCommandOutput;
    type Input = WeatherDailyGraphInput;
    type Output = WeatherDailyGraphOutput;
    type Init = WeatherDailyGraphInit;

    view! {
        #[root]
        gtk::Stack {
            add_css_class: "weather-menu-widget",
            set_transition_type: gtk::StackTransitionType::Crossfade,
            set_transition_duration: 250,
            set_vhomogeneous: false,

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
                        sender.input(WeatherDailyGraphInput::RetryClicked);
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

                #[name = "column"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_hexpand: true,
                }
            },

            #[watch]
            set_visible_child_name: match model.loading_state {
                LoadingState::Loading => "loading",
                LoadingState::Loaded => "loaded",
                LoadingState::Error => "error",
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_weather_watcher(&sender, || WeatherDailyGraphCommandOutput::WeatherChanged);

        let model = WeatherDailyGraphModel {
            daily_graph_controller: None,
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
            WeatherDailyGraphInput::RetryClicked => {
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
            WeatherDailyGraphCommandOutput::WeatherChanged => {
                let service = weather_service();

                match service.status.get() {
                    WeatherStatus::Loading => self.loading_state = LoadingState::Loading,
                    WeatherStatus::Loaded => {
                        if let Some(weather) = service.weather.get() {
                            if let Some(controller) = &self.daily_graph_controller {
                                // Already built: push the new data to each widget.
                                controller.emit(DailyGraphInput::Update(weather.daily.clone()));
                            } else {
                                let daily_graph_controller = DailyGraphModel::builder()
                                    .launch(DailyGraphInit {
                                        daily: weather.daily.clone(),
                                    })
                                    .detach();
                                widgets.column.append(daily_graph_controller.widget());

                                self.daily_graph_controller = Some(daily_graph_controller);
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

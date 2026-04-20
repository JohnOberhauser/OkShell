use std::ops::Not;
use std::time::Duration;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use wayle_weather::{WeatherErrorKind, WeatherStatus};
use okshell_services::weather_service;
use crate::menus::menu_widgets::weather::current::{CurrentInit, CurrentInput, CurrentModel};
use crate::menus::menu_widgets::weather::daily::{DailyInit, DailyInput, DailyModel};
use crate::menus::menu_widgets::weather::hourly::{HourlyInit, HourlyInput, HourlyModel};
use okshell_utils::weather::{spawn_weather_watcher};

const PAGE_CURRENT: &str = "current";
const PAGE_HOURLY: &str = "hourly";
const PAGE_DAILY: &str = "daily";

enum LoadingState {
    Loading,
    Loaded,
    Error,
}

pub(crate) struct WeatherModel {
    current_weather_controller: Option<Controller<CurrentModel>>,
    hourly_controller: Option<Controller<HourlyModel>>,
    daily_controller: Option<Controller<DailyModel>>,
    location: String,
    current_page: WeatherPage,
    previous_button_sensitive: bool,
    next_button_sensitive: bool,
    loading_state: LoadingState,
    error_msg: String,
}

#[derive(Debug, PartialEq, Clone)]
enum WeatherPage {
    Current,
    Hourly,
    Daily,
}

impl WeatherPage {
    fn stack_name(&self) -> &'static str {
        match self {
            WeatherPage::Current => PAGE_CURRENT,
            WeatherPage::Hourly => PAGE_HOURLY,
            WeatherPage::Daily => PAGE_DAILY,
        }
    }
}

#[derive(Debug)]
pub(crate) enum WeatherInput {
    PreviousClicked,
    NextClicked,
    RetryClicked,
}

#[derive(Debug)]
pub(crate) enum WeatherOutput {}

pub(crate) struct WeatherInit {}

#[derive(Debug)]
pub(crate) enum WeatherCommandOutput {
    WeatherChanged,
}

#[relm4::component(pub)]
impl Component for WeatherModel {
    type CommandOutput = WeatherCommandOutput;
    type Input = WeatherInput;
    type Output = WeatherOutput;
    type Init = WeatherInit;

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
                        sender.input(WeatherInput::RetryClicked);
                    },

                    gtk::Label {
                        add_css_class: "label-small",
                        set_label: "Retry",
                    },
                },
            },

            add_named[Some("loaded")] = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Label {
                        add_css_class: "label-small-bold-variant",
                        #[watch]
                        set_label: model.location.as_str(),
                        set_hexpand: true,
                        set_xalign: 0.0
                    },

                    gtk::Button {
                        add_css_class: "ok-button-surface",
                        set_hexpand: false,
                        set_vexpand: false,
                        #[watch]
                        set_sensitive: model.previous_button_sensitive,
                        connect_clicked[sender] => move |_| {
                            sender.input(WeatherInput::PreviousClicked);
                        },

                        gtk::Image {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_icon_name: Some("menu-left-symbolic"),
                        },
                    },

                    gtk::Button {
                        add_css_class: "ok-button-surface",
                        set_hexpand: false,
                        set_vexpand: false,
                        #[watch]
                        set_sensitive: model.next_button_sensitive,
                        connect_clicked[sender] => move |_| {
                            sender.input(WeatherInput::NextClicked);
                        },

                        gtk::Image {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_icon_name: Some("menu-right-symbolic"),
                        },
                    },
                },

                #[name = "stack"]
                gtk::Stack {
                    set_transition_type: gtk::StackTransitionType::SlideLeftRight,
                    set_transition_duration: 200,
                    set_hexpand: true,
                    set_vexpand: false,
                }
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        spawn_weather_watcher(
            &sender,
            || WeatherCommandOutput::WeatherChanged,
        );

        let model = WeatherModel {
            current_weather_controller: None,
            hourly_controller: None,
            daily_controller: None,
            location: "".to_string(),
            current_page: WeatherPage::Current,
            previous_button_sensitive: false,
            next_button_sensitive: false,
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
            WeatherInput::PreviousClicked => {
                self.current_page = match self.current_page {
                    WeatherPage::Current => {
                        WeatherPage::Current
                    }
                    WeatherPage::Hourly => {
                        WeatherPage::Current
                    }
                    WeatherPage::Daily => {
                        WeatherPage::Hourly
                    }
                };
            }
            WeatherInput::NextClicked => {
                self.current_page = match self.current_page {
                    WeatherPage::Current => {
                        WeatherPage::Hourly
                    }
                    WeatherPage::Hourly => {
                        WeatherPage::Daily
                    }
                    WeatherPage::Daily => {
                        WeatherPage::Daily
                    }
                };
            }
            WeatherInput::RetryClicked => {
                // there is no retry, so set the poll interval to the same normal value to
                // kick off restarting the polling service.
                weather_service().set_poll_interval(Duration::from_mins(15));
            }
        }

        self.update_page_state(&widgets.stack);
        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            WeatherCommandOutput::WeatherChanged => {
                let service = weather_service();

                match service.status.get() {
                    WeatherStatus::Loading => {
                        self.loading_state = LoadingState::Loading
                    }
                    WeatherStatus::Loaded => {
                        if let Some(weather) = service.weather.get() {
                            if self.current_weather_controller.is_some() {
                                if let Some(controller) = &self.current_weather_controller {
                                    controller.emit(CurrentInput::Update(
                                        weather.current.clone(),
                                        weather.astronomy.clone(),
                                    ));
                                }
                                if let Some(controller) = &self.hourly_controller {
                                    controller.emit(HourlyInput::Update(weather.hourly.clone()));
                                }

                                if let Some(controller) = &self.daily_controller {
                                    controller.emit(DailyInput::Update(weather.daily.clone()));
                                }
                            } else {
                                let current_controller = CurrentModel::builder()
                                    .launch(CurrentInit {
                                        current_weather: weather.current.clone(),
                                        astronomy: weather.astronomy.clone(),
                                    })
                                    .detach();

                                widgets.stack.add_titled(
                                    current_controller.widget(),
                                    Some(PAGE_CURRENT),
                                    "Current",
                                );

                                let hourly_controller = HourlyModel::builder()
                                    .launch(HourlyInit {
                                        hourly: weather.hourly.clone(),
                                    })
                                    .detach();

                                widgets.stack.add_titled(
                                    hourly_controller.widget(),
                                    Some(PAGE_HOURLY),
                                    "Hourly",
                                );

                                let daily_controller = DailyModel::builder()
                                    .launch(DailyInit {
                                        daily: weather.daily.clone(),
                                    })
                                    .detach();

                                widgets.stack.add_titled(
                                    daily_controller.widget(),
                                    Some(PAGE_DAILY),
                                    "Daily",
                                );

                                self.current_weather_controller = Some(current_controller);
                                self.hourly_controller = Some(hourly_controller);
                                self.daily_controller = Some(daily_controller);

                                self.update_page_state(&widgets.stack);
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
                        match error {
                            WeatherErrorKind::Network => {
                                self.error_msg = "Error loading weather. Check network.".to_string();
                            }
                            WeatherErrorKind::ApiKeyMissing { provider: _ } => {
                                self.error_msg = "Error loading weather. Api key missing.".to_string();
                            }
                            WeatherErrorKind::LocationNotFound { query: _ } => {
                                self.error_msg = "Error loading weather. Location not found.".to_string();
                            }
                            WeatherErrorKind::RateLimited => {
                                self.error_msg = "Error loading weather. Too many requests.".to_string();
                            }
                            WeatherErrorKind::Other => {
                                self.error_msg = "Error loading weather.".to_string();
                            }
                        };
                    }
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl WeatherModel {
    fn update_page_state(&mut self, stack: &gtk::Stack) {
        let has_pages = self.current_weather_controller.is_some();
        self.previous_button_sensitive = has_pages && self.current_page != WeatherPage::Current;
        self.next_button_sensitive = has_pages && self.current_page != WeatherPage::Daily;
        if has_pages {
            stack.set_visible_child_name(self.current_page.stack_name());
        }
    }
}
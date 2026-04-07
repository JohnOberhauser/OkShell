use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender, Controller};
use relm4::gtk::{
    glib,
};
use relm4::gtk::prelude::{BoxExt, ButtonExt, CastNone, ListModelExt, ObjectExt, OrientableExt, WidgetExt};
use okshell_common::scoped_effects::EffectScope;
use okshell_common::text_entry_dialog::{TextEntryDialogInit, TextEntryDialogModel, TextEntryDialogOutput};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, GeneralStoreFields};
use okshell_config::schema::location_query::{LocationQueryConfig, LocationQueryType, OrdF64};
use okshell_config::schema::temperature::TemperatureUnitConfig;

pub(crate) struct GeneralSettingsModel {
    active_profile: Option<String>,
    available_profiles: gtk::StringList,
    new_profile_dialog: Option<Controller<TextEntryDialogModel>>,
    time_format_24_h: bool,
    location_query_types: gtk::StringList,
    active_location_query_type: LocationQueryType,
    location_lat_lon: String,
    location_city: String,
    lat_lon_dialog: Option<Controller<TextEntryDialogModel>>,
    city_dialog: Option<Controller<TextEntryDialogModel>>,
    weather_unit_types: gtk::StringList,
    active_weather_unit_type: TemperatureUnitConfig,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum GeneralSettingsInput {
    TimeFormat24HToggled(bool),
    TimeFormat24HChanged(bool),
    ActiveProfileChanged(Option<String>),
    AvailableProfilesChanged(Vec<String>),
    NewProfileClicked,
    ActiveProfileSelected(Option<String>),
    NewProfileNameChosen(String),
    DialogCanceled,
    DeleteProfileClicked,
    LocationQueryTypeSelected(LocationQueryType),
    LocationQueryChanged(LocationQueryConfig),
    ChangeCoordinatesClicked,
    ChangeCityClicked,
    LatLonChosen(String, String),
    CityChosen(String, String),
    WeatherUnitTypeSelected(TemperatureUnitConfig),
    WeatherUnitTypeChanged(TemperatureUnitConfig),
}

#[derive(Debug)]
pub(crate) enum GeneralSettingsOutput {}

pub(crate) struct GeneralSettingsInit {}

#[derive(Debug)]
pub(crate) enum GeneralSettingsCommandOutput {}

#[relm4::component(pub)]
impl Component for GeneralSettingsModel {
    type CommandOutput = GeneralSettingsCommandOutput;
    type Input = GeneralSettingsInput;
    type Output = GeneralSettingsOutput;
    type Init = GeneralSettingsInit;

    view! {
        #[root]
        gtk::ScrolledWindow {
            set_vscrollbar_policy: gtk::PolicyType::Automatic,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_height: false,
            set_propagate_natural_width: false,
            set_hexpand: true,
            set_vexpand: true,

            gtk::Box {
                add_css_class: "settings-page",
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 16,

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Profile",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    #[name = "profile_dropdown"]
                    gtk::DropDown {
                        set_hexpand: true,
                        set_model: Some(&model.available_profiles),
                        set_selected: (0..model.available_profiles.n_items())
                            .find(|&i| model.available_profiles.string(i).as_deref() == model.active_profile.as_deref())
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(GeneralSettingsInput::ActiveProfileSelected(selected));
                        },
                    },

                    gtk::Button {
                        set_css_classes: &["label-medium", "ok-button-primary"],
                        set_label: "New Profile",
                        set_halign: gtk::Align::Start,
                        set_hexpand: false,
                        connect_clicked[sender] => move |_| {
                            sender.input(GeneralSettingsInput::NewProfileClicked);
                        },
                    },

                    gtk::Button {
                        set_css_classes: &["label-medium", "ok-button-primary"],
                        #[watch]
                        set_sensitive: model.available_profiles.n_items() > 1,
                        set_label: "Delete Profile",
                        set_halign: gtk::Align::Start,
                        set_hexpand: false,
                        connect_clicked[sender] => move |_| {
                            sender.input(GeneralSettingsInput::DeleteProfileClicked);
                        },
                    },
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Clock",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Label {
                        add_css_class: "label-small",
                        set_halign: gtk::Align::Start,
                        set_label: "24 hour time format",
                        set_hexpand: true,
                    },

                    gtk::Switch {
                        #[watch]
                        #[block_signal(time_format_handler)]
                        set_active: model.time_format_24_h,
                        connect_state_set[sender] => move |_, enabled| {
                            sender.input(GeneralSettingsInput::TimeFormat24HToggled(enabled));
                            glib::Propagation::Proceed
                        } @time_format_handler,
                    }
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Weather",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Location Query Type",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "How to determine the location.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    #[name = "location_query_type_dropdown"]
                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.location_query_types),
                        #[watch]
                        #[block_signal(lqt_handler)]
                        set_selected: LocationQueryType::all()
                            .iter()
                            .position(|k| k == &model.active_location_query_type)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = LocationQueryType::all().get(idx) {
                                sender.input(GeneralSettingsInput::LocationQueryTypeSelected(*kind));
                            }
                        } @lqt_handler,
                    },
                },

                gtk::Box {
                    #[watch]
                    set_visible: model.active_location_query_type == LocationQueryType::Coordinates,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Lat Lon",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            #[watch]
                            set_label: model.location_lat_lon.as_str(),
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::Button {
                        set_css_classes: &["label-medium", "ok-button-primary"],
                        set_label: "Change Coordinates",
                        set_halign: gtk::Align::Start,
                        set_hexpand: false,
                        connect_clicked[sender] => move |_| {
                            sender.input(GeneralSettingsInput::ChangeCoordinatesClicked);
                        },
                    },
                },

                gtk::Box {
                    #[watch]
                    set_visible: model.active_location_query_type == LocationQueryType::City,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "City, Country",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            #[watch]
                            set_label: model.location_city.as_str(),
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::Button {
                        set_css_classes: &["label-medium", "ok-button-primary"],
                        set_label: "Change Location",
                        set_halign: gtk::Align::Start,
                        set_hexpand: false,
                        connect_clicked[sender] => move |_| {
                            sender.input(GeneralSettingsInput::ChangeCityClicked);
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Weather units",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Units in which weather information should be displayed.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    #[name = "weather_unit_type_dropdown"]
                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.weather_unit_types),
                        #[watch]
                        #[block_signal(unit_handler)]
                        set_selected: TemperatureUnitConfig::all()
                            .iter()
                            .position(|k| k == &model.active_weather_unit_type)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = TemperatureUnitConfig::all().get(idx) {
                                sender.input(GeneralSettingsInput::WeatherUnitTypeSelected(*kind));
                            }
                        } @unit_handler,
                    },
                },
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let active_profile = config_manager().active_profile().get();
            sender_clone.input(GeneralSettingsInput::ActiveProfileChanged(active_profile));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let available_profiles = config_manager().available_profiles().get();
            sender_clone.input(GeneralSettingsInput::AvailableProfilesChanged(available_profiles));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let format = config.general().clock_format_24_h().get();
            sender_clone.input(GeneralSettingsInput::TimeFormat24HChanged(format));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let location_query = config.general().weather_location_query().get();
            sender_clone.input(GeneralSettingsInput::LocationQueryChanged(location_query));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.general().temperature_unit().get();
            sender_clone.input(GeneralSettingsInput::WeatherUnitTypeChanged(value));
        });

        let location_query_types = gtk::StringList::new(
            &LocationQueryType::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let weather_unit_types = gtk::StringList::new(
            &TemperatureUnitConfig::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let model = GeneralSettingsModel {
            active_profile: None,
            available_profiles: gtk::StringList::new(&[]),
            new_profile_dialog: None,
            time_format_24_h: false,
            location_query_types,
            active_location_query_type: config_manager().config().general().weather_location_query().get_untracked().kind(),
            location_lat_lon: "0.0, 0.0".to_string(),
            location_city: "Nowhere".to_string(),
            lat_lon_dialog: None,
            city_dialog: None,
            weather_unit_types,
            active_weather_unit_type: config_manager().config().general().temperature_unit().get_untracked(),
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
            GeneralSettingsInput::ActiveProfileSelected(selected_profile) => {
                config_manager().set_active_profile(selected_profile);
            }
            GeneralSettingsInput::ActiveProfileChanged(profile) => {
                self.active_profile = profile;
                let idx = (0..self.available_profiles.n_items())
                    .find(|&i| {
                        self.available_profiles.string(i).as_deref() == self.active_profile.as_deref()
                    })
                    .unwrap_or(0);
                widgets.profile_dropdown.set_selected(idx);
            }
            GeneralSettingsInput::AvailableProfilesChanged(profiles) => {
                // Rebuild the list in-place
                while self.available_profiles.n_items() > 0 {
                    self.available_profiles.remove(0);
                }
                for p in &profiles {
                    self.available_profiles.append(p);
                }
                // Re-sync selected index
                let idx = (0..self.available_profiles.n_items())
                    .find(|&i| {
                        self.available_profiles.string(i).as_deref() == self.active_profile.as_deref()
                    })
                    .unwrap_or(0);
                widgets.profile_dropdown.set_selected(idx);
            }
            GeneralSettingsInput::NewProfileClicked => {
                let dialog = TextEntryDialogModel::builder()
                    .launch(TextEntryDialogInit {
                        message: "Enter new profile name".to_string(),
                        negative_label: "Cancel".to_string(),
                        positive_label: "Create".to_string(),
                        entry_placeholder: "Profile name".to_string(),
                        entry2_placeholder: String::new(),
                        show_second_entry: false,
                    })
                    .forward(sender.input_sender(), |msg| {
                        match msg {
                            TextEntryDialogOutput::PositiveSelected(name, _) => {
                                GeneralSettingsInput::NewProfileNameChosen(name)
                            }
                            TextEntryDialogOutput::NegativeSelected => {
                                GeneralSettingsInput::DialogCanceled
                            }
                        }
                    });

                self.new_profile_dialog = Some(dialog);
            }
            GeneralSettingsInput::NewProfileNameChosen(name) => {
                let _ = config_manager().create_profile(name.as_str());
            }
            GeneralSettingsInput::DialogCanceled => {
                // do nothing
            }
            GeneralSettingsInput::DeleteProfileClicked => {
                if let Some(active) = &self.active_profile {
                    let _ = config_manager().delete_profile(active.as_str());
                }
            }
            GeneralSettingsInput::TimeFormat24HToggled(format) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.general.clock_format_24_h = format;
                });
            }
            GeneralSettingsInput::TimeFormat24HChanged(format) => {
                self.time_format_24_h = format;
            }
            GeneralSettingsInput::LocationQueryTypeSelected(query_type) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.general.weather_location_query = if query_type == LocationQueryType::Coordinates {
                        LocationQueryConfig::Coordinates {
                            lat: OrdF64(0.0),
                            lon: OrdF64(0.0),
                        }
                    } else {
                        LocationQueryConfig::City {
                            name: String::new(),
                            country: String::new(),
                        }
                    };
                });
            }
            GeneralSettingsInput::LocationQueryChanged(query) => {
                match query {
                    LocationQueryConfig::Coordinates { lat, lon } => {
                        self.location_lat_lon = format!("{}, {}", lat.0, lon.0);
                        self.active_location_query_type = LocationQueryType::Coordinates;
                    }
                    LocationQueryConfig::City { name, country } => {
                        self.location_city = format!("{}, {}", name, country);
                        self.active_location_query_type = LocationQueryType::City;
                    }
                }
            }
            GeneralSettingsInput::ChangeCoordinatesClicked => {
                let dialog = TextEntryDialogModel::builder()
                    .launch(TextEntryDialogInit {
                        message: "Enter location coordinates".to_string(),
                        negative_label: "Cancel".to_string(),
                        positive_label: "Done".to_string(),
                        entry_placeholder: "lat".to_string(),
                        entry2_placeholder: "lon".to_string(),
                        show_second_entry: true,
                    })
                    .forward(sender.input_sender(), |msg| {
                        match msg {
                            TextEntryDialogOutput::PositiveSelected(lat, lon) => {
                                GeneralSettingsInput::LatLonChosen(lat, lon)
                            }
                            TextEntryDialogOutput::NegativeSelected => {
                                GeneralSettingsInput::DialogCanceled
                            }
                        }
                    });

                self.lat_lon_dialog = Some(dialog);
            }
            GeneralSettingsInput::ChangeCityClicked => {
                let dialog = TextEntryDialogModel::builder()
                    .launch(TextEntryDialogInit {
                        message: "Enter location details".to_string(),
                        negative_label: "Cancel".to_string(),
                        positive_label: "Done".to_string(),
                        entry_placeholder: "City".to_string(),
                        entry2_placeholder: "Country Code".to_string(),
                        show_second_entry: true,
                    })
                    .forward(sender.input_sender(), |msg| {
                        match msg {
                            TextEntryDialogOutput::PositiveSelected(city, country) => {
                                GeneralSettingsInput::CityChosen(city, country)
                            }
                            TextEntryDialogOutput::NegativeSelected => {
                                GeneralSettingsInput::DialogCanceled
                            }
                        }
                    });

                self.city_dialog = Some(dialog);
            }
            GeneralSettingsInput::LatLonChosen(lat, lon) => {
                if let (Ok(lat), Ok(lon)) = (lat.parse::<f64>(), lon.parse::<f64>()) {
                    config_manager().update_config(|config| {
                        config.general.weather_location_query = LocationQueryConfig::Coordinates {
                            lat: OrdF64(lat),
                            lon: OrdF64(lon),
                        };
                    });
                }
            }
            GeneralSettingsInput::CityChosen(city, country) => {
                config_manager().update_config(|config| {
                    config.general.weather_location_query = LocationQueryConfig::City {
                        name: city,
                        country,
                    }
                })
            }
            GeneralSettingsInput::WeatherUnitTypeSelected(unit) => {
                config_manager().update_config(|config| {
                    config.general.temperature_unit = unit;
                })
            }
            GeneralSettingsInput::WeatherUnitTypeChanged(unit) => {
                self.active_weather_unit_type = unit;
            }
        }

        self.update_view(widgets, sender);
    }
}
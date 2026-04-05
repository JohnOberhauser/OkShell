use std::path::PathBuf;
use reactive_graph::prelude::{GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::{glib};
use relm4::gtk::prelude::{BoxExt, CastNone, ListModelExt, OrientableExt, RangeExt, WidgetExt};
use relm4::prelude::FactoryVecDeque;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, MatugenStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::{MatugenContrast, MatugenMode, MatugenPreference, MatugenType, Themes, WindowOpacity};
use okshell_style::user_css::style_utils::list_available_styles;
use crate::theme_card::{ThemeCardInput, ThemeCardModel, ThemeCardOutput};

#[derive(Debug)]
pub(crate) struct ThemeSettingsModel {
    available_shell_icon_themes: gtk::StringList,
    active_shell_theme: String,
    available_app_icon_themes: gtk::StringList,
    active_apps_theme: String,
    available_css: gtk::StringList,
    active_css: String,
    matugen_preferences: gtk::StringList,
    active_matugen_preference: MatugenPreference,
    matugen_types: gtk::StringList,
    active_matugen_type: MatugenType,
    matugen_modes: gtk::StringList,
    active_matugen_mode: MatugenMode,
    matugen_contrast: f64,
    matugen_contrast_debounce: Option<glib::JoinHandle<()>>,
    window_opacity: f64,
    window_opacity_debounce: Option<glib::JoinHandle<()>>,
    theme_cards: Option<FactoryVecDeque<ThemeCardModel>>,
}

#[derive(Debug)]
pub(crate) enum ThemeSettingsInput {
    ShellIconThemeSelected(Option<String>),
    AppIconThemeSelected(Option<String>),
    MatugenPreferencesSelected(MatugenPreference),
    MatugenTypeSelected(MatugenType),
    MatugenModeSelected(MatugenMode),
    MatugenContrastChanged(f64),
    WindowOpacityChanged(f64),
    ThemeSelected(Themes),
    CssFileSelected(Option<String>),
}

#[derive(Debug)]
pub(crate) enum ThemeSettingsOutput {}

pub(crate) struct ThemeSettingsInit {}

#[derive(Debug)]
pub(crate) enum ThemeSettingsCommandOutput {}

#[relm4::component(pub)]
impl Component for ThemeSettingsModel {
    type CommandOutput = ThemeSettingsCommandOutput;
    type Input = ThemeSettingsInput;
    type Output = ThemeSettingsOutput;
    type Init = ThemeSettingsInit;

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
                    set_label: "Icon Theme",
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
                            set_label: "Shell",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The icons used in OkShell.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    #[name = "shell_icons_dropdown"]
                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.available_shell_icon_themes),
                        set_selected: (0..model.available_shell_icon_themes.n_items())
                            .find(|&i| model.available_shell_icon_themes.string(i).as_deref() == Some(model.active_shell_theme.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::ShellIconThemeSelected(selected));
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
                            set_label: "Apps",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The icons used to represent apps.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    #[name = "app_icons_dropdown"]
                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.available_app_icon_themes),
                        set_selected: (0..model.available_app_icon_themes.n_items())
                            .find(|&i| model.available_app_icon_themes.string(i).as_deref() == Some(model.active_apps_theme.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::AppIconThemeSelected(selected));
                        },
                    },
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Custom CSS",
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
                            set_label: "CSS file",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Custom styles sheets go in ~/.config/okshell/styles/",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.available_css),
                        set_selected: (0..model.available_css.n_items())
                            .find(|&i| model.available_css.string(i).as_deref() == Some(model.active_css.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::CssFileSelected(selected));
                        },
                    },
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Layer Windows",
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
                            set_label: "Opacity",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Value from 0.5 to 1 where 1 is fully opaque.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::Scale {
                        add_css_class: "ok-progress-bar",
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_can_focus: false,
                        set_focus_on_click: false,
                        set_range: (0.5, 1.0),
                        set_increments: (0.1, 0.1),
                        set_value: model.window_opacity,
                        connect_value_changed[sender] => move |scale| {
                            sender.input(ThemeSettingsInput::WindowOpacityChanged(scale.value()));
                        },
                    },
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Matugen",
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
                            set_label: "Type",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Sets a custom color scheme type.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    #[name = "matugen_type_dropdown"]
                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.matugen_types),
                        set_selected: MatugenType::all()
                            .iter()
                            .position(|k| k == &model.active_matugen_type)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = MatugenType::all().get(idx) {
                                sender.input(ThemeSettingsInput::MatugenTypeSelected(*kind));
                            }
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
                            set_label: "Preference",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "When multiple colors can be extracted from an image, this will decide which to pick.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    #[name = "matugen_preference_dropdown"]
                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.matugen_preferences),
                        set_selected: MatugenPreference::all()
                            .iter()
                            .position(|k| k == &model.active_matugen_preference)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = MatugenPreference::all().get(idx) {
                                sender.input(ThemeSettingsInput::MatugenPreferencesSelected(*kind));
                            }
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
                            set_label: "Mode",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Light or dark mode.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    #[name = "matugen_mode_dropdown"]
                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.matugen_modes),
                        set_selected: MatugenMode::all()
                            .iter()
                            .position(|k| k == &model.active_matugen_mode)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = MatugenMode::all().get(idx) {
                                sender.input(ThemeSettingsInput::MatugenModeSelected(*kind));
                            }
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
                            set_label: "Contrast",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Value from -1 to 1. -1 represents minimum contrast, 0 represents standard (i.e. the design as spec'd), and 1 represents maximum contrast.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::Scale {
                        add_css_class: "ok-progress-bar",
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_can_focus: false,
                        set_focus_on_click: false,
                        set_range: (-1.0, 1.0),
                        set_increments: (0.1, 0.1),
                        set_value: model.matugen_contrast,
                        connect_value_changed[sender] => move |scale| {
                            sender.input(ThemeSettingsInput::MatugenContrastChanged(scale.value()));
                        },
                    },
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Color Scheme",
                    set_halign: gtk::Align::Start,
                },

                #[name = "flow_box"]
                gtk::FlowBox {
                    set_max_children_per_line: 2,
                    set_min_children_per_line: 2,
                    set_selection_mode: gtk::SelectionMode::None,
                }
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let app_icon_themes = available_app_icon_themes();
        let theme_refs: Vec<&str> = app_icon_themes.iter().map(|s| s.as_str()).collect();
        let available_app_icon_themes = gtk::StringList::new(&theme_refs);

        let shell_icon_themes = available_shell_icon_themes();
        let shell_theme_refs: Vec<&str> = shell_icon_themes.iter().map(|s| s.as_str()).collect();
        let available_shell_icon_themes = gtk::StringList::new(&shell_theme_refs);

        let mut style_sheets = list_available_styles();
        style_sheets.insert(0, "(none)".to_string());
        let style_refs: Vec<&str> = style_sheets.iter().map(|s| s.as_str()).collect();
        let available_css = gtk::StringList::new(&style_refs);

        let matugen_preferences = gtk::StringList::new(
            &MatugenPreference::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let matugen_types = gtk::StringList::new(
            &MatugenType::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let matugen_modes = gtk::StringList::new(
            &MatugenMode::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let mut model = ThemeSettingsModel {
            available_shell_icon_themes,
            active_shell_theme: config_manager().config().theme().shell_icon_theme().get_untracked(),
            available_app_icon_themes,
            active_apps_theme: config_manager().config().theme().app_icon_theme().get_untracked(),
            available_css,
            active_css: {
                let css = config_manager().config().theme().css_file().get_untracked();
                if css.is_empty() { "(none)".to_string() } else { css }
            },
            matugen_preferences,
            active_matugen_preference: config_manager().config().theme().matugen().preference().get_untracked(),
            matugen_types,
            active_matugen_type: config_manager().config().theme().matugen().scheme_type().get_untracked(),
            matugen_modes,
            active_matugen_mode: config_manager().config().theme().matugen().mode().get_untracked(),
            matugen_contrast: config_manager().config().theme().matugen().contrast().get_untracked().get(),
            matugen_contrast_debounce: None,
            window_opacity: config_manager().config().theme().window_opacity().get_untracked().get(),
            window_opacity_debounce: None,
            theme_cards: None,
        };

        let widgets = view_output!();

        let mut theme_cards = FactoryVecDeque::builder()
            .launch(widgets.flow_box.clone())
            .forward(sender.input_sender(), |msg| match msg {
                ThemeCardOutput::Selected(theme) => ThemeSettingsInput::ThemeSelected(theme),
            });

        {
            let mut guard = theme_cards.guard();
            for theme in Themes::all() {
                guard.push_back(theme.clone());
            }
        }

        model.theme_cards = Some(theme_cards);

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
            ThemeSettingsInput::ShellIconThemeSelected(theme) => {
                if let Some(theme) = theme {
                    config_manager().update_config(|config| {
                        config.theme.shell_icon_theme = theme;
                    });
                }
            }
            ThemeSettingsInput::AppIconThemeSelected(theme) => {
                if let Some(theme) = theme {
                    config_manager().update_config(|config| {
                        config.theme.app_icon_theme = theme;
                    });
                }
            }
            ThemeSettingsInput::MatugenPreferencesSelected(preference) => {
                config_manager().update_config(|config| {
                    config.theme.matugen.preference = preference;
                });
            }
            ThemeSettingsInput::MatugenTypeSelected(scheme_type) => {
                config_manager().update_config(|config| {
                    config.theme.matugen.scheme_type = scheme_type;
                });
            }
            ThemeSettingsInput::MatugenModeSelected(mode) => {
                config_manager().update_config(|config| {
                    config.theme.matugen.mode = mode;
                });
            }
            ThemeSettingsInput::MatugenContrastChanged(contrast) => {
                if let Some(handle) = self.matugen_contrast_debounce.take() {
                    handle.abort();
                }
                self.matugen_contrast_debounce = Some(glib::spawn_future_local(async move {
                    glib::timeout_future(std::time::Duration::from_millis(500)).await;
                    config_manager().update_config(|config| {
                        config.theme.matugen.contrast = MatugenContrast::new(contrast);
                    });
                }));
            }
            ThemeSettingsInput::WindowOpacityChanged(opacity) => {
                if let Some(handle) = self.window_opacity_debounce.take() {
                    handle.abort();
                }
                self.window_opacity_debounce = Some(glib::spawn_future_local(async move {
                    glib::timeout_future(std::time::Duration::from_millis(500)).await;
                    config_manager().update_config(|config| {
                        config.theme.window_opacity = WindowOpacity::new(opacity);
                    });
                }));
            }
            ThemeSettingsInput::ThemeSelected(theme) => {
                config_manager().update_config(|config| {
                    config.theme.theme = theme.clone();
                });

                if let Some(theme_cards) = &mut self.theme_cards {
                    let guard = theme_cards.guard();
                    for i in 0..guard.len() {
                        guard.send(i, ThemeCardInput::SelectionChanged(theme.clone()));
                    }
                }
            }
            ThemeSettingsInput::CssFileSelected(css_file) => {
                config_manager().update_config(|config| {
                    match css_file.as_deref() {
                        Some("(none)") | None => config.theme.css_file = String::new(),
                        Some(file) => config.theme.css_file = file.to_string(),
                    }
                });
            }
        }

        self.update_view(widgets, sender);
    }
}

fn available_shell_icon_themes() -> Vec<String> {
    let mut themes = std::collections::HashSet::new();

    let search_paths = [
        dirs::home_dir().map(|h| h.join(".config/okshell/icons")),
    ];

    for path in search_paths.iter().flatten() {
        let Ok(entries) = std::fs::read_dir(path) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("index.theme").exists() {
                if let Some(name) = entry.file_name().to_str() {
                    themes.insert(name.to_string());
                }
            }
        }
    }

    themes.insert("OkMaterial".to_string());
    themes.insert("OkPhosphor".to_string());

    let mut themes: Vec<_> = themes.into_iter().collect();
    themes.sort();
    themes
}

fn available_app_icon_themes() -> Vec<String> {
    let mut themes = std::collections::HashSet::new();

    let search_paths = [
        dirs::home_dir().map(|h| h.join(".local/share/icons")),
        Some(PathBuf::from("/usr/share/icons")),
        Some(PathBuf::from("/usr/local/share/icons")),
    ];

    for path in search_paths.iter().flatten() {
        let Ok(entries) = std::fs::read_dir(path) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("index.theme").exists() {
                if let Some(name) = entry.file_name().to_str() {
                    themes.insert(name.to_string());
                }
            }
        }
    }
    
    themes.remove("OkPhosphor");
    themes.remove("OkMaterial");

    let mut themes: Vec<_> = themes.into_iter().collect();
    themes.sort();
    themes
}

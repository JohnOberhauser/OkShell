use std::path::PathBuf;
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::{glib};
use relm4::gtk::prelude::{BoxExt, CastNone, ListModelExt, OrientableExt, RangeExt, WidgetExt};
use relm4::prelude::FactoryVecDeque;
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, FontStoreFields, MatugenStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::{MatugenContrast, MatugenMode, MatugenPreference, MatugenType, Themes, WindowOpacity};
use okshell_style::user_css::style_utils::list_available_styles;
use crate::theme_settings::theme_card::{ThemeCardInput, ThemeCardModel, ThemeCardOutput};

#[derive(Debug)]
pub(crate) struct ThemeSettingsModel {
    available_shell_icon_themes: gtk::StringList,
    active_shell_theme: String,
    available_app_icon_themes: gtk::StringList,
    active_apps_theme: String,
    available_fonts: gtk::StringList,
    active_primary_font: String,
    active_secondary_font: String,
    active_tertiary_font: String,
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
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum ThemeSettingsInput {
    ShellIconThemeSelected(Option<String>),
    AppIconThemeSelected(Option<String>),
    MatugenPreferenceSelected(MatugenPreference),
    MatugenTypeSelected(MatugenType),
    MatugenModeSelected(MatugenMode),
    MatugenContrastSelected(f64),
    WindowOpacitySelected(f64),
    ThemeSelected(Themes),
    CssFileSelected(Option<String>),
    PrimaryFontSelected(Option<String>),
    SecondaryFontSelected(Option<String>),
    TertiaryFontSelected(Option<String>),

    ShellIconEffect(String),
    AppIconEffect(String),
    CssFileEffect(String),
    WindowOpacityEffect(f64),
    MatugenTypeEffect(MatugenType),
    MatugenPreferenceEffect(MatugenPreference),
    MatugenModeEffect(MatugenMode),
    MatugenContrastEffect(f64),
    ThemeEffect(Themes),
    PrimaryFontEffect(String),
    SecondaryFontEffect(String),
    TertiaryFontEffect(String),
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
                        #[watch]
                        #[block_signal(shell_handler)]
                        set_selected: (0..model.available_shell_icon_themes.n_items())
                            .find(|&i| model.available_shell_icon_themes.string(i).as_deref() == Some(model.active_shell_theme.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::ShellIconThemeSelected(selected));
                        } @shell_handler,
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
                        #[watch]
                        #[block_signal(app_handler)]
                        set_selected: (0..model.available_app_icon_themes.n_items())
                            .find(|&i| model.available_app_icon_themes.string(i).as_deref() == Some(model.active_apps_theme.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::AppIconThemeSelected(selected));
                        } @app_handler,
                    },
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Font",
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
                            set_label: "Primary font",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "The primary font in OkShell. Sent to matugen as okshell.font.primary",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.available_fonts),
                        #[watch]
                        #[block_signal(primary_font_handler)]
                        set_selected: (0..model.available_fonts.n_items())
                            .find(|&i| model.available_fonts.string(i).as_deref() == Some(model.active_primary_font.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::PrimaryFontSelected(selected));
                        } @primary_font_handler,
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
                            set_label: "Secondary font",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Sent to matugen as okshell.font.secondary",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.available_fonts),
                        #[watch]
                        #[block_signal(secondary_font_handler)]
                        set_selected: (0..model.available_fonts.n_items())
                            .find(|&i| model.available_fonts.string(i).as_deref() == Some(model.active_secondary_font.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::SecondaryFontSelected(selected));
                        } @secondary_font_handler,
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
                            set_label: "Tertiary font",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Sent to matugen as okshell.font.tertiary",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 200,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&model.available_fonts),
                        #[watch]
                        #[block_signal(tertiary_font_handler)]
                        set_selected: (0..model.available_fonts.n_items())
                            .find(|&i| model.available_fonts.string(i).as_deref() == Some(model.active_tertiary_font.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::TertiaryFontSelected(selected));
                        } @tertiary_font_handler,
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
                        #[watch]
                        #[block_signal(css_handler)]
                        set_selected: (0..model.available_css.n_items())
                            .find(|&i| model.available_css.string(i).as_deref() == Some(model.active_css.as_str()))
                            .unwrap_or(0),
                        connect_selected_notify[sender] => move |dd| {
                            let selected = dd.selected_item()
                                .and_downcast::<gtk::StringObject>()
                                .map(|s| s.string().to_string());
                            sender.input(ThemeSettingsInput::CssFileSelected(selected));
                        } @css_handler,
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
                        #[watch]
                        #[block_signal(opacity_handler)]
                        set_value: model.window_opacity,
                        connect_value_changed[sender] => move |scale| {
                            sender.input(ThemeSettingsInput::WindowOpacitySelected(scale.value()));
                        } @opacity_handler,
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
                        #[watch]
                        #[block_signal(type_handler)]
                        set_selected: MatugenType::all()
                            .iter()
                            .position(|k| k == &model.active_matugen_type)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = MatugenType::all().get(idx) {
                                sender.input(ThemeSettingsInput::MatugenTypeSelected(*kind));
                            }
                        } @type_handler,
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
                        #[watch]
                        #[block_signal(preference_handler)]
                        set_selected: MatugenPreference::all()
                            .iter()
                            .position(|k| k == &model.active_matugen_preference)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = MatugenPreference::all().get(idx) {
                                sender.input(ThemeSettingsInput::MatugenPreferenceSelected(*kind));
                            }
                        } @preference_handler,
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
                        #[watch]
                        #[block_signal(mode_handler)]
                        set_selected: MatugenMode::all()
                            .iter()
                            .position(|k| k == &model.active_matugen_mode)
                            .unwrap_or(0) as u32,
                        connect_selected_notify[sender] => move |dd| {
                            let idx = dd.selected() as usize;
                            if let Some(kind) = MatugenMode::all().get(idx) {
                                sender.input(ThemeSettingsInput::MatugenModeSelected(*kind));
                            }
                        } @mode_handler,
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
                        #[watch]
                        #[block_signal(contrast_handler)]
                        set_value: model.matugen_contrast,
                        connect_value_changed[sender] => move |scale| {
                            sender.input(ThemeSettingsInput::MatugenContrastSelected(scale.value()));
                        } @contrast_handler,
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

        let mut fonts = available_fonts();
        fonts.insert(0, "(none)".to_string());
        let font_refs: Vec<&str> = fonts.iter().map(|s| s.as_str()).collect();
        let available_fonts = gtk::StringList::new(&font_refs);

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

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let theme = config.theme().shell_icon_theme().get();
            sender_clone.input(ThemeSettingsInput::ShellIconEffect(theme));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let theme = config.theme().app_icon_theme().get();
            sender_clone.input(ThemeSettingsInput::AppIconEffect(theme));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().css_file().get();
            sender_clone.input(ThemeSettingsInput::CssFileEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().window_opacity().get();
            sender_clone.input(ThemeSettingsInput::WindowOpacityEffect(value.get()));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().scheme_type().get();
            sender_clone.input(ThemeSettingsInput::MatugenTypeEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().preference().get();
            sender_clone.input(ThemeSettingsInput::MatugenPreferenceEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().mode().get();
            sender_clone.input(ThemeSettingsInput::MatugenModeEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().contrast().get();
            sender_clone.input(ThemeSettingsInput::MatugenContrastEffect(value.get()));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().theme().get();
            sender_clone.input(ThemeSettingsInput::ThemeEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().font().primary().get();
            sender_clone.input(ThemeSettingsInput::PrimaryFontEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().font().secondary().get();
            sender_clone.input(ThemeSettingsInput::SecondaryFontEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().font().tertiary().get();
            sender_clone.input(ThemeSettingsInput::TertiaryFontEffect(value));
        });

        let mut model = ThemeSettingsModel {
            available_shell_icon_themes,
            active_shell_theme: config_manager().config().theme().shell_icon_theme().get_untracked(),
            available_app_icon_themes,
            active_apps_theme: config_manager().config().theme().app_icon_theme().get_untracked(),
            available_fonts,
            active_primary_font: config_manager().config().theme().font().primary().get_untracked(),
            active_secondary_font: config_manager().config().theme().font().secondary().get_untracked(),
            active_tertiary_font: config_manager().config().theme().font().tertiary().get_untracked(),
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
            _effects: effects,
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
            ThemeSettingsInput::MatugenPreferenceSelected(preference) => {
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
            ThemeSettingsInput::MatugenContrastSelected(contrast) => {
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
            ThemeSettingsInput::WindowOpacitySelected(opacity) => {
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
            ThemeSettingsInput::PrimaryFontSelected(font) => {
                config_manager().update_config(|config| {
                    match font.as_deref() {
                        Some("(none)") | None => config.theme.font.primary = String::new(),
                        Some(font) => config.theme.font.primary = font.to_string(),
                    }
                });
            }
            ThemeSettingsInput::SecondaryFontSelected(font) => {
                config_manager().update_config(|config| {
                    match font.as_deref() {
                        Some("(none)") | None => config.theme.font.secondary = String::new(),
                        Some(font) => config.theme.font.secondary = font.to_string(),
                    }
                });
            }
            ThemeSettingsInput::TertiaryFontSelected(font) => {
                config_manager().update_config(|config| {
                    match font.as_deref() {
                        Some("(none)") | None => config.theme.font.tertiary = String::new(),
                        Some(font) => config.theme.font.tertiary = font.to_string(),
                    }
                });
            }

            ThemeSettingsInput::ShellIconEffect(theme) => {
                self.active_shell_theme = theme;
            }
            ThemeSettingsInput::AppIconEffect(theme) => {
                self.active_apps_theme = theme;
            }
            ThemeSettingsInput::CssFileEffect(file) => {
                self.active_css = file;
            }
            ThemeSettingsInput::WindowOpacityEffect(opacity) => {
                self.window_opacity = opacity;
            }
            ThemeSettingsInput::MatugenTypeEffect(matugen_type) => {
                self.active_matugen_type = matugen_type;
            }
            ThemeSettingsInput::MatugenPreferenceEffect(preference) => {
                self.active_matugen_preference = preference;
            }
            ThemeSettingsInput::MatugenModeEffect(matugen_mode) => {
                self.active_matugen_mode = matugen_mode;
            }
            ThemeSettingsInput::MatugenContrastEffect(matugen_contrast) => {
                self.matugen_contrast = matugen_contrast;
            }
            ThemeSettingsInput::ThemeEffect(theme) => {
                if let Some(theme_cards) = &mut self.theme_cards {
                    let guard = theme_cards.guard();
                    for i in 0..guard.len() {
                        guard.send(i, ThemeCardInput::SelectionChanged(theme.clone()));
                    }
                }
            }
            ThemeSettingsInput::PrimaryFontEffect(font) => {
                self.active_primary_font = font;
            }
            ThemeSettingsInput::SecondaryFontEffect(font) => {
                self.active_secondary_font = font;
            }
            ThemeSettingsInput::TertiaryFontEffect(font) => {
                self.active_tertiary_font = font;
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

fn available_fonts() -> Vec<String> {
    let Some(fc) = fontconfig::Fontconfig::new() else {
        return vec![];
    };

    let pattern = fontconfig::Pattern::new(&fc);
    let font_set = fontconfig::list_fonts(&pattern, None);

    let mut families = std::collections::HashSet::new();
    for pattern in font_set.iter() {
        if let Some(family) = pattern.get_string(fontconfig::FC_FAMILY) {
            families.insert(family.to_string());
        }
    }

    let mut families: Vec<_> = families.into_iter().collect();
    families.sort();
    families
}

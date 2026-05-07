use crate::menus::menu_widgets::theme_picker::theme_card::{
    ThemeCardInput, ThemeCardModel, ThemeCardOutput,
};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::Themes;
use okshell_matugen::static_theme_mapping::static_theme;
use okshell_utils::scroll_extensions::wire_vertical_to_horizontal;
use reactive_graph::prelude::Get;
use reactive_graph::traits::GetUntracked;
use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, gtk};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeFilter {
    All,
    Light,
    Dark,
}

impl ThemeFilter {
    pub fn all() -> &'static [ThemeFilter] {
        &[ThemeFilter::All, ThemeFilter::Light, ThemeFilter::Dark]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ThemeFilter::All => "All",
            ThemeFilter::Light => "Light",
            ThemeFilter::Dark => "Dark",
        }
    }
}

#[derive(Debug)]
pub(crate) struct ThemePickerMenuWidgetModel {
    theme_cards: Option<FactoryVecDeque<ThemeCardModel>>,
    theme_filters: gtk::StringList,
    active_theme_filter: ThemeFilter,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum ThemePickerMenuWidgetInput {
    ThemeSelected(Themes),
    ThemeEffect(Themes),
    ThemeFilterSelected(ThemeFilter),
}

#[derive(Debug)]
pub(crate) enum ThemePickerMenuWidgetOutput {}

pub(crate) struct ThemePickerMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum ThemePickerMenuWidgetCommandOutput {}

#[relm4::component(pub)]
impl Component for ThemePickerMenuWidgetModel {
    type CommandOutput = ThemePickerMenuWidgetCommandOutput;
    type Input = ThemePickerMenuWidgetInput;
    type Output = ThemePickerMenuWidgetOutput;
    type Init = ThemePickerMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "theme-picker-menu-widget",
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 26,
                set_spacing: 20,

                gtk::Label {
                    add_css_class: "label-xl-bold",
                    set_label: "Color Scheme",
                    set_halign: gtk::Align::Start,
                },

                #[name = "theme_filter_dropdown"]
                gtk::DropDown {
                    set_width_request: 120,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::End,
                    set_model: Some(&model.theme_filters),
                    #[watch]
                    #[block_signal(filter_handler)]
                    set_selected: ThemeFilter::all()
                        .iter()
                        .position(|f| f == &model.active_theme_filter)
                        .unwrap_or(0) as u32,
                    connect_selected_notify[sender] => move |dd| {
                        let idx = dd.selected() as usize;
                        if let Some(filter) = ThemeFilter::all().get(idx) {
                            sender.input(ThemePickerMenuWidgetInput::ThemeFilterSelected(*filter));
                        }
                    } @filter_handler,
                },
            },

            gtk::Overlay {
                add_overlay = &gtk::Box {
                    add_css_class: "wallpaper-shadow",
                    set_hexpand: true,
                    set_vexpand: true,
                    set_can_target: false,
                },

                #[name = "scroll_window"]
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: false,
                    set_vscrollbar_policy: gtk::PolicyType::Never,
                    set_hscrollbar_policy: gtk::PolicyType::External,
                    set_propagate_natural_height: true,

                    #[name = "flow_box"]
                    gtk::Box {
                        add_css_class: "wallpaper-grid",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                    }
                }
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
            let config = config_manager().config();
            let value = config.theme().theme().get();
            sender_clone.input(ThemePickerMenuWidgetInput::ThemeEffect(value));
        });

        let theme_filters = gtk::StringList::new(
            &ThemeFilter::all()
                .iter()
                .map(|f| f.label())
                .collect::<Vec<_>>(),
        );

        let mut model = ThemePickerMenuWidgetModel {
            theme_cards: None,
            theme_filters,
            active_theme_filter: ThemeFilter::All,
            _effects: effects,
        };

        let widgets = view_output!();

        let mut theme_cards = FactoryVecDeque::builder()
            .launch(widgets.flow_box.clone())
            .forward(sender.input_sender(), |msg| match msg {
                ThemeCardOutput::Selected(theme) => {
                    ThemePickerMenuWidgetInput::ThemeSelected(theme)
                }
            });

        {
            let mut guard = theme_cards.guard();
            for theme in Themes::all() {
                guard.push_back(*theme);
            }
        }

        model.theme_cards = Some(theme_cards);

        wire_vertical_to_horizontal(&widgets.scroll_window, 64.0);

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
            ThemePickerMenuWidgetInput::ThemeSelected(theme) => {
                config_manager().update_config(|config| {
                    config.theme.theme = theme;
                });

                if let Some(theme_cards) = &mut self.theme_cards {
                    let guard = theme_cards.guard();
                    for i in 0..guard.len() {
                        guard.send(i, ThemeCardInput::SelectionChanged(theme));
                    }
                }
            }
            ThemePickerMenuWidgetInput::ThemeEffect(theme) => {
                if let Some(theme_cards) = &mut self.theme_cards {
                    let guard = theme_cards.guard();
                    for i in 0..guard.len() {
                        guard.send(i, ThemeCardInput::SelectionChanged(theme));
                    }
                }
            }
            ThemePickerMenuWidgetInput::ThemeFilterSelected(filter) => {
                self.active_theme_filter = filter;
                let Some(theme_cards) = &mut self.theme_cards else {
                    return;
                };

                let mut guard = theme_cards.guard();
                guard.clear();

                for theme in Themes::all() {
                    let is_dark = static_theme(theme, None).map(|t| t.is_dark_mode);

                    let matches = match (self.active_theme_filter, is_dark) {
                        (ThemeFilter::All, _) => true,
                        (ThemeFilter::Light, Some(false)) => true,
                        (ThemeFilter::Dark, Some(true)) => true,
                        _ => false,
                    };

                    if matches {
                        guard.push_back(*theme);
                    }
                }

                // Re-sync selection state on freshly populated cards
                let current = config_manager().config().theme().theme().get_untracked();
                for i in 0..guard.len() {
                    guard.send(i, ThemeCardInput::SelectionChanged(current));
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

use reactive_graph::prelude::Get;
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::Themes;
use okshell_utils::scroll_extensions::wire_vertical_to_horizontal;
use crate::menus::menu_widgets::theme_picker::theme_card::{ThemeCardInput, ThemeCardModel, ThemeCardOutput};

#[derive(Debug)]
pub(crate) struct ThemePickerMenuWidgetModel {
    theme_cards: Option<FactoryVecDeque<ThemeCardModel>>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum ThemePickerMenuWidgetInput {
    ThemeSelected(Themes),
    ThemeEffect(Themes),
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
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 26,
                set_spacing: 20,

                gtk::Label {
                    add_css_class: "label-xl-bold",
                    set_label: "Color Scheme",
                    set_halign: gtk::Align::Start,
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
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
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

        let mut model = ThemePickerMenuWidgetModel {
            theme_cards: None,
            _effects: effects,
        };

        let widgets = view_output!();

        let mut theme_cards = FactoryVecDeque::builder()
            .launch(widgets.flow_box.clone())
            .forward(sender.input_sender(), |msg| match msg {
                ThemeCardOutput::Selected(theme) => ThemePickerMenuWidgetInput::ThemeSelected(theme),
            });

        {
            let mut guard = theme_cards.guard();
            for theme in Themes::all() {
                guard.push_back(theme.clone());
            }
        }

        model.theme_cards = Some(theme_cards);

        wire_vertical_to_horizontal(
            &widgets.scroll_window,
            64.0,
        );

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
                    config.theme.theme = theme.clone();
                });

                if let Some(theme_cards) = &mut self.theme_cards {
                    let guard = theme_cards.guard();
                    for i in 0..guard.len() {
                        guard.send(i, ThemeCardInput::SelectionChanged(theme.clone()));
                    }
                }
            }
            ThemePickerMenuWidgetInput::ThemeEffect(theme) => {
                if let Some(theme_cards) = &mut self.theme_cards {
                    let guard = theme_cards.guard();
                    for i in 0..guard.len() {
                        guard.send(i, ThemeCardInput::SelectionChanged(theme.clone()));
                    }
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
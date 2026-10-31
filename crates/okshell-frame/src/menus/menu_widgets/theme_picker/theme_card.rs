use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::Themes;
use okshell_matugen::static_theme_mapping::static_theme;
use reactive_graph::prelude::GetUntracked;
use relm4::gtk::gdk;
use relm4::gtk::prelude::{BoxExt, DrawingAreaExtManual, OrientableExt, WidgetExt};
use relm4::prelude::FactoryComponent;
use relm4::{FactorySender, gtk};

#[derive(Debug)]
pub(crate) struct ThemeCardModel {
    pub(crate) theme: Themes,
    pub(crate) is_selected: bool,
    pub(crate) colors: Option<[String; 7]>,
}

#[derive(Debug)]
pub(crate) enum ThemeCardInput {
    Clicked,
    SelectionChanged(Themes),
}

#[derive(Debug)]
pub(crate) enum ThemeCardOutput {
    Selected(Themes),
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for ThemeCardModel {
    type Init = Themes;
    type Input = ThemeCardInput;
    type Output = ThemeCardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Overlay {
            set_width_request: 100,
            add_css_class: "theme-card",
            set_overflow: gtk::Overflow::Hidden,

            #[name = "bg"]
            gtk::DrawingArea {
                set_hexpand: true,
                set_height_request: 120,
                set_can_target: false,
            },

            add_overlay = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_vexpand: false,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,
                set_spacing: 4,

                gtk::Label {
                    add_css_class: "label-small",
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    set_vexpand: true,
                    set_margin_top: 4,
                    set_margin_start: 4,
                    set_margin_end: 4,
                    set_wrap: true,
                    set_justify: gtk::Justification::Center,
                    set_xalign: 0.5,
                    #[watch]
                    set_markup: &if let Some(colors) = &self.colors {
                        format!("<span foreground=\"{}\">{}</span>", &colors[1], self.theme.label())
                    } else {
                        self.theme.label().to_string()
                    },
                },

                gtk::Image {
                    add_css_class: "image-surface",
                    set_visible: self.theme == Themes::Default,
                    set_halign: gtk::Align::Center,
                    set_icon_name: Some("palette-symbolic"),
                    set_pixel_size: 60,
                },

                gtk::Image {
                    add_css_class: "image-surface",
                    set_visible: self.theme == Themes::Wallpaper,
                    set_halign: gtk::Align::Center,
                    set_icon_name: Some("wallpaper-symbolic"),
                    set_pixel_size: 60,
                },

                gtk::Box {
                    set_visible: self.theme != Themes::Wallpaper && self.theme != Themes::Default,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_spacing: 8,
                    add_css_class: "theme-swatches",

                    #[name = "swatch1"] gtk::DrawingArea {
                        set_size_request: (16, 16),
                        add_css_class: "theme-card-swatch",
                        set_overflow: gtk::Overflow::Hidden,
                    },
                    #[name = "swatch2"] gtk::DrawingArea {
                       set_size_request: (16, 16),
                       add_css_class: "theme-card-swatch",
                       set_overflow: gtk::Overflow::Hidden,
                    },
                    #[name = "swatch3"] gtk::DrawingArea {
                        set_size_request: (16, 16),
                        add_css_class: "theme-card-swatch",
                        set_overflow: gtk::Overflow::Hidden,
                    },
                },

                gtk::Box {
                    set_visible: self.theme != Themes::Wallpaper && self.theme != Themes::Default,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_spacing: 8,
                    add_css_class: "theme-swatches",

                    #[name = "swatch4"] gtk::DrawingArea {
                        set_size_request: (16, 16),
                        add_css_class: "theme-card-swatch",
                        set_overflow: gtk::Overflow::Hidden,
                    },
                    #[name = "swatch5"] gtk::DrawingArea {
                        set_size_request: (16, 16),
                        add_css_class: "theme-card-swatch",
                        set_overflow: gtk::Overflow::Hidden,
                    },
                    #[name = "swatch6"] gtk::DrawingArea {
                        set_size_request: (16, 16),
                        add_css_class: "theme-card-swatch",
                        set_overflow: gtk::Overflow::Hidden,
                    },
                },

                gtk::Box {
                    set_height_request: 26,
                }
            },

            add_overlay = &gtk::Box {
                gtk::Image {
                    add_css_class: "image-surface",
                    set_icon_name: Some("check-circle-symbolic"),
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::End,
                    set_pixel_size: 20,
                    set_margin_start: 6,
                    set_margin_bottom: 6,
                    #[watch]
                    set_visible: self.is_selected,
                },
            },

            add_controller = gtk::GestureClick::builder().button(1).build() {
                connect_released[sender] => move |_, _, _, _| {
                    sender.input(ThemeCardInput::Clicked);
                },
            },
        }
    }

    fn init_model(
        theme: Self::Init,
        _index: &relm4::prelude::DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        let active_theme = config_manager().config().theme().theme().get_untracked();
        let colors = theme_swatch_colors(&theme);
        Self {
            is_selected: theme == active_theme,
            theme,
            colors,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &relm4::prelude::DynamicIndex,
        root: Self::Root,
        _returned_widget: &gtk::Widget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        self.set_draw_funcs(&widgets);

        widgets
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        match message {
            ThemeCardInput::Clicked => {
                sender.output(ThemeCardOutput::Selected(self.theme)).ok();
            }
            ThemeCardInput::SelectionChanged(active_theme) => {
                self.is_selected = self.theme == active_theme;
            }
        }
        self.update_view(widgets, sender);
    }
}

impl ThemeCardModel {
    fn set_draw_funcs(&self, widgets: &ThemeCardModelWidgets) {
        // Background
        if let Some(colors) = &self.colors {
            let bg_color = colors[0].clone();
            widgets.bg.set_draw_func(move |_, cr, _, _| {
                if let Ok(rgba) = gdk::RGBA::parse(&bg_color) {
                    cr.set_source_rgba(
                        rgba.red() as f64,
                        rgba.green() as f64,
                        rgba.blue() as f64,
                        rgba.alpha() as f64,
                    );
                    let _ = cr.paint();
                }
            });

            // Swatches
            let swatches = [
                (&widgets.swatch1, &colors[1]),
                (&widgets.swatch2, &colors[2]),
                (&widgets.swatch3, &colors[3]),
                (&widgets.swatch4, &colors[4]),
                (&widgets.swatch5, &colors[5]),
                (&widgets.swatch6, &colors[6]),
            ];
            for (swatch, color) in swatches {
                let color = color.clone();
                swatch.set_draw_func(move |_, cr, _, _| {
                    if let Ok(rgba) = gdk::RGBA::parse(&color) {
                        cr.set_source_rgba(
                            rgba.red() as f64,
                            rgba.green() as f64,
                            rgba.blue() as f64,
                            rgba.alpha() as f64,
                        );
                        let _ = cr.paint();
                    }
                });
            }
        }
    }
}

fn theme_swatch_colors(theme: &Themes) -> Option<[String; 7]> {
    let matugen_theme = static_theme(theme, None)?;
    Some([
        matugen_theme.colors.surface.default.color.clone(),
        matugen_theme.colors.on_surface.default.color.clone(),
        matugen_theme.colors.primary.default.color.clone(),
        matugen_theme.colors.secondary.default.color.clone(),
        matugen_theme.colors.tertiary.default.color.clone(),
        matugen_theme.colors.error.default.color.clone(),
        matugen_theme.colors.outline.default.color.clone(),
    ])
}

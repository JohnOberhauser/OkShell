use std::path::PathBuf;
use reactive_graph::effect::Effect;
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::{gdk, CssProvider, STYLE_PROVIDER_PRIORITY_USER};
use tracing::{error, info};
use okshell_cache::wallpaper::{current_wallpaper, wallpaper_store, WallpaperStateStoreFields};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, Font, FontStoreFields, Matugen, SizingStoreFields, ThemeAttributes, ThemeAttributesStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::{Themes, WindowOpacity};
use crate::compiled_css;
use crate::matugen::matugen::{apply_matugen_debounced, apply_matugen_from_theme_debounced};
use crate::matugen::json_struct::{MatugenTheme, MatugenThemeCustomOnly, OkShell};
use crate::matugen::static_theme_mapping::static_theme;
use crate::style_manager::StyleManagerInput::*;
use crate::style_manager::StyleManagerOutput::QueueFrameRedraw;
use crate::user_css::style::StyleStoreFields;
use crate::user_css::user_style_manager::style_manager;

pub struct StyleManagerModel {
    user_css_provider: CssProvider,
    theme_css_provider: CssProvider,
    attributes_css_provider: CssProvider,
}

#[derive(Debug)]
pub enum StyleManagerInput {
    ReloadUserCss(String),
    ReloadTheme(Themes),
    WallpaperUpdate(Option<PathBuf>),
    SetMatugenCssWithWallpaper(PathBuf, Matugen),
    MatugenUpdate(Matugen),
    SetMatugenCssWithStaticTheme(MatugenTheme),
    MatugenComplete(anyhow::Result<String>),
    AttributesUpdate(ThemeAttributes),
}

#[derive(Debug)]
pub enum StyleManagerOutput {
    QueueFrameRedraw,
}

#[relm4::component(pub)]
impl Component for StyleManagerModel {
    type Input = StyleManagerInput;
    type Init = ();
    type Output = StyleManagerOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {}
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let base_css_provider = CssProvider::new();
        let user_css_provider = CssProvider::new();
        let theme_css_provider = CssProvider::new();
        let attributes_css_provider = CssProvider::new();

        let display = gdk::Display::default().expect("No GDK display available");
        gtk::style_context_add_provider_for_display(
            &display,
            &base_css_provider,
            STYLE_PROVIDER_PRIORITY_USER,
        );

        gtk::style_context_add_provider_for_display(
            &display,
            &theme_css_provider,
            STYLE_PROVIDER_PRIORITY_USER + 1,
        );

        gtk::style_context_add_provider_for_display(
            &display,
            &attributes_css_provider,
            STYLE_PROVIDER_PRIORITY_USER + 2,
        );

        gtk::style_context_add_provider_for_display(
            &display,
            &user_css_provider,
            STYLE_PROVIDER_PRIORITY_USER + 3,
        );

        base_css_provider.load_from_string(compiled_css());

        style_manager().watch_style();

        let base_style = style_manager().style();
        let style_sender = sender.clone();
        Effect::new(move || {
            let style = base_style.clone();
            let css = style.css().get();
            style_sender.input(ReloadUserCss(css));
        });

        Effect::new(move || {
            let config = config_manager().config();
            let _ = config.theme().css_file().get();
            style_manager().reload_style();
        });

        let sender_clone = sender.clone();
        Effect::new(move || {
            let config = config_manager().config();
            let theme = config.theme().theme().get();
            sender_clone.input(ReloadTheme(theme));
        });

        let sender_clone = sender.clone();
        Effect::new(move || {
            let store = wallpaper_store();
            let path = store.path().get();
            sender_clone.input(WallpaperUpdate(path));
        });

        let sender_clone = sender.clone();
        Effect::new(move || {
            let config = config_manager().config();
            let matugen = config.theme().matugen().get();
            sender_clone.input(MatugenUpdate(matugen));
        });

        let sender_clone = sender.clone();
        Effect::new(move || {
            let config = config_manager().config();
            let attributes = config.theme().attributes().get();
            sender_clone.input(AttributesUpdate(attributes));
        });

        let model = StyleManagerModel {
            user_css_provider,
            theme_css_provider,
            attributes_css_provider,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ReloadUserCss(css) => {
                self.user_css_provider.load_from_string(&css);
                let _ = sender.output(QueueFrameRedraw);
            }
            ReloadTheme(theme) => {
                if let Some(static_theme) = static_theme(&theme, Some(build_okshell_matugen())) {
                    sender.input(SetMatugenCssWithStaticTheme(static_theme));
                } else {
                    if theme == Themes::Default {
                        self.theme_css_provider.load_from_string("");
                    } else if theme == Themes::Matugen {
                        if let Some(current_wallpaper) = current_wallpaper() {
                            let matugen = config_manager().config().theme().matugen().get_untracked();
                            sender.input(SetMatugenCssWithWallpaper(current_wallpaper, matugen));
                        } else {
                            self.theme_css_provider.load_from_string("");
                        }
                    }
                }
                let _ = sender.output(QueueFrameRedraw);
            }
            WallpaperUpdate(path) => {
                if let Some(path) = path {
                    if config_manager().config().theme().theme().get_untracked() == Themes::Matugen {
                        let matugen = config_manager().config().theme().matugen().get_untracked();
                        sender.input(SetMatugenCssWithWallpaper(path, matugen));
                    }
                }
            }
            MatugenUpdate(matugen) => {
                if config_manager().config().theme().theme().get_untracked() == Themes::Matugen {
                    if let Some(current_wallpaper) = current_wallpaper() {
                        sender.input(SetMatugenCssWithWallpaper(current_wallpaper, matugen));
                    }
                }
            }
            SetMatugenCssWithStaticTheme(theme) => {
                let sender = sender.clone();
                apply_matugen_from_theme_debounced(theme, move |result| {
                    sender.input(MatugenComplete(result));
                });
            }
            SetMatugenCssWithWallpaper(path, matugen) => {
                let theme_overrides = MatugenThemeCustomOnly {
                    okshell: build_okshell_matugen(),
                };
                let sender = sender.clone();
                apply_matugen_debounced(path, matugen, theme_overrides, move |result| {
                    sender.input(MatugenComplete(result));
                });
            }
            MatugenComplete(result) => {
                match result {
                    Ok(css) => {
                        self.theme_css_provider.load_from_string(&css);

                        let _ = sender.output(QueueFrameRedraw);
                    }
                    Err(e) => {
                        error!("Error loading matugen theme: {}", e);
                    }
                }
            }
            AttributesUpdate(attributes) => {
                self.attributes_css_provider.load_from_string(&format!(
                    r#":root {{
                        --font-family-primary: {};
                        --font-family-secondary: {};
                        --font-family-tertiary: {};
                        --window-opacity: {};
                        --radius-widget: {}px;
                        --radius-window: {}px;
                        --border-width: {}px;
                    }}"#,
                    if attributes.font.primary.is_empty() { "inherit" } else { &attributes.font.primary },
                    if attributes.font.secondary.is_empty() { "inherit" } else { &attributes.font.secondary },
                    if attributes.font.tertiary.is_empty() { "inherit" } else { &attributes.font.tertiary },
                    attributes.window_opacity.get(),
                    attributes.sizing.radius_widget,
                    attributes.sizing.radius_window,
                    attributes.sizing.border_width,
                ));

                sender.input(ReloadTheme(config_manager().config().theme().theme().get_untracked()));
            }
        }
    }
}

fn build_okshell_matugen() -> OkShell {
    OkShell {
        font: crate::matugen::json_struct::Font {
            primary: config_manager().config().theme().attributes().font().primary().get_untracked(),
            secondary: config_manager().config().theme().attributes().font().primary().get_untracked(),
            tertiary: config_manager().config().theme().attributes().font().primary().get_untracked(),
        },
        sizing: crate::matugen::json_struct::Sizing {
            radius_widget: config_manager().config().theme().attributes().sizing().radius_widget().get_untracked(),
            radius_window: config_manager().config().theme().attributes().sizing().radius_window().get_untracked(),
            border_width: config_manager().config().theme().attributes().sizing().border_width().get_untracked(),
        },
        opacity: config_manager().config().theme().attributes().window_opacity().get_untracked().get(),
    }
}
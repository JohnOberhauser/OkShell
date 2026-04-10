use std::path::PathBuf;
use gtk4::glib::property::PropertyGet;
use reactive_graph::effect::Effect;
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::{gdk, CssProvider, STYLE_PROVIDER_PRIORITY_USER};
use tracing::{error, info};
use okshell_cache::wallpaper::{current_wallpaper, wallpaper_store, WallpaperStateStoreFields};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, Font, Matugen, ThemeStoreFields};
use okshell_config::schema::themes::{Themes, WindowOpacity};
use crate::compiled_css;
use crate::matugen::matugen::{apply_matugen_debounced, apply_matugen_from_theme_debounced};
use crate::matugen::json_struct::MatugenTheme;
use crate::matugen::static_theme_mapping::static_theme;
use crate::style_manager::StyleManagerInput::*;
use crate::style_manager::StyleManagerOutput::QueueFrameRedraw;
use crate::user_css::style::StyleStoreFields;
use crate::user_css::user_style_manager::style_manager;

pub struct StyleManagerModel {
    user_css_provider: CssProvider,
    theme_css_provider: CssProvider,
    opacity_css_provider: CssProvider,
    font_css_provider: CssProvider,
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
    WindowOpacityUpdate(WindowOpacity),
    FontUpdate(Font),
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
        let opacity_css_provider = CssProvider::new();
        let font_css_provider = CssProvider::new();

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
            &opacity_css_provider,
            STYLE_PROVIDER_PRIORITY_USER + 2,
        );

        gtk::style_context_add_provider_for_display(
            &display,
            &font_css_provider,
            STYLE_PROVIDER_PRIORITY_USER + 3,
        );

        gtk::style_context_add_provider_for_display(
            &display,
            &user_css_provider,
            STYLE_PROVIDER_PRIORITY_USER + 4,
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
            let window_opacity = config.theme().window_opacity().get();
            sender_clone.input(WindowOpacityUpdate(window_opacity));
        });

        let sender_clone = sender.clone();
        Effect::new(move || {
            let config = config_manager().config();
            let font = config.theme().font().get();
            sender_clone.input(FontUpdate(font));
        });

        let model = StyleManagerModel {
            user_css_provider,
            theme_css_provider,
            opacity_css_provider,
            font_css_provider,
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
                if let Some(static_theme) = static_theme(&theme) {
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
                let sender = sender.clone();
                apply_matugen_debounced(path, matugen, move |result| {
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
            WindowOpacityUpdate(opacity) => {
                self.opacity_css_provider.load_from_string(
                    &format!(r#":root {{ --window-opacity: {}; }}"#, opacity.get())
                );
            }
            FontUpdate(font) => {
                info!("applying fonts, {}, {}, {}", font.primary, font.secondary, font.tertiary);
                self.font_css_provider.load_from_string(&format!(
                    r#":root {{
                        --font-family-primary: {};
                        --font-family-secondary: {};
                        --font-family-tertiary: {};
                    }}"#,
                    if font.primary.is_empty() { "inherit" } else { &font.primary },
                    if font.secondary.is_empty() { "inherit" } else { &font.secondary },
                    if font.tertiary.is_empty() { "inherit" } else { &font.tertiary },
                ));
            }
        }
    }
}
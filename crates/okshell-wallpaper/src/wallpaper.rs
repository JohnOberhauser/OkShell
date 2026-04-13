use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use gtk4::prelude::Cast;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::gdk;
use relm4::gtk::glib;
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};
use okshell_cache::wallpaper::{wallpaper_store, WallpaperStateStoreFields};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields, WallpaperStoreFields};
use okshell_config::schema::content_fit::ContentFit;
use okshell_config::schema::themes::Themes;
use okshell_image::lut::{apply_theme_filter};

const TRANSITION_DURATION_MS: u32 = 200;

#[derive(Debug, Clone)]
pub struct WallpaperModel {
    content_fit: ContentFit,
    apply_theme_filter: bool,
    filter_strength: f64,
    theme: Themes,
    path: Option<PathBuf>,
    cancel_token: Arc<AtomicBool>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub enum WallpaperInput {
    PathUpdated(Option<PathBuf>),
    ContentFitChanged(ContentFit),
    ThemeChanged(Themes),
    ApplyThemeChanged(bool),
    FilterStrengthChanged(f64),
    SetWallpaper(Option<PathBuf>, Themes, bool, f64),
}

#[derive(Debug)]
pub enum WallpaperOutput {}

pub struct WallpaperInit {
    pub monitor: gdk::Monitor,
}

#[derive(Debug)]
pub enum WallpaperCommandOutput {
    FilteredReady {
        name: String,
        buf: Vec<u8>,
        width: u32,
        height: u32,
        content_fit: gtk::ContentFit,
    },
}

#[relm4::component(pub)]
impl Component for WallpaperModel {
    type CommandOutput = WallpaperCommandOutput;
    type Input = WallpaperInput;
    type Output = WallpaperOutput;
    type Init = WallpaperInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Window {
            add_css_class: "wallpaper-window",
            set_decorated: false,
            set_visible: true,

            #[name = "stack"]
            gtk::Stack {
                set_transition_type: gtk::StackTransitionType::Crossfade,
                set_transition_duration: TRANSITION_DURATION_MS,
                set_hexpand: true,
                set_vexpand: true,
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_monitor(Some(&params.monitor));
        root.set_namespace(Some("okshell-wallpaper"));
        root.set_layer(Layer::Background);
        root.set_exclusive_zone(-1);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Bottom, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let wallpaper = wallpaper_store();
            let path = wallpaper.path().get();
            sender_clone.input(WallpaperInput::PathUpdated(path));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().content_fit().get();
            sender_clone.input(WallpaperInput::ContentFitChanged(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().apply_theme_filter().get();
            sender_clone.input(WallpaperInput::ApplyThemeChanged(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().theme().theme().get();
            sender_clone.input(WallpaperInput::ThemeChanged(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().theme_filter_strength().get();
            sender_clone.input(WallpaperInput::FilterStrengthChanged(value.get()));
        });

        let model = WallpaperModel {
            content_fit: config_manager().config().wallpaper().content_fit().get_untracked(),
            apply_theme_filter: config_manager().config().wallpaper().apply_theme_filter().get_untracked(),
            filter_strength: config_manager().config().wallpaper().theme_filter_strength().get_untracked().get(),
            theme: config_manager().config().theme().theme().get_untracked(),
            path: None,
            cancel_token: Arc::new(AtomicBool::new(false)),
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
            WallpaperInput::PathUpdated(path) => {
                self.path = path;
                sender.input(WallpaperInput::SetWallpaper(
                    self.path.clone(),
                    self.theme,
                    self.apply_theme_filter,
                    self.filter_strength,
                ))
            }
            WallpaperInput::ContentFitChanged(content_fit) => {
                self.content_fit = content_fit;
                let fit = gtk_content_fit(&self.content_fit);
                let mut child = widgets.stack.first_child();
                while let Some(widget) = child {
                    child = widget.next_sibling();
                    if let Some(picture) = widget.downcast_ref::<gtk::Picture>() {
                        picture.set_content_fit(fit);
                    }
                }
            }
            WallpaperInput::ThemeChanged(theme) => {
                self.theme = theme;
                if self.apply_theme_filter {
                    sender.input(WallpaperInput::SetWallpaper(
                        self.path.clone(),
                        self.theme,
                        self.apply_theme_filter,
                        self.filter_strength,
                    ))
                }
            }
            WallpaperInput::ApplyThemeChanged(apply_theme) => {
                let changed = self.apply_theme_filter != apply_theme;
                self.apply_theme_filter = apply_theme;
                if changed {
                    sender.input(WallpaperInput::SetWallpaper(
                        self.path.clone(),
                        self.theme,
                        self.apply_theme_filter,
                        self.filter_strength,
                    ))
                }
            }
            WallpaperInput::FilterStrengthChanged(filter_strength) => {
                self.filter_strength = filter_strength;
                if self.apply_theme_filter {
                    sender.input(WallpaperInput::SetWallpaper(
                        self.path.clone(),
                        self.theme,
                        self.apply_theme_filter,
                        self.filter_strength,
                    ))
                }
            }
            WallpaperInput::SetWallpaper(
                path,
                theme,
                apply_theme,
                filter_strength,
            ) => {
                if let Some(path) = path {
                    let new_name = format!(
                        "{}{}{}",
                        path.to_string_lossy(),
                        theme.label(),
                        if apply_theme { "t" } else { "f" }
                    );

                    if apply_theme && theme != Themes::Default && theme != Themes::Wallpaper {
                        // cancel any in-flight job
                        self.cancel_token.store(true, Ordering::Relaxed);
                        let cancel_token = Arc::new(AtomicBool::new(false));
                        self.cancel_token = cancel_token.clone();

                        let content_fit = gtk_content_fit(&self.content_fit);

                        sender.command(move |out, _shutdown| async move {
                            let result = tokio::task::spawn_blocking(move || {
                                apply_theme_filter(
                                    &path,
                                    &theme,
                                    &cancel_token,
                                )
                            }).await.ok().flatten();

                            if let Some(result) = result {
                                out.send(WallpaperCommandOutput::FilteredReady {
                                    name: new_name,
                                    buf: result.buf,
                                    width: result.width,
                                    height: result.height,
                                    content_fit,
                                }).ok();
                            }
                        });
                    } else {
                        let stack = &widgets.stack;
                        if let Some(existing) = stack.child_by_name(&new_name) {
                            stack.remove(&existing);
                        }
                        let widget = make_wallpaper_widget(&path, gtk_content_fit(&self.content_fit));
                        let old_child = stack.visible_child();
                        stack.add_named(&widget, Some(&new_name));
                        transition_stack(stack, &new_name, old_child);
                    }
                }
            }
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            WallpaperCommandOutput::FilteredReady { name, buf, width, height, content_fit } => {
                let stack = &widgets.stack;
                if let Some(existing) = stack.child_by_name(&name) {
                    stack.remove(&existing);
                }

                let bytes = glib::Bytes::from_owned(buf);
                let texture = gdk::MemoryTexture::new(
                    width as i32,
                    height as i32,
                    gdk::MemoryFormat::R8g8b8a8,
                    &bytes,
                    (width * 4) as usize,
                );

                let picture = gtk::Picture::for_paintable(&texture);
                picture.set_hexpand(true);
                picture.set_vexpand(true);
                picture.set_content_fit(content_fit);
                picture.set_can_shrink(true);

                let old_child = stack.visible_child();
                stack.add_named(&picture.upcast::<gtk::Widget>(), Some(&name));
                transition_stack(stack, &name, old_child);
            }
        }
    }
}

fn transition_stack(stack: &gtk::Stack, new_name: &str, old_child: Option<gtk::Widget>) {
    let stack_clone = stack.clone();
    let new_name = new_name.to_string();
    glib::idle_add_local_once(move || {
        stack_clone.set_visible_child_name(&new_name);

        if let Some(old) = old_child {
            let stack_clone2 = stack_clone.clone();
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(TRANSITION_DURATION_MS as u64 + 50),
                move || {
                    if old.parent().as_ref() == Some(stack_clone2.upcast_ref()) {
                        stack_clone2.remove(&old);
                    }
                },
            );
        }
    });
}

fn make_wallpaper_widget(
    path: &Path,
    content_fit: gtk::ContentFit,
) -> gtk::Widget {
    let picture = gtk::Picture::for_filename(&path);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_content_fit(content_fit);
    picture.set_can_shrink(true);
    picture.upcast()
}

fn gtk_content_fit(content_fit: &ContentFit) -> gtk::ContentFit {
    match content_fit {
        ContentFit::Contain => gtk::ContentFit::Contain,
        ContentFit::Cover => gtk::ContentFit::Cover,
        ContentFit::Fill => gtk::ContentFit::Fill,
        ContentFit::ScaleDown => gtk::ContentFit::ScaleDown,
    }
}
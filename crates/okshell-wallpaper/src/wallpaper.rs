use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::sync::atomic::{AtomicBool, Ordering};
use gtk4::prelude::{Cast};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use rayon::prelude::*;
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
use okshell_style::matugen::json_struct::{MatugenTheme};
use okshell_style::matugen::static_theme_mapping::static_theme;

const TRANSITION_DURATION_MS: u32 = 200;

#[derive(Debug, Clone)]
pub struct WallpaperModel {
    content_fit: ContentFit,
    apply_theme_filter: bool,
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
    SetWallpaper(Option<PathBuf>, Themes, bool),
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

        let model = WallpaperModel {
            content_fit: config_manager().config().wallpaper().content_fit().get_untracked(),
            apply_theme_filter: config_manager().config().wallpaper().apply_theme_filter().get_untracked(),
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
                    self.apply_theme_filter
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
                sender.input(WallpaperInput::SetWallpaper(
                    self.path.clone(),
                    self.theme,
                    self.apply_theme_filter
                ))
            }
            WallpaperInput::ApplyThemeChanged(apply_theme) => {
                self.apply_theme_filter = apply_theme;
                sender.input(WallpaperInput::SetWallpaper(
                    self.path.clone(),
                    self.theme,
                    self.apply_theme_filter
                ))
            }
            WallpaperInput::SetWallpaper(path, theme, apply_theme) => {
                if let Some(path) = path {
                    let new_name = format!(
                        "{}{}{}",
                        path.to_string_lossy(),
                        theme.label(),
                        if apply_theme { "t" } else { "f" }
                    );

                    let static_theme = static_theme(&theme, None);

                    if apply_theme && static_theme.is_some() {
                        // cancel any in-flight job
                        self.cancel_token.store(true, Ordering::Relaxed);
                        let cancel_token = Arc::new(AtomicBool::new(false));
                        self.cancel_token = cancel_token.clone();

                        let palette = extract_palette(&static_theme.unwrap());
                        let content_fit = gtk_content_fit(&self.content_fit);

                        sender.command(move |out, _shutdown| async move {
                            let result = tokio::task::spawn_blocking(move || {
                                let img = image::open(&path).ok()?.into_rgba8();
                                let (w, h) = img.dimensions();
                                let mut buf = img.into_raw();
                                apply_palette_remap(&mut buf, &palette, 1.0, &cancel_token)?;
                                Some((buf, w, h))
                            }).await.ok().flatten();

                            if let Some((buf, w, h)) = result {
                                out.send(WallpaperCommandOutput::FilteredReady {
                                    name: new_name,
                                    buf,
                                    width: w,
                                    height: h,
                                    content_fit,
                                }).ok();
                            }
                        });
                    } else {
                        // unfiltered path — apply directly on main thread
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

fn extract_palette(theme: &MatugenTheme) -> Vec<(u8, u8, u8)> {
    let mut colors = vec![
        theme.colors.surface.default.as_rgb(),
        theme.colors.on_surface.default.as_rgb(),
        theme.colors.primary.default.as_rgb(),
        theme.colors.secondary.default.as_rgb(),
        theme.colors.tertiary.default.as_rgb(),
        theme.colors.outline.default.as_rgb(),
    ];

    // deduplicate near-identical colors in OkLAB
    dedup_by_perceptual_distance(&mut colors, 0.03);
    colors
}

fn dedup_by_perceptual_distance(colors: &mut Vec<(u8, u8, u8)>, min_distance: f32) {
    let mut kept: Vec<(u8, u8, u8)> = Vec::with_capacity(colors.len());

    for &color in colors.iter() {
        let lab = srgb_to_oklab(color.0, color.1, color.2);

        let too_close = kept.iter().any(|&k| {
            let k_lab = srgb_to_oklab(k.0, k.1, k.2);
            let dl = lab.0 - k_lab.0;
            let da = lab.1 - k_lab.1;
            let db = lab.2 - k_lab.2;
            (dl * dl + da * da + db * db).sqrt() < min_distance
        });

        if !too_close {
            kept.push(color);
        }
    }

    *colors = kept;
}

fn apply_palette_remap(
    buf: &mut [u8],
    palette: &[(u8, u8, u8)],
    strength: f32,
    cancel: &AtomicBool,
) -> Option<()> {
    let lab_palette: Vec<_> = palette.iter()
        .map(|&(r, g, b)| srgb_to_oklab(r, g, b))
        .collect();

    let cancelled = AtomicBool::new(false);

    buf.par_chunks_exact_mut(4).for_each(|pixel| {
        if cancelled.load(Ordering::Relaxed) {
            return;
        }
        if cancel.load(Ordering::Relaxed) {
            cancelled.store(true, Ordering::Relaxed);
            return;
        }

        let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
        let (l, a, bp) = srgb_to_oklab(pixel[0], pixel[1], pixel[2]);

        let (_, best) = lab_palette.iter().enumerate()
            .min_by(|(_, a_lab), (_, b_lab)| {
                let da = (a_lab.0 - l).powi(2) + (a_lab.1 - a).powi(2) + (a_lab.2 - bp).powi(2);
                let db = (b_lab.0 - l).powi(2) + (b_lab.1 - a).powi(2) + (b_lab.2 - bp).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .unwrap();

        let remapped = oklab_to_srgb(l, best.1, best.2);
        pixel[0] = lerp_u8(r, remapped.0, strength);
        pixel[1] = lerp_u8(g, remapped.1, strength);
        pixel[2] = lerp_u8(b, remapped.2, strength);
    });

    if cancel.load(Ordering::Relaxed) { None } else { Some(()) }
}

static SRGB_TO_LINEAR: LazyLock<[f32; 256]> = LazyLock::new(|| {
    let mut table = [0.0f32; 256];
    for i in 0..256 {
        let x = i as f32 / 255.0;
        table[i] = if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        };
    }
    table
});

static LINEAR_TO_SRGB: LazyLock<[u8; 4096]> = LazyLock::new(|| {
    let mut table = [0u8; 4096];
    for i in 0..4096 {
        let x = i as f32 / 4095.0;
        let v = if x <= 0.0031308 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        };
        table[i] = (v * 255.0).clamp(0.0, 255.0) as u8;
    }
    table
});

fn oklab_to_srgb(l: f32, a: f32, b: f32) -> (u8, u8, u8) {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    (
        LINEAR_TO_SRGB[(r.clamp(0.0, 1.0) * 4095.0) as usize],
        LINEAR_TO_SRGB[(g.clamp(0.0, 1.0) * 4095.0) as usize],
        LINEAR_TO_SRGB[(b.clamp(0.0, 1.0) * 4095.0) as usize],
    )
}

fn linearize_inv(x: f32) -> f32 {
    if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8
}

fn srgb_to_oklab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = SRGB_TO_LINEAR[r as usize];
    let g = SRGB_TO_LINEAR[g as usize];
    let b = SRGB_TO_LINEAR[b as usize];

    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    let l = l.cbrt();
    let m = m.cbrt();
    let s = s.cbrt();

    (
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    )
}

fn gtk_content_fit(content_fit: &ContentFit) -> gtk::ContentFit {
    match content_fit {
        ContentFit::Contain => {
            gtk::ContentFit::Contain
        }
        ContentFit::Cover => {
            gtk::ContentFit::Cover
        }
        ContentFit::Fill => {
            gtk::ContentFit::Fill
        }
        ContentFit::ScaleDown => {
            gtk::ContentFit::ScaleDown
        }
    }
}
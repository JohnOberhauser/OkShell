use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use reactive_graph::effect::Effect;
use reactive_graph::prelude::{Get, GetUntracked, Update};
use reactive_stores::{ArcStore, Store};
use relm4::gtk::glib;
use tracing::info;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields, WallpaperStoreFields};
use okshell_config::schema::themes::Themes;
use okshell_image::lut::apply_theme_filter;

// ── Cache paths ──────────────────────────────────────────────────────────────

fn cache_dir() -> PathBuf {
    glib::user_cache_dir().join("okshell")
}

/// The original wallpaper as provided by the user.
pub fn source_path() -> PathBuf {
    cache_dir().join("wallpaper_source.png")
}

/// The wallpaper consumers should display (filtered or copy of source).
pub fn display_path() -> PathBuf {
    cache_dir().join("wallpaper.png")
}

// ── Store ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Store)]
pub struct WallpaperState {
    /// Monotonic counter bumped every time `wallpaper.png` is updated.
    /// Consumers watch this to know when to reload.
    pub revision: u64,
}

struct WallpaperInner {
    cancel_token: Arc<AtomicBool>,
}

static WALLPAPER: LazyLock<ArcStore<WallpaperState>> = LazyLock::new(|| {
    ArcStore::new(WallpaperState {
        revision: if display_path().exists() { 1 } else { 0 },
    })
});

static WALLPAPER_INNER: LazyLock<std::sync::Mutex<WallpaperInner>> = LazyLock::new(|| {
    // React to theme changes
    Effect::new(move |_| {
        let _theme = config_manager().config().theme().theme().get();
        refilter();
    });

    // React to filter toggle
    Effect::new(move |_| {
        let _apply = config_manager().config().wallpaper().apply_theme_filter().get();
        info!("apply_theme_filter effect fired: {_apply}");
        refilter();
    });

    // React to filter strength changes
    Effect::new(move |_| {
        let _strength = config_manager().config().wallpaper().theme_filter_strength().get();
        refilter();
    });

    std::sync::Mutex::new(WallpaperInner {
        cancel_token: Arc::new(AtomicBool::new(false)),
    })
});

pub fn wallpaper_store() -> ArcStore<WallpaperState> {
    // Ensure effects are initialized
    let _ = &*WALLPAPER_INNER;
    WALLPAPER.clone()
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Set a new wallpaper from a source image. Copies it to the cache dir
/// and applies the current theme filter if enabled.
pub fn set_wallpaper(path: &Path) {
    info!("set wallpaper to {}", path.display());
    let dir = cache_dir();
    fs::create_dir_all(&dir).ok();

    // Copy source to cache
    if let Err(e) = fs::copy(path, source_path()) {
        eprintln!("Failed to copy wallpaper source: {e}");
        return;
    }

    refilter();
}

/// Clear the wallpaper entirely.
pub fn clear_wallpaper() {
    fs::remove_file(source_path()).ok();
    fs::remove_file(display_path()).ok();
    bump_revision();
}

/// Returns true if a wallpaper is currently set.
pub fn has_wallpaper() -> bool {
    source_path().exists()
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Re-apply the current filter settings to the source image and write
/// the result to the display path. Called when the wallpaper, theme,
/// or filter config changes.
fn refilter() {
    info!("refilter");
    let source = source_path();
    if !source.exists() {
        return;
    }

    // Cancel any in-flight filter job
    if let Ok(mut inner) = WALLPAPER_INNER.lock() {
        inner.cancel_token.store(true, Ordering::Relaxed);
        let new_token = Arc::new(AtomicBool::new(false));
        inner.cancel_token = new_token.clone();

        let cancel_token = new_token;

        // Determine current config
        let theme = config_manager().config().theme().theme().get_untracked();
        let apply = config_manager().config().wallpaper().apply_theme_filter().get_untracked();
        let strength = config_manager().config().wallpaper().theme_filter_strength().get_untracked().get();

        let should_filter = apply
            && strength != 0.0
            && theme != Themes::Default
            && theme != Themes::Wallpaper;

        std::thread::spawn(move || {
            let success = if should_filter {
                let remap = apply_theme_filter(
                    &source,
                    &theme,
                    strength,
                    &cancel_token,
                );

                if cancel_token.load(Ordering::Relaxed) {
                    false
                } else if let Some(remap) = remap {
                    let img = image::RgbaImage::from_raw(remap.width, remap.height, remap.buf);
                    img.and_then(|img| img.save(display_path()).ok()).is_some()
                } else {
                    false
                }
            } else {
                if cancel_token.load(Ordering::Relaxed) {
                    false
                } else {
                    fs::copy(source, display_path()).is_ok()
                }
            };

            if success {
                // Marshal back to the main thread to bump the reactive store
                glib::idle_add_once(|| {
                    bump_revision();
                });
            }
        });
    }
}

fn bump_revision() {
    WALLPAPER.update(|s| s.revision += 1);
}
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields, WallpaperStoreFields};
use okshell_config::schema::themes::Themes;
use okshell_image::lut::apply_theme_filter;
use reactive_graph::effect::Effect;
use reactive_graph::prelude::{Get, GetUntracked, Update};
use reactive_stores::{ArcStore, Store};
use relm4::gtk::glib;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use tracing::info;

// ── Cache paths ──────────────────────────────────────────────────────────────

fn cache_dir() -> PathBuf {
    glib::user_cache_dir().join("okshell")
}

/// The original wallpaper as provided by the user (raw bytes, any format).
pub fn source_path() -> PathBuf {
    cache_dir().join("wallpaper_source")
}

/// Persisted display wallpaper on disk: [u32 width][u32 height][RGBA pixels]
fn display_cache_path() -> PathBuf {
    cache_dir().join("wallpaper.raw")
}

// ── In-memory wallpaper buffer ───────────────────────────────────────────────

/// Shared RGBA image data ready for direct use with MemoryTexture.
#[derive(Debug, Clone)]
pub struct WallpaperImage {
    pub buf: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

// ── Store ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Store)]
pub struct WallpaperState {
    /// Monotonic counter bumped every time the wallpaper is updated.
    /// Consumers watch this to know when to reload.
    pub revision: u64,
}

struct WallpaperInner {
    cancel_token: Arc<AtomicBool>,
    image: Option<WallpaperImage>,
}

static WALLPAPER: LazyLock<ArcStore<WallpaperState>> =
    LazyLock::new(|| ArcStore::new(WallpaperState { revision: 0 }));

static WALLPAPER_INNER: LazyLock<std::sync::Mutex<WallpaperInner>> = LazyLock::new(|| {
    // Load persisted image from disk if available
    let image = load_from_disk();

    // React to theme changes
    Effect::new(move |_| {
        let _theme = config_manager().config().theme().theme().get();
        refilter();
    });

    // React to filter toggle
    Effect::new(move |_| {
        let _apply = config_manager()
            .config()
            .wallpaper()
            .apply_theme_filter()
            .get();
        refilter();
    });

    // React to filter strength changes
    Effect::new(move |_| {
        let _strength = config_manager()
            .config()
            .wallpaper()
            .theme_filter_strength()
            .get();
        refilter();
    });

    let has_image = image.is_some();

    let inner = std::sync::Mutex::new(WallpaperInner {
        cancel_token: Arc::new(AtomicBool::new(false)),
        image,
    });

    // If we loaded a persisted image, bump revision so consumers pick it up
    if has_image {
        WALLPAPER.update(|s| s.revision = 1);
    }

    inner
});

pub fn wallpaper_store() -> ArcStore<WallpaperState> {
    // Ensure effects are initialized
    let _ = &*WALLPAPER_INNER;
    WALLPAPER.clone()
}

/// Get the current in-memory wallpaper image, if any.
pub fn current_wallpaper_image() -> Option<WallpaperImage> {
    WALLPAPER_INNER.lock().ok()?.image.clone()
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Set a new wallpaper from a source image. Copies it to the cache dir
/// and applies the current theme filter if enabled.
pub fn set_wallpaper(path: &Path) {
    info!("set wallpaper to {}", path.display());
    let dir = cache_dir();
    fs::create_dir_all(&dir).ok();

    if let Err(e) = fs::copy(path, source_path()) {
        eprintln!("Failed to copy wallpaper source: {e}");
        return;
    }

    refilter();
}

/// Clear the wallpaper entirely.
pub fn clear_wallpaper() {
    fs::remove_file(source_path()).ok();
    fs::remove_file(display_cache_path()).ok();
    if let Ok(mut inner) = WALLPAPER_INNER.lock() {
        inner.image = None;
    }
    bump_revision();
}

/// Returns true if a wallpaper is currently set.
pub fn has_wallpaper() -> bool {
    source_path().exists()
}

// ── Internal ─────────────────────────────────────────────────────────────────

fn refilter() {
    let source = source_path();
    if !source.exists() {
        return;
    }

    if let Ok(mut inner) = WALLPAPER_INNER.lock() {
        inner.cancel_token.store(true, Ordering::Relaxed);
        let new_token = Arc::new(AtomicBool::new(false));
        inner.cancel_token = new_token.clone();

        let cancel_token = new_token;

        let theme = config_manager().config().theme().theme().get_untracked();
        let apply = config_manager()
            .config()
            .wallpaper()
            .apply_theme_filter()
            .get_untracked();
        let strength = config_manager()
            .config()
            .wallpaper()
            .theme_filter_strength()
            .get_untracked()
            .get();

        let should_filter =
            apply && strength != 0.0 && theme != Themes::Default && theme != Themes::Wallpaper;

        std::thread::spawn(move || {
            let result = if should_filter {
                apply_theme_filter(&source, &theme, strength, 1.0, 0.0, &cancel_token).map(
                    |remap| WallpaperImage {
                        buf: Arc::new(remap.buf),
                        width: remap.width,
                        height: remap.height,
                    },
                )
            } else if !cancel_token.load(Ordering::Relaxed) {
                decode_source(&source)
            } else {
                None
            };

            if cancel_token.load(Ordering::Relaxed) {
                return;
            }

            if let Some(image) = result {
                // Store in memory
                if let Ok(mut inner) = WALLPAPER_INNER.lock() {
                    inner.image = Some(image.clone());
                }

                glib::idle_add_once(|| {
                    bump_revision();
                });

                // Persist to disk in background (cheap — just a write)
                persist_to_disk(&image);
            }
        });
    }
}

/// Decode any image format into an RGBA WallpaperImage.
fn decode_source(path: &Path) -> Option<WallpaperImage> {
    let img = okshell_image::lut::decode_pixbuf_rgba(path)?;
    let (width, height) = img.dimensions();
    Some(WallpaperImage {
        buf: Arc::new(img.into_raw()),
        width,
        height,
    })
}

// ── Disk persistence ─────────────────────────────────────────────────────────
// Format: [u32 LE width][u32 LE height][width*height*4 bytes RGBA]

fn persist_to_disk(image: &WallpaperImage) {
    let path = display_cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut file = match fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to persist wallpaper: {e}");
            return;
        }
    };
    file.write_all(&image.width.to_le_bytes()).ok();
    file.write_all(&image.height.to_le_bytes()).ok();
    file.write_all(&image.buf).ok();
}

fn load_from_disk() -> Option<WallpaperImage> {
    let path = display_cache_path();
    let mut file = fs::File::open(&path).ok()?;

    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok()?;
    let width = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let height = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);

    let expected_len = (width as usize) * (height as usize) * 4;
    let mut buf = vec![0u8; expected_len];
    file.read_exact(&mut buf).ok()?;

    Some(WallpaperImage {
        buf: Arc::new(buf),
        width,
        height,
    })
}

fn bump_revision() {
    info!("bumping revision");
    WALLPAPER.update(|s| s.revision += 1);
}

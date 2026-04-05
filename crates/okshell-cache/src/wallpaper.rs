use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use reactive_graph::prelude::{ReadUntracked, Update};
use reactive_stores::{ArcStore, Store};
use relm4::gtk::glib;

#[derive(Debug, Clone, PartialEq, Eq, Store)]
pub struct WallpaperState {
    pub path: Option<PathBuf>,
}

static WALLPAPER: LazyLock<ArcStore<WallpaperState>> = LazyLock::new(|| {
    ArcStore::new(WallpaperState {
        path: load_wallpaper(),
    })
});

pub fn wallpaper_store() -> ArcStore<WallpaperState> {
    WALLPAPER.clone()
}

pub fn current_wallpaper() -> Option<PathBuf> {
    wallpaper_store().read_untracked().path.clone()
}

pub fn set_wallpaper(path: PathBuf) {
    wallpaper_store().update(|s| s.path = Some(path));
    persist();
}

pub fn clear_wallpaper() {
    wallpaper_store().update(|s| s.path = None);
    persist();
}

fn persist() {
    let path = wallpaper_store().read_untracked().path.clone();
    if let Err(e) = save_wallpaper(path.as_deref()) {
        eprintln!("Failed to save wallpaper path: {e}");
    }
}

fn wallpaper_file_path() -> PathBuf {
    glib::user_cache_dir().join("okshell").join("wallpaper.txt")
}

fn load_wallpaper() -> Option<PathBuf> {
    let file_path = wallpaper_file_path();
    let file = fs::File::open(&file_path).ok()?;
    let line = BufReader::new(file)
        .lines()
        .find_map(|l| l.ok())
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())?;
    Some(PathBuf::from(line))
}

fn save_wallpaper(path: Option<&Path>) -> std::io::Result<()> {
    let file_path = wallpaper_file_path();
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(&file_path)?;
    if let Some(p) = path {
        writeln!(file, "{}", p.display())?;
    }
    Ok(())
}
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::LazyLock;
use reactive_graph::prelude::{ReadUntracked, Update};
use reactive_stores::{ArcStore, Store};
use relm4::gtk::glib;

#[derive(Debug, Clone, PartialEq, Eq, Store)]
pub struct HiddenAppsState {
    pub apps: Vec<String>, // desktop_ids
}

static HIDDEN_APPS: LazyLock<ArcStore<HiddenAppsState>> = LazyLock::new(|| {
    ArcStore::new(HiddenAppsState {
        apps: load_hidden_apps(),
    })
});

pub fn hidden_apps_store() -> ArcStore<HiddenAppsState> {
    HIDDEN_APPS.clone()
}

pub fn is_hidden(desktop_id: &str) -> bool {
    hidden_apps_store()
        .read_untracked()
        .apps
        .iter()
        .any(|a| a == desktop_id)
}

pub fn hide_app(desktop_id: String) {
    let store = hidden_apps_store();
    if store.read_untracked().apps.iter().any(|a| a == &desktop_id) {
        return;
    }
    store.update(|s| s.apps.push(desktop_id));
    persist();
}

pub fn unhide_app(desktop_id: String) {
    let store = hidden_apps_store();
    let len_before = store.read_untracked().apps.len();
    store.update(|s| s.apps.retain(|a| a != desktop_id.as_str()));
    let changed = store.read_untracked().apps.len() != len_before;
    if changed {
        persist();
    }
}

fn persist() {
    let apps = hidden_apps_store().read_untracked().apps.clone();
    if let Err(e) = save_hidden_apps(&apps) {
        eprintln!("Failed to save hidden apps: {e}");
    }
}

fn hidden_apps_path() -> PathBuf {
    glib::user_cache_dir().join("okshell").join("hidden_apps.txt")
}

fn load_hidden_apps() -> Vec<String> {
    let path = hidden_apps_path();
    match fs::File::open(&path) {
        Ok(file) => BufReader::new(file)
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn save_hidden_apps(apps: &[String]) -> std::io::Result<()> {
    let path = hidden_apps_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(&path)?;
    for app in apps {
        writeln!(file, "{}", app)?;
    }
    Ok(())
}
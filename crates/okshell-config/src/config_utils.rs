use figment::{
    Figment,
    providers::{Format, Serialized, Yaml},
};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use tracing::info;

use notify::{Event, EventKind};
use reactive_stores::{ArcStore, Patch};

use crate::paths::{active_profile_cache_path, profile_path, profiles_dir};
use crate::schema::config::Config;
use reactive_graph::prelude::ReadUntracked;
use serde::Serialize;
use std::fs;
use std::path::Path;

pub(crate) fn read_active_profile_from_cache() -> Option<String> {
    let p = active_profile_cache_path();
    let s = fs::read_to_string(p).ok()?;
    let name = s.trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

pub(crate) fn write_active_profile_to_cache(name: Option<&str>) {
    let p = active_profile_cache_path();
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match name {
        Some(n) => {
            let _ = fs::write(p, n);
        }
        None => {
            let _ = fs::remove_file(p);
        }
    }
}

pub fn list_available_profiles() -> Vec<String> {
    let dir = profiles_dir();
    let mut out = Vec::new();

    let Ok(rd) = fs::read_dir(dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            out.push(stem.to_string());
        }
    }
    out.sort();
    out
}

pub(crate) fn load_effective_config(
    active_profile: Option<&str>,
) -> Result<Config, figment::Error> {
    let mut figment = Figment::from(Serialized::defaults(Config::default()));

    if let Some(name) = active_profile {
        figment = figment.merge(Yaml::file(profile_path(name)));
    }

    figment.extract::<Config>()
}

pub(crate) fn watch_config_loop(
    rx: mpsc::Receiver<notify::Result<Event>>,
    active_profile: ArcStore<Option<String>>,
    available_profiles: ArcStore<Vec<String>>,
    config: ArcStore<Config>,
) {
    let mut pending = false;
    let mut last_event_at = Instant::now();
    const DEBOUNCE_MS: u64 = 200;
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                if is_relevant_config_event(&event) {
                    pending = true;
                    last_event_at = Instant::now();
                }
            }
            Ok(Err(e)) => eprintln!("config: watch error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if pending && last_event_at.elapsed() >= Duration::from_millis(DEBOUNCE_MS) {
            pending = false;
            let active = active_profile.read_untracked();
            let active = active.as_deref();
            match load_effective_config(active) {
                Ok(new_cfg) => {
                    config.patch(new_cfg);
                    available_profiles.patch(list_available_profiles());
                    info!("New config loaded in watch loop");
                }
                Err(e) => eprintln!("config: reload failed (keeping last-good): {e}"),
            }
        }
    }
}

pub(crate) fn is_relevant_config_event(event: &Event) -> bool {
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {}
        _ => return false,
    }
    event.paths.iter().any(|path| {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name.ends_with('~')
                || name.ends_with(".swp")
                || name.ends_with(".swx")
                || name.ends_with(".tmp")
                || name.starts_with(".#")
                || name.starts_with('#')
            {
                return false;
            }
        }
        path.extension().and_then(|s| s.to_str()) == Some("yaml")
    })
}

pub(crate) fn persist_config_layer<T: Serialize>(
    value: &T,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml = serde_yaml::to_string(value)?;

    // Atomic write: write to a sibling temp file, then rename over the target.
    // This prevents the file watcher from seeing a half-written file.
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &yaml)?;
    fs::rename(&tmp, path)?;

    Ok(())
}

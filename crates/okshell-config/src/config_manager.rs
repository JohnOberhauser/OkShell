use std::{fs, sync::{
    mpsc,
    OnceLock,
}, thread};

use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use reactive_graph::prelude::{ReadUntracked};
use reactive_stores::{ArcStore, Patch};
use tracing::{error, info};

use crate::schema::config::{Config};
use crate::paths::*;
use crate::config_utils::*;
use crate::errors::{ProfileCreateError, ProfileDeleteError};

pub struct ConfigManager {
    active_profile: ArcStore<Option<String>>,
    available_profiles: ArcStore<Vec<String>>,
    config: ArcStore<Config>,
}

static CONFIG_MANAGER: OnceLock<ConfigManager> = OnceLock::new();

pub fn config_manager() -> &'static ConfigManager {
    CONFIG_MANAGER.get_or_init(|| {
        ConfigManager::new()
    })
}

impl ConfigManager {

    fn new() -> Self {
        info!("Creating new ConfigManager");
        let active_profile = read_active_profile_from_cache();
        let config = ArcStore::new(
            load_effective_config(active_profile.as_deref())
                .unwrap_or_else(|e| {
                    error!("Error loading config: {}", e);
                    Config::default()
                })
        );
        let active_profile = ArcStore::new(active_profile);

        let available_profiles = ArcStore::new(list_available_profiles());

        Self {
            active_profile,
            available_profiles,
            config,
        }
    }

    pub fn config(&self) -> ArcStore<Config> {
        self.config.clone()
    }

    pub fn active_profile(&self) -> ArcStore<Option<String>> {
        self.active_profile.clone()
    }

    pub fn available_profiles(&self) -> ArcStore<Vec<String>> {
        self.available_profiles.clone()
    }

    /// Sets active profile name (without ".yaml"), persists it, reloads immediately
    pub fn set_active_profile(
        &self,
        name: Option<String>,
    ) {
        self.active_profile.patch(name.clone());

        write_active_profile_to_cache(name.as_deref());

        self.reload_config();
    }

    pub fn create_profile(&self, name: &str) -> Result<(), ProfileCreateError> {
        let path = profile_path(name);
        if path.exists() {
            return Err(ProfileCreateError::AlreadyExists);
        }

        if let Err(e) = persist_config_layer(&Config::default(), &path) {
            return Err(ProfileCreateError::Io(e));
        }

        self.available_profiles.patch(list_available_profiles());
        self.set_active_profile(Some(name.to_string()));
        Ok(())
    }

    pub fn delete_profile(&self, name: &str) -> Result<(), ProfileDeleteError> {
        let path = profile_path(name);
        if !path.exists() {
            return Err(ProfileDeleteError::NotFound);
        }

        // If deleting the active profile, switch away first
        let active = self.active_profile.read_untracked();
        if active.as_deref() == Some(name) {
            drop(active); // release borrow before mutating

            let available = list_available_profiles();
            let fallback = available.iter()
                .find(|p| p.as_str() != name)
                .cloned();
            self.set_active_profile(fallback);
        }

        fs::remove_file(&path).map_err(ProfileDeleteError::Io)?;
        self.available_profiles.patch(list_available_profiles());
        Ok(())
    }

    /// Apply a mutation to the current effective config, persist the change to
    /// the appropriate layer (active profile if one is set, otherwise global
    /// config), then reload so the full effective config reflects it.
    pub fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut Config),
    {
        // Snapshot current effective config and apply the mutation.
        let mut updated = self.config.read_untracked().clone();
        f(&mut updated);

        // Determine which file owns this layer.
        let active = self.active_profile.read_untracked();
        let layer_path = match active.as_deref() {
            Some(name) => profile_path(name),
            None => default_config_path(),
        };

        if let Err(e) = persist_config_layer(&updated, &layer_path) {
            eprintln!("config: failed to persist update: {e}");
            return;
        }

        // Reload so the ArcStore reflects the new effective config.
        if active.is_some() {
            self.reload_config();
        } else {
            self.set_active_profile(Some(DEFAULT_PROFILE_NAME.to_string()));
        }
    }

    pub fn reload_config(&self) {
        let active = self.active_profile.read_untracked();
        let active = active.as_deref();

        match load_effective_config(active) {
            Ok(new_cfg) => {
                self.config.patch(new_cfg);
                info!("Config reloaded");
            }
            Err(e) => {
                eprintln!("config: reload failed (keeping last-good): {e}");
            }
        }
    }

    pub fn watch_config(&self) {
        let active_profile = self.active_profile.clone();
        let available_profiles = self.available_profiles.clone();
        let config = self.config.clone();

        thread::spawn(move || {
            let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
            let mut watcher: RecommendedWatcher =
                RecommendedWatcher::new(tx, NotifyConfig::default())
                    .expect("config: failed to create watcher");

            let prof_dir = profiles_dir();

            if let Err(e) = fs::create_dir_all(&prof_dir) {
                error!("failed to create profiles dir: {e}");
                return;
            }

            let _ = watcher.watch(&prof_dir, RecursiveMode::NonRecursive);

            watch_config_loop(rx, active_profile, available_profiles, config);
        });
    }
}

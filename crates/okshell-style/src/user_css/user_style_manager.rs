use std::{sync::{
    mpsc,
    OnceLock,
}, thread};

use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use reactive_graph::prelude::GetUntracked;
use reactive_stores::{ArcStore, Patch};
use tracing::{error, info};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields};
use crate::user_css::paths::styles_dir;
use crate::user_css::style::Style;
use crate::user_css::style_utils::{load_style, watch_style_loop};

pub struct UserStyleManager {
    style: ArcStore<Style>,
}

static STYLE_MANAGER: OnceLock<UserStyleManager> = OnceLock::new();

pub fn style_manager() -> &'static UserStyleManager {
    STYLE_MANAGER.get_or_init(|| {
        UserStyleManager::new()
    })
}

impl UserStyleManager {
    fn new() -> Self {
        let active_style = config_manager().config().theme().css_file().get_untracked();
        let style = ArcStore::new(
            load_style(active_style)
                .unwrap_or_else(|e| {
                    error!("Error loading style: {}", e);
                    Style::default()
                })
        );

        Self {
            style,
        }
    }

    pub fn style(&self) -> ArcStore<Style> {
        self.style.clone()
    }

    pub fn reload_style(&self) {
        let active_style = config_manager().config().theme().css_file().get_untracked();

        match load_style(active_style) {
            Ok(new_style) => {
                self.style.patch(new_style);
                info!("New style loaded");
            }
            Err(e) => {
                // keep last-good
                eprintln!("reload failed (keeping last-good): {e}");
            }
        }
    }

    pub fn watch_style(&self) {
        let style = self.style.clone();

        thread::spawn(move || {
            let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
            let mut watcher: RecommendedWatcher =
                RecommendedWatcher::new(tx, NotifyConfig::default())
                    .expect("config: failed to create watcher");

            // Watch directories (best for atomic saves: temp + rename)
            let styles_dir = styles_dir();

            if let Err(e) = std::fs::create_dir_all(&styles_dir) {
                error!("failed to create styles dir: {e}");
                return;
            }

            let _ = watcher.watch(&styles_dir, RecursiveMode::NonRecursive);

            watch_style_loop(rx, style);
        });
    }
}

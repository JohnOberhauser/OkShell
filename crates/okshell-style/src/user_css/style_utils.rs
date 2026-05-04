use crate::user_css::paths::{style_path, styles_dir};
use crate::user_css::style::Style;
use notify::{Event, EventKind};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields};
use reactive_graph::prelude::GetUntracked;
use reactive_stores::{ArcStore, Patch};
use std::ops::Not;
use std::{
    fs, io,
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use tracing::info;

pub(crate) fn load_style(active_style: String) -> Result<Style, io::Error> {
    if active_style.is_empty().not() {
        let css = fs::read_to_string(style_path(active_style.as_str()))?;
        Ok(Style { css })
    } else {
        Ok(Style::default())
    }
}

pub fn list_available_styles() -> Vec<String> {
    let dir = styles_dir();
    let mut out = Vec::new();

    let Ok(rd) = fs::read_dir(dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("css") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            out.push(stem.to_string());
        }
    }
    out.sort();
    out
}

pub(crate) fn watch_style_loop(rx: mpsc::Receiver<notify::Result<Event>>, style: ArcStore<Style>) {
    let mut pending = false;
    let mut last_event_at = Instant::now();
    const DEBOUNCE_MS: u64 = 200;

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                let active_style = config_manager().config().theme().css_file().get_untracked();
                let active_style_path: PathBuf = style_path(active_style.as_str());

                if is_relevant_style_event(&event, &active_style_path) {
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

            let active = config_manager().config().theme().css_file().get_untracked();

            match load_style(active) {
                Ok(new_style) => {
                    style.patch(new_style);
                    info!("New style loaded in watch loop");
                }
                Err(e) => eprintln!("style: reload failed (keeping last-good): {e}"),
            }
        }
    }
}

pub(crate) fn is_relevant_style_event(event: &Event, active_style_path: &PathBuf) -> bool {
    // Only respond to writes/creates/removes/renames (not Access/Other)
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {}
        _ => return false,
    }

    event.paths.iter().any(|path| {
        // ignore editor temp files
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name.ends_with("~")
                || name.ends_with(".swp")
                || name.ends_with(".swx")
                || name.ends_with(".tmp")
                || name.starts_with(".#")
                || name.starts_with('#')
            {
                return false;
            }
        }

        if path == active_style_path {
            return true;
        }
        false
    })
}

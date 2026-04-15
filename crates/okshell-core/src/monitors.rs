use std::collections::HashMap;
use relm4::{
    gtk::prelude::*,
    gtk::prelude::DisplayExt,
    gtk::gdk,
    prelude::*,
};
use tracing::info;
use okshell_utils::gtk as utils;
use crate::relm_app::{Shell, ShellInput, WindowGroup};

pub(crate) fn setup_monitor_watcher(
    sender: &ComponentSender<Shell>
) {
    let display = gdk::Display::default().expect("No display");
    let monitors = display.monitors();

    let sender = sender.clone();

    monitors.connect_items_changed(move |model, position, removed, added| {
        if let Some(monitor) = utils::monitor_at_position(model, position) {
            monitor.model();
            monitor.connector();
        };

        info!(position, removed, added, "Monitor changed");
        sender.input(ShellInput::SyncMonitors);
    });
}

pub(crate) fn sync_monitors(
    window_groups: &HashMap<String, WindowGroup>,
    sender: &ComponentSender<Shell>,
) {
    let display = gdk::Display::default().expect("No display");
    let monitors = utils::list_model_to_monitors(&display.monitors());

    // Remove stale windows
    info!("Checking for stale windows");
    let connectors_in_monitors: Vec<String> = monitors
        .iter()
        .filter_map(|m| m.connector().map(|c| c.to_string()))
        .collect();

    let stale_connectors: Vec<String> = window_groups
        .keys()
        .filter(|connector| !connectors_in_monitors.contains(&connector))
        .cloned()
        .collect();

    for connector in stale_connectors {
        sender.input(ShellInput::RemoveWindowGroup(connector));
    }

    // Add windows to monitor
    info!("Adding windows to new monitors");
    monitors.iter().for_each(|monitor| {
        if let Some(connector) = monitor.connector() {
            let connector = connector.to_string();
            if !window_groups.contains_key(&connector) {
                sender.input(ShellInput::AddWindowGroup(connector, monitor.clone()))
            }
        }
    })
}

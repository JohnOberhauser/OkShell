use crate::relm_app::{Shell, ShellInput, WindowGroup};
use okshell_utils::gtk as utils;
use relm4::gtk::glib::SignalHandlerId;
use relm4::{gtk::gdk, gtk::prelude::DisplayExt, gtk::prelude::*, prelude::*};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::info;

pub(crate) fn setup_monitor_watcher(sender: &ComponentSender<Shell>) {
    let display = gdk::Display::default().expect("No display");
    let monitors = display.monitors();
    let sender_clone = sender.clone();

    monitors.connect_items_changed(move |model, position, _removed, added| {
        for i in position..position + added {
            if let Some(monitor) = utils::monitor_at_position(model, i) {
                let sender_inner = sender_clone.clone();
                let handler_id: Rc<RefCell<Option<SignalHandlerId>>> = Rc::new(RefCell::new(None));
                let handler_id_clone = handler_id.clone();

                let id = monitor.connect_notify_local(Some("connector"), move |m, _| {
                    if m.connector().is_some() {
                        sender_inner.input(ShellInput::SyncMonitors);
                        if let Some(id) = handler_id_clone.borrow_mut().take() {
                            m.disconnect(id);
                        }
                    }
                });
                *handler_id.borrow_mut() = Some(id);
            }
        }
        sender_clone.input(ShellInput::SyncMonitors);
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

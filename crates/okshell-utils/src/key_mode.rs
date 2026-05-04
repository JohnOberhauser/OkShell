use gtk4_layer_shell;
use gtk4_layer_shell::LayerShell;
use relm4::gtk::prelude::WidgetExt;
use relm4::{RelmWidgetExt, gtk};

pub fn wire_entry_focus(entry: &gtk::Entry) {
    entry.connect_realize(|entry| {
        let fc = gtk::EventControllerFocus::new();
        if let Some(window) = entry.toplevel_window() {
            let window_clone = window.clone();
            fc.connect_enter(move |_| {
                window_clone.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            });

            let window_clone = window.clone();
            fc.connect_leave(move |_| {
                window_clone.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
            });

            entry.add_controller(fc);
        }
    });
}

use crate::settings::{SettingsWindowInit, SettingsWindowModel};
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{Component, ComponentController};
use std::cell::RefCell;

mod bar_settings;
mod general_settings;
mod menu_settings;
mod notification_settings;
pub mod settings;
mod theme_settings;
mod wallpaper_settings;

thread_local! {
    static SETTINGS_ROOT: RefCell<Option<relm4::Controller<SettingsWindowModel>>> = const { RefCell::new(None) };
}

pub fn open_settings() {
    let already_open =
        SETTINGS_ROOT.with(|w| w.borrow().as_ref().is_some_and(|c| c.widget().is_visible()));

    if already_open {
        SETTINGS_ROOT.with(|w| {
            if let Some(c) = w.borrow().as_ref() {
                c.widget().present();
            }
        });
        return;
    }

    let controller = SettingsWindowModel::builder()
        .launch(SettingsWindowInit {})
        .detach();

    SETTINGS_ROOT.with(|w| {
        *w.borrow_mut() = Some(controller);
    });
}

pub fn close_settings() {
    SETTINGS_ROOT.with(|w| {
        if let Some(c) = w.borrow().as_ref() {
            c.widget().close();
        }
    });
    // Drop the controller outside the borrow to avoid the double-borrow panic
    SETTINGS_ROOT.with(|w| {
        *w.borrow_mut() = None;
    });
}

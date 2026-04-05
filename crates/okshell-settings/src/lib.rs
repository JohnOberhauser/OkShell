use std::cell::RefCell;
use relm4::{gtk, Component, ComponentController};
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};
use crate::settings::{SettingsWindowInit, SettingsWindowModel};

mod general_settings;
pub mod settings;
mod wallpaper_settings;
mod theme_settings;
mod theme_card;
mod bar_settings;
mod menu_settings;
mod notification_settings;

thread_local! {
    static SETTINGS_ROOT: RefCell<Option<gtk::Window>> = RefCell::new(None);
}

pub fn open_settings() {
    let already_open = SETTINGS_ROOT.with(|w| {
        w.borrow().as_ref().is_some_and(|win| win.is_visible())
    });
    if already_open {
        // Optionally present/focus the existing window
        SETTINGS_ROOT.with(|w| {
            if let Some(win) = w.borrow().as_ref() {
                win.present();
            }
        });
        return;
    }
    let controller = SettingsWindowModel::builder()
        .launch(SettingsWindowInit {})
        .detach();
    let window = controller.widget().clone();
    SETTINGS_ROOT.with(|w| {
        *w.borrow_mut() = Some(window);
    });
    std::mem::forget(controller);
}

pub fn close_settings() {
    let already_open = SETTINGS_ROOT.with(|w| {
        w.borrow().as_ref().is_some_and(|win| win.is_visible())
    });
    if already_open {
        // Optionally present/focus the existing window
        SETTINGS_ROOT.with(|w| {
            if let Some(win) = w.borrow().as_ref() {
                win.close();
                SETTINGS_ROOT.set(None);
            }
        });
        return;
    }
}
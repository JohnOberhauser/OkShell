use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use relm4::gtk;
use relm4::gtk::{gio, glib};
use relm4::gtk::gio::DesktopAppInfo;
use relm4::gtk::prelude::{AppInfoExt, Cast, FileExt};
use okshell_config::schema::themes::Themes;
use okshell_image::lut::{apply_theme_filter, embedded_clut, rgba_to_texture};
use crate::app_icon::icon_index::IconIndex;

pub fn set_icon(
    app_info: &Option<DesktopAppInfo>,
    hyprland_class: &Option<String>,
    image: &gtk::Image,
    theme: String,
    color_theme: &Themes,
    apply_filter: bool,
) {
    let should_recolor = apply_filter && embedded_clut(color_theme).is_some();

    let app = match app_info {
        Some(app) => app,
        None => {
            image.set_icon_name(Some("application-x-executable"));
            return;
        }
    };
    let icon = match app.icon() {
        Some(icon) => icon,
        None => {
            image.set_icon_name(Some("application-x-executable"));
            return;
        }
    };

    let icon_name = resolve_icon_name(&icon, hyprland_class);
    let image = image.clone();
    let color_theme = color_theme.clone();

    // Also grab the direct file path if it's a FileIcon
    let file_icon_path = icon.downcast_ref::<gio::FileIcon>()
        .and_then(|fi| fi.file().path());

    glib::spawn_future_local(async move {
        // Build/fetch the index off-thread (cached after first call)
        let theme_clone = theme.clone();
        let icon_name_clone = icon_name.clone();
        let path = gio::spawn_blocking(move || {
            let index = IconIndex::get_or_build(&theme_clone);
            icon_name_clone.and_then(|name| index.lookup(&name).cloned())
        })
            .await
            .ok()
            .flatten()
            .or(file_icon_path);

        let Some(path) = path else {
            image.set_icon_name(Some("application-x-executable"));
            return;
        };

        if should_recolor {
            let result = gio::spawn_blocking(move || {
                let cancel = AtomicBool::new(false);
                apply_theme_filter(&path, &color_theme, 1.0, &cancel)
            })
                .await;

            if let Ok(Some(r)) = result {
                if let Some(texture) = rgba_to_texture(&r.buf, r.width, r.height) {
                    image.set_paintable(Some(&texture));
                    return;
                }
            }
        } else {
            let result = gio::spawn_blocking(move || {
                gtk::gdk::Texture::from_filename(&path).ok()
            })
                .await;

            if let Ok(Some(texture)) = result {
                image.set_paintable(Some(&texture));
                return;
            }
        }

        image.set_icon_name(Some("application-x-executable"));
    });
}

fn resolve_icon_name(icon: &gio::Icon, hyprland_class: &Option<String>) -> Option<String> {
    if let Some(themed) = icon.downcast_ref::<gio::ThemedIcon>() {
        return themed.names().first().map(|n| n.to_string());
    }
    if let Some(class) = hyprland_class {
        return Some(class.to_lowercase());
    }
    None
}

fn spawn_load_and_recolor(image: &gtk::Image, path: PathBuf, color_theme: Themes) {
    let image = image.clone();
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(move || {
            let cancel = AtomicBool::new(false);
            apply_theme_filter(&path, &color_theme, 1.0, &cancel)
        })
            .await;

        if let Ok(Some(r)) = result {
            if let Some(texture) = rgba_to_texture(&r.buf, r.width, r.height) {
                image.set_paintable(Some(&texture));
                return;
            }
        }
        image.set_icon_name(Some("application-x-executable"));
    });
}

fn spawn_load(image: &gtk::Image, path: PathBuf) {
    let image = image.clone();
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(move || {
            gtk::gdk::Texture::from_filename(&path).ok()
        })
            .await;

        if let Ok(Some(texture)) = result {
            image.set_paintable(Some(&texture));
        } else {
            image.set_icon_name(Some("application-x-executable"));
        }
    });
}
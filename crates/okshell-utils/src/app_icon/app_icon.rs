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
    filter_strength: f64,
    monochrome_strength: f64,
    contrast_strength: f64,
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

    let image = image.clone();
    let color_theme = color_theme.clone();
    let hyprland_class = hyprland_class.clone();

    // Also grab the direct file path if it's a FileIcon
    let file_icon_path = icon.downcast_ref::<gio::FileIcon>()
        .and_then(|fi| fi.file().path());

    glib::spawn_future_local(async move {
        // Build/fetch the index off-thread (cached after first call)
        let theme_clone = theme.clone();
        let candidates = resolve_icon_candidates(&icon, &hyprland_class);
        let path = gio::spawn_blocking(move || {
            let index = IconIndex::get_or_build(&theme_clone);
            for name in &candidates {
                if let Some(path) = index.lookup(name) {
                    return Some(path.clone());
                }
            }
            None
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
                apply_theme_filter(&path, &color_theme, filter_strength, contrast_strength, monochrome_strength, &cancel)
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

fn resolve_icon_candidates(
    icon: &gio::Icon,
    hyprland_class: &Option<String>,
) -> Vec<String> {
    let mut candidates = Vec::new();

    // ThemedIcon names (first is primary, rest are fallbacks)
    if let Some(themed) = icon.downcast_ref::<gio::ThemedIcon>() {
        for name in themed.names() {
            let name_string = name.to_string();
            // Some apps like jetbrains IDEs add uuid suffixes to their icons.  Try removing the
            // suffix if it's there.
            if let Some(stripped) = strip_uuid_suffix(&name_string) {
                candidates.push(stripped);
            }
            candidates.push(name_string);
        }
    }

    // FileIcon path walk — derive names from parent directories
    if let Some(file_icon) = icon.downcast_ref::<gio::FileIcon>() {
        if let Some(path) = file_icon.file().path() {
            let mut dir = path.as_path();
            for _ in 0..4 {
                if let Some(stem) = dir.file_stem().and_then(|s| s.to_str()) {
                    candidates.push(stem.to_string());
                }
                match dir.parent() {
                    Some(p) => dir = p,
                    None => break,
                }
            }
        }
    }

    // Hyprland class
    if let Some(class) = hyprland_class {
        candidates.push(class.to_lowercase());
    }

    candidates
}

fn strip_uuid_suffix(name: &str) -> Option<String> {
    // Match trailing -xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    let re = regex::Regex::new(r"-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
    let stripped = re.replace(name, "");
    if stripped != name {
        Some(stripped.into_owned())
    } else {
        None
    }
}

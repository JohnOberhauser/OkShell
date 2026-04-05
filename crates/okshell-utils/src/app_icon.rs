use relm4::gtk;
use relm4::gtk::gio;
use relm4::gtk::gio::{DesktopAppInfo};
use relm4::gtk::prelude::{AppInfoExt, Cast, FileExt};

pub fn set_icon(
    app_info: &Option<DesktopAppInfo>,
    hyprland_class: &Option<String>,
    image: &gtk::Image, 
    theme: String
) {
    let icon_theme = gtk::IconTheme::new();
    icon_theme.set_theme_name(Some(theme.as_str()));

    let app = match app_info {
        Some(app) => app,
        None => {
            image.set_icon_name(Some("application-x-executable"));
            return
        },
    };

    let icon = match app.icon() {
        Some(icon) => icon,
        None => {
            image.set_icon_name(Some("application-x-executable"));
            return
        },
    };

    // Strategy 1: ThemedIcon — try all names from the icon
    if let Some(themed) = icon.downcast_ref::<gio::ThemedIcon>() {
        let names = themed.names();
        let name_strs: Vec<&str> = names.iter().map(|n| n.as_str()).collect();
        if icon_theme.has_icon(name_strs[0]) {
            let paintable = lookup_icon(&icon_theme, name_strs[0], &name_strs[1..]);
            image.set_paintable(Some(&paintable));
            return;
        }
    }

    // Strategy 2: Hyprland class name
    if let Some(class) = hyprland_class {
        let class_lower = class.to_lowercase();
        if icon_theme.has_icon(&class_lower) {
            let paintable = lookup_icon(&icon_theme, &class_lower, &[]);
            image.set_paintable(Some(&paintable));
            return;
        }
    }

    // Strategy 3: Walk parent directories of a FileIcon path
    if let Some(file_icon) = icon.downcast_ref::<gio::FileIcon>() {
        if let Some(name) = find_icon_from_path(&icon_theme, &file_icon.file()) {
            let paintable = lookup_icon(&icon_theme, &name, &[]);
            image.set_paintable(Some(&paintable));
            return;
        }
    }

    // Strategy 4: use the raw GIcon
    if let Some(icon) = app.icon().filter(|i| has_usable_icon(i)) {
        image.set_from_gicon(&icon);
        return;
    }

    // Fallback: use a default icon
    image.set_icon_name(Some("application-x-executable"));
}

fn has_usable_icon(icon: &gio::Icon) -> bool {
    if let Some(themed) = icon.downcast_ref::<gio::ThemedIcon>() {
        let display = gtk::gdk::Display::default().unwrap();
        let default_theme = gtk::IconTheme::for_display(&display);
        themed.names().iter().any(|n| default_theme.has_icon(n.as_str()))
    } else if let Some(file_icon) = icon.downcast_ref::<gio::FileIcon>() {
        file_icon.file().path().map_or(false, |p| p.exists())
    } else {
        false
    }
}

fn lookup_icon(
    icon_theme: &gtk::IconTheme, 
    name: &str, 
    fallbacks: &[&str]
) -> gtk::IconPaintable {
    icon_theme.lookup_icon(
        name,
        fallbacks,
        48,
        1,
        gtk::TextDirection::Ltr,
        gtk::IconLookupFlags::empty(),
    )
}

fn find_icon_from_path(
    icon_theme: &gtk::IconTheme, 
    file: &gio::File
) -> Option<String> {
    let path = file.path()?;
    let mut dir = path.parent()?;
    for _ in 0..3 {
        let dir_name = dir.file_name()?.to_str()?;
        if icon_theme.has_icon(dir_name) {
            return Some(dir_name.to_string());
        }
        dir = dir.parent()?;
    }
    None
}
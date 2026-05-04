use relm4::gtk::gio;
use relm4::gtk::gio::DesktopAppInfo;
use relm4::gtk::prelude::{AppInfoExt, Cast};

/// Attempts to find a DesktopAppInfo matching a Hyprland client class name.
///
/// Tries several strategies:
/// 1. Direct lookup by desktop ID (e.g., "firefox" -> "firefox.desktop")
/// 2. Case-insensitive match against desktop IDs
/// 3. Case-insensitive match against WM_CLASS / StartupWMClass field
/// 4. Case-insensitive substring match against app Name
pub fn find_app_info(class: &str) -> Option<DesktopAppInfo> {
    // 1. Direct lookup (most common case)
    if let Some(app) = DesktopAppInfo::new(&format!("{class}.desktop")) {
        return Some(app);
    }

    // Also try lowercase
    let class_lower = class.to_lowercase();
    if let Some(app) = DesktopAppInfo::new(&format!("{class_lower}.desktop")) {
        return Some(app);
    }

    // Search through all apps
    let all_apps = gio::AppInfo::all();
    let mut name_match: Option<DesktopAppInfo> = None;

    for app in &all_apps {
        let Some(desktop) = app.downcast_ref::<DesktopAppInfo>() else {
            continue;
        };

        // 2. Case-insensitive desktop ID match
        if let Some(id) = desktop.id() {
            let id_str = id.to_string();
            let id_stem = id_str.strip_suffix(".desktop").unwrap_or(&id_str);
            if id_stem.eq_ignore_ascii_case(class) {
                return Some(desktop.clone());
            }
        }

        // 3. Match against StartupWMClass
        if let Some(wm_class) = desktop.string("StartupWMClass") {
            if wm_class.eq_ignore_ascii_case(class) {
                return Some(desktop.clone());
            }
        }

        // 4. Exact name match
        if desktop.name().eq_ignore_ascii_case(class) {
            return Some(desktop.clone());
        }

        // 5. Fuzzy substring name match (last resort)
        if name_match.is_none() && desktop.name().to_lowercase().contains(&class_lower) {
            name_match = Some(desktop.clone());
        }
    }

    None
}

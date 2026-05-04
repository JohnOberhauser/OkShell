use crate::OutputInfo;
use gtk4::prelude::{Cast, ListModelExt, MonitorExt};
use okshell_services::hyprland_service;
use relm4::gtk::gdk;
use std::path::PathBuf;

/// Try to find the GDK monitor matching a Hyprland output by connector name.
pub(crate) fn find_gdk_monitor(
    monitors: &gdk::gio::ListModel,
    output: &OutputInfo,
) -> Option<gdk::Monitor> {
    for i in 0..monitors.n_items() {
        if let Some(obj) = monitors.item(i)
            && let Ok(monitor) = obj.downcast::<gdk::Monitor>()
            && monitor.connector().as_deref() == Some(&output.name)
        {
            return Some(monitor);
        }
    }
    None
}

pub(crate) fn default_screenshot_path() -> PathBuf {
    let dir = dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Screenshots");

    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(
            &time::format_description::parse("[year]_[month]_[day]_[hour]_[minute]_[second]")
                .unwrap(),
        )
        .unwrap_or_else(|_| "screenshot".into());

    dir.join(format!("{timestamp}_screenshot.png"))
}

pub(crate) fn default_recording_path() -> PathBuf {
    let dir = dirs::video_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ScreenRecordings");

    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(
            &time::format_description::parse("[year]_[month]_[day]_[hour]_[minute]_[second]")
                .unwrap(),
        )
        .unwrap_or_else(|_| "record".into());

    dir.join(format!("{timestamp}_record.mp4"))
}

/// Query Hyprland for all connected outputs via hyprctl.
pub fn query_outputs() -> crate::common::Result<Vec<OutputInfo>> {
    let hyprland = hyprland_service();
    let monitors = hyprland.monitors.get();

    Ok(monitors
        .into_iter()
        .map(|m| OutputInfo {
            name: m.name.get(),
            x: m.x.get(),
            y: m.y.get(),
            width: m.width.get() as i32,
            height: m.height.get() as i32,
            scale: m.scale.get() as f64,
        })
        .collect())
}

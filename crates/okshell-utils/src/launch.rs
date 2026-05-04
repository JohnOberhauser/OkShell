use relm4::gtk::gio;
use relm4::gtk::prelude::AppInfoExt;
use std::os::unix::prelude::CommandExt;
use std::process::{Command, Stdio};

pub fn launch_detached(app: &gio::DesktopAppInfo) {
    if let Some(id) = app.id() {
        let _ = Command::new("gtk-launch")
            .arg(id.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map(|mut child| child.wait());
    }
}

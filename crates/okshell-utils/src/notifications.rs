use std::path::PathBuf;
use notify_rust::Notification;
use relm4::{Component, ComponentSender};
use okshell_common::watch;
use okshell_services::notification_service;

pub fn show_file_saved_notification(summary: String, path: PathBuf) {
    std::thread::spawn(move || {
        let file_path = path.display().to_string();
        let handle = Notification::new()
            .summary(summary.as_str())
            .body(&format!("Saved to {file_path}"))
            .appname("okshell")
            .action("view", "View")
            .action("open_dir", "Show in Files")
            .show();

        if let Ok(handle) = handle {
            handle.wait_for_action(|action| {
                match action {
                    "view" => {
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&file_path)
                            .spawn();
                    }
                    "open_dir" => {
                        if let Some(parent) = PathBuf::from(&file_path).parent() {
                            let _ = std::process::Command::new("xdg-open")
                                .arg(parent)
                                .spawn();
                        }
                    }
                    _ => {}
                }
            });
        }
    });
}

pub fn spawn_notifications_watcher<C>(
    sender: &ComponentSender<C>,
    map_notifications: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = notification_service();
    let notifications = service.notifications.clone();

    watch!(
        sender,
        [notifications.watch()],
        |out| {
            let _ = out.send(map_notifications());
        }
    );
}

pub fn spawn_notification_popups_watcher<C>(
    sender: &ComponentSender<C>,
    map_notifications: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = notification_service();
    let notifications = service.popups.clone();

    watch!(
        sender,
        [notifications.watch()],
        |out| {
            let _ = out.send(map_notifications());
        }
    );
}

pub fn spawn_dnd_watcher<C>(
    sender: &ComponentSender<C>,
    map_dnd: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = notification_service();
    let dnd = service.dnd.clone();

    watch!(
        sender,
        [dnd.watch()],
        |out| {
            let _ = out.send(map_dnd());
        }
    );
}
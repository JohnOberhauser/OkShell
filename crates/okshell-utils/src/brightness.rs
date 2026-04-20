use relm4::{Component, ComponentSender};
use okshell_common::watch;
use okshell_services::brightness_service;

pub fn get_brightness_icon(percentage: f64) -> &'static str {
    if percentage > 66f64 {
        "brightness-high-symbolic"
    } else if percentage > 33f64 {
        "brightness-medium-symbolic"
    } else {
        "brightness-low-symbolic"
    }
}

pub fn spawn_brightness_watcher<C>(
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    if let Some(service) = brightness_service() {
        let device = service.primary.get();

        if let Some(device) = device {
            let brightness = device.brightness.clone();

            watch!(
                sender,
                [
                    brightness.watch(),
                ],
                |out| {
                    let _ = out.send(map_state());
                }
            );
        }
    }
}
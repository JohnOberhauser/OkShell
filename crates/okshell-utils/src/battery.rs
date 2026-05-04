use okshell_common::watch;
use okshell_services::{battery_service, line_power_service};
use relm4::{Component, ComponentSender};

pub fn get_battery_icon(percent: f64) -> &'static str {
    if percent > 99.0 {
        "battery-level-100-symbolic"
    } else if percent > 90.0 {
        "battery-level-90-symbolic"
    } else if percent > 80.0 {
        "battery-level-80-symbolic"
    } else if percent > 70.0 {
        "battery-level-70-symbolic"
    } else if percent > 60.0 {
        "battery-level-60-symbolic"
    } else if percent > 50.0 {
        "battery-level-50-symbolic"
    } else if percent > 40.0 {
        "battery-level-40-symbolic"
    } else if percent > 30.0 {
        "battery-level-30-symbolic"
    } else if percent > 20.0 {
        "battery-level-20-symbolic"
    } else if percent > 10.0 {
        "battery-level-10-symbolic"
    } else {
        "battery-level-0-symbolic"
    }
}

pub fn get_charging_battery_icon(percent: f64) -> &'static str {
    if percent > 99.0 {
        "battery-level-100-charging-symbolic"
    } else if percent > 90.0 {
        "battery-level-90-charging-symbolic"
    } else if percent > 80.0 {
        "battery-level-80-charging-symbolic"
    } else if percent > 70.0 {
        "battery-level-70-charging-symbolic"
    } else if percent > 60.0 {
        "battery-level-60-charging-symbolic"
    } else if percent > 50.0 {
        "battery-level-50-charging-symbolic"
    } else if percent > 40.0 {
        "battery-level-40-charging-symbolic"
    } else if percent > 30.0 {
        "battery-level-30-charging-symbolic"
    } else if percent > 20.0 {
        "battery-level-20-charging-symbolic"
    } else if percent > 10.0 {
        "battery-level-10-charging-symbolic"
    } else {
        "battery-level-0-charging-symbolic"
    }
}

pub fn spawn_battery_watcher<C>(
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = battery_service();
    let percentage = service.device.percentage.clone();
    let state = service.device.state.clone();
    let is_present = service.device.is_present.clone();

    watch!(
        sender,
        [percentage.watch(), state.watch(), is_present.watch(),],
        |out| {
            let _ = out.send(map_state());
        }
    );
}

pub fn spawn_battery_online_watcher<C>(
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    if let Some(service) = line_power_service() {
        let online = service.device.online.clone();

        watch!(sender, [online.watch(),], |out| {
            let _ = out.send(map_state());
        });
    }
}

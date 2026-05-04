use okshell_common::{watch, watch_cancellable};
use okshell_services::network_service;
use relm4::{Component, ComponentSender, gtk};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use wayle_network::NetworkService;
use wayle_network::types::connectivity::ConnectionType;
use wayle_network::types::states::NetworkStatus;

pub fn set_network_icon(image: &gtk::Image) {
    let network = network_service();
    let primary = network.primary.get();

    match primary {
        ConnectionType::Wired => {
            if let Some(icon) = get_wired_icon(&network) {
                image.set_icon_name(Some(icon));
            } else {
                image.set_icon_name(Some("network-wired-disconnected-symbolic"));
            }
        }
        ConnectionType::Wifi => {
            if let Some(icon) = get_wifi_icon(&network) {
                image.set_icon_name(Some(icon));
            } else {
                image.set_icon_name(Some("network-wireless-disabled-symbolic"));
            }
        }
        _ => {
            if let Some(icon) = get_wifi_icon(&network) {
                image.set_icon_name(Some(icon));
            } else if let Some(icon) = get_wired_icon(&network) {
                image.set_icon_name(Some(icon));
            } else {
                image.set_icon_name(Some("network-wireless-no-route-symbolic"));
            }
        }
    }
}

fn get_wifi_icon(network: &Arc<NetworkService>) -> Option<&'static str> {
    if let Some(wifi) = network.wifi.get() {
        if !wifi.enabled.get() {
            return Some("network-wireless-disabled-symbolic");
        }

        match wifi.connectivity.get() {
            NetworkStatus::Connecting => Some("network-wireless-acquiring-symbolic"),
            NetworkStatus::Disconnected => Some("network-wireless-offline-symbolic"),
            NetworkStatus::Connected => {
                if let Some(strength) = wifi.strength.get() {
                    Some(get_wifi_icon_for_strength(strength))
                } else {
                    Some("network-wireless-signal-none-symbolic")
                }
            }
        }
    } else {
        None
    }
}

fn get_wired_icon(network: &Arc<NetworkService>) -> Option<&'static str> {
    if let Some(wired) = network.wired.get() {
        match wired.connectivity.get() {
            NetworkStatus::Connecting => Some("network-wired-acquiring-symbolic"),
            NetworkStatus::Disconnected => Some("network-wired-disconnected-symbolic"),
            NetworkStatus::Connected => Some("network-wired-symbolic"),
        }
    } else {
        None
    }
}

pub fn get_wifi_icon_for_strength(strength: u8) -> &'static str {
    if strength > 75 {
        "network-wireless-signal-excellent-symbolic"
    } else if strength > 50 {
        "network-wireless-signal-good-symbolic"
    } else if strength > 25 {
        "network-wireless-signal-ok-symbolic"
    } else if strength > 0 {
        "network-wireless-signal-weak-symbolic"
    } else {
        "network-wireless-signal-none-symbolic"
    }
}

pub fn set_network_label(label: &gtk::Label) {
    let network = network_service();
    let primary = network.primary.get();

    match primary {
        ConnectionType::Wired => {
            if let Some(wired_label) = get_wired_label(&network) {
                label.set_label(wired_label);
            } else {
                label.set_label("Not Connected");
            }
        }
        ConnectionType::Wifi => {
            if let Some(wifi_label) = get_wifi_label(&network) {
                label.set_label(wifi_label.as_str());
            } else {
                label.set_label("Not Connected");
            }
        }
        _ => {
            if let Some(wifi_label) = get_wifi_label(&network) {
                label.set_label(wifi_label.as_str());
            } else if let Some(wired_label) = get_wired_label(&network) {
                label.set_label(wired_label);
            } else {
                label.set_label("Not Connected");
            }
        }
    }
}

fn get_wifi_label(network: &Arc<NetworkService>) -> Option<String> {
    if let Some(wifi) = network.wifi.get() {
        if !wifi.enabled.get() {
            return Some("Not Connected".to_string());
        }

        if let Some(ssdi) = wifi.ssid.get() {
            return Some(ssdi);
        }

        match wifi.connectivity.get() {
            NetworkStatus::Connecting => {
                if let Some(ssdi) = wifi.ssid.get() {
                    Some(ssdi)
                } else {
                    Some("Connecting…".to_string())
                }
            }
            NetworkStatus::Disconnected => Some("Not Connected".to_string()),
            NetworkStatus::Connected => {
                if let Some(ssdi) = wifi.ssid.get() {
                    Some(ssdi)
                } else {
                    Some("Wifi Connected".to_string())
                }
            }
        }
    } else {
        None
    }
}

fn get_wired_label(network: &Arc<NetworkService>) -> Option<&'static str> {
    if let Some(wired) = network.wired.get() {
        match wired.connectivity.get() {
            NetworkStatus::Connecting => Some("Connecting…"),
            NetworkStatus::Disconnected => Some("Not Connected"),
            NetworkStatus::Connected => Some("Wired"),
        }
    } else {
        None
    }
}

pub fn spawn_network_watcher<C>(
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    map_wifi: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    map_wired: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let network = network_service();
    let primary = network.primary.clone();
    let wifi = network.wifi.clone();
    let wired = network.wired.clone();
    watch!(sender, [primary.watch()], |out| {
        let _ = out.send(map_state());
    });
    watch!(sender, [wifi.watch()], |out| {
        let _ = out.send(map_wifi());
    });
    watch!(sender, [wired.watch()], |out| {
        let _ = out.send(map_wired());
    });
}

pub fn spawn_wifi_available_watcher<C>(
    sender: &ComponentSender<C>,
    map_wifi: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let network = network_service();
    let wifi = network.wifi.clone();
    watch!(sender, [wifi.watch()], |out| {
        let _ = out.send(map_wifi());
    });
}

pub fn spawn_wifi_enabled_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: CancellationToken,
    map: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let network = network_service();
    let Some(wifi) = network.wifi.get() else {
        return;
    };
    let enabled = wifi.enabled.clone();
    watch_cancellable!(sender, cancellation_token, [enabled.watch()], |out| {
        let _ = out.send(map());
    });
}

pub fn spawn_wifi_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: CancellationToken,
    map: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let network = network_service();
    let Some(wifi) = network.wifi.get() else {
        return;
    };
    let enabled = wifi.enabled.clone();
    let connectivity = wifi.connectivity.clone();
    let ssid = wifi.ssid.clone();
    let strength = wifi.strength.clone();
    watch_cancellable!(
        sender,
        cancellation_token,
        [
            enabled.watch(),
            connectivity.watch(),
            ssid.watch(),
            strength.watch()
        ],
        |out| {
            let _ = out.send(map());
        }
    );
}

pub fn spawn_available_wifi_networks_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: CancellationToken,
    map: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let network = network_service();
    let Some(wifi) = network.wifi.get() else {
        return;
    };
    let ssid = wifi.ssid.clone();
    let access_points = wifi.access_points.clone();
    watch_cancellable!(
        sender,
        cancellation_token,
        [ssid.watch(), access_points.watch(),],
        |out| {
            let _ = out.send(map());
        }
    );
}

pub fn spawn_wired_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: CancellationToken,
    map: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let network = network_service();
    let Some(wired) = network.wired.get() else {
        return;
    };
    let connectivity = wired.connectivity.clone();
    watch_cancellable!(sender, cancellation_token, [connectivity.watch()], |out| {
        let _ = out.send(map());
    });
}

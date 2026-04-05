use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing::info;
use wayle_audio::AudioService;
use wayle_battery::BatteryService;
use wayle_bluetooth::BluetoothService;
use wayle_brightness::BrightnessService;
use wayle_hyprland::HyprlandService;
use wayle_media::MediaService;
use wayle_network::NetworkService;
use wayle_notification::NotificationService;
use wayle_power_profiles::PowerProfilesService;
use wayle_sysinfo::SysinfoService;
use wayle_systray::SystemTrayService;
use wayle_weather::{LocationQuery, TemperatureUnit, WeatherService, WeatherServiceBuilder};
use zbus::{Connection, Error};
use zbus::zvariant::OwnedObjectPath;

pub async fn init_services(
    location_query: LocationQuery,
    temperature_unit: TemperatureUnit,
) -> anyhow::Result<()> {
    info!("Initializing services...");
    let audio = AudioService::new();
    let battery = BatteryService::new();
    let bluetooth = BluetoothService::new();
    let brightness = BrightnessService::new();
    let hyprland = HyprlandService::new();
    let media = MediaService::new();
    let network = NetworkService::new();
    let notifications = NotificationService::new();
    let power_profiles = PowerProfilesService::new();
    let sysinfo = SysinfoService::builder().build();
    let systray = SystemTrayService::builder().build();
    let weather = WeatherServiceBuilder::new()
        .poll_interval(Duration::from_mins(15))
        .location(location_query)
        .units(temperature_unit)
        .build();

    if let Some(path) = find_line_power_path().await? {
        let line_power = BatteryService::builder()
            .device_path(path)
            .build();

        LINE_POWER_SERVICE.set(Some(Arc::new(line_power.await?))).ok();
    } else {
        LINE_POWER_SERVICE.set(None).ok();
    }

    AUDIO_SERVICE.set(audio.await?).ok();
    BATTERY_SERVICE.set(Arc::new(battery.await?)).ok();
    BLUETOOTH_SERVICE.set(Arc::new(bluetooth.await?)).ok();
    BRIGHTNESS_SERVICE.set(brightness.await?).ok();
    HYPRLAND_SERVICE.set(hyprland.await?).ok();
    MEDIA_SERVICE.set(media.await?).ok();
    NETWORK_SERVICE.set(Arc::new(network.await?)).ok();
    NOTIFICATION_SERVICE.set(notifications.await?).ok();
    POWER_PROFILE_SERVICE.set(power_profiles.await?).ok();
    SYS_INFO_SERVICE.set(Arc::new(sysinfo)).ok();
    SYS_TRAY_SERVICE.set(systray.await?).ok();
    WEATHER_SERVICE.set(Arc::new(weather)).ok();

    info!("Done");

    Ok(())
}

pub async fn find_line_power_path() -> Result<Option<OwnedObjectPath>, Error> {
    let connection = Connection::system().await?;

    let reply = connection
        .call_method(
            Some("org.freedesktop.UPower"),
            "/org/freedesktop/UPower",
            Some("org.freedesktop.UPower"),
            "EnumerateDevices",
            &(),
        )
        .await?;

    let devices: Vec<OwnedObjectPath> = reply.body().deserialize()?;

    Ok(devices.into_iter().find(|p| p.as_str().contains("line_power")))
}

static AUDIO_SERVICE: OnceLock<Arc<AudioService>> = OnceLock::new();

pub fn audio_service() -> Arc<AudioService> {
    AUDIO_SERVICE.get().expect("AudioService not initialized").clone()
}

static BATTERY_SERVICE: OnceLock<Arc<BatteryService>> = OnceLock::new();

pub fn battery_service() -> Arc<BatteryService> {
    BATTERY_SERVICE.get().expect("BatteryService not initialized").clone()
}

static BLUETOOTH_SERVICE: OnceLock<Arc<BluetoothService>> = OnceLock::new();

pub fn bluetooth_service() -> Arc<BluetoothService> {
    BLUETOOTH_SERVICE.get().expect("BluetoothService not initialized").clone()
}

static BRIGHTNESS_SERVICE: OnceLock<Option<Arc<BrightnessService>>> = OnceLock::new();

pub fn brightness_service() -> Option<Arc<BrightnessService>> {
    BRIGHTNESS_SERVICE.get().expect("BrightnessService not initialized").clone()
}

static HYPRLAND_SERVICE: OnceLock<Arc<HyprlandService>> = OnceLock::new();

pub fn hyprland_service() -> Arc<HyprlandService> {
    HYPRLAND_SERVICE.get().expect("HyprlandService not initialized").clone()
}

static LINE_POWER_SERVICE: OnceLock<Option<Arc<BatteryService>>> = OnceLock::new();

pub fn line_power_service() -> Option<Arc<BatteryService>> {
    LINE_POWER_SERVICE.get().expect("LinePower not initialized").clone()
}

static MEDIA_SERVICE: OnceLock<Arc<MediaService>> = OnceLock::new();

pub fn media_service() -> Arc<MediaService> {
    MEDIA_SERVICE.get().expect("MediaService not initialized").clone()
}

static NETWORK_SERVICE: OnceLock<Arc<NetworkService>> = OnceLock::new();

pub fn network_service() -> Arc<NetworkService> {
    NETWORK_SERVICE.get().expect("NetworkService not initialized").clone()
}

static NOTIFICATION_SERVICE: OnceLock<Arc<NotificationService>> = OnceLock::new();

pub fn notification_service() -> Arc<NotificationService> {
    NOTIFICATION_SERVICE.get().expect("NotificationService not initialized").clone()
}

static POWER_PROFILE_SERVICE: OnceLock<Arc<PowerProfilesService>> = OnceLock::new();

pub fn power_profile_service() -> Arc<PowerProfilesService> {
    POWER_PROFILE_SERVICE.get().expect("PowerProfilesService not initialized").clone()
}

static SYS_INFO_SERVICE: OnceLock<Arc<SysinfoService>> = OnceLock::new();

pub fn sys_info_service() -> Arc<SysinfoService> {
    SYS_INFO_SERVICE.get().expect("SysinfoService not initialized").clone()
}

static SYS_TRAY_SERVICE: OnceLock<Arc<SystemTrayService>> = OnceLock::new();

pub fn sys_tray_service() -> Arc<SystemTrayService> {
    SYS_TRAY_SERVICE.get().expect("SystemTrayService not initialized").clone()
}

static WEATHER_SERVICE: OnceLock<Arc<WeatherService>> = OnceLock::new();

pub fn weather_service() -> Arc<WeatherService> {
    WEATHER_SERVICE.get().expect("WeatherService not initialized").clone()
}
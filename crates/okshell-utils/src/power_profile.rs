use okshell_common::{watch, watch_cancellable};
use okshell_services::power_profile_service;
use relm4::{Component, ComponentSender};
use tokio_util::sync::CancellationToken;
use wayle_power_profiles::types::profile::PowerProfile;

pub fn get_active_power_profile_icon() -> &'static str {
    let profile = power_profile_service().power_profiles.active_profile.get();
    get_power_profile_icon(&profile)
}

pub fn get_power_profile_icon(profile: &PowerProfile) -> &'static str {
    match profile {
        PowerProfile::PowerSaver => "power-profile-power-saver-symbolic",
        PowerProfile::Balanced => "power-profile-balanced-symbolic",
        PowerProfile::Performance => "power-profile-performance-symbolic",
        PowerProfile::Unknown => "power-profile-balanced-symbolic",
    }
}

pub fn get_power_profile_label(profile: &PowerProfile) -> &'static str {
    match profile {
        PowerProfile::Balanced => "Balanced",
        PowerProfile::Performance => "Performance",
        PowerProfile::PowerSaver => "Power Saver",
        PowerProfile::Unknown => "Unknown",
    }
}

pub fn spawn_active_profile_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: Option<CancellationToken>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let active = power_profile_service()
        .power_profiles
        .active_profile
        .clone();

    if let Some(cancellation_token) = cancellation_token {
        watch_cancellable!(sender, cancellation_token, [active.watch()], |out| {
            let _ = out.send(map_state());
        });
    } else {
        watch!(sender, [active.watch()], |out| {
            let _ = out.send(map_state());
        });
    }
}

pub fn spawn_profiles_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: Option<CancellationToken>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let profiles = power_profile_service().power_profiles.profiles.clone();

    if let Some(cancellation_token) = cancellation_token {
        watch_cancellable!(sender, cancellation_token, [profiles.watch()], |out| {
            let _ = out.send(map_state());
        });
    } else {
        watch!(sender, [profiles.watch()], |out| {
            let _ = out.send(map_state());
        });
    }
}

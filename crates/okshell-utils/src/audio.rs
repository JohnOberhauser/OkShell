use std::sync::Arc;
use relm4::{Component, ComponentSender};
use tokio_util::sync::CancellationToken;
use wayle_audio::core::device::input::InputDevice;
use wayle_audio::core::device::output::OutputDevice;
use okshell_common::{watch, watch_cancellable};
use okshell_services::audio_service;

pub fn get_audio_out_icon(device: &Arc<OutputDevice>) -> &'static str {
    if device.muted.get() {
        return "audio-volume-muted-symbolic"
    }
    let percentage = device.volume.get().average_percentage().round() as u16;
    if percentage > 66 {
        "audio-volume-high-symbolic"
    } else if percentage > 33 {
        "audio-volume-medium-symbolic"
    } else if percentage > 0 {
        "audio-volume-low-symbolic"
    } else {
        "audio-volume-muted-symbolic"
    }
}

pub fn get_audio_in_icon(device: &Arc<InputDevice>) -> &'static str {
    if device.muted.get() {
        return "microphone-sensitivity-muted-symbolic";
    }
    let percentage = device.volume.get().average_percentage().round() as u16;
    if percentage > 66 {
        "microphone-sensitivity-high-symbolic"
    } else if percentage > 33 {
        "microphone-sensitivity-medium-symbolic"
    } else if percentage > 0 {
        "microphone-sensitivity-low-symbolic"
    } else {
        "microphone-sensitivity-muted-symbolic"
    }
}

pub fn spawn_default_output_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: Option<CancellationToken>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let default_output = audio_service().default_output.clone();
    
    if let Some(cancellation_token) = cancellation_token {
        watch_cancellable!(
            sender, 
            cancellation_token,
            [default_output.watch()],
            |out| {
                let _ = out.send(map_state());
            }
        );
    } else {
        watch!(
            sender, 
            [default_output.watch()], 
            |out| {
                let _ = out.send(map_state());
            }
        );
    }
}

pub fn spawn_output_devices_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: CancellationToken,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let out_devices = audio_service().output_devices.clone();

    watch_cancellable!(
        sender, 
        cancellation_token,
        [out_devices.watch()],
        |out| {
            let _ = out.send(map_state());
        }
    );
}

pub fn spawn_output_device_volume_mute_watcher<C>(
    output_device: &Arc<OutputDevice>,
    cancellation_token: CancellationToken,
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let volume = output_device.volume.clone();
    let muted = output_device.muted.clone();
    watch_cancellable!(
        sender, 
        cancellation_token, 
        [volume.watch(), muted.watch()], 
        |out| {
            let _ = out.send(map_state());
        }
    );
}

pub fn spawn_default_input_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: Option<CancellationToken>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let default = audio_service().default_input.clone();

    if let Some(cancellation_token) = cancellation_token {
        watch_cancellable!(
            sender, 
            cancellation_token,
            [default.watch()],
            |out| {
                let _ = out.send(map_state());
            }
        );
    } else {
        watch!(
            sender, 
            [default.watch()], 
            |out| {
                let _ = out.send(map_state());
            }
        );
    }
}

pub fn spawn_input_devices_watcher<C>(
    sender: &ComponentSender<C>,
    cancellation_token: CancellationToken,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let devices = audio_service().input_devices.clone();

    watch_cancellable!(
        sender, 
        cancellation_token,
        [devices.watch()],
        |out| {
            let _ = out.send(map_state());
        }
    );
}

pub fn spawn_input_device_volume_mute_watcher<C>(
    input_device: &Arc<InputDevice>,
    cancellation_token: CancellationToken,
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let volume = input_device.volume.clone();
    let muted = input_device.muted.clone();
    watch_cancellable!(
        sender, 
        cancellation_token, 
        [volume.watch(), muted.watch()], 
        |out| {
            let _ = out.send(map_state());
        }
    );
}
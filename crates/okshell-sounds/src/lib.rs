use std::io::Cursor;
use std::time::Duration;

const CAMERA_SHUTTER_SOUND: &[u8] = include_bytes!("../assets/camera-shutter.ogg");
const AUDIO_VOLUME_CHANGED_SOUND: &[u8] = include_bytes!("../assets/audio-volume-change.ogg");
const BATTERY_LOW_SOUND: &[u8] = include_bytes!("../assets/battery-low.ogg");
const POWER_PLUG_SOUND: &[u8] = include_bytes!("../assets/power-plug.ogg");
const POWER_UNPLUG_SOUND: &[u8] = include_bytes!("../assets/power-unplug.ogg");

pub fn play_shutter() {
    std::thread::spawn(|| {
        let mut handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio device");
        handle.log_on_drop(false);
        let cursor = Cursor::new(CAMERA_SHUTTER_SOUND);
        if let Ok(player) = rodio::play(&handle.mixer(), cursor) {
            player.sleep_until_end();
        }
    });
}

pub fn play_audio_volume_change() {
    std::thread::spawn(|| {
        // give volume changes a moment to happen
        std::thread::sleep(Duration::from_millis(50));
        let mut handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio device");
        handle.log_on_drop(false);
        let cursor = Cursor::new(AUDIO_VOLUME_CHANGED_SOUND);
        if let Ok(player) = rodio::play(&handle.mixer(), cursor) {
            player.sleep_until_end();
            // sleep to make sure the sounds plays.  It's very short and might not without the sleep.
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}

pub fn play_battery_low() {
    std::thread::spawn(|| {
        let mut handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio device");
        handle.log_on_drop(false);
        let cursor = Cursor::new(BATTERY_LOW_SOUND);
        if let Ok(player) = rodio::play(&handle.mixer(), cursor) {
            player.sleep_until_end();
        }
    });
}

pub fn play_power_plug_sound() {
    std::thread::spawn(|| {
        let mut handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio device");
        handle.log_on_drop(false);
        let cursor = Cursor::new(POWER_PLUG_SOUND);
        if let Ok(player) = rodio::play(&handle.mixer(), cursor) {
            player.sleep_until_end();
        }
    });
}

pub fn play_power_unplug_sound() {
    std::thread::spawn(|| {
        let mut handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio device");
        handle.log_on_drop(false);
        let cursor = Cursor::new(POWER_UNPLUG_SOUND);
        if let Ok(player) = rodio::play(&handle.mixer(), cursor) {
            player.sleep_until_end();
        }
    });
}

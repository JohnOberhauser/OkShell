use std::sync::Arc;
use wayle_audio::core::device::input::InputDevice;
use crate::common_widgets::option_list::{OptionItem, OptionsList};

#[derive(Debug, Clone)]
pub struct AudioOption {
    pub value: Option<Arc<InputDevice>>,
}

impl OptionItem for AudioOption {
    fn label(&self) -> String {
        get_audio_option_label(self)
    }

    fn icon_name(&self) -> Option<String> {
        Some(get_audio_option_icon_name(self))
    }
}

pub fn get_audio_option_label(option: &AudioOption) -> String {
    if let Some(source) = &option.value {
        source.description.get()
    } else {
        "No Audio".to_string()
    }
}

pub fn get_audio_option_icon_name(option: &AudioOption) -> String {
    if let Some(source) = &option.value {
        if source.is_monitor.get() {
            "audio-volume-medium-symbolic".to_string()
        } else {
            "microphone-sensitivity-medium-symbolic".to_string()
        }
    } else {
        "audio-volume-muted-symbolic".to_string()
    }
}

pub type AudioOptionsList = OptionsList<AudioOption>;
use crate::common_widgets::option_list::{OptionItem, OptionsList};

#[derive(Debug, Clone)]
pub struct DelayOption {
    pub value: u32,
    pub icon_name: String,
}

impl OptionItem for DelayOption {
    fn label(&self) -> String {
        if self.value == 1 {
            format!("{} second", self.value)
        } else {
            format!("{} seconds", self.value)
        }
    }

    fn icon_name(&self) -> Option<String> {
        Some(self.icon_name.clone())
    }
}

pub type DelayOptionsList = OptionsList<DelayOption>;
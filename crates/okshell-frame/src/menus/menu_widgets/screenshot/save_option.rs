use crate::common_widgets::option_list::{OptionItem, OptionsList};
use okshell_screenshot::OutputTarget;

#[derive(Debug, Clone)]
pub struct SaveOptionRow {
    pub value: OutputTarget,
    pub icon_name: String,
}

impl OptionItem for SaveOptionRow {
    fn label(&self) -> String {
        match self.value {
            OutputTarget::FileAndClipboard => "Save to file and clipboard".to_string(),
            OutputTarget::File => "Save to file".to_string(),
            OutputTarget::Clipboard => "Save to clipboard".to_string(),
        }
    }

    fn icon_name(&self) -> Option<String> {
        match self.value {
            OutputTarget::FileAndClipboard => Some("screenshot-save-both-symbolic".to_string()),
            OutputTarget::File => Some("screenshot-save-file-symbolic".to_string()),
            OutputTarget::Clipboard => Some("screenshot-save-clipboard-symbolic".to_string()),
        }
    }
}

pub type SaveOptionsList = OptionsList<SaveOptionRow>;

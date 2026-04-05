use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
pub enum BarWidget {
    AudioInput,
    AudioOutput,
    Battery,
    Bluetooth,
    Clipboard,
    Clock,
    HyprlandDock,
    HyprlandWorkspaces,
    HyprPicker,
    Lock,
    Logout,
    Network,
    Notifications,
    PowerProfile,
    QuickSettings,
    Reboot,
    RecordingIndicator,
    Screenshot,
    Shutdown,
    Tray,
    VpnIndicator,
    Wallpaper,
}

impl PatchField for BarWidget {
    fn patch_field(
        &mut self,
        new: Self,
        path: &StorePath,
        notify: &mut dyn FnMut(&StorePath),
        _keys: Option<&KeyMap>,
    ) {
        if *self != new {
            *self = new;
            notify(path);
        }
    }
}

impl BarWidget {
    pub fn display_name(&self) -> &'static str {
        match self {
            BarWidget::AudioInput => "Audio Input",
            BarWidget::AudioOutput => "Audio Output",
            BarWidget::Battery => "Battery",
            BarWidget::Bluetooth => "Bluetooth",
            BarWidget::Clipboard => "Clipboard",
            BarWidget::Clock => "Clock",
            BarWidget::HyprlandDock => "Dock",
            BarWidget::HyprlandWorkspaces => "Workspaces",
            BarWidget::HyprPicker => "Color Picker",
            BarWidget::Lock => "Lock",
            BarWidget::Logout => "Logout",
            BarWidget::Network => "Network",
            BarWidget::Notifications => "Notifications",
            BarWidget::PowerProfile => "Power Profile",
            BarWidget::QuickSettings => "Quick Settings",
            BarWidget::Reboot => "Reboot",
            BarWidget::RecordingIndicator => "Recording",
            BarWidget::Screenshot => "Screenshot",
            BarWidget::Shutdown => "Shutdown",
            BarWidget::Tray => "System Tray",
            BarWidget::VpnIndicator => "VPN",
            BarWidget::Wallpaper => "Wallpaper",
        }
    }

    pub fn action_name(&self) -> String {
        format!("{:?}", self).to_lowercase().replace(' ', "-")
    }

    pub fn all() -> &'static [BarWidget] {
        &[
            BarWidget::AudioInput,
            BarWidget::AudioOutput,
            BarWidget::Battery,
            BarWidget::Bluetooth,
            BarWidget::Clipboard,
            BarWidget::Clock,
            BarWidget::HyprlandDock,
            BarWidget::HyprlandWorkspaces,
            BarWidget::HyprPicker,
            BarWidget::Lock,
            BarWidget::Logout,
            BarWidget::Network,
            BarWidget::Notifications,
            BarWidget::PowerProfile,
            BarWidget::QuickSettings,
            BarWidget::Reboot,
            BarWidget::RecordingIndicator,
            BarWidget::Screenshot,
            BarWidget::Shutdown,
            BarWidget::Tray,
            BarWidget::VpnIndicator,
            BarWidget::Wallpaper,
        ]
    }
}
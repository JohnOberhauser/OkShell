use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::location_query::OrdF64;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum BarWidget {
    AudioInput,
    AudioOutput,
    Battery,
    Bluetooth,
    Cava(CavaConfig),
    Clipboard,
    Clock,
    CpuUsage,
    DiskUsage,
    HyprlandDock,
    HyprlandLayoutSwitcher,
    HyprlandWorkspaces,
    HyprPicker,
    Lock,
    Logout,
    MemoryUsage,
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
            BarWidget::Cava(_) => "Cava",
            BarWidget::Clipboard => "Clipboard",
            BarWidget::Clock => "Clock",
            BarWidget::CpuUsage => "CPU Usage",
            BarWidget::DiskUsage => "Disk Usage",
            BarWidget::HyprlandDock => "Hyprland Dock",
            BarWidget::HyprlandLayoutSwitcher => "Hyprland Layout Switcher",
            BarWidget::HyprlandWorkspaces => "Hyprland Workspaces",
            BarWidget::HyprPicker => "HyprPicker",
            BarWidget::Lock => "Lock",
            BarWidget::Logout => "Logout",
            BarWidget::MemoryUsage => "Memory Usage",
            BarWidget::Network => "Network",
            BarWidget::Notifications => "Notifications",
            BarWidget::PowerProfile => "Power Profile",
            BarWidget::QuickSettings => "Quick Settings",
            BarWidget::Reboot => "Reboot",
            BarWidget::RecordingIndicator => "Recording Indicator",
            BarWidget::Screenshot => "Screenshot",
            BarWidget::Shutdown => "Shutdown",
            BarWidget::Tray => "Tray",
            BarWidget::VpnIndicator => "VPN Indicator",
            BarWidget::Wallpaper => "Wallpaper",
        }
    }

    pub fn action_name(&self) -> String {
        self.display_name().to_lowercase().replace(' ', "-")
    }

    pub fn all_defaults() -> Vec<BarWidget> {
        vec![
            BarWidget::AudioInput,
            BarWidget::AudioOutput,
            BarWidget::Battery,
            BarWidget::Bluetooth,
            BarWidget::Cava(CavaConfig::default()),
            BarWidget::Clipboard,
            BarWidget::Clock,
            BarWidget::CpuUsage,
            BarWidget::DiskUsage,
            BarWidget::HyprlandDock,
            BarWidget::HyprlandLayoutSwitcher,
            BarWidget::HyprlandWorkspaces,
            BarWidget::HyprPicker,
            BarWidget::Lock,
            BarWidget::Logout,
            BarWidget::MemoryUsage,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct CavaConfig {
    pub bars: u16,
    pub bar_width: u32,
    pub autosens: bool,
    pub stereo: bool,
    pub noise_reduction: OrdF64,
    pub monstercat: OrdF64,
    pub waves: u32,
    pub low_cutoff: u32,
    pub high_cutoff: u32,
}

impl Default for CavaConfig {
    fn default() -> Self {
        Self {
            bars: 32,
            bar_width: 3,
            autosens: true,
            stereo: false,
            noise_reduction: OrdF64(0.77),
            monstercat: OrdF64(1.0),
            waves: 0,
            low_cutoff: 50,
            high_cutoff: 10000,
        }
    }
}

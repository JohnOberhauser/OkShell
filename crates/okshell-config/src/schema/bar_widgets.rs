use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::location_query::OrdF64;
use crate::schema::widget_enum::widget_enum;

widget_enum! {
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Store, JsonSchema)]
    pub enum BarWidget {
        AudioInput => "Audio Input",
        AudioOutput => "Audio Output",
        Battery => "Battery",
        Bluetooth => "Bluetooth",
        Cava(CavaConfig) => "Cava",
        Clipboard => "Clipboard",
        Clock => "Clock",
        CpuUsage => "CPU Usage",
        DiskUsage => "Disk Usage",
        HyprlandDock => "Hyprland Dock",
        HyprlandLayoutSwitcher => "Hyprland Layout Switcher",
        HyprlandWorkspaces(HyprlandWorkspacesConfig) => "Hyprland Workspaces",
        HyprPicker => "HyprPicker",
        Lock => "Lock",
        Logout => "Logout",
        MemoryUsage => "Memory Usage",
        Network => "Network",
        Notifications => "Notifications",
        PowerProfile => "Power Profile",
        QuickSettings => "Quick Settings",
        Reboot => "Reboot",
        RecordingIndicator => "Recording Indicator",
        Screenshot => "Screenshot",
        Shutdown => "Shutdown",
        Tray => "Tray",
        VpnIndicator => "VPN Indicator",
        Wallpaper => "Wallpaper",
    }
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
    pub fn action_name(&self) -> String {
        self.display_name().to_lowercase().replace(' ', "-")
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct HyprlandWorkspacesConfig {
    pub font: String,
}

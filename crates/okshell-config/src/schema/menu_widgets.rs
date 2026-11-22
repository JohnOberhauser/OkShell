use crate::schema::position::Orientation;
use crate::schema::widget_enum::widget_enum;
use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

widget_enum! {
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Store, JsonSchema)]
    pub enum MenuWidget {
        AppLauncher => "App Launcher",
        AudioInput => "Audio Input",
        AudioOutput => "Audio Output",
        Bluetooth => "Bluetooth",
        Calendar => "Calendar",
        Clipboard => "Clipboard",
        Clock => "Clock",
        Container(ContainerConfig) => "Container",
        CpuUsage => "CPU Usage",
        DiskUsage => "Disk Usage",
        Divider => "Divider",
        MediaPlayer => "Media Player",
        MemoryUsage => "Memory Usage",
        Network => "Network",
        Notifications => "Notifications",
        PowerProfiles => "Power Profiles",
        QuickActions(QuickActionsConfig) => "Quick Actions",
        Screenshots => "Screenshots",
        ScreenRecording => "Screen Recording",
        Spacer(SpacerConfig) => "Spacer",
        ThemePicker => "Theme Picker",
        Wallpaper => "Wallpaper",
        Weather => "Weather",
        WeatherDailyGraph => "Weather Daily Graph",
        WeatherHourlyGraph => "Weather Hourly Graph",
    }
}

impl PatchField for MenuWidget {
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

impl MenuWidget {
    pub fn action_name(&self) -> String {
        self.display_name().to_lowercase().replace(' ', "-")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema, Default)]
#[serde(default)]
pub struct QuickActionsConfig {
    pub widgets: Vec<QuickActionWidget>,
}

widget_enum! {
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Store, JsonSchema)]
    pub enum QuickActionWidget {
        AirplaneMode => "Airplane Mode",
        DoNotDisturb => "Do Not Disturb",
        HyprPicker => "Color Picker",
        IdleInhibitor => "Idle Inhibitor",
        Lock => "Lock",
        Logout => "Logout",
        Nightlight => "Night Light",
        Reboot => "Reboot",
        Settings => "Settings",
        Shutdown => "Shutdown",
    }
}

impl QuickActionWidget {
    pub fn action_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
#[serde(default)]
pub struct SpacerConfig {
    pub size: i32,
}

impl Default for SpacerConfig {
    fn default() -> Self {
        Self { size: 16 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
#[serde(default)]
pub struct ContainerConfig {
    pub widgets: Vec<MenuWidget>,
    pub spacing: i32,
    pub orientation: Orientation,
    pub minimum_width: i32,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            spacing: 0,
            orientation: Orientation::Horizontal,
            minimum_width: 0,
        }
    }
}

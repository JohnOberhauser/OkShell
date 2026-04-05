use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::schema::position::Orientation;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum MenuWidget {
    AppLauncher,
    AudioInput,
    AudioOutput,
    Bluetooth,
    Calendar,
    Clipboard,
    Clock,
    Container(ContainerConfig),
    Divider,
    MediaPlayer,
    Network,
    Notifications,
    PowerProfiles,
    QuickActions(QuickActionsConfig),
    Screenshots,
    ScreenRecording,
    Spacer(SpacerConfig),
    Wallpaper,
    WallpaperDropdown,
    Weather,
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
    pub fn display_name(&self) -> &'static str {
        match self {
            MenuWidget::AppLauncher => "App Launcher",
            MenuWidget::AudioInput => "Audio Input",
            MenuWidget::AudioOutput => "Audio Output",
            MenuWidget::Bluetooth => "Bluetooth",
            MenuWidget::Calendar => "Calendar",
            MenuWidget::Clipboard => "Clipboard",
            MenuWidget::Clock => "Clock",
            MenuWidget::Container(_) => "Container",
            MenuWidget::Divider => "Divider",
            MenuWidget::MediaPlayer => "Media Player",
            MenuWidget::Network => "Network",
            MenuWidget::Notifications => "Notifications",
            MenuWidget::PowerProfiles => "Power Profiles",
            MenuWidget::QuickActions(_) => "Quick Actions",
            MenuWidget::Screenshots => "Screenshots",
            MenuWidget::ScreenRecording => "Screen Recording",
            MenuWidget::Spacer(_) => "Spacer",
            MenuWidget::Wallpaper => "Wallpaper",
            MenuWidget::WallpaperDropdown => "Wallpaper Dropdown",
            MenuWidget::Weather => "Weather",
        }
    }

    pub fn action_name(&self) -> String {
        self.display_name().to_lowercase().replace(' ', "-")
    }

    /// Returns all widget types with default configs
    pub fn all_defaults() -> Vec<MenuWidget> {
        vec![
            MenuWidget::AppLauncher,
            MenuWidget::AudioInput,
            MenuWidget::AudioOutput,
            MenuWidget::Bluetooth,
            MenuWidget::Calendar,
            MenuWidget::Clipboard,
            MenuWidget::Clock,
            MenuWidget::Container(ContainerConfig::default()),
            MenuWidget::Divider,
            MenuWidget::MediaPlayer,
            MenuWidget::Network,
            MenuWidget::Notifications,
            MenuWidget::PowerProfiles,
            MenuWidget::QuickActions(QuickActionsConfig::default()),
            MenuWidget::Screenshots,
            MenuWidget::ScreenRecording,
            MenuWidget::Spacer(SpacerConfig { size: 16 }),
            MenuWidget::Wallpaper,
            MenuWidget::WallpaperDropdown,
            MenuWidget::Weather,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub struct QuickActionsConfig {
    pub widgets: Vec<QuickActionWidget>
}

impl Default for QuickActionsConfig {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum QuickActionWidget {
    AirplaneMode,
    DoNotDisturb,
    HyprPicker,
    Lock,
    Logout,
    Nightlight,
    Reboot,
    Settings,
    Shutdown,
}

impl QuickActionWidget {
    pub fn display_name(&self) -> &'static str {
        match self {
            QuickActionWidget::AirplaneMode => "Airplane Mode",
            QuickActionWidget::DoNotDisturb => "Do Not Disturb",
            QuickActionWidget::HyprPicker => "Color Picker",
            QuickActionWidget::Lock => "Lock",
            QuickActionWidget::Logout => "Logout",
            QuickActionWidget::Nightlight => "Night Light",
            QuickActionWidget::Reboot => "Reboot",
            QuickActionWidget::Settings => "Settings",
            QuickActionWidget::Shutdown => "Shutdown",
        }
    }

    pub fn action_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn all() -> &'static [QuickActionWidget] {
        &[
            QuickActionWidget::AirplaneMode,
            QuickActionWidget::DoNotDisturb,
            QuickActionWidget::HyprPicker,
            QuickActionWidget::Lock,
            QuickActionWidget::Logout,
            QuickActionWidget::Nightlight,
            QuickActionWidget::Reboot,
            QuickActionWidget::Settings,
            QuickActionWidget::Shutdown,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub struct SpacerConfig {
    pub size: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
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
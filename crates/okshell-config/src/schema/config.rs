use reactive_stores::{KeyMap, Patch, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::span::Attributes;
use crate::schema::bar_widgets::BarWidget;
use crate::schema::content_fit::ContentFit;
use crate::schema::location_query::{LocationQueryConfig, OrdF64};
use crate::schema::menu_widgets::{MenuWidget, QuickActionWidget, QuickActionsConfig, SpacerConfig};
use crate::schema::position::{NotificationPosition, Position};
use crate::schema::temperature::TemperatureUnitConfig;
use crate::schema::themes::{MatugenContrast, MatugenMode, MatugenPreference, MatugenType, Themes, WindowOpacity};
use crate::schema::wallpaper::ThemeFilterStrength;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Config {
    pub general: General,
    pub theme: Theme,
    pub bars: Bars,
    pub menus: Menus,
    pub notifications: Notifications,
    pub wallpaper: Wallpaper,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General::default(),
            theme: Theme::default(),
            bars: Bars::default(),
            menus: Menus::default(),
            notifications: Notifications::default(),
            wallpaper: Wallpaper::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct General {
    pub clock_format_24_h: bool,
    pub weather_location_query: LocationQueryConfig,
    pub temperature_unit: TemperatureUnitConfig,
}

impl Default for General {
    fn default() -> Self {
        Self {
            clock_format_24_h: false,
            weather_location_query: LocationQueryConfig::Coordinates {
                lat: OrdF64(0.0),
                lon: OrdF64(0.0),
            },
            temperature_unit: TemperatureUnitConfig::Metric,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Theme {
    pub shell_icon_theme: String,
    pub app_icon_theme: String,
    pub theme: Themes,
    pub matugen: Matugen,
    pub css_file: String,
    pub attributes: ThemeAttributes,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            shell_icon_theme: "OkMaterial".to_string(),
            app_icon_theme: "OkMaterial".to_string(),
            theme: Themes::Default,
            matugen: Matugen::default(),
            css_file: String::new(),
            attributes: ThemeAttributes::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct ThemeAttributes {
    pub font: Font,
    pub sizing: Sizing,
    pub window_opacity: WindowOpacity,
}

impl Default for ThemeAttributes {
    fn default() -> Self {
        Self {
            window_opacity: WindowOpacity::new(1.0),
            font: Font::default(),
            sizing: Sizing::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Font {
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            primary: String::new(),
            secondary: String::new(),
            tertiary: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Sizing {
    pub radius_widget: i32,
    pub radius_window: i32,
    pub border_width: i32,
}

impl Default for Sizing {
    fn default() -> Self {
        Self {
            radius_widget: 8,
            radius_window: 8,
            border_width: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Matugen {
    pub preference: MatugenPreference,
    pub scheme_type: MatugenType,
    pub mode: MatugenMode,
    pub contrast: MatugenContrast,
}

impl Default for Matugen {
    fn default() -> Self {
        Self {
            preference: MatugenPreference::Darkness,
            scheme_type: MatugenType::TonalSpot,
            mode: MatugenMode::Dark,
            contrast: MatugenContrast::new(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Bars {
    pub frame: Frame,
    pub top_bar: HorizontalBar,
    pub bottom_bar: HorizontalBar,
    pub left_bar: VerticalBar,
    pub right_bar: VerticalBar,
}

impl Default for Bars {
    fn default() -> Self {
        Self {
            frame: Frame::default() ,
            top_bar: HorizontalBar::default(),
            bottom_bar: HorizontalBar::default(),
            left_bar: VerticalBar {
                minimum_width: 0,
                top_widgets: vec![
                    BarWidget::QuickSettings,
                    BarWidget::HyprlandWorkspaces,
                ],
                center_widgets: vec![
                    BarWidget::HyprlandDock,
                ],
                bottom_widgets: vec![
                    BarWidget::RecordingIndicator,
                    BarWidget::Tray,
                    BarWidget::Screenshot,
                    BarWidget::Wallpaper,
                    BarWidget::Clipboard,
                    BarWidget::Notifications,
                    BarWidget::AudioInput,
                    BarWidget::AudioOutput,
                    BarWidget::Bluetooth,
                    BarWidget::Network,
                    BarWidget::Battery,
                    BarWidget::Clock,
                ],
            },
            right_bar: VerticalBar::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Menus {
    pub clock_menu: Menu,
    pub clipboard_menu: Menu,
    pub quick_settings_menu: Menu,
    pub notification_menu: Menu,
    pub screenshot_menu: Menu,
    pub app_launcher_menu: Menu,
    pub wallpaper_menu: Menu,
    pub screenshare_menu: ScreenshareMenu,
    pub left_menu_expansion_type: VerticalMenuExpansion,
    pub right_menu_expansion_type: VerticalMenuExpansion,
}

impl Default for Menus {
    fn default() -> Self {
        Self {
            clock_menu: Menu {
                position: Position::Left,
                widgets: vec![
                    MenuWidget::Calendar,
                    MenuWidget::Spacer(SpacerConfig {
                        size: 20,
                    }),
                    MenuWidget::Weather,
                ],
                minimum_width: 410,
            },
            clipboard_menu: Menu {
                position: Position::Left,
                widgets: vec![MenuWidget::Clipboard],
                minimum_width: 410,
            },
            quick_settings_menu: Menu {
                position: Position::Left,
                widgets: vec![
                    MenuWidget::Clock,
                    MenuWidget::Network,
                    MenuWidget::Bluetooth,
                    MenuWidget::AudioOutput,
                    MenuWidget::AudioInput,
                    MenuWidget::PowerProfiles,
                    MenuWidget::MediaPlayer,
                    MenuWidget::Spacer(
                        SpacerConfig {
                            size: 20,
                        }
                    ),
                    MenuWidget::QuickActions(
                        QuickActionsConfig {
                            widgets: vec![
                                QuickActionWidget::AirplaneMode,
                                QuickActionWidget::Nightlight,
                                QuickActionWidget::HyprPicker,
                                QuickActionWidget::Settings,
                            ]
                        }
                    ),
                    MenuWidget::Spacer(
                        SpacerConfig {
                            size: 20,
                        }
                    ),
                    MenuWidget::QuickActions(
                        QuickActionsConfig {
                            widgets: vec![
                                QuickActionWidget::Logout,
                                QuickActionWidget::Lock,
                                QuickActionWidget::Reboot,
                                QuickActionWidget::Shutdown,
                            ]
                        }
                    ),
                ],
                minimum_width: 410,
            },
            notification_menu: Menu {
                position: Position::Left,
                widgets: vec![MenuWidget::Notifications],
                minimum_width: 410,
            },
            screenshot_menu: Menu {
                position: Position::Left,
                widgets: vec![MenuWidget::Screenshots, MenuWidget::Divider, MenuWidget::ScreenRecording],
                minimum_width: 410,
            },
            app_launcher_menu: Menu {
                position: Position::Left,
                widgets: vec![MenuWidget::AppLauncher],
                minimum_width: 410,
            },
            wallpaper_menu: Menu {
                position: Position::Bottom,
                widgets: vec![MenuWidget::ThemePicker, MenuWidget::Wallpaper],
                minimum_width: 1200,
            },
            screenshare_menu: ScreenshareMenu {
                position: Position::Left,
            },
            left_menu_expansion_type: VerticalMenuExpansion::AlwaysExpanded,
            right_menu_expansion_type: VerticalMenuExpansion::AlwaysExpanded,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Notifications {
    pub notification_position: NotificationPosition,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            notification_position: NotificationPosition::Right,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Wallpaper {
    pub wallpaper_dir: String,
    pub content_fit: ContentFit,
    pub apply_theme_filter: bool,
    pub theme_filter_strength: ThemeFilterStrength,
}

impl Default for Wallpaper {
    fn default() -> Self {
        Self {
            wallpaper_dir: "".to_string(),
            content_fit: ContentFit::Cover,
            apply_theme_filter: false,
            theme_filter_strength: ThemeFilterStrength::new(1.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Frame {
    pub enable_frame: bool,
    pub monitor_filter: Vec<String>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            enable_frame: true,
            monitor_filter: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct Menu {
    pub position: Position,
    pub widgets: Vec<MenuWidget>,
    pub minimum_width: i32,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            position: Position::Left,
            widgets: Vec::new(),
            minimum_width: 410,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct ScreenshareMenu {
    pub position: Position,
}

impl Default for ScreenshareMenu {
    fn default() -> Self {
        Self {
            position: Position::Left,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct HorizontalBar {
    pub minimum_height: i32,
    pub left_widgets: Vec<BarWidget>,
    pub center_widgets: Vec<BarWidget>,
    pub right_widgets: Vec<BarWidget>,
}

impl Default for HorizontalBar {
    fn default() -> Self {
        Self {
            minimum_height: 0,
            left_widgets: Vec::new(),
            center_widgets: Vec::new(),
            right_widgets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, Patch, JsonSchema)]
#[serde(default)]
pub struct VerticalBar {
    pub minimum_width: i32,
    pub top_widgets: Vec<BarWidget>,
    pub center_widgets: Vec<BarWidget>,
    pub bottom_widgets: Vec<BarWidget>,
}

impl Default for VerticalBar {
    fn default() -> Self {
        Self {
            minimum_width: 0,
            top_widgets: Vec::new(),
            center_widgets: Vec::new(),
            bottom_widgets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum VerticalMenuExpansion {
    AlwaysExpanded,
    ExpandBothWays,
    ExpandUp,
    ExpandDown,
}

impl PatchField for VerticalMenuExpansion {
    fn patch_field(
        &mut self, 
        new: Self, 
        path: &StorePath, 
        notify: &mut dyn FnMut(&StorePath), 
        _keys: Option<&KeyMap>
    ) {
        if *self != new {
            *self = new;
            notify(path);
        }
    }
}

impl VerticalMenuExpansion {
    pub fn to_index(&self) -> u32 {
        match self {
            VerticalMenuExpansion::AlwaysExpanded => 0,
            VerticalMenuExpansion::ExpandBothWays => 1,
            VerticalMenuExpansion::ExpandUp => 2,
            VerticalMenuExpansion::ExpandDown => 3,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => VerticalMenuExpansion::AlwaysExpanded,
            1 => VerticalMenuExpansion::ExpandBothWays,
            2 => VerticalMenuExpansion::ExpandUp,
            _ => VerticalMenuExpansion::ExpandDown,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            VerticalMenuExpansion::AlwaysExpanded => "Always Expanded",
            VerticalMenuExpansion::ExpandBothWays => "Expand Both Ways",
            VerticalMenuExpansion::ExpandUp => "Expand Up",
            VerticalMenuExpansion::ExpandDown => "Expand Down",
        }
    }

    pub fn display_names() -> Vec<&'static str> {
        Self::all().iter().map(|p| p.display_name()).collect()
    }

    pub fn all() -> &'static [VerticalMenuExpansion] {
        &[
            VerticalMenuExpansion::AlwaysExpanded,
            VerticalMenuExpansion::ExpandBothWays,
            VerticalMenuExpansion::ExpandUp,
            VerticalMenuExpansion::ExpandDown,
        ]
    }
}

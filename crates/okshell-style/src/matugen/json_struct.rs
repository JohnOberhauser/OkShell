use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct MatugenTheme {
    pub base16: Base16,
    pub colors: Colors,
    pub image: String,
    pub is_dark_mode: bool,
    pub mode: String,
    pub palettes: Palettes,
    pub okshell: OkShell,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct MatugenThemeCustomOnly {
    pub okshell: OkShell,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorEntry {
    pub color: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ThemedColor {
    pub dark: ColorEntry,
    pub default: ColorEntry,
    pub light: ColorEntry,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Base16 {
    pub base00: ThemedColor,
    pub base01: ThemedColor,
    pub base02: ThemedColor,
    pub base03: ThemedColor,
    pub base04: ThemedColor,
    pub base05: ThemedColor,
    pub base06: ThemedColor,
    pub base07: ThemedColor,
    pub base08: ThemedColor,
    pub base09: ThemedColor,
    #[serde(rename = "base0a")]
    pub base0a: ThemedColor,
    #[serde(rename = "base0b")]
    pub base0b: ThemedColor,
    #[serde(rename = "base0c")]
    pub base0c: ThemedColor,
    #[serde(rename = "base0d")]
    pub base0d: ThemedColor,
    #[serde(rename = "base0e")]
    pub base0e: ThemedColor,
    #[serde(rename = "base0f")]
    pub base0f: ThemedColor,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Colors {
    pub background: ThemedColor,
    pub error: ThemedColor,
    pub error_container: ThemedColor,
    pub inverse_on_surface: ThemedColor,
    pub inverse_primary: ThemedColor,
    pub inverse_surface: ThemedColor,
    pub on_background: ThemedColor,
    pub on_error: ThemedColor,
    pub on_error_container: ThemedColor,
    pub on_primary: ThemedColor,
    pub on_primary_container: ThemedColor,
    pub on_primary_fixed: ThemedColor,
    pub on_primary_fixed_variant: ThemedColor,
    pub on_secondary: ThemedColor,
    pub on_secondary_container: ThemedColor,
    pub on_secondary_fixed: ThemedColor,
    pub on_secondary_fixed_variant: ThemedColor,
    pub on_surface: ThemedColor,
    pub on_surface_variant: ThemedColor,
    pub on_tertiary: ThemedColor,
    pub on_tertiary_container: ThemedColor,
    pub on_tertiary_fixed: ThemedColor,
    pub on_tertiary_fixed_variant: ThemedColor,
    pub outline: ThemedColor,
    pub outline_variant: ThemedColor,
    pub primary: ThemedColor,
    pub primary_container: ThemedColor,
    pub primary_fixed: ThemedColor,
    pub primary_fixed_dim: ThemedColor,
    pub scrim: ThemedColor,
    pub secondary: ThemedColor,
    pub secondary_container: ThemedColor,
    pub secondary_fixed: ThemedColor,
    pub secondary_fixed_dim: ThemedColor,
    pub shadow: ThemedColor,
    pub source_color: ThemedColor,
    pub surface: ThemedColor,
    pub surface_bright: ThemedColor,
    pub surface_container: ThemedColor,
    pub surface_container_high: ThemedColor,
    pub surface_container_highest: ThemedColor,
    pub surface_container_low: ThemedColor,
    pub surface_container_lowest: ThemedColor,
    pub surface_dim: ThemedColor,
    pub surface_tint: ThemedColor,
    pub surface_variant: ThemedColor,
    pub tertiary: ThemedColor,
    pub tertiary_container: ThemedColor,
    pub tertiary_fixed: ThemedColor,
    pub tertiary_fixed_dim: ThemedColor,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Palettes {
    pub error: Palette,
    pub neutral: Palette,
    pub neutral_variant: Palette,
    pub primary: Palette,
    pub secondary: Palette,
    pub tertiary: Palette,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Palette {
    #[serde(rename = "0")]   pub t0: ColorEntry,
    #[serde(rename = "5")]   pub t5: ColorEntry,
    #[serde(rename = "10")]  pub t10: ColorEntry,
    #[serde(rename = "15")]  pub t15: ColorEntry,
    #[serde(rename = "20")]  pub t20: ColorEntry,
    #[serde(rename = "25")]  pub t25: ColorEntry,
    #[serde(rename = "30")]  pub t30: ColorEntry,
    #[serde(rename = "35")]  pub t35: ColorEntry,
    #[serde(rename = "40")]  pub t40: ColorEntry,
    #[serde(rename = "50")]  pub t50: ColorEntry,
    #[serde(rename = "60")]  pub t60: ColorEntry,
    #[serde(rename = "70")]  pub t70: ColorEntry,
    #[serde(rename = "80")]  pub t80: ColorEntry,
    #[serde(rename = "90")]  pub t90: ColorEntry,
    #[serde(rename = "95")]  pub t95: ColorEntry,
    #[serde(rename = "98")]  pub t98: ColorEntry,
    #[serde(rename = "99")]  pub t99: ColorEntry,
    #[serde(rename = "100")] pub t100: ColorEntry,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct OkShell {
    pub font: Font,
    pub sizing: Sizing,
    pub opacity: f64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Font {
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Sizing {
    pub radius_small: i32,
    pub radius_medium: i32,
    pub border_width: i32,
}

impl Default for ColorEntry {
    fn default() -> Self {
        Self { color: "#000000".to_string() }
    }
}

impl Default for ThemedColor {
    fn default() -> Self {
        let entry = ColorEntry::default();
        Self {
            dark: entry.clone(),
            default: entry.clone(),
            light: entry,
        }
    }
}

impl Default for Base16 {
    fn default() -> Self {
        Self {
            base00: Default::default(),
            base01: Default::default(),
            base02: Default::default(),
            base03: Default::default(),
            base04: Default::default(),
            base05: Default::default(),
            base06: Default::default(),
            base07: Default::default(),
            base08: Default::default(),
            base09: Default::default(),
            base0a: Default::default(),
            base0b: Default::default(),
            base0c: Default::default(),
            base0d: Default::default(),
            base0e: Default::default(),
            base0f: Default::default(),
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            t0: Default::default(),
            t5: Default::default(),
            t10: Default::default(),
            t15: Default::default(),
            t20: Default::default(),
            t25: Default::default(),
            t30: Default::default(),
            t35: Default::default(),
            t40: Default::default(),
            t50: Default::default(),
            t60: Default::default(),
            t70: Default::default(),
            t80: Default::default(),
            t90: Default::default(),
            t95: Default::default(),
            t98: Default::default(),
            t99: Default::default(),
            t100: Default::default(),
        }
    }
}

impl Default for Palettes {
    fn default() -> Self {
        Self {
            error: Default::default(),
            neutral: Default::default(),
            neutral_variant: Default::default(),
            primary: Default::default(),
            secondary: Default::default(),
            tertiary: Default::default(),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background: Default::default(),
            error: Default::default(),
            error_container: Default::default(),
            inverse_on_surface: Default::default(),
            inverse_primary: Default::default(),
            inverse_surface: Default::default(),
            on_background: Default::default(),
            on_error: Default::default(),
            on_error_container: Default::default(),
            on_primary: Default::default(),
            on_primary_container: Default::default(),
            on_primary_fixed: Default::default(),
            on_primary_fixed_variant: Default::default(),
            on_secondary: Default::default(),
            on_secondary_container: Default::default(),
            on_secondary_fixed: Default::default(),
            on_secondary_fixed_variant: Default::default(),
            on_surface: Default::default(),
            on_surface_variant: Default::default(),
            on_tertiary: Default::default(),
            on_tertiary_container: Default::default(),
            on_tertiary_fixed: Default::default(),
            on_tertiary_fixed_variant: Default::default(),
            outline: Default::default(),
            outline_variant: Default::default(),
            primary: Default::default(),
            primary_container: Default::default(),
            primary_fixed: Default::default(),
            primary_fixed_dim: Default::default(),
            scrim: Default::default(),
            secondary: Default::default(),
            secondary_container: Default::default(),
            secondary_fixed: Default::default(),
            secondary_fixed_dim: Default::default(),
            shadow: Default::default(),
            source_color: Default::default(),
            surface: Default::default(),
            surface_bright: Default::default(),
            surface_container: Default::default(),
            surface_container_high: Default::default(),
            surface_container_highest: Default::default(),
            surface_container_low: Default::default(),
            surface_container_lowest: Default::default(),
            surface_dim: Default::default(),
            surface_tint: Default::default(),
            surface_variant: Default::default(),
            tertiary: Default::default(),
            tertiary_container: Default::default(),
            tertiary_fixed: Default::default(),
            tertiary_fixed_dim: Default::default(),
        }
    }
}

impl Default for OkShell {
    fn default() -> Self {
        Self {
            font: Default::default(),
            sizing: Default::default(),
            opacity: 1.0,
        }
    }
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

impl Default for Sizing {
    fn default() -> Self {
        Self {
            radius_small: 0,
            radius_medium: 0,
            border_width: 0,
        }
    }
}

impl Default for MatugenTheme {
    fn default() -> Self {
        Self {
            base16: Default::default(),
            colors: Default::default(),
            image: String::new(),
            is_dark_mode: true,
            mode: "dark".to_string(),
            palettes: Default::default(),
            okshell: Default::default(),
        }
    }
}

pub fn color(hex: &str) -> ThemedColor {
    let entry = ColorEntry { color: hex.to_string() };
    ThemedColor {
        dark: entry.clone(),
        default: entry.clone(),
        light: entry,
    }
}

pub fn tc(dark: &str, light: &str) -> ThemedColor {
    let entry_dark = ColorEntry { color: dark.to_string() };
    let entry_light = ColorEntry { color: light.to_string() };
    ThemedColor {
        dark: entry_dark.clone(),
        default: entry_dark,
        light: entry_light,
    }
}
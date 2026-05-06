use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum Themes {
    Default,
    Wallpaper,
    Bauhaus,
    BloodRust,
    CatppuccinFrappe,
    CatppuccinLatte,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Cyberpunk,
    DesertPower,
    Dracula,
    Eldritch,
    EverforestDarkHard,
    EverforestDarkMedium,
    EverforestDarkSoft,
    EverforestLightHard,
    EverforestLightMedium,
    EverforestLightSoft,
    GruvboxDarkHard,
    GruvboxDarkMedium,
    GruvboxDarkSoft,
    GruvboxLightHard,
    GruvboxLightMedium,
    GruvboxLightSoft,
    Hackerman,
    InkyPinky,
    KanagawaDragon,
    KanagawaLotus,
    KanagawaWave,
    NordDark,
    NordLight,
    OceanicNext,
    OneDark,
    Poimandres,
    RosePine,
    RosePineDawn,
    RosePineMoon,
    Seoul,
    SolarizedDark,
    SolarizedLight,
    Synthwave84,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    Varda,
}

impl PatchField for Themes {
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

impl Themes {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Wallpaper => "Wallpaper",
            Self::Bauhaus => "Bauhaus",
            Self::BloodRust => "Blood Rust",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::Cyberpunk => "Cyberpunk",
            Self::DesertPower => "Desert Power",
            Self::Dracula => "Dracula",
            Self::Eldritch => "Eldritch",
            Self::EverforestDarkHard => "Everforest Dark Hard",
            Self::EverforestDarkMedium => "Everforest Dark Medium",
            Self::EverforestDarkSoft => "Everforest Dark Soft",
            Self::EverforestLightHard => "Everforest Light Hard",
            Self::EverforestLightMedium => "Everforest Light Medium",
            Self::EverforestLightSoft => "Everforest Light Soft",
            Self::GruvboxDarkHard => "Gruvbox Dark Hard",
            Self::GruvboxDarkMedium => "Gruvbox Dark Medium",
            Self::GruvboxDarkSoft => "Gruvbox Dark Soft",
            Self::GruvboxLightHard => "Gruvbox Light Hard",
            Self::GruvboxLightMedium => "Gruvbox Light Medium",
            Self::GruvboxLightSoft => "Gruvbox Light Soft",
            Self::Hackerman => "Hackerman",
            Self::InkyPinky => "InkyPinky",
            Self::KanagawaDragon => "Kanagawa Dragon",
            Self::KanagawaLotus => "Kanagawa Lotus",
            Self::KanagawaWave => "Kanagawa Wave",
            Self::NordDark => "Nord Dark",
            Self::NordLight => "Nord Light",
            Self::OneDark => "One Dark",
            Self::OceanicNext => "Oceanic Next",
            Self::Poimandres => "Poimandres",
            Self::RosePine => "Rosé Pine",
            Self::RosePineDawn => "Rosé Pine Dawn",
            Self::RosePineMoon => "Rosé Pine Moon",
            Self::Seoul => "Seoul",
            Self::SolarizedDark => "Solarized Dark",
            Self::SolarizedLight => "Solarized Light",
            Self::Synthwave84 => "Synthwave 84",
            Self::TokyoNight => "Tokyo Night",
            Self::TokyoNightStorm => "Tokyo Night Storm",
            Self::TokyoNightLight => "Tokyo Night Light",
            Self::Varda => "Varda",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::Wallpaper,
            Self::Bauhaus,
            Self::BloodRust,
            Self::CatppuccinFrappe,
            Self::CatppuccinLatte,
            Self::CatppuccinMacchiato,
            Self::CatppuccinMocha,
            Self::Cyberpunk,
            Self::DesertPower,
            Self::Dracula,
            Self::Eldritch,
            Self::EverforestDarkHard,
            Self::EverforestDarkMedium,
            Self::EverforestDarkSoft,
            Self::EverforestLightHard,
            Self::EverforestLightMedium,
            Self::EverforestLightSoft,
            Self::GruvboxDarkHard,
            Self::GruvboxDarkMedium,
            Self::GruvboxDarkSoft,
            Self::GruvboxLightHard,
            Self::GruvboxLightMedium,
            Self::GruvboxLightSoft,
            Self::Hackerman,
            Self::InkyPinky,
            Self::KanagawaDragon,
            Self::KanagawaLotus,
            Self::KanagawaWave,
            Self::NordDark,
            Self::NordLight,
            Self::OceanicNext,
            Self::OneDark,
            Self::Poimandres,
            Self::RosePine,
            Self::RosePineDawn,
            Self::RosePineMoon,
            Self::Seoul,
            Self::SolarizedDark,
            Self::SolarizedLight,
            Self::Synthwave84,
            Self::TokyoNight,
            Self::TokyoNightStorm,
            Self::TokyoNightLight,
            Self::Varda,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum MatugenPreference {
    Darkness,
    Lightness,
    Saturation,
    LessSaturation,
    Value,
}

impl PatchField for MatugenPreference {
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

impl fmt::Display for MatugenPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Darkness => "darkness",
            Self::Lightness => "lightness",
            Self::Saturation => "saturation",
            Self::LessSaturation => "less-saturation",
            Self::Value => "value",
        };
        f.write_str(s)
    }
}

impl MatugenPreference {
    pub fn all() -> &'static [Self] {
        &[
            Self::Darkness,
            Self::Lightness,
            Self::Saturation,
            Self::LessSaturation,
            Self::Value,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Darkness => "Darkness",
            Self::Lightness => "Lightness",
            Self::Saturation => "Saturation",
            Self::LessSaturation => "Less Saturation",
            Self::Value => "Value",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum MatugenType {
    Content,
    Expressive,
    Fidelity,
    FruitSalad,
    Monochrome,
    Neutral,
    Rainbow,
    TonalSpot,
    Vibrant,
}

impl PatchField for MatugenType {
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

impl fmt::Display for MatugenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Content => "scheme-content",
            Self::Expressive => "scheme-expressive",
            Self::Fidelity => "scheme-fidelity",
            Self::FruitSalad => "scheme-fruit-salad",
            Self::Monochrome => "scheme-monochrome",
            Self::Neutral => "scheme-neutral",
            Self::Rainbow => "scheme-rainbow",
            Self::TonalSpot => "scheme-tonal-spot",
            Self::Vibrant => "scheme-vibrant",
        };
        f.write_str(s)
    }
}

impl MatugenType {
    pub fn all() -> &'static [Self] {
        &[
            Self::Content,
            Self::Expressive,
            Self::Fidelity,
            Self::FruitSalad,
            Self::Monochrome,
            Self::Neutral,
            Self::Rainbow,
            Self::TonalSpot,
            Self::Vibrant,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Content => "Content",
            Self::Expressive => "Expressive",
            Self::Fidelity => "Fidelity",
            Self::FruitSalad => "Fruit Salad",
            Self::Monochrome => "Monochrome",
            Self::Neutral => "Neutral",
            Self::Rainbow => "Rainbow",
            Self::TonalSpot => "Tonal Spot",
            Self::Vibrant => "Vibrant",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum MatugenMode {
    Light,
    Dark,
}

impl PatchField for MatugenMode {
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

impl fmt::Display for MatugenMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Light => "light",
            Self::Dark => "dark",
        };
        f.write_str(s)
    }
}

impl MatugenMode {
    pub fn all() -> &'static [Self] {
        &[Self::Light, Self::Dark]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Store, JsonSchema)]
pub struct MatugenContrast(f64);

impl MatugenContrast {
    pub fn new(v: f64) -> Self {
        Self(v.clamp(-1.0, 1.0))
    }
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for MatugenContrast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl PatchField for MatugenContrast {
    fn patch_field(
        &mut self,
        new: Self,
        path: &StorePath,
        notify: &mut dyn FnMut(&StorePath),
        _keys: Option<&KeyMap>,
    ) {
        let clamped = MatugenContrast::new(new.0);
        if self.0 != clamped.0 {
            *self = clamped;
            notify(path);
        }
    }
}

impl PartialEq for MatugenContrast {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for MatugenContrast {}

#[derive(Clone, Debug, Serialize, Deserialize, Store, JsonSchema)]
pub struct WindowOpacity(f64);

impl WindowOpacity {
    pub fn new(v: f64) -> Self {
        Self(v.clamp(0.0, 1.0))
    }
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for WindowOpacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl PatchField for WindowOpacity {
    fn patch_field(
        &mut self,
        new: Self,
        path: &StorePath,
        notify: &mut dyn FnMut(&StorePath),
        _keys: Option<&KeyMap>,
    ) {
        let clamped = WindowOpacity::new(new.0);
        if self.0 != clamped.0 {
            *self = clamped;
            notify(path);
        }
    }
}

impl PartialEq for WindowOpacity {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for WindowOpacity {}

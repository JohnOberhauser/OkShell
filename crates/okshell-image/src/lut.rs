use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use image::{ImageBuffer, Rgb};
use lutgen::identity::correct_image;
use okshell_config::schema::themes::Themes;

const CLUT_BLOOD_RUST: &[u8] = include_bytes!("../cluts/blood_rust.bin");
const CLUT_CATPPUCCIN_FRAPPE: &[u8] = include_bytes!("../cluts/catppuccin_frappe.bin");
const CLUT_CATPPUCCIN_LATTE: &[u8] = include_bytes!("../cluts/catppuccin_latte.bin");
const CLUT_CATPPUCCIN_MACCHIATO: &[u8] = include_bytes!("../cluts/catppuccin_macchiato.bin");
const CLUT_CATPPUCCIN_MOCHA: &[u8] = include_bytes!("../cluts/catppuccin_mocha.bin");
const CLUT_DESERT_POWER: &[u8] = include_bytes!("../cluts/desert_power.bin");
const CLUT_DRACULA: &[u8] = include_bytes!("../cluts/dracula.bin");
const CLUT_EVERFOREST_DARK_HARD: &[u8] = include_bytes!("../cluts/everforest_dark_hard.bin");
const CLUT_EVERFOREST_DARK_MEDIUM: &[u8] = include_bytes!("../cluts/everforest_dark_medium.bin");
const CLUT_EVERFOREST_DARK_SOFT: &[u8] = include_bytes!("../cluts/everforest_dark_soft.bin");
const CLUT_EVERFOREST_LIGHT_HARD: &[u8] = include_bytes!("../cluts/everforest_light_hard.bin");
const CLUT_EVERFOREST_LIGHT_MEDIUM: &[u8] = include_bytes!("../cluts/everforest_light_medium.bin");
const CLUT_EVERFOREST_LIGHT_SOFT: &[u8] = include_bytes!("../cluts/everforest_light_soft.bin");
const CLUT_GRUVBOX_DARK_HARD: &[u8] = include_bytes!("../cluts/gruvbox_dark_hard.bin");
const CLUT_GRUVBOX_DARK_MEDIUM: &[u8] = include_bytes!("../cluts/gruvbox_dark_medium.bin");
const CLUT_GRUVBOX_DARK_SOFT: &[u8] = include_bytes!("../cluts/gruvbox_dark_soft.bin");
const CLUT_GRUVBOX_LIGHT_HARD: &[u8] = include_bytes!("../cluts/gruvbox_light_hard.bin");
const CLUT_GRUVBOX_LIGHT_MEDIUM: &[u8] = include_bytes!("../cluts/gruvbox_light_medium.bin");
const CLUT_GRUVBOX_LIGHT_SOFT: &[u8] = include_bytes!("../cluts/gruvbox_light_soft.bin");
const CLUT_KANAGAWA_DRAGON: &[u8] = include_bytes!("../cluts/kanagawa_dragon.bin");
const CLUT_KANAGAWA_LOTUS: &[u8] = include_bytes!("../cluts/kanagawa_lotus.bin");
const CLUT_KANAGAWA_WAVE: &[u8] = include_bytes!("../cluts/kanagawa_wave.bin");
const CLUT_NORD_DARK: &[u8] = include_bytes!("../cluts/nord_dark.bin");
const CLUT_NORD_LIGHT: &[u8] = include_bytes!("../cluts/nord_light.bin");
const CLUT_ONE_DARK: &[u8] = include_bytes!("../cluts/one_dark.bin");
const CLUT_POIMANDRES: &[u8] = include_bytes!("../cluts/poimandres.bin");
const CLUT_ROSE_PINE: &[u8] = include_bytes!("../cluts/rose_pine.bin");
const CLUT_ROSE_PINE_DAWN: &[u8] = include_bytes!("../cluts/rose_pine_dawn.bin");
const CLUT_ROSE_PINE_MOON: &[u8] = include_bytes!("../cluts/rose_pine_moon.bin");
const CLUT_SOLARIZED_DARK: &[u8] = include_bytes!("../cluts/solarized_dark.bin");
const CLUT_SOLARIZED_LIGHT: &[u8] = include_bytes!("../cluts/solarized_light.bin");
const CLUT_TOKYO_NIGHT: &[u8] = include_bytes!("../cluts/tokyo_night.bin");
const CLUT_TOKYO_NIGHT_STORM: &[u8] = include_bytes!("../cluts/tokyo_night_storm.bin");
const CLUT_TOKYO_NIGHT_LIGHT: &[u8] = include_bytes!("../cluts/tokyo_night_light.bin");
const CLUT_VARDA: &[u8] = include_bytes!("../cluts/varda.bin");

/// Look up the precomputed Hald CLUT for a static theme.
/// Returns `None` for `Default` and `Wallpaper` (dynamic themes).
pub fn embedded_clut(theme: &Themes) -> Option<&'static [u8]> {
    match theme {
        Themes::Default | Themes::Wallpaper => None,
        Themes::BloodRust => Some(CLUT_BLOOD_RUST),
        Themes::CatppuccinFrappe => Some(CLUT_CATPPUCCIN_FRAPPE),
        Themes::CatppuccinLatte => Some(CLUT_CATPPUCCIN_LATTE),
        Themes::CatppuccinMacchiato => Some(CLUT_CATPPUCCIN_MACCHIATO),
        Themes::CatppuccinMocha => Some(CLUT_CATPPUCCIN_MOCHA),
        Themes::DesertPower => Some(CLUT_DESERT_POWER),
        Themes::Dracula => Some(CLUT_DRACULA),
        Themes::EverforestDarkHard => Some(CLUT_EVERFOREST_DARK_HARD),
        Themes::EverforestDarkMedium => Some(CLUT_EVERFOREST_DARK_MEDIUM),
        Themes::EverforestDarkSoft => Some(CLUT_EVERFOREST_DARK_SOFT),
        Themes::EverforestLightHard => Some(CLUT_EVERFOREST_LIGHT_HARD),
        Themes::EverforestLightMedium => Some(CLUT_EVERFOREST_LIGHT_MEDIUM),
        Themes::EverforestLightSoft => Some(CLUT_EVERFOREST_LIGHT_SOFT),
        Themes::GruvboxDarkHard => Some(CLUT_GRUVBOX_DARK_HARD),
        Themes::GruvboxDarkMedium => Some(CLUT_GRUVBOX_DARK_MEDIUM),
        Themes::GruvboxDarkSoft => Some(CLUT_GRUVBOX_DARK_SOFT),
        Themes::GruvboxLightHard => Some(CLUT_GRUVBOX_LIGHT_HARD),
        Themes::GruvboxLightMedium => Some(CLUT_GRUVBOX_LIGHT_MEDIUM),
        Themes::GruvboxLightSoft => Some(CLUT_GRUVBOX_LIGHT_SOFT),
        Themes::KanagawaDragon => Some(CLUT_KANAGAWA_DRAGON),
        Themes::KanagawaLotus => Some(CLUT_KANAGAWA_LOTUS),
        Themes::KanagawaWave => Some(CLUT_KANAGAWA_WAVE),
        Themes::NordDark => Some(CLUT_NORD_DARK),
        Themes::NordLight => Some(CLUT_NORD_LIGHT),
        Themes::OneDark => Some(CLUT_ONE_DARK),
        Themes::Poimandres => Some(CLUT_POIMANDRES),
        Themes::RosePine => Some(CLUT_ROSE_PINE),
        Themes::RosePineDawn => Some(CLUT_ROSE_PINE_DAWN),
        Themes::RosePineMoon => Some(CLUT_ROSE_PINE_MOON),
        Themes::SolarizedDark => Some(CLUT_SOLARIZED_DARK),
        Themes::SolarizedLight => Some(CLUT_SOLARIZED_LIGHT),
        Themes::TokyoNight => Some(CLUT_TOKYO_NIGHT),
        Themes::TokyoNightStorm => Some(CLUT_TOKYO_NIGHT_STORM),
        Themes::TokyoNightLight => Some(CLUT_TOKYO_NIGHT_LIGHT),
        Themes::Varda => Some(CLUT_VARDA),
    }
}

/// Hald CLUT level used for all precomputed CLUTs.
pub const HALD_LEVEL: u8 = 8;

/// Image dimensions for a level-8 Hald CLUT (8^3 = 512).
const HALD8_DIM: u32 = 512;

/// Result of a successful palette remap operation.
pub struct RemapResult {
    pub buf: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Apply a precomputed theme filter to an image file.
///
/// Uses the embedded Hald CLUT for the given theme. Returns `None` if:
/// - The theme has no embedded CLUT (Default/Wallpaper)
/// - The image cannot be opened
/// - The operation was cancelled
pub fn apply_theme_filter(
    path: &Path,
    theme: &Themes,
    strength: f64,
    cancel: &AtomicBool,
) -> Option<RemapResult> {
    let clut_bytes = embedded_clut(theme)?;

    if cancel.load(Ordering::Relaxed) {
        return None;
    }

    let hald_clut = load_embedded_clut(clut_bytes);

    let mut img = image::open(path).ok()?.into_rgba8();
    let (width, height) = img.dimensions();

    let original = if strength < 1.0 {
        Some(img.clone())
    } else {
        None
    };

    correct_image(&mut img, &hald_clut);

    if cancel.load(Ordering::Relaxed) {
        return None;
    }

    if let Some(original) = original {
        blend_buffers(img.as_mut(), original.as_raw(), strength as f32);
    }

    Some(RemapResult {
        buf: img.into_raw(),
        width,
        height,
    })
}

fn load_embedded_clut(bytes: &[u8]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    ImageBuffer::from_raw(HALD8_DIM, HALD8_DIM, bytes.to_vec())
        .expect("embedded CLUT data is invalid")
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8
}

fn blend_buffers(dst: &mut [u8], src: &[u8], t: f32) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = lerp_u8(*s, *d, t);
    }
}
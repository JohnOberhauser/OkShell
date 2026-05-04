use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use image::{ImageBuffer, Rgb, RgbaImage};
use lutgen::identity::correct_image;
use okshell_config::schema::themes::Themes;
use okshell_matugen::static_theme_mapping::static_theme;
use relm4::gtk;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::gtk::prelude::{Cast, GskRendererExt, SnapshotExt, TextureExt, TextureExtManual};

const CLUT_BLOOD_RUST: &[u8] = include_bytes!("../cluts/blood_rust.bin");
const CLUT_CATPPUCCIN_FRAPPE: &[u8] = include_bytes!("../cluts/catppuccin_frappe.bin");
const CLUT_CATPPUCCIN_LATTE: &[u8] = include_bytes!("../cluts/catppuccin_latte.bin");
const CLUT_CATPPUCCIN_MACCHIATO: &[u8] = include_bytes!("../cluts/catppuccin_macchiato.bin");
const CLUT_CATPPUCCIN_MOCHA: &[u8] = include_bytes!("../cluts/catppuccin_mocha.bin");
const CLUT_CYBERPUNK: &[u8] = include_bytes!("../cluts/cyberpunk.bin");
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
const CLUT_OCEANIC_NEXT: &[u8] = include_bytes!("../cluts/oceanic_next.bin");
const CLUT_ONE_DARK: &[u8] = include_bytes!("../cluts/one_dark.bin");
const CLUT_POIMANDRES: &[u8] = include_bytes!("../cluts/poimandres.bin");
const CLUT_ROSE_PINE: &[u8] = include_bytes!("../cluts/rose_pine.bin");
const CLUT_ROSE_PINE_DAWN: &[u8] = include_bytes!("../cluts/rose_pine_dawn.bin");
const CLUT_ROSE_PINE_MOON: &[u8] = include_bytes!("../cluts/rose_pine_moon.bin");
const CLUT_SEOUL: &[u8] = include_bytes!("../cluts/seoul.bin");
const CLUT_SOLARIZED_DARK: &[u8] = include_bytes!("../cluts/solarized_dark.bin");
const CLUT_SOLARIZED_LIGHT: &[u8] = include_bytes!("../cluts/solarized_light.bin");
const CLUT_SYNTHWAVE_84: &[u8] = include_bytes!("../cluts/synthwave_84.bin");
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
        Themes::Cyberpunk => Some(CLUT_CYBERPUNK),
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
        Themes::OceanicNext => Some(CLUT_OCEANIC_NEXT),
        Themes::OneDark => Some(CLUT_ONE_DARK),
        Themes::Poimandres => Some(CLUT_POIMANDRES),
        Themes::RosePine => Some(CLUT_ROSE_PINE),
        Themes::RosePineDawn => Some(CLUT_ROSE_PINE_DAWN),
        Themes::RosePineMoon => Some(CLUT_ROSE_PINE_MOON),
        Themes::Seoul => Some(CLUT_SEOUL),
        Themes::SolarizedDark => Some(CLUT_SOLARIZED_DARK),
        Themes::SolarizedLight => Some(CLUT_SOLARIZED_LIGHT),
        Themes::Synthwave84 => Some(CLUT_SYNTHWAVE_84),
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
    contrast_adjustment: f64,
    monochrome: f64,
    cancel: &AtomicBool,
) -> Option<RemapResult> {
    let clut_bytes = embedded_clut(theme)?;

    if cancel.load(Ordering::Relaxed) {
        return None;
    }

    let hald_clut = load_embedded_clut(clut_bytes);

    let mut img = decode_pixbuf_rgba(path)?;
    let (width, height) = img.dimensions();

    if monochrome > 0.0 {
        let tint = static_theme(theme, None)
            .map(|t| {
                let rgb = t.colors.on_surface.default.as_rgb();
                [rgb.0, rgb.1, rgb.2]
            })
            .unwrap_or([255, 255, 255]);
        apply_monochrome(img.as_mut(), tint, monochrome);
    }

    let original = if strength < 1.0 {
        Some(img.clone())
    } else {
        None
    };

    correct_image(&mut img, &hald_clut);

    adjust_contrast(img.as_mut(), contrast_adjustment);

    if cancel.load(Ordering::Relaxed) {
        return None;
    }

    if let Some(original) = original {
        blend_buffers(img.as_mut(), original.as_raw(), strength);
    }

    Some(RemapResult {
        buf: img.into_raw(),
        width,
        height,
    })
}

/// Decode an image file to RGBA using gdk-pixbuf.
pub fn decode_pixbuf_rgba(path: &Path) -> Option<RgbaImage> {
    let pixbuf = Pixbuf::from_file(path).ok()?;

    let width = pixbuf.width() as u32;
    let height = pixbuf.height() as u32;
    let n_channels = pixbuf.n_channels() as u32;
    let rowstride = pixbuf.rowstride() as u32;
    let has_alpha = pixbuf.has_alpha();
    let pixels = unsafe { pixbuf.pixels() };

    let mut rgba_buf = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        let row_offset = (y * rowstride) as usize;
        for x in 0..width {
            let src = row_offset + (x * n_channels) as usize;
            let dst = ((y * width + x) * 4) as usize;

            rgba_buf[dst] = pixels[src]; // R
            rgba_buf[dst + 1] = pixels[src + 1]; // G
            rgba_buf[dst + 2] = pixels[src + 2]; // B
            rgba_buf[dst + 3] = if has_alpha { pixels[src + 3] } else { 255 };
        }
    }

    RgbaImage::from_raw(width, height, rgba_buf)
}

fn load_embedded_clut(bytes: &[u8]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    ImageBuffer::from_raw(HALD8_DIM, HALD8_DIM, bytes.to_vec())
        .expect("embedded CLUT data is invalid")
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).clamp(0.0, 255.0) as u8
}

fn blend_buffers(dst: &mut [u8], src: &[u8], t: f64) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = lerp_u8(*s, *d, t);
    }
}

/// Render a paintable to pixels via snapshot, apply the CLUT, return a new texture.
pub fn snapshot_and_recolor(
    paintable: &gtk::IconPaintable,
    color_theme: &Themes,
) -> Option<gtk::gdk::Texture> {
    use gtk::graphene;
    use gtk::prelude::PaintableExt;

    let w = paintable.intrinsic_width().max(48) as f64;
    let h = paintable.intrinsic_height().max(48) as f64;

    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(snapshot.upcast_ref::<gtk::gdk::Snapshot>(), w, h);
    let node = snapshot.to_node()?;

    // Simpler approach: use CairoRenderer which doesn't need a surface
    let renderer = gtk::gsk::CairoRenderer::new();
    renderer.realize(None::<&gtk::gdk::Surface>).ok()?;

    let rect = graphene::Rect::new(0.0, 0.0, w as f32, h as f32);
    let texture = renderer.render_texture(&node, Some(&rect));
    renderer.unrealize();

    let width = texture.width() as u32;
    let height = texture.height() as u32;
    let stride = width * 4;
    let mut pixels = vec![0u8; (stride * height) as usize];
    texture.download(&mut pixels, stride as usize);

    // Build an RgbaImage from the downloaded pixels and apply the CLUT
    let mut img = RgbaImage::from_raw(width, height, pixels)?;
    let clut_bytes = embedded_clut(color_theme)?;
    let hald_clut = ImageBuffer::from_raw(512, 512, clut_bytes.to_vec())?;
    correct_image(&mut img, &hald_clut);

    rgba_to_texture(img.as_raw(), width, height)
}

pub fn rgba_to_texture(buf: &[u8], width: u32, height: u32) -> Option<gtk::gdk::Texture> {
    let bytes = gtk::glib::Bytes::from(buf);
    let texture = gtk::gdk::MemoryTexture::new(
        width as i32,
        height as i32,
        gtk::gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (width * 4) as usize,
    );
    Some(texture.upcast())
}

/// Adjust contrast of an RGBA buffer in-place.
/// `factor` > 1.0 increases contrast, < 1.0 decreases it.
/// 1.0 is no change.
fn adjust_contrast(buf: &mut [u8], factor: f64) {
    for chunk in buf.chunks_exact_mut(4) {
        chunk[0] =
            ((((chunk[0] as f64 / 255.0) - 0.5) * factor + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
        chunk[1] =
            ((((chunk[1] as f64 / 255.0) - 0.5) * factor + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
        chunk[2] =
            ((((chunk[2] as f64 / 255.0) - 0.5) * factor + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
        // leave alpha (chunk[3]) untouched
    }
}

/// Desaturate toward a tint color.
/// `factor` 0.0 = no change, 1.0 = fully desaturated and tinted.
fn apply_monochrome(buf: &mut [u8], tint: [u8; 3], factor: f64) {
    for chunk in buf.chunks_exact_mut(4) {
        // Rec. 709 luminance
        let luma = (chunk[0] as f32 * 0.2126 + chunk[1] as f32 * 0.7152 + chunk[2] as f32 * 0.0722)
            .clamp(0.0, 255.0);

        // Blend tint color by luma (preserves light/dark variation)
        let tinted_r = tint[0] as f32 * (luma / 255.0);
        let tinted_g = tint[1] as f32 * (luma / 255.0);
        let tinted_b = tint[2] as f32 * (luma / 255.0);

        chunk[0] = lerp_u8(chunk[0], tinted_r as u8, factor);
        chunk[1] = lerp_u8(chunk[1], tinted_g as u8, factor);
        chunk[2] = lerp_u8(chunk[2], tinted_b as u8, factor);
        // alpha untouched
    }
}

use std::fs;
use std::path::PathBuf;

use image::RgbImage;
use lutgen::GenerateLut;
use lutgen::interpolation::GaussianRemapper;
use okshell_config::schema::themes::Themes;
use okshell_matugen::json_struct::MatugenTheme;
use okshell_matugen::static_theme_mapping::static_theme;

const HALD_LEVEL: u8 = 8;
const GAUSSIAN_SHAPE: f64 = 96.0;
const GAUSSIAN_NEAREST: usize = 0;
const LUM_FACTOR: f64 = 1.0;
const PRESERVE: bool = false;

fn extract_palette(theme: &MatugenTheme) -> Vec<[u8; 3]> {
    let c = &theme.colors;
    let b = &theme.base16;

    let colors = vec![
        c.background.default.as_rgb(),
        c.error.default.as_rgb(),
        c.error_container.default.as_rgb(),
        c.inverse_on_surface.default.as_rgb(),
        c.inverse_primary.default.as_rgb(),
        c.inverse_surface.default.as_rgb(),
        c.on_background.default.as_rgb(),
        c.on_error.default.as_rgb(),
        c.on_error_container.default.as_rgb(),
        c.on_primary.default.as_rgb(),
        c.on_primary_container.default.as_rgb(),
        c.on_primary_fixed.default.as_rgb(),
        c.on_primary_fixed_variant.default.as_rgb(),
        c.on_secondary.default.as_rgb(),
        c.on_secondary_container.default.as_rgb(),
        c.on_secondary_fixed.default.as_rgb(),
        c.on_secondary_fixed_variant.default.as_rgb(),
        c.on_surface.default.as_rgb(),
        c.on_surface_variant.default.as_rgb(),
        c.on_tertiary.default.as_rgb(),
        c.on_tertiary_container.default.as_rgb(),
        c.on_tertiary_fixed.default.as_rgb(),
        c.on_tertiary_fixed_variant.default.as_rgb(),
        c.outline.default.as_rgb(),
        c.outline_variant.default.as_rgb(),
        c.primary.default.as_rgb(),
        c.primary_container.default.as_rgb(),
        c.primary_fixed.default.as_rgb(),
        c.primary_fixed_dim.default.as_rgb(),
        c.scrim.default.as_rgb(),
        c.secondary.default.as_rgb(),
        c.secondary_container.default.as_rgb(),
        c.secondary_fixed.default.as_rgb(),
        c.secondary_fixed_dim.default.as_rgb(),
        c.shadow.default.as_rgb(),
        c.source_color.default.as_rgb(),
        c.surface.default.as_rgb(),
        c.surface_bright.default.as_rgb(),
        c.surface_container.default.as_rgb(),
        c.surface_container_high.default.as_rgb(),
        c.surface_container_highest.default.as_rgb(),
        c.surface_container_low.default.as_rgb(),
        c.surface_container_lowest.default.as_rgb(),
        c.surface_dim.default.as_rgb(),
        c.surface_tint.default.as_rgb(),
        c.surface_variant.default.as_rgb(),
        c.tertiary.default.as_rgb(),
        c.tertiary_container.default.as_rgb(),
        c.tertiary_fixed.default.as_rgb(),
        c.tertiary_fixed_dim.default.as_rgb(),
        b.base00.default.as_rgb(),
        b.base01.default.as_rgb(),
        b.base02.default.as_rgb(),
        b.base03.default.as_rgb(),
        b.base04.default.as_rgb(),
        b.base05.default.as_rgb(),
        b.base06.default.as_rgb(),
        b.base07.default.as_rgb(),
        b.base08.default.as_rgb(),
        b.base09.default.as_rgb(),
        b.base0a.default.as_rgb(),
        b.base0b.default.as_rgb(),
        b.base0c.default.as_rgb(),
        b.base0d.default.as_rgb(),
        b.base0e.default.as_rgb(),
        b.base0f.default.as_rgb(),
    ];

    colors.into_iter().map(|(r, g, b)| [r, g, b]).collect()
}

fn static_themes() -> Vec<(&'static str, Themes)> {
    vec![
        ("bauhaus", Themes::Bauhaus),
        ("blood_rust", Themes::BloodRust),
        ("catppuccin_frappe", Themes::CatppuccinFrappe),
        ("catppuccin_latte", Themes::CatppuccinLatte),
        ("catppuccin_macchiato", Themes::CatppuccinMacchiato),
        ("catppuccin_mocha", Themes::CatppuccinMocha),
        ("cyberpunk", Themes::Cyberpunk),
        ("desert_power", Themes::DesertPower),
        ("dracula", Themes::Dracula),
        ("eldritch", Themes::Eldritch),
        ("everforest_dark_hard", Themes::EverforestDarkHard),
        ("everforest_dark_medium", Themes::EverforestDarkMedium),
        ("everforest_dark_soft", Themes::EverforestDarkSoft),
        ("everforest_light_hard", Themes::EverforestLightHard),
        ("everforest_light_medium", Themes::EverforestLightMedium),
        ("everforest_light_soft", Themes::EverforestLightSoft),
        ("gruvbox_dark_hard", Themes::GruvboxDarkHard),
        ("gruvbox_dark_medium", Themes::GruvboxDarkMedium),
        ("gruvbox_dark_soft", Themes::GruvboxDarkSoft),
        ("gruvbox_light_hard", Themes::GruvboxLightHard),
        ("gruvbox_light_medium", Themes::GruvboxLightMedium),
        ("gruvbox_light_soft", Themes::GruvboxLightSoft),
        ("hackerman", Themes::Hackerman),
        ("inky_pinky", Themes::InkyPinky),
        ("kanagawa_dragon", Themes::KanagawaDragon),
        ("kanagawa_lotus", Themes::KanagawaLotus),
        ("kanagawa_wave", Themes::KanagawaWave),
        ("miasma", Themes::Miasma),
        ("monokai_classic", Themes::MonokaiClassic),
        ("nord_dark", Themes::NordDark),
        ("nord_light", Themes::NordLight),
        ("oceanic_next", Themes::OceanicNext),
        ("one_dark", Themes::OneDark),
        ("poimandres", Themes::Poimandres),
        ("rose_pine", Themes::RosePine),
        ("rose_pine_dawn", Themes::RosePineDawn),
        ("rose_pine_moon", Themes::RosePineMoon),
        ("saga", Themes::Saga),
        ("seoul", Themes::Seoul),
        ("solarized_dark", Themes::SolarizedDark),
        ("solarized_light", Themes::SolarizedLight),
        ("synthwave_84", Themes::Synthwave84),
        ("tokyo_night", Themes::TokyoNight),
        ("tokyo_night_storm", Themes::TokyoNightStorm),
        ("tokyo_night_light", Themes::TokyoNightLight),
        ("varda", Themes::Varda),
    ]
}

fn generate_clut(palette: &[[u8; 3]]) -> RgbImage {
    let remapper = GaussianRemapper::new(
        palette,
        GAUSSIAN_SHAPE,
        GAUSSIAN_NEAREST,
        LUM_FACTOR,
        PRESERVE,
    );
    remapper.par_generate_lut(HALD_LEVEL)
}

fn main() {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("okshell_clut_gen must be in a workspace")
        .join("okshell-image")
        .join("cluts");

    fs::create_dir_all(&out_dir).expect("failed to create cluts directory");

    let themes = static_themes();
    let total = themes.len();

    for (i, (name, theme_variant)) in themes.iter().enumerate() {
        let theme = static_theme(theme_variant, None)
            .unwrap_or_else(|| panic!("static_theme returned None for {name}"));
        let palette = extract_palette(&theme);
        let clut = generate_clut(&palette);
        let raw = clut.into_raw();

        let path = out_dir.join(format!("{name}.bin"));
        fs::write(&path, &raw).expect("failed to write CLUT file");

        println!(
            "[{}/{}] Generated {} ({} bytes)",
            i + 1,
            total,
            name,
            raw.len()
        );
    }

    println!("\nDone! Generated {total} CLUTs in {}", out_dir.display());
}

use crate::json_struct::{ThemedColor, color};

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn parse_hex(hex: &str) -> [f32; 3] {
    let h = hex.trim_start_matches('#');
    let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0) as f32 / 255.0;
    [ch(0), ch(2), ch(4)]
}

fn to_oklab(rgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb.map(srgb_to_linear);

    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();

    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

fn from_oklab(lab: [f32; 3]) -> [f32; 3] {
    let [ll, a, b] = lab;

    let l = (ll + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (ll - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (ll - 0.0894841775 * a - 1.2914855480 * b).powi(3);

    [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ]
    .map(|c| linear_to_srgb(c.clamp(0.0, 1.0)))
}

/// Blend `accent` into `base` by `t` (0.0 = base, 1.0 = accent) in OKLab.
/// Returns the hex string so it can feed `color()` or `tc()`.
pub fn mix_hex(base: &str, accent: &str, t: f32) -> String {
    let a = to_oklab(parse_hex(base));
    let b = to_oklab(parse_hex(accent));
    let t = t.clamp(0.0, 1.0);

    let lab = [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ];

    let [r, g, bl] = from_oklab(lab).map(|c| (c * 255.0).round() as u8);
    format!("#{r:02X}{g:02X}{bl:02X}")
}

/// Single-value version, matches `color()`.
pub fn mix(base: &str, accent: &str, t: f32) -> ThemedColor {
    color(&mix_hex(base, accent, t))
}

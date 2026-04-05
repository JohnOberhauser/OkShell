/// Convert a color temperature in Kelvin to (R, G, B) multipliers in [0.0, 1.0].
///
/// Uses the Tanner Helland piecewise polynomial approximation — the same
/// basis as wlsunset and redshift.  Valid range: 1000 – 25000 K.
pub fn kelvin_to_rgb(temp_k: u32) -> (f64, f64, f64) {
    let t = (temp_k as f64).clamp(1000.0, 25000.0) / 100.0;

    let r = if t <= 66.0 {
        1.0
    } else {
        (329.698_727_446 * (t - 60.0).powf(-0.133_204_759_2) / 255.0).clamp(0.0, 1.0)
    };

    let g = if t <= 66.0 {
        ((99.470_802_586_1 * t.ln() - 161.119_568_166_1) / 255.0).clamp(0.0, 1.0)
    } else {
        (288.122_169_528_3 * (t - 60.0).powf(-0.075_514_849_2) / 255.0).clamp(0.0, 1.0)
    };

    let b = if t >= 66.0 {
        1.0
    } else if t <= 19.0 {
        0.0
    } else {
        ((138.517_731_223_1 * (t - 10.0).ln() - 305.044_792_730_7) / 255.0).clamp(0.0, 1.0)
    };

    (r, g, b)
}

/// Build a gamma ramp buffer for `zwlr_gamma_control_v1.set_gamma`.
///
/// Layout: `[R₀..Rₙ, G₀..Gₙ, B₀..Bₙ]` — `ramp_size * 3` u16 values.
/// Passing `temp_k = 6500` and `gamma = 1.0` produces a neutral linear ramp.
pub fn build_ramp(temp_k: u32, gamma: f64, ramp_size: usize) -> Vec<u16> {
    let (r_mul, g_mul, b_mul) = kelvin_to_rgb(temp_k);
    let n = ramp_size;
    let mut buf = vec![0u16; n * 3];

    for i in 0..n {
        let v = i as f64 / (n - 1) as f64;
        let curve = v.powf(1.0 / gamma);
        buf[i]         = (curve * r_mul * 65535.0).round() as u16;
        buf[n + i]     = (curve * g_mul * 65535.0).round() as u16;
        buf[2 * n + i] = (curve * b_mul * 65535.0).round() as u16;
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_is_white() {
        let (r, g, b) = kelvin_to_rgb(6500);
        assert!((r - 1.0).abs() < 0.02, "r={r}");
        assert!((g - 1.0).abs() < 0.02, "g={g}");
        assert!((b - 1.0).abs() < 0.02, "b={b}");
    }

    #[test]
    fn warm_reduces_blue() {
        let (r, _, b) = kelvin_to_rgb(2700);
        assert!(r > 0.9, "r={r}");
        assert!(b < 0.4, "b={b}");
    }

    #[test]
    fn ramp_layout() {
        let ramp = build_ramp(6500, 1.0, 256);
        assert_eq!(ramp.len(), 768);
        assert_eq!(ramp[0], 0);           // first R entry is black
        assert!(ramp[255] > 65000);       // last R entry is near-white
    }
}
use crate::json_struct::{Base16, Colors, MatugenTheme, OkShell, Palettes, color};

pub fn radioactive(okshell: OkShell) -> MatugenTheme {
    MatugenTheme {
        okshell,
        image: String::new(),
        is_dark_mode: true,
        mode: "dark".to_string(),
        base16: Base16 {
            base00: color("#1a1a14"), // bg (derived, no explicit bg given)
            base01: color("#22221a"),
            base02: color("#282822"), // cloud0
            base03: color("#74745d"), // cloud8 - comments
            base04: color("#a2a284"), // cloud14
            base05: color("#d9d9cf"), // fg/cloud15
            base06: color("#dfdfa9"), // cloud6
            base07: color("#a39778"), // cloud7
            base08: color("#cd749c"), // cloud9 - red/pink
            base09: color("#a38c78"), // cloud10 - orange (muted)
            base0a: color("#e3e3a8"), // cloud1 - yellow
            base0b: color("#aeae98"), // cloud3 - green (muted olive)
            base0c: color("#bbbc57"), // cloud11 - cyan slot (yellow-green here)
            base0d: color("#7b6c97"), // cloud4 - blue (purple-ish)
            base0e: color("#56417a"), // cloud2 - purple
            base0f: color("#424234"), // cloud13 - brown
        },
        palettes: Palettes::default(),
        colors: Colors {
            surface: color("#1a1a14"),
            on_surface: color("#d9d9cf"),
            surface_variant: color("#22221a"),
            on_surface_variant: color("#a2a284"),
            surface_container_highest: color("#34342a"),
            surface_container_high: color("#2a2a22"),
            surface_container: color("#22221a"),
            surface_container_low: color("#1d1d16"),
            surface_container_lowest: color("#1a1a14"),
            inverse_surface: color("#d9d9cf"),
            inverse_on_surface: color("#1a1a14"),
            surface_tint: color("#bbbc57"),
            primary: color("#bbbc57"),
            on_primary: color("#1a1a14"),
            primary_container: color("#282822"),
            on_primary_container: color("#bbbc57"),
            secondary: color("#7b6c97"),
            on_secondary: color("#1a1a14"),
            secondary_container: color("#282822"),
            on_secondary_container: color("#7b6c97"),
            tertiary: color("#e3e3a8"),
            on_tertiary: color("#1a1a14"),
            tertiary_container: color("#282822"),
            on_tertiary_container: color("#e3e3a8"),
            error: color("#cd749c"),
            on_error: color("#1a1a14"),
            error_container: color("#282822"),
            on_error_container: color("#cd749c"),
            outline: color("#74745d"),
            outline_variant: color("#282822"),
            background: color("#1a1a14"),
            on_background: color("#d9d9cf"),
            inverse_primary: color("#bbbc57"),
            primary_fixed: color("#bbbc57"),
            primary_fixed_dim: color("#dfdfa9"),
            on_primary_fixed: color("#1a1a14"),
            on_primary_fixed_variant: color("#d9d9cf"),
            secondary_fixed: color("#7b6c97"),
            secondary_fixed_dim: color("#56417a"),
            on_secondary_fixed: color("#1a1a14"),
            on_secondary_fixed_variant: color("#d9d9cf"),
            tertiary_fixed: color("#e3e3a8"),
            tertiary_fixed_dim: color("#aeae98"),
            on_tertiary_fixed: color("#1a1a14"),
            on_tertiary_fixed_variant: color("#d9d9cf"),
            scrim: color("#1a1a14"),
            shadow: color("#1a1a14"),
            source_color: color("#bbbc57"),
            surface_bright: color("#34342a"),
            surface_dim: color("#1a1a14"),
        },
    }
}

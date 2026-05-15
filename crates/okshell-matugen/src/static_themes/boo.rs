use crate::json_struct::{Base16, Colors, MatugenTheme, OkShell, Palettes, color};

pub fn boo(okshell: OkShell) -> MatugenTheme {
    MatugenTheme {
        okshell,
        image: String::new(),
        is_dark_mode: true,
        mode: "dark".to_string(),
        base16: Base16 {
            base00: color("#111113"), // bg
            base01: color("#1a1a1f"),
            base02: color("#222827"), // cloud0
            base03: color("#5d6f74"), // cloud8 - comments
            base04: color("#849da2"), // cloud14
            base05: color("#e4dcec"), // fg
            base06: color("#e6ebe5"), // cloud7
            base07: color("#d9d6cf"), // cloud15
            base08: color("#cd749c"), // cloud9 - red/pink
            base09: color("#d5a8e4"), // cloud1 - orange-ish (lavender)
            base0a: color("#c0c0dd"), // cloud11 - yellow (pale)
            base0b: color("#9c75dd"), // cloud2 - green slot (purple here)
            base0c: color("#63b0b0"), // cloud10 - cyan
            base0d: color("#5786bc"), // cloud12 - blue
            base0e: color("#654a96"), // cloud4 - purple
            base0f: color("#3f3442"), // cloud13 - brown
        },
        palettes: Palettes::default(),
        colors: Colors {
            surface: color("#111113"),
            on_surface: color("#e4dcec"),
            surface_variant: color("#1a1a1f"),
            on_surface_variant: color("#849da2"),
            surface_container_highest: color("#2a2a30"),
            surface_container_high: color("#222227"),
            surface_container: color("#1b1b20"),
            surface_container_low: color("#161618"),
            surface_container_lowest: color("#111113"),
            inverse_surface: color("#e4dcec"),
            inverse_on_surface: color("#111113"),
            surface_tint: color("#9c75dd"),
            primary: color("#9c75dd"),
            on_primary: color("#111113"),
            primary_container: color("#222227"),
            on_primary_container: color("#9c75dd"),
            secondary: color("#63b0b0"),
            on_secondary: color("#111113"),
            secondary_container: color("#222227"),
            on_secondary_container: color("#63b0b0"),
            tertiary: color("#5786bc"),
            on_tertiary: color("#111113"),
            tertiary_container: color("#222227"),
            on_tertiary_container: color("#5786bc"),
            error: color("#cd749c"),
            on_error: color("#111113"),
            error_container: color("#222227"),
            on_error_container: color("#cd749c"),
            outline: color("#5d6f74"),
            outline_variant: color("#222827"),
            background: color("#111113"),
            on_background: color("#e4dcec"),
            inverse_primary: color("#9c75dd"),
            primary_fixed: color("#9c75dd"),
            primary_fixed_dim: color("#d5a8e4"),
            on_primary_fixed: color("#111113"),
            on_primary_fixed_variant: color("#e4dcec"),
            secondary_fixed: color("#63b0b0"),
            secondary_fixed_dim: color("#a9d1df"),
            on_secondary_fixed: color("#111113"),
            on_secondary_fixed_variant: color("#e4dcec"),
            tertiary_fixed: color("#5786bc"),
            tertiary_fixed_dim: color("#849da2"),
            on_tertiary_fixed: color("#111113"),
            on_tertiary_fixed_variant: color("#e4dcec"),
            scrim: color("#111113"),
            shadow: color("#111113"),
            source_color: color("#9c75dd"),
            surface_bright: color("#2a2a30"),
            surface_dim: color("#111113"),
        },
    }
}

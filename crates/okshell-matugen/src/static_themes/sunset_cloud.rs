use crate::json_struct::{Base16, Colors, MatugenTheme, OkShell, Palettes, color};

pub fn sunset_cloud(okshell: OkShell) -> MatugenTheme {
    MatugenTheme {
        okshell,
        image: String::new(),
        is_dark_mode: true,
        mode: "dark".to_string(),
        base16: Base16 {
            base00: color("#0e0f06"), // bg
            base01: color("#1a1c0d"),
            base02: color("#262904"), // cloud0
            base03: color("#5d6f74"), // cloud8 - comments
            base04: color("#809d7b"), // cloud14
            base05: color("#ffffcf"), // fg
            base06: color("#e0ec7a"), // cloud7
            base07: color("#dad36d"), // cloud15
            base08: color("#bc5050"), // cloud9 - red
            base09: color("#cd6d91"), // cloud1 - orange (pinkish)
            base0a: color("#bbc373"), // cloud11 - yellow
            base0b: color("#8d9a59"), // cloud3 - green
            base0c: color("#71d17c"), // cloud6 - cyan (green-leaning)
            base0d: color("#40719c"), // cloud12 - blue
            base0e: color("#947fbc"), // cloud2 - purple
            base0f: color("#3c2e1a"), // cloud13 - brown
        },
        palettes: Palettes::default(),
        colors: Colors {
            surface: color("#0e0f06"),
            on_surface: color("#ffffcf"),
            surface_variant: color("#1a1c0d"),
            on_surface_variant: color("#809d7b"),
            surface_container_highest: color("#2e3110"),
            surface_container_high: color("#262904"),
            surface_container: color("#1e2008"),
            surface_container_low: color("#181a07"),
            surface_container_lowest: color("#0e0f06"),
            inverse_surface: color("#ffffcf"),
            inverse_on_surface: color("#0e0f06"),
            surface_tint: color("#bbc373"),
            primary: color("#bbc373"),
            on_primary: color("#0e0f06"),
            primary_container: color("#262904"),
            on_primary_container: color("#bbc373"),
            secondary: color("#cd6d91"),
            on_secondary: color("#0e0f06"),
            secondary_container: color("#262904"),
            on_secondary_container: color("#cd6d91"),
            tertiary: color("#71d17c"),
            on_tertiary: color("#0e0f06"),
            tertiary_container: color("#262904"),
            on_tertiary_container: color("#71d17c"),
            error: color("#bc5050"),
            on_error: color("#0e0f06"),
            error_container: color("#262904"),
            on_error_container: color("#bc5050"),
            outline: color("#5d6f74"),
            outline_variant: color("#262904"),
            background: color("#0e0f06"),
            on_background: color("#ffffcf"),
            inverse_primary: color("#bbc373"),
            primary_fixed: color("#bbc373"),
            primary_fixed_dim: color("#dad36d"),
            on_primary_fixed: color("#0e0f06"),
            on_primary_fixed_variant: color("#ffffcf"),
            secondary_fixed: color("#cd6d91"),
            secondary_fixed_dim: color("#947fbc"),
            on_secondary_fixed: color("#0e0f06"),
            on_secondary_fixed_variant: color("#ffffcf"),
            tertiary_fixed: color("#71d17c"),
            tertiary_fixed_dim: color("#8d9a59"),
            on_tertiary_fixed: color("#0e0f06"),
            on_tertiary_fixed_variant: color("#ffffcf"),
            scrim: color("#0e0f06"),
            shadow: color("#0e0f06"),
            source_color: color("#bbc373"),
            surface_bright: color("#2e3110"),
            surface_dim: color("#0e0f06"),
        },
    }
}

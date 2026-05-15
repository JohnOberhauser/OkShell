use crate::json_struct::{Base16, Colors, MatugenTheme, OkShell, Palettes, color};

pub fn crimson_moonlight(okshell: OkShell) -> MatugenTheme {
    MatugenTheme {
        okshell,
        image: String::new(),
        is_dark_mode: true,
        mode: "dark".to_string(),
        base16: Base16 {
            base00: color("#0f0e0e"), // bg
            base01: color("#1a1414"),
            base02: color("#201c1c"), // cloud0
            base03: color("#796769"), // cloud8 - comments
            base04: color("#ab9295"), // cloud14
            base05: color("#f9f2f3"), // fg
            base06: color("#e9bfc4"), // cloud6
            base07: color("#f9f1f2"), // cloud7
            base08: color("#dd5571"), // cloud9 - red
            base09: color("#e15774"), // cloud2 - orange (pink-red)
            base0a: color("#fca2ae"), // cloud1 - yellow slot (rose)
            base0b: color("#bc969a"), // cloud3 - green slot (muted rose)
            base0c: color("#d19299"), // cloud10 - cyan slot (rose)
            base0d: color("#b96a76"), // cloud12 - blue slot (deep rose)
            base0e: color("#714e75"), // cloud4 - purple
            base0f: color("#473234"), // cloud13 - brown
        },
        palettes: Palettes::default(),
        colors: Colors {
            surface: color("#0f0e0e"),
            on_surface: color("#f9f2f3"),
            surface_variant: color("#1a1414"),
            on_surface_variant: color("#ab9295"),
            surface_container_highest: color("#2c2424"),
            surface_container_high: color("#241c1c"),
            surface_container: color("#1d1717"),
            surface_container_low: color("#161212"),
            surface_container_lowest: color("#0f0e0e"),
            inverse_surface: color("#f9f2f3"),
            inverse_on_surface: color("#0f0e0e"),
            surface_tint: color("#e15774"),
            primary: color("#e15774"),
            on_primary: color("#0f0e0e"),
            primary_container: color("#201c1c"),
            on_primary_container: color("#e15774"),
            secondary: color("#fca2ae"),
            on_secondary: color("#0f0e0e"),
            secondary_container: color("#201c1c"),
            on_secondary_container: color("#fca2ae"),
            tertiary: color("#714e75"),
            on_tertiary: color("#0f0e0e"),
            tertiary_container: color("#201c1c"),
            on_tertiary_container: color("#714e75"),
            error: color("#dd5571"),
            on_error: color("#0f0e0e"),
            error_container: color("#201c1c"),
            on_error_container: color("#dd5571"),
            outline: color("#796769"),
            outline_variant: color("#201c1c"),
            background: color("#0f0e0e"),
            on_background: color("#f9f2f3"),
            inverse_primary: color("#e15774"),
            primary_fixed: color("#e15774"),
            primary_fixed_dim: color("#dd5571"),
            on_primary_fixed: color("#0f0e0e"),
            on_primary_fixed_variant: color("#f9f2f3"),
            secondary_fixed: color("#fca2ae"),
            secondary_fixed_dim: color("#e9bfc4"),
            on_secondary_fixed: color("#0f0e0e"),
            on_secondary_fixed_variant: color("#f9f2f3"),
            tertiary_fixed: color("#714e75"),
            tertiary_fixed_dim: color("#b96a76"),
            on_tertiary_fixed: color("#0f0e0e"),
            on_tertiary_fixed_variant: color("#f9f2f3"),
            scrim: color("#0f0e0e"),
            shadow: color("#0f0e0e"),
            source_color: color("#e15774"),
            surface_bright: color("#2c2424"),
            surface_dim: color("#0f0e0e"),
        },
    }
}

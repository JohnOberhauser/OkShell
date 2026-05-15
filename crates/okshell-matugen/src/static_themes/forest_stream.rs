use crate::json_struct::{Base16, Colors, MatugenTheme, OkShell, Palettes, color};

pub fn forest_stream(okshell: OkShell) -> MatugenTheme {
    MatugenTheme {
        okshell,
        image: String::new(),
        is_dark_mode: true,
        mode: "dark".to_string(),
        base16: Base16 {
            base00: color("#0b0c0b"), // bg
            base01: color("#101c14"),
            base02: color("#0d1f0f"), // cloud0
            base03: color("#3c7153"), // cloud8 - comments
            base04: color("#609f83"), // cloud14
            base05: color("#e3f5e7"), // fg
            base06: color("#89d9d0"), // cloud6
            base07: color("#e8f4e7"), // cloud7
            base08: color("#c7566f"), // cloud9 - red
            base09: color("#a5c1cd"), // cloud11 - orange slot (pale slate)
            base0a: color("#caade1"), // cloud1 - yellow slot (lavender)
            base0b: color("#41b193"), // cloud10 - green
            base0c: color("#89d9d0"), // cloud6 - cyan
            base0d: color("#3788a2"), // cloud12 - blue
            base0e: color("#7a77cd"), // cloud2 - purple
            base0f: color("#243528"), // cloud13 - brown (deep forest)
        },
        palettes: Palettes::default(),
        colors: Colors {
            surface: color("#0b0c0b"),
            on_surface: color("#e3f5e7"),
            surface_variant: color("#101c14"),
            on_surface_variant: color("#609f83"),
            surface_container_highest: color("#1d3025"),
            surface_container_high: color("#16261c"),
            surface_container: color("#101c14"),
            surface_container_low: color("#0d1610"),
            surface_container_lowest: color("#0b0c0b"),
            inverse_surface: color("#e3f5e7"),
            inverse_on_surface: color("#0b0c0b"),
            surface_tint: color("#41b193"),
            primary: color("#41b193"),
            on_primary: color("#0b0c0b"),
            primary_container: color("#101c14"),
            on_primary_container: color("#41b193"),
            secondary: color("#3788a2"),
            on_secondary: color("#0b0c0b"),
            secondary_container: color("#101c14"),
            on_secondary_container: color("#3788a2"),
            tertiary: color("#7a77cd"),
            on_tertiary: color("#0b0c0b"),
            tertiary_container: color("#101c14"),
            on_tertiary_container: color("#7a77cd"),
            error: color("#c7566f"),
            on_error: color("#0b0c0b"),
            error_container: color("#101c14"),
            on_error_container: color("#c7566f"),
            outline: color("#3c7153"),
            outline_variant: color("#0d1f0f"),
            background: color("#0b0c0b"),
            on_background: color("#e3f5e7"),
            inverse_primary: color("#41b193"),
            primary_fixed: color("#41b193"),
            primary_fixed_dim: color("#89d9d0"),
            on_primary_fixed: color("#0b0c0b"),
            on_primary_fixed_variant: color("#e3f5e7"),
            secondary_fixed: color("#3788a2"),
            secondary_fixed_dim: color("#a5c1cd"),
            on_secondary_fixed: color("#0b0c0b"),
            on_secondary_fixed_variant: color("#e3f5e7"),
            tertiary_fixed: color("#7a77cd"),
            tertiary_fixed_dim: color("#caade1"),
            on_tertiary_fixed: color("#0b0c0b"),
            on_tertiary_fixed_variant: color("#e3f5e7"),
            scrim: color("#0b0c0b"),
            shadow: color("#0b0c0b"),
            source_color: color("#41b193"),
            surface_bright: color("#1d3025"),
            surface_dim: color("#0b0c0b"),
        },
    }
}

use crate::json_struct::MatugenTheme;

pub fn to_css(theme: &MatugenTheme) -> String {
    let c = &theme.colors;
    let mode = &theme.mode;

    macro_rules! col {
        ($field:expr) => {
            match mode.as_str() {
                "dark" => &$field.dark.color,
                "light" => &$field.light.color,
                _ => &$field.default.color,
            }
        };
    }

    format!(
        r#":root {{
    --surface: {surface};
    --on-surface: {on_surface};
    --surface-variant: {surface_variant};
    --on-surface-variant: {on_surface_variant};
    --surface-container-highest: {surface_container_highest};
    --surface-container-high: {surface_container_high};
    --surface-container: {surface_container};
    --surface-container-low: {surface_container_low};
    --surface-container-lowest: {surface_container_lowest};
    --inverse-surface: {inverse_surface};
    --inverse-on-surface: {inverse_on_surface};
    --surface-tint: {surface_tint};
    --surface-tint-color: {surface_tint};
    --primary: {primary};
    --on-primary: {on_primary};
    --primary-container: {primary_container};
    --on-primary-container: {on_primary_container};
    --secondary: {secondary};
    --on-secondary: {on_secondary};
    --secondary-container: {secondary_container};
    --on-secondary-container: {on_secondary_container};
    --tertiary: {tertiary};
    --on-tertiary: {on_tertiary};
    --tertiary-container: {tertiary_container};
    --on-tertiary-container: {on_tertiary_container};
    --error: {error};
    --on-error: {on_error};
    --error-container: {error_container};
    --on-error-container: {on_error_container};
    --outline: {outline};
    --outline-variant: {outline_variant};
}}"#,
        surface = col!(c.surface),
        on_surface = col!(c.on_surface),
        surface_variant = col!(c.surface_variant),
        on_surface_variant = col!(c.on_surface_variant),
        surface_container_highest = col!(c.surface_container_highest),
        surface_container_high = col!(c.surface_container_high),
        surface_container = col!(c.surface_container),
        surface_container_low = col!(c.surface_container_low),
        surface_container_lowest = col!(c.surface_container_lowest),
        inverse_surface = col!(c.inverse_surface),
        inverse_on_surface = col!(c.inverse_on_surface),
        surface_tint = col!(c.surface_tint),
        primary = col!(c.primary),
        on_primary = col!(c.on_primary),
        primary_container = col!(c.primary_container),
        on_primary_container = col!(c.on_primary_container),
        secondary = col!(c.secondary),
        on_secondary = col!(c.on_secondary),
        secondary_container = col!(c.secondary_container),
        on_secondary_container = col!(c.on_secondary_container),
        tertiary = col!(c.tertiary),
        on_tertiary = col!(c.on_tertiary),
        tertiary_container = col!(c.tertiary_container),
        on_tertiary_container = col!(c.on_tertiary_container),
        error = col!(c.error),
        on_error = col!(c.on_error),
        error_container = col!(c.error_container),
        on_error_container = col!(c.on_error_container),
        outline = col!(c.outline),
        outline_variant = col!(c.outline_variant),
    )
}

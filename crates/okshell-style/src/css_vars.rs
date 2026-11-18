//! Shared CSS custom-property helpers.
//!
//! GTK has no public API for reading custom properties (`--foo`) off a widget,
//! so we print the widget's computed style and parse the `--name: value` lines.
//! Used by FrameDrawWidget and the weather graph.

use relm4::gtk::{self, prelude::*};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub type Rgba = (f64, f64, f64, f64);

pub fn collect_css_vars(css: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for line in css.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("--")
            && let Some((name, value)) = rest.split_once(':')
        {
            let name = format!("--{}", name.trim());
            let value = value.trim().trim_end_matches(';').trim().to_string();
            vars.insert(name, value);
        }
    }
    vars
}

pub fn resolve_var<'a>(value: &'a str, vars: &'a HashMap<String, String>) -> &'a str {
    let trimmed = value.trim();
    if let Some(inner) = trimmed
        .strip_prefix("var(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let var_name = inner.trim();
        if let Some(resolved) = vars.get(var_name) {
            return resolve_var(resolved, vars);
        }
    }
    trimmed
}

pub fn parse_color(s: &str) -> Option<Rgba> {
    let s = s.trim();

    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex_color(hex);
    }

    if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let r = parse_channel(parts[0])?;
            let g = parse_channel(parts[1])?;
            let b = parse_channel(parts[2])?;
            let a = parse_channel(parts[3])?;
            return Some((r, g, b, a));
        }
    }

    if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parse_channel(parts[0])?;
            let g = parse_channel(parts[1])?;
            let b = parse_channel(parts[2])?;
            return Some((r, g, b, 1.0));
        }
    }

    None
}

fn parse_channel(s: &str) -> Option<f64> {
    let s = s.trim();
    let v: f64 = s.parse().ok()?;
    if v > 1.0 {
        Some((v / 255.0).clamp(0.0, 1.0))
    } else {
        Some(v.clamp(0.0, 1.0))
    }
}

fn parse_hex_color(hex: &str) -> Option<Rgba> {
    let hex = hex.trim();
    let byte = |s: &str| u8::from_str_radix(s, 16).ok().map(|v| v as f64 / 255.0);
    match hex.len() {
        3 => Some((
            byte(&hex[0..1].repeat(2))?,
            byte(&hex[1..2].repeat(2))?,
            byte(&hex[2..3].repeat(2))?,
            1.0,
        )),
        4 => Some((
            byte(&hex[0..1].repeat(2))?,
            byte(&hex[1..2].repeat(2))?,
            byte(&hex[2..3].repeat(2))?,
            byte(&hex[3..4].repeat(2))?,
        )),
        6 => Some((byte(&hex[0..2])?, byte(&hex[2..4])?, byte(&hex[4..6])?, 1.0)),
        8 => Some((
            byte(&hex[0..2])?,
            byte(&hex[2..4])?,
            byte(&hex[4..6])?,
            byte(&hex[6..8])?,
        )),
        _ => None,
    }
}

pub fn parse_number(s: &str) -> Option<f64> {
    s.trim().trim_end_matches("px").parse().ok()
}

pub fn hash_str(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Caches a widget's parsed custom properties; only re-parses when the
/// computed style text actually changes (e.g. on theme switch).
#[derive(Debug, Default)]
pub struct CssVarCache {
    hash: Cell<u64>,
    vars: RefCell<HashMap<String, String>>,
}

impl CssVarCache {
    /// Re-read the widget's computed style. Call once per draw before lookups.
    pub fn refresh(&self, widget: &impl IsA<gtk::Widget>) {
        #[allow(deprecated)]
        let css = widget
            .as_ref()
            .style_context()
            .to_string(gtk::StyleContextPrintFlags::SHOW_STYLE);
        let new_hash = hash_str(css.as_str());
        if new_hash != self.hash.get() {
            *self.vars.borrow_mut() = collect_css_vars(css.as_str());
            self.hash.set(new_hash);
        }
    }

    /// Look up a color variable, following `var(...)` chains.
    pub fn color(&self, name: &str) -> Option<Rgba> {
        let vars = self.vars.borrow();
        vars.get(name)
            .and_then(|raw| parse_color(resolve_var(raw, &vars)))
    }

    /// Look up a numeric variable (`px` suffix allowed).
    pub fn number(&self, name: &str) -> Option<f64> {
        let vars = self.vars.borrow();
        vars.get(name)
            .and_then(|raw| parse_number(resolve_var(raw, &vars)))
    }

    pub fn invalidate(&self) {
        self.hash.set(0);
    }
}

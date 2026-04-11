use relm4::gtk::{
    self,
    glib::{
        self,
        object_subclass,
    },
    prelude::*,
    subclass::prelude::*,
    cairo::{
        Context, Operator, LineJoin, LineCap, Region, RectangleInt
    }
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct FrameStyle {
    pub draw_frame: bool,
    pub border_radius: f64,
    pub background_rgba: (f64, f64, f64, f64),
    pub border_rgba: (f64, f64, f64, f64),
    pub border_width: f64,
    pub left_thickness: f64,
    pub right_thickness: f64,
    pub top_thickness: f64,
    pub bottom_thickness: f64,
    pub left_top_expander_height: f64,
    pub right_top_expander_height: f64,
    pub right_bottom_expander_height: f64,
    pub left_bottom_expander_height: f64,
    pub left_expander_width: f64,
    pub right_expander_width: f64,
    pub top_revealer_size: (f64, f64),
    pub bottom_revealer_size: (f64, f64),
    pub top_left_revealer_size: (f64, f64),
    pub top_right_revealer_size: (f64, f64),
    pub bottom_left_revealer_size: (f64, f64),
    pub bottom_right_revealer_size: (f64, f64),
}

impl Default for FrameStyle {
    fn default() -> Self {
        Self {
            draw_frame: true,
            border_radius: 24.0,
            background_rgba: (0.0, 0.0, 0.0, 1.0),
            border_rgba: (1.0, 1.0, 1.0, 1.0),
            border_width: 2.0,
            left_thickness: 50.0,
            right_thickness: 50.0,
            top_thickness: 50.0,
            bottom_thickness: 50.0,
            left_top_expander_height: 0.0,
            right_top_expander_height: 0.0,
            left_bottom_expander_height: 0.0,
            right_bottom_expander_height: 0.0,
            left_expander_width: 0.0,
            right_expander_width: 0.0,
            top_revealer_size: (0.0, 0.0),
            bottom_revealer_size: (0.0, 0.0),
            top_left_revealer_size: (0.0, 0.0),
            top_right_revealer_size: (0.0, 0.0),
            bottom_left_revealer_size: (0.0, 0.0),
            bottom_right_revealer_size: (0.0, 0.0),
        }
    }
}

// ---------------------------------------------------------------------------
// CSS custom-property helpers
// ---------------------------------------------------------------------------

fn collect_css_vars(css: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for line in css.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("--") {
            if let Some((name, value)) = rest.split_once(':') {
                let name = format!("--{}", name.trim());
                let value = value.trim().trim_end_matches(';').trim().to_string();
                vars.insert(name, value);
            }
        }
    }
    vars
}

fn resolve_var<'a>(value: &'a str, vars: &'a HashMap<String, String>) -> &'a str {
    let trimmed = value.trim();
    if let Some(inner) = trimmed.strip_prefix("var(").and_then(|s| s.strip_suffix(')')) {
        let var_name = inner.trim();
        if let Some(resolved) = vars.get(var_name) {
            return resolve_var(resolved, vars);
        }
    }
    trimmed
}

fn parse_color(s: &str) -> Option<(f64, f64, f64, f64)> {
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

fn parse_hex_color(hex: &str) -> Option<(f64, f64, f64, f64)> {
    let hex = hex.trim();
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()? as f64 / 255.0;
            Some((r, g, b, 1.0))
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()? as f64 / 255.0;
            let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()? as f64 / 255.0;
            Some((r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
            Some((r, g, b, 1.0))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f64 / 255.0;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

fn parse_number(s: &str) -> Option<f64> {
    s.trim().trim_end_matches("px").parse().ok()
}

fn apply_css_vars(style: &mut FrameStyle, vars: &HashMap<String, String>) {
    if let Some(raw) = vars.get("--frame-bg") {
        let resolved = resolve_var(raw, vars);
        if let Some(c) = parse_color(resolved) {
            style.background_rgba = c;
        }
    }

    if let Some(raw) = vars.get("--frame-border") {
        let resolved = resolve_var(raw, vars);
        if let Some(c) = parse_color(resolved) {
            style.border_rgba = c;
        }
    }

    if let Some(raw) = vars.get("--frame-border-width") {
        let resolved = resolve_var(raw, vars);
        if let Some(n) = parse_number(resolved) {
            style.border_width = n;
        }
    }

    if let Some(raw) = vars.get("--frame-border-radius") {
        let resolved = resolve_var(raw, vars);
        if let Some(n) = parse_number(resolved) {
            style.border_radius = n;
        }
    }
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------
// Polygon-based hole path algorithm
// ---------------------------------------------------------------------------
//
// The hole is described as a polygon of (x, y) vertices in clockwise order.
// `build_hole_vertices` collects all the vertices based on which features
// are active.  `trace_rounded_polygon` then traces the path with rounded
// corners, automatically determining convex vs concave at each vertex.
//
// Understanding the layout:
//
//   left_thickness includes the left bar + left revealer (menu) width.
//   right_thickness includes the right bar + right revealer width.
//   top_thickness / bottom_thickness include the top/bottom bar heights.
//
//   The inner hole rectangle is:
//     hole_x = left_thickness,  hole_y = top_thickness
//     hole_width  = total_w - left_thickness - right_thickness
//     hole_height = total_h - top_thickness - bottom_thickness
//
//   left_expander_width / right_expander_width: the width of the side
//   menus (revealers).  These are already accounted for in left/right
//   thickness.  The expander HEIGHTS represent empty space (no menu
//   content) above/below the menu within that revealer area.  These
//   empty regions are "negative space" — the hole extends leftward
//   into them.
//
//   So the left side has two notches extending LEFT from the hole edge:
//     Top-left notch:  from hole_y to hole_y + left_top_expander_height,
//                      width = left_expander_width, going left from hole_x
//     Bottom-left notch: from hole_y + hole_height - left_bottom_expander_height
//                         to hole_y + hole_height,
//                         width = left_expander_width, going left from hole_x
//
//   Same pattern mirrored for the right side (extending right from hole_x + hole_width).
//
//   Revealers (top, bottom, top_left, top_right, bottom_left, bottom_right)
//   create inward notches that poke INTO the hole from the top/bottom edges.
//   The frame is painted over these areas.  Corner revealers are anchored
//   flush with the hole's left/right edge.

/// Build the hole polygon vertices in clockwise order.
#[allow(clippy::too_many_arguments)]
fn build_hole_vertices(style: &FrameStyle, hole_x: f64, hole_y: f64, hole_width: f64, hole_height: f64) -> Vec<(f64, f64)> {
    let mut vertices: Vec<(f64, f64)> = Vec::with_capacity(64);

    let left_expander_width = style.left_expander_width;
    let right_expander_width = style.right_expander_width;
    let left_top_expander_height = style.left_top_expander_height;
    let left_bottom_expander_height = style.left_bottom_expander_height;
    let right_top_expander_height = style.right_top_expander_height;
    let right_bottom_expander_height = style.right_bottom_expander_height;

    let (top_revealer_width, top_revealer_height) = style.top_revealer_size;
    let (bottom_revealer_width, bottom_revealer_height) = style.bottom_revealer_size;
    let (top_left_revealer_width, top_left_revealer_height) = style.top_left_revealer_size;
    let (top_right_revealer_width, top_right_revealer_height) = style.top_right_revealer_size;
    let (bottom_left_revealer_width, bottom_left_revealer_height) = style.bottom_left_revealer_size;
    let (bottom_right_revealer_width, bottom_right_revealer_height) = style.bottom_right_revealer_size;

    let has_left_top_expander = left_top_expander_height > 0.0 && left_expander_width > 0.0;
    let has_right_top_expander = right_top_expander_height > 0.0 && right_expander_width > 0.0;
    let has_left_bottom_expander = left_bottom_expander_height > 0.0 && left_expander_width > 0.0;
    let has_right_bottom_expander = right_bottom_expander_height > 0.0 && right_expander_width > 0.0;

    let has_top_left_revealer = top_left_revealer_width > 0.0 && top_left_revealer_height > 0.0;
    let has_top_right_revealer = top_right_revealer_width > 0.0 && top_right_revealer_height > 0.0;
    let has_bottom_left_revealer = bottom_left_revealer_width > 0.0 && bottom_left_revealer_height > 0.0;
    let has_bottom_right_revealer = bottom_right_revealer_width > 0.0 && bottom_right_revealer_height > 0.0;
    let has_top_revealer = top_revealer_width > 0.0 && top_revealer_height > 0.0;
    let has_bottom_revealer = bottom_revealer_width > 0.0 && bottom_revealer_height > 0.0;

    let hole_right = hole_x + hole_width;
    let hole_bottom = hole_y + hole_height;

    // =====================================================================
    // Walk clockwise starting from the top-left corner (hole_x, hole_y).
    //
    // The path traces the outline of everything that should be "hole"
    // (transparent / cleared).  Expander notches are detours that
    // extend LEFT/RIGHT from the main hole edge and return.  Revealer
    // notches are detours that extend DOWN/UP into the hole from the
    // top/bottom edges.
    //
    // The walk always starts and ends on the main hole edge so that
    // close_path connects cleanly.
    // =====================================================================

    // Track whether corner revealers have already consumed the expander
    // space, so the expander detours on the left/right edges can be skipped.
    let mut left_top_expander_consumed = false;
    let mut left_bottom_expander_consumed = false;
    let mut right_top_expander_consumed = false;
    let mut right_bottom_expander_consumed = false;

    // === TOP-LEFT CORNER (hole_x, hole_y) ===

    // Top-left revealer: inward notch going down from the top edge.
    //
    // The revealer notch always extends downward from hole_y by
    // top_left_revealer_height.  The question is what happens on its
    // left side, which depends on whether the left menu is revealed
    // (left_expander_width > 0) and whether there is empty space above
    // the left menu content (left_top_expander_height > 0).
    //
    // Cases:
    //   A. left_expander_width == 0: No left menu panel.  Revealer is
    //      flush against the left frame band.  Merges with corner.
    //
    //   B. left_expander_width > 0 and left_top_expander_height == 0:
    //      Left menu is fully expanded (no empty space above it).
    //      There is no gap to the left of the revealer — the menu
    //      content fills that area.  Revealer is a standalone notch
    //      that does NOT extend leftward.
    //
    //   C. left_expander_width > 0 and left_top_expander_height > 0
    //      and revealer height >= expander height:
    //      Joined L-shape.  The hole extends left for the expander's
    //      height, then steps right to hole_x, then continues down
    //      to the revealer's height.  The expander detour is consumed.
    //
    //   D. left_expander_width > 0 and left_top_expander_height > 0
    //      and revealer height < expander height:
    //      The hole extends left for the revealer's height.  The
    //      expander detour on the left edge handles the remaining
    //      gap below.
    if has_top_left_revealer {
        if left_expander_width > 0.0 && left_top_expander_height > 0.0 {
            if top_left_revealer_height >= left_top_expander_height {
                // Case C: joined L-shape — expander consumed
                left_top_expander_consumed = true;
                vertices.push((hole_x - left_expander_width, hole_y));
                vertices.push((hole_x - left_expander_width, hole_y + left_top_expander_height));
                vertices.push((hole_x, hole_y + left_top_expander_height));
                vertices.push((hole_x, hole_y + top_left_revealer_height));
                vertices.push((hole_x + top_left_revealer_width, hole_y + top_left_revealer_height));
                vertices.push((hole_x + top_left_revealer_width, hole_y));
            } else {
                // Case D: revealer shorter than expander
                vertices.push((hole_x - left_expander_width, hole_y));
                vertices.push((hole_x, hole_y));
                vertices.push((hole_x, hole_y + top_left_revealer_height));
                vertices.push((hole_x + top_left_revealer_width, hole_y + top_left_revealer_height));
                vertices.push((hole_x + top_left_revealer_width, hole_y));
            }
        } else if left_expander_width > 0.0 {
            // Case B: left menu visible but fully expanded, no gap
            vertices.push((hole_x, hole_y));
            vertices.push((hole_x, hole_y + top_left_revealer_height));
            vertices.push((hole_x + top_left_revealer_width, hole_y + top_left_revealer_height));
            vertices.push((hole_x + top_left_revealer_width, hole_y));
        } else {
            // Case A: no left menu, merged with left edge
            vertices.push((hole_x, hole_y));
            vertices.push((hole_x, hole_y + top_left_revealer_height));
            vertices.push((hole_x + top_left_revealer_width, hole_y + top_left_revealer_height));
            vertices.push((hole_x + top_left_revealer_width, hole_y));
        }
    } else {
        vertices.push((hole_x, hole_y));
    }

    // === TOP EDGE (left → right) ===

    // Center top revealer: inward notch centered on the top edge.
    // Its position must account for the corner revealer widths — it is
    // centered in the remaining span between them.
    if has_top_revealer {
        let edge_start = hole_x + top_left_revealer_width;
        let edge_end = hole_right - top_right_revealer_width;
        let edge_center = (edge_start + edge_end) / 2.0;
        let notch_left = edge_center - top_revealer_width / 2.0;
        let notch_right = edge_center + top_revealer_width / 2.0;
        vertices.push((notch_left, hole_y));
        vertices.push((notch_left, hole_y + top_revealer_height));
        vertices.push((notch_right, hole_y + top_revealer_height));
        vertices.push((notch_right, hole_y));
    }

    // Top-right revealer: same four cases, mirrored for the right side.
    if has_top_right_revealer {
        if right_expander_width > 0.0 && right_top_expander_height > 0.0 {
            if top_right_revealer_height >= right_top_expander_height {
                // Case C: joined L-shape — expander consumed
                right_top_expander_consumed = true;
                vertices.push((hole_right - top_right_revealer_width, hole_y));
                vertices.push((hole_right - top_right_revealer_width, hole_y + top_right_revealer_height));
                vertices.push((hole_right, hole_y + top_right_revealer_height));
                vertices.push((hole_right, hole_y + right_top_expander_height));
                vertices.push((hole_right + right_expander_width, hole_y + right_top_expander_height));
                vertices.push((hole_right + right_expander_width, hole_y));
            } else {
                // Case D: revealer shorter than expander
                vertices.push((hole_right - top_right_revealer_width, hole_y));
                vertices.push((hole_right - top_right_revealer_width, hole_y + top_right_revealer_height));
                vertices.push((hole_right, hole_y + top_right_revealer_height));
                vertices.push((hole_right, hole_y));
                vertices.push((hole_right + right_expander_width, hole_y));
            }
        } else if right_expander_width > 0.0 {
            // Case B: right menu visible but fully expanded, no gap
            vertices.push((hole_right - top_right_revealer_width, hole_y));
            vertices.push((hole_right - top_right_revealer_width, hole_y + top_right_revealer_height));
            vertices.push((hole_right, hole_y + top_right_revealer_height));
            vertices.push((hole_right, hole_y));
        } else {
            // Case A: no right menu, merged with right edge
            vertices.push((hole_right - top_right_revealer_width, hole_y));
            vertices.push((hole_right - top_right_revealer_width, hole_y + top_right_revealer_height));
            vertices.push((hole_right, hole_y + top_right_revealer_height));
            vertices.push((hole_right, hole_y));
        }
    }

    // === TOP-RIGHT CORNER (hole_right, hole_y) ===

    // Right top expander: detour rightward into the frame band, then return.
    // Skipped if the top-right revealer already consumed this space.
    if has_right_top_expander && !right_top_expander_consumed {
        if !has_top_right_revealer {
            vertices.push((hole_right, hole_y));
        }
        vertices.push((hole_right + right_expander_width, hole_y));
        vertices.push((hole_right + right_expander_width, hole_y + right_top_expander_height));
        vertices.push((hole_right, hole_y + right_top_expander_height));
    } else if !has_top_right_revealer && !right_top_expander_consumed {
        vertices.push((hole_right, hole_y));
    }

    // === RIGHT EDGE (top → bottom) ===

    // Right bottom expander: detour rightward, then return.
    // Skipped if the bottom-right revealer already consumed this space.
    if has_right_bottom_expander && !right_bottom_expander_consumed {
        vertices.push((hole_right, hole_bottom - right_bottom_expander_height));
        vertices.push((hole_right + right_expander_width, hole_bottom - right_bottom_expander_height));
        vertices.push((hole_right + right_expander_width, hole_bottom));
        vertices.push((hole_right, hole_bottom));
    } else {
        vertices.push((hole_right, hole_bottom));
    }

    // === BOTTOM-RIGHT CORNER (hole_right, hole_bottom) ===

    // Bottom-right revealer: same four cases, mirrored for bottom-right.
    if has_bottom_right_revealer {
        if right_expander_width > 0.0 && right_bottom_expander_height > 0.0 {
            if bottom_right_revealer_height >= right_bottom_expander_height {
                // Case C: joined L-shape — expander consumed
                right_bottom_expander_consumed = true;
                vertices.push((hole_right + right_expander_width, hole_bottom));
                vertices.push((hole_right + right_expander_width, hole_bottom - right_bottom_expander_height));
                vertices.push((hole_right, hole_bottom - right_bottom_expander_height));
                vertices.push((hole_right, hole_bottom - bottom_right_revealer_height));
                vertices.push((hole_right - bottom_right_revealer_width, hole_bottom - bottom_right_revealer_height));
                vertices.push((hole_right - bottom_right_revealer_width, hole_bottom));
            } else {
                // Case D: revealer shorter than expander
                vertices.push((hole_right + right_expander_width, hole_bottom));
                vertices.push((hole_right, hole_bottom));
                vertices.push((hole_right, hole_bottom - bottom_right_revealer_height));
                vertices.push((hole_right - bottom_right_revealer_width, hole_bottom - bottom_right_revealer_height));
                vertices.push((hole_right - bottom_right_revealer_width, hole_bottom));
            }
        } else if right_expander_width > 0.0 {
            // Case B: right menu visible but fully expanded, no gap
            vertices.push((hole_right, hole_bottom - bottom_right_revealer_height));
            vertices.push((hole_right - bottom_right_revealer_width, hole_bottom - bottom_right_revealer_height));
            vertices.push((hole_right - bottom_right_revealer_width, hole_bottom));
        } else {
            // Case A: no right menu, merged with right edge
            vertices.push((hole_right, hole_bottom - bottom_right_revealer_height));
            vertices.push((hole_right - bottom_right_revealer_width, hole_bottom - bottom_right_revealer_height));
            vertices.push((hole_right - bottom_right_revealer_width, hole_bottom));
        }
    }

    // === BOTTOM EDGE (right → left) ===

    // Center bottom revealer: same offset logic as center top.
    if has_bottom_revealer {
        let edge_start = hole_x + bottom_left_revealer_width;
        let edge_end = hole_right - bottom_right_revealer_width;
        let edge_center = (edge_start + edge_end) / 2.0;
        let notch_left = edge_center - bottom_revealer_width / 2.0;
        let notch_right = edge_center + bottom_revealer_width / 2.0;
        vertices.push((notch_right, hole_bottom));
        vertices.push((notch_right, hole_bottom - bottom_revealer_height));
        vertices.push((notch_left, hole_bottom - bottom_revealer_height));
        vertices.push((notch_left, hole_bottom));
    }

    // Bottom-left revealer: same four cases, mirrored for bottom-left.
    if has_bottom_left_revealer {
        if left_expander_width > 0.0 && left_bottom_expander_height > 0.0 {
            if bottom_left_revealer_height >= left_bottom_expander_height {
                // Case C: joined L-shape — expander consumed
                left_bottom_expander_consumed = true;
                vertices.push((hole_x + bottom_left_revealer_width, hole_bottom));
                vertices.push((hole_x + bottom_left_revealer_width, hole_bottom - bottom_left_revealer_height));
                vertices.push((hole_x, hole_bottom - bottom_left_revealer_height));
                vertices.push((hole_x, hole_bottom - left_bottom_expander_height));
                vertices.push((hole_x - left_expander_width, hole_bottom - left_bottom_expander_height));
                vertices.push((hole_x - left_expander_width, hole_bottom));
            } else {
                // Case D: revealer shorter than expander
                vertices.push((hole_x + bottom_left_revealer_width, hole_bottom));
                vertices.push((hole_x + bottom_left_revealer_width, hole_bottom - bottom_left_revealer_height));
                vertices.push((hole_x, hole_bottom - bottom_left_revealer_height));
                vertices.push((hole_x, hole_bottom));
                vertices.push((hole_x - left_expander_width, hole_bottom));
            }
        } else if left_expander_width > 0.0 {
            // Case B: left menu visible but fully expanded, no gap
            vertices.push((hole_x + bottom_left_revealer_width, hole_bottom));
            vertices.push((hole_x + bottom_left_revealer_width, hole_bottom - bottom_left_revealer_height));
            vertices.push((hole_x, hole_bottom - bottom_left_revealer_height));
            vertices.push((hole_x, hole_bottom));
        } else {
            // Case A: no left menu, merged with left edge
            vertices.push((hole_x + bottom_left_revealer_width, hole_bottom));
            vertices.push((hole_x + bottom_left_revealer_width, hole_bottom - bottom_left_revealer_height));
            vertices.push((hole_x, hole_bottom - bottom_left_revealer_height));
            vertices.push((hole_x, hole_bottom));
        }
    }

    // === BOTTOM-LEFT CORNER (hole_x, hole_bottom) ===

    // Left bottom expander: detour leftward into the frame band, then return.
    // Skipped if the bottom-left revealer already consumed this space.
    if has_left_bottom_expander && !left_bottom_expander_consumed {
        if !has_bottom_left_revealer {
            vertices.push((hole_x, hole_bottom));
        }
        vertices.push((hole_x - left_expander_width, hole_bottom));
        vertices.push((hole_x - left_expander_width, hole_bottom - left_bottom_expander_height));
        vertices.push((hole_x, hole_bottom - left_bottom_expander_height));
    } else if !has_bottom_left_revealer && !left_bottom_expander_consumed {
        vertices.push((hole_x, hole_bottom));
    }

    // === LEFT EDGE (bottom → top) ===

    // Left top expander: detour leftward into the frame band, then return.
    // Skipped if the top-left revealer already consumed this space.
    if has_left_top_expander && !left_top_expander_consumed {
        vertices.push((hole_x, hole_y + left_top_expander_height));
        vertices.push((hole_x - left_expander_width, hole_y + left_top_expander_height));
        vertices.push((hole_x - left_expander_width, hole_y));
        vertices.push((hole_x, hole_y));
    }

    // Path closes back to the first vertex at (hole_x, hole_y).

    vertices
}

/// Determine the cross product z-component at vertex `current`.
/// Positive = left turn (convex in CW winding).
/// Negative = right turn (concave in CW winding).
fn cross_z(previous: (f64, f64), current: (f64, f64), next: (f64, f64)) -> f64 {
    let incoming = (current.0 - previous.0, current.1 - previous.1);
    let outgoing = (next.0 - current.0, next.1 - current.1);
    incoming.0 * outgoing.1 - incoming.1 * outgoing.0
}

/// Trace the polygon with rounded corners.
///
/// At each vertex the turn direction is computed via cross product:
///   - CW convex (cross > 0): standard `arc`
///   - CW concave (cross < 0): `arc_negative`
///   - Collinear: skip
///
/// Radii are clamped so adjacent corners don't overlap on shared edges.
fn trace_rounded_polygon(cr: &Context, vertices: &[(f64, f64)], max_radius: f64) {
    let vertex_count = vertices.len();
    if vertex_count < 3 {
        return;
    }

    // Remove collinear points (where the path continues in the same direction)
    let mut cleaned: Vec<(f64, f64)> = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let previous = vertices[(i + vertex_count - 1) % vertex_count];
        let current = vertices[i];
        let next = vertices[(i + 1) % vertex_count];
        if cross_z(previous, current, next).abs() > 1e-6 {
            cleaned.push(current);
        }
    }

    let vertex_count = cleaned.len();
    if vertex_count < 3 {
        return;
    }

    // Compute radius for each corner, clamped by adjacent edge lengths
    let mut radii: Vec<f64> = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let previous = cleaned[(i + vertex_count - 1) % vertex_count];
        let current = cleaned[i];
        let next = cleaned[(i + 1) % vertex_count];

        let incoming_length = ((current.0 - previous.0).powi(2) + (current.1 - previous.1).powi(2)).sqrt();
        let outgoing_length = ((next.0 - current.0).powi(2) + (next.1 - current.1).powi(2)).sqrt();

        radii.push(max_radius.min(incoming_length / 2.0).min(outgoing_length / 2.0));
    }

    // Ensure adjacent corners don't overlap on shared edges
    for i in 0..vertex_count {
        let next_index = (i + 1) % vertex_count;
        let edge_length = {
            let (x0, y0) = cleaned[i];
            let (x1, y1) = cleaned[next_index];
            ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
        };
        let combined_radii = radii[i] + radii[next_index];
        if combined_radii > edge_length && combined_radii > 1e-6 {
            let scale = edge_length / combined_radii;
            radii[i] *= scale;
            radii[next_index] *= scale;
        }
    }

    cr.new_path();

    for i in 0..vertex_count {
        let previous = cleaned[(i + vertex_count - 1) % vertex_count];
        let current = cleaned[i];
        let next = cleaned[(i + 1) % vertex_count];
        let radius = radii[i];

        if radius < 1e-6 {
            if i == 0 {
                cr.move_to(current.0, current.1);
            } else {
                cr.line_to(current.0, current.1);
            }
            continue;
        }

        let incoming_length = ((current.0 - previous.0).powi(2) + (current.1 - previous.1).powi(2)).sqrt();
        let outgoing_length = ((next.0 - current.0).powi(2) + (next.1 - current.1).powi(2)).sqrt();

        if incoming_length < 1e-9 || outgoing_length < 1e-9 {
            if i == 0 {
                cr.move_to(current.0, current.1);
            } else {
                cr.line_to(current.0, current.1);
            }
            continue;
        }

        let incoming_dir_x = (current.0 - previous.0) / incoming_length;
        let incoming_dir_y = (current.1 - previous.1) / incoming_length;
        let outgoing_dir_x = (next.0 - current.0) / outgoing_length;
        let outgoing_dir_y = (next.1 - current.1) / outgoing_length;

        // Tangent points where arc meets edges
        let tangent_incoming = (current.0 - incoming_dir_x * radius, current.1 - incoming_dir_y * radius);
        let tangent_outgoing = (current.0 + outgoing_dir_x * radius, current.1 + outgoing_dir_y * radius);

        let cross = incoming_dir_x * outgoing_dir_y - incoming_dir_y * outgoing_dir_x;

        // Normal pointing toward arc center
        let (normal_x, normal_y) = if cross > 0.0 {
            (-incoming_dir_y, incoming_dir_x)
        } else {
            (incoming_dir_y, -incoming_dir_x)
        };

        let arc_center = (tangent_incoming.0 + normal_x * radius, tangent_incoming.1 + normal_y * radius);

        let angle_start = (tangent_incoming.1 - arc_center.1).atan2(tangent_incoming.0 - arc_center.0);
        let angle_end = (tangent_outgoing.1 - arc_center.1).atan2(tangent_outgoing.0 - arc_center.0);

        if i == 0 {
            cr.move_to(tangent_incoming.0, tangent_incoming.1);
        } else {
            cr.line_to(tangent_incoming.0, tangent_incoming.1);
        }

        if cross > 0.0 {
            cr.arc(arc_center.0, arc_center.1, radius, angle_start, angle_end);
        } else {
            cr.arc_negative(arc_center.0, arc_center.1, radius, angle_start, angle_end);
        }
    }

    cr.close_path();
}

// --- Subclass internals ---

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FrameDrawWidget {
        pub(super) style: RefCell<FrameStyle>,
        pub(super) resolved_style: RefCell<Option<FrameStyle>>,
        pub(super) css_hash: Cell<u64>,
    }

    #[object_subclass]
    impl ObjectSubclass for FrameDrawWidget {
        const NAME: &'static str = "FrameDrawWidget";
        type Type = super::FrameDrawWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("frame-draw");
        }
    }

    impl ObjectImpl for FrameDrawWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let widget = self.obj();
            widget.set_hexpand(true);
            widget.set_vexpand(true);
            widget.set_can_focus(false);
            widget.set_can_target(false);
            widget.set_sensitive(false);

            widget.connect_notify(Some("css-classes"), |w, _| {
                w.imp().invalidate_css_cache();
                w.queue_draw();
            });

            if let Some(settings) = gtk::Settings::default() {
                let w = widget.downgrade();
                settings.connect_notify_local(Some("gtk-theme-name"), move |_, _| {
                    if let Some(w) = w.upgrade() {
                        w.imp().invalidate_css_cache();
                        w.queue_draw();
                    }
                });

                let w = widget.downgrade();
                settings.connect_notify_local(Some("gtk-application-prefer-dark-theme"), move |_, _| {
                    if let Some(w) = w.upgrade() {
                        w.imp().invalidate_css_cache();
                        w.queue_draw();
                    }
                });
            }
        }
    }

    impl WidgetImpl for FrameDrawWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let total_width = widget.width() as f64;
            let total_height = widget.height() as f64;

            if total_width <= 0.0 || total_height <= 0.0 {
                return;
            }

            let style = self.get_resolved_style();

            let total_top_thickness = style.top_thickness;
            let total_bottom_thickness = style.bottom_thickness;
            let total_left_thickness = style.left_thickness;
            let total_right_thickness = style.right_thickness;

            let hole_x = total_left_thickness;
            let hole_y = total_top_thickness;
            let hole_width = (total_width - total_left_thickness - total_right_thickness).max(0.0);
            let hole_height = (total_height - total_top_thickness - total_bottom_thickness).max(0.0);
            let border_radius = style.border_radius.clamp(0.0, hole_width.min(hole_height) / 2.0);

            let vertices = build_hole_vertices(&style, hole_x, hole_y, hole_width, hole_height);

            if style.draw_frame && !vertices.is_empty() {
                let bounds = gtk::graphene::Rect::new(0.0, 0.0, total_width as f32, total_height as f32);
                let cr = snapshot.append_cairo(&bounds);

                // 1) Paint full background
                cr.set_operator(Operator::Over);
                let (bg_r, bg_g, bg_b, bg_a) = style.background_rgba;
                cr.set_source_rgba(bg_r, bg_g, bg_b, bg_a);
                cr.rectangle(0.0, 0.0, total_width, total_height);
                let _ = cr.fill();

                // 2) Clear the hole
                cr.set_operator(Operator::Clear);
                trace_rounded_polygon(&cr, &vertices, border_radius);
                let _ = cr.fill();

                // 3) Border
                let (border_r, border_g, border_b, border_a) = style.border_rgba;
                if style.border_width > 0.0 && border_a > 0.0 {
                    cr.set_operator(Operator::Over);
                    trace_rounded_polygon(&cr, &vertices, border_radius);
                    cr.clip();
                    trace_rounded_polygon(&cr, &vertices, border_radius);
                    cr.set_source_rgba(border_r, border_g, border_b, border_a);
                    cr.set_line_width(style.border_width * 2.0);
                    cr.set_line_join(LineJoin::Round);
                    cr.set_line_cap(LineCap::Round);
                    let _ = cr.stroke();
                    cr.reset_clip();
                }
            }

            self.update_input_region(&style, total_width as i32, total_height as i32, hole_x, hole_y, hole_width, hole_height);
        }
    }

    impl FrameDrawWidget {
        fn get_resolved_style(&self) -> FrameStyle {
            let widget = self.obj();

            #[allow(deprecated)]
            let ctx = widget.style_context();
            #[allow(deprecated)]
            let css_str = ctx.to_string(gtk::StyleContextPrintFlags::SHOW_STYLE);
            let new_hash = hash_str(css_str.as_str());

            if new_hash != self.css_hash.get() || self.resolved_style.borrow().is_none() {
                let mut style = self.style.borrow().clone();
                let vars = collect_css_vars(css_str.as_str());
                apply_css_vars(&mut style, &vars);

                self.css_hash.set(new_hash);
                *self.resolved_style.borrow_mut() = Some(style);
            }

            self.resolved_style.borrow().clone().unwrap()
        }

        pub(super) fn invalidate_css_cache(&self) {
            self.css_hash.set(0);
            *self.resolved_style.borrow_mut() = None;
        }

        fn update_input_region(
            &self,
            style: &FrameStyle,
            window_width: i32, window_height: i32,
            hole_x: f64, hole_y: f64, hole_width: f64, hole_height: f64,
        ) {
            let widget = self.obj();

            let Some(native) = widget.native() else { return };
            let Some(surface) = native.surface() else { return };

            let hole_left = hole_x.floor().max(0.0) as i32;
            let hole_top = hole_y.floor().max(0.0) as i32;
            let hole_right = (hole_x + hole_width).ceil().min(window_width as f64) as i32;
            let hole_bottom = (hole_y + hole_height).ceil().min(window_height as f64) as i32;

            let region = Region::create();
            region
                .union_rectangle(&RectangleInt::new(0, 0, window_width, window_height))
                .expect("region union");

            let border_width = style.border_width.ceil() as i32;
            let border_width_f = style.border_width;

            // Subtract main hole (inset by border_width so the border stays in the input region)
            region
                .subtract_rectangle(&RectangleInt::new(
                    hole_left + border_width,
                    hole_top + border_width,
                    (hole_right - hole_left - border_width * 2).max(0),
                    (hole_bottom - hole_top - border_width * 2).max(0),
                ))
                .expect("region subtract hole");

            let left_expander_width = style.left_expander_width.ceil() as i32;
            let right_expander_width = style.right_expander_width.ceil() as i32;

            // Subtract side expander negative-space areas (inset by border_width)
            // Left top expander: shrink width from the right (hole edge) by border_width
            subtract_rect(&region,
                          hole_left - left_expander_width,
                          hole_top,
                          (left_expander_width - border_width).max(0),
                          style.left_top_expander_height.ceil() as i32);
            // Left bottom expander
            subtract_rect(&region,
                          hole_left - left_expander_width,
                          hole_bottom - style.left_bottom_expander_height.ceil() as i32,
                          (left_expander_width - border_width).max(0),
                          style.left_bottom_expander_height.ceil() as i32);
            // Right top expander: shift x rightward by border_width, shrink width
            subtract_rect(&region,
                          hole_right + border_width,
                          hole_top,
                          (right_expander_width - border_width).max(0),
                          style.right_top_expander_height.ceil() as i32);
            // Right bottom expander
            subtract_rect(&region,
                          hole_right + border_width,
                          hole_bottom - style.right_bottom_expander_height.ceil() as i32,
                          (right_expander_width - border_width).max(0),
                          style.right_bottom_expander_height.ceil() as i32);

            // Union back inward revealer notches so frame captures input there.
            // Expand each notch by border_width on sides that touch the hole edge
            // so the border strip is also covered.
            let (top_revealer_width, top_revealer_height) = style.top_revealer_size;
            let (bottom_revealer_width, bottom_revealer_height) = style.bottom_revealer_size;
            let (top_left_revealer_width, top_left_revealer_height) = style.top_left_revealer_size;
            let (top_right_revealer_width, top_right_revealer_height) = style.top_right_revealer_size;
            let (bottom_left_revealer_width, bottom_left_revealer_height) = style.bottom_left_revealer_size;
            let (bottom_right_revealer_width, bottom_right_revealer_height) = style.bottom_right_revealer_size;

            // Center top revealer: touches top hole edge, expand upward
            union_inward_notch(&region,
                               hole_x + hole_width / 2.0 - top_revealer_width / 2.0,
                               hole_y - border_width_f,
                               top_revealer_width,
                               top_revealer_height + border_width_f);

            // Center bottom revealer: touches bottom hole edge, expand downward
            union_inward_notch(&region,
                               hole_x + hole_width / 2.0 - bottom_revealer_width / 2.0,
                               hole_y + hole_height - bottom_revealer_height,
                               bottom_revealer_width,
                               bottom_revealer_height + border_width_f);

            // Top-left corner revealer: touches top and left hole edges
            union_inward_notch(&region,
                               hole_x - border_width_f,
                               hole_y - border_width_f,
                               top_left_revealer_width + border_width_f,
                               top_left_revealer_height + border_width_f);

            // Top-right corner revealer: touches top and right hole edges
            union_inward_notch(&region,
                               hole_x + hole_width - top_right_revealer_width,
                               hole_y - border_width_f,
                               top_right_revealer_width + border_width_f,
                               top_right_revealer_height + border_width_f);

            // Bottom-left corner revealer: touches bottom and left hole edges
            union_inward_notch(&region,
                               hole_x - border_width_f,
                               hole_y + hole_height - bottom_left_revealer_height,
                               bottom_left_revealer_width + border_width_f,
                               bottom_left_revealer_height + border_width_f);

            // Bottom-right corner revealer: touches bottom and right hole edges
            union_inward_notch(&region,
                               hole_x + hole_width - bottom_right_revealer_width,
                               hole_y + hole_height - bottom_right_revealer_height,
                               bottom_right_revealer_width + border_width_f,
                               bottom_right_revealer_height + border_width_f);

            surface.set_input_region(&region);
        }
    }

    fn subtract_rect(region: &Region, x: i32, y: i32, width: i32, height: i32) {
        if width > 0 && height > 0 {
            let _ = region.subtract_rectangle(&RectangleInt::new(x, y, width, height));
        }
    }

    fn union_inward_notch(region: &Region, x: f64, y: f64, width: f64, height: f64) {
        if width > 0.0 && height > 0.0 {
            let _ = region.union_rectangle(&RectangleInt::new(
                x.floor() as i32, y.floor() as i32,
                width.ceil() as i32, height.ceil() as i32,
            ));
        }
    }
}

// --- Public wrapper ---

glib::wrapper! {
    pub struct FrameDrawWidget(ObjectSubclass<imp::FrameDrawWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl FrameDrawWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn update_style(&self, f: impl FnOnce(&mut FrameStyle)) {
        f(&mut self.imp().style.borrow_mut());
        self.imp().invalidate_css_cache();
        self.queue_draw();
    }

    pub fn set_style(&self, style: FrameStyle) {
        *self.imp().style.borrow_mut() = style;
        self.imp().invalidate_css_cache();
        self.queue_draw();
    }

    pub fn set_draw_frame(&self, draw: bool) {
        self.imp().style.borrow_mut().draw_frame = draw;
        self.imp().invalidate_css_cache();
        self.queue_draw();
    }

    pub fn border_width(&self) -> f64 {
        self.imp().style.borrow().border_width
    }

    pub fn border_radius(&self) -> f64 {
        self.imp().style.borrow().border_radius
    }

    pub fn set_border_width(&self, width: f64) {
        self.imp().style.borrow_mut().border_width = width;
        self.imp().invalidate_css_cache();
        self.queue_draw();
    }

    pub fn set_border_radius(&self, radius: f64) {
        self.imp().style.borrow_mut().border_radius = radius;
        self.imp().invalidate_css_cache();
        self.queue_draw();
    }
}

impl Default for FrameDrawWidget {
    fn default() -> Self {
        Self::new()
    }
}
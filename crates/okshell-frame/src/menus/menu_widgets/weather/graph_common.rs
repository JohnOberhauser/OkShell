//! Shared layout and drawing helpers for the weather graphs (hourly, daily).
//!
//! Each graph has the same shape:
//!
//! ```text
//! [axis spacer]  [ icon row (gtk::Fixed)  ]   <- scrolls
//! [axis area  ]  [ plot area (DrawingArea)]   <- scrolls
//! ```
//!
//! This module owns the geometry, theme colors, temperature scale, and the
//! drawing primitives. Each graph only decides *what* to plot.

use std::cell::RefCell;
use std::rc::Rc;

use okshell_style::css_vars::{CssVarCache, Rgba};
use relm4::gtk::prelude::*;
use relm4::gtk::{self, cairo, pango};
use wayle_weather::TemperatureUnit;

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

pub(crate) const GRAPH_HEIGHT: i32 = 178;

/// Weather icon row above the plot. Keep ICON_SIZE below the item spacing
/// (px per hour / day) so neighbouring icons don't touch.
pub(crate) const ICON_SIZE: i32 = 18;
pub(crate) const ICON_ROW_HEIGHT: i32 = ICON_SIZE + 2;
/// Horizontal inset of the first/last item from the edges: half an icon,
/// so every icon can sit exactly centered on its item.
pub(crate) const X_INSET: f64 = ICON_SIZE as f64 / 2.0;

/// Vertical padding inside the plot/axis areas.
pub(crate) const TOP_PAD: f64 = 8.0; // half a label height so the top tick isn't clipped
/// Gap between the bottom of the plot and the x-axis / unit labels.
pub(crate) const BOTTOM_LABEL_GAP: f64 = 12.0;
pub(crate) const BOTTOM_PAD: f64 = 16.0 + BOTTOM_LABEL_GAP; // label height + gap

/// Gap between the innermost axis labels and the plot edge.
pub(crate) const AXIS_LABEL_PAD: f64 = 6.0;

/// Number of horizontal gridline divisions (Y_DIVISIONS + 1 lines).
/// Every y-axis on a graph uses this, so their ticks share gridlines.
pub(crate) const Y_DIVISIONS: usize = 3;

/// X position of item `i` (hour, day, ...) given the spacing between items.
/// Shared by the plot and the icon row so they line up.
pub(crate) fn item_x(i: usize, spacing: f64) -> f64 {
    X_INSET + spacing * i as f64
}

/// Spacing that spreads `count` items across the actual allocated width.
///
/// The plot's minimum width is `content_width(count, min_spacing)`, so this is
/// never less than the minimum spacing. When the container is wider than that,
/// items spread out to fill it instead of leaving a dead stretch at the end.
pub(crate) fn item_spacing(width: f64, count: usize) -> f64 {
    let gaps = count.saturating_sub(1).max(1) as f64;
    ((width - ICON_SIZE as f64) / gaps).max(0.0)
}

/// Minimum plot / icon row width for `count` items at `min_spacing` apart.
/// Past this, the graph scrolls.
pub(crate) fn content_width(count: usize, min_spacing: i32) -> i32 {
    min_spacing * count.saturating_sub(1) as i32 + ICON_SIZE // ICON_SIZE == 2 * X_INSET
}

/// Vertical geometry shared by the axis and the plot so they line up.
pub(crate) struct Geometry {
    pub top: f64,
    pub bottom: f64,
}

impl Geometry {
    pub fn new(h: f64) -> Self {
        Self {
            top: TOP_PAD,
            bottom: h - BOTTOM_PAD,
        }
    }

    /// Map `v` in `lo..=hi` to a y coordinate.
    pub fn y(&self, v: f64, lo: f64, hi: f64) -> f64 {
        let span = (hi - lo).max(1e-6);
        self.bottom - (v - lo) / span * (self.bottom - self.top)
    }

    /// Baseline for the x-axis labels and axis captions.
    pub fn caption_y(&self) -> f64 {
        self.bottom + BOTTOM_LABEL_GAP
    }
}

// ---------------------------------------------------------------------------
// Temperature scale
// ---------------------------------------------------------------------------

/// A temperature range with exactly Y_DIVISIONS clean steps.
#[derive(Clone, Copy)]
pub(crate) struct TempScale {
    pub lo: f64,
    pub hi: f64,
    pub step: f64,
}

impl Default for TempScale {
    fn default() -> Self {
        Self {
            lo: 0.0,
            hi: Y_DIVISIONS as f64,
            step: 1.0,
        }
    }
}

impl TempScale {
    /// Smallest clean step whose Y_DIVISIONS steps cover all `values`.
    pub fn fit(values: impl IntoIterator<Item = f64>) -> Self {
        let (min, max) = values
            .into_iter()
            .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)));
        if min > max {
            return Self::default(); // no values
        }

        const STEPS: [f64; 15] = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0,
        ];
        let divisions = Y_DIVISIONS as f64;
        for step in STEPS {
            let lo = (min / step).floor() * step;
            let hi = lo + step * divisions;
            if hi >= max {
                return Self { lo, hi, step };
            }
        }
        // Absurd range; just span it evenly.
        let step = ((max - min) / divisions).ceil().max(1.0);
        let lo = min.floor();
        Self {
            lo,
            hi: lo + step * divisions,
            step,
        }
    }

    pub fn ticks(&self) -> impl Iterator<Item = f64> {
        let s = *self;
        (0..=Y_DIVISIONS).map(move |i| s.lo + i as f64 * s.step)
    }

    pub fn y(&self, g: &Geometry, t: f64) -> f64 {
        g.y(t, self.lo, self.hi)
    }
}

/// Unit caption for the temperature axis.
/// Adjust the variant names if TemperatureUnit's differ.
pub(crate) fn temp_unit_label(unit: &TemperatureUnit) -> &'static str {
    match unit {
        TemperatureUnit::Metric => "°C",
        TemperatureUnit::Imperial => "°F",
    }
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// Graph colors, read from the theme palette on every draw.
/// Anything missing falls back to the widget's CSS `color`.
pub(crate) struct GraphTheme {
    pub primary: Rgba,
    pub secondary: Rgba,
    /// Horizontal gridlines and minor vertical lines.
    pub grid: Rgba,
    /// Major vertical lines (labeled hours, days).
    pub grid_strong: Rgba,
    /// x-axis labels and neutral text.
    pub label: Rgba,
}

impl GraphTheme {
    pub fn resolve(area: &gtk::DrawingArea, css: &CssVarCache) -> Self {
        css.refresh(area);
        let c = area.color();
        let fg = |a: f64| (c.red() as f64, c.green() as f64, c.blue() as f64, a);
        Self {
            primary: css.color("--primary").unwrap_or(fg(1.0)),
            secondary: css.color("--secondary").unwrap_or(fg(0.6)),
            grid: css.color("--outline-variant").unwrap_or(fg(0.08)),
            grid_strong: css.color("--outline").unwrap_or(fg(0.2)),
            label: css.color("--on-surface").unwrap_or(fg(1.0)),
        }
    }
}

/// Install a draw func that resolves the theme and borrows shared graph data.
pub(crate) fn set_themed_draw_func<T: 'static>(
    area: &gtk::DrawingArea,
    data: Rc<RefCell<T>>,
    draw: impl Fn(&gtk::DrawingArea, &cairo::Context, f64, f64, &T, &GraphTheme) + 'static,
) {
    let css = CssVarCache::default();
    area.set_draw_func(move |area, cr, w, h| {
        let theme = GraphTheme::resolve(area, &css);
        draw(area, cr, w as f64, h as f64, &data.borrow(), &theme);
    });
}

// ---------------------------------------------------------------------------
// Icon row
// ---------------------------------------------------------------------------

/// Replace the icon row with one icon per item, in order.
/// Real gtk::Images (not cairo) so symbolic icons get recolored by CSS.
///
/// `area` is the plot the icons sit above; its current width decides the
/// spacing, same as in the draw func.
pub(crate) fn rebuild_icon_row<S: AsRef<str>>(
    icons: &gtk::Fixed,
    area: &gtk::DrawingArea,
    min_width: i32,
    icon_names: impl IntoIterator<Item = S>,
    css_class: &str,
) {
    while let Some(child) = icons.first_child() {
        icons.remove(&child);
    }
    icons.set_width_request(min_width);

    for name in icon_names {
        let image = gtk::Image::from_icon_name(name.as_ref());
        image.set_pixel_size(ICON_SIZE);
        image.add_css_class(css_class);
        icons.put(&image, 0.0, 0.0);
    }

    // Before the first allocation area.width() is 0; fall back to the minimum.
    relayout_icon_row(icons, area.width().max(min_width) as f64);
}

/// Keep the icon row in step with the plot when the plot is resized.
/// Call once in init.
pub(crate) fn follow_plot_width(area: &gtk::DrawingArea, icons: &gtk::Fixed) {
    let icons = icons.clone();
    area.connect_resize(move |_, w, _| relayout_icon_row(&icons, w as f64));
}

/// Center each icon on its item's x for the given plot width.
fn relayout_icon_row(icons: &gtk::Fixed, width: f64) {
    let mut children = Vec::new();
    let mut child = icons.first_child();
    while let Some(c) = child {
        child = c.next_sibling();
        children.push(c);
    }

    let spacing = item_spacing(width, children.len());
    for (i, c) in children.iter().enumerate() {
        icons.move_(c, item_x(i, spacing) - X_INSET, 0.0);
    }
}

// ---------------------------------------------------------------------------
// Composite drawing
// ---------------------------------------------------------------------------

/// Horizontal gridlines at the temperature ticks.
pub(crate) fn draw_horizontal_grid(
    cr: &cairo::Context,
    g: &Geometry,
    scale: &TempScale,
    w: f64,
    theme: &GraphTheme,
) {
    cr.set_line_width(1.0);
    set_color(cr, theme.grid, 1.0);
    for t in scale.ticks() {
        hline(cr, scale.y(g, t), w);
    }
    let _ = cr.stroke();
}

/// Vertical lines at `(x, is_major)`. Major lines use the stronger outline color.
pub(crate) fn draw_vertical_grid(
    cr: &cairo::Context,
    g: &Geometry,
    lines: impl IntoIterator<Item = (f64, bool)>,
    theme: &GraphTheme,
) {
    cr.set_line_width(1.0);
    for (x, major) in lines {
        if major {
            set_color(cr, theme.grid_strong, 0.75);
        } else {
            set_color(cr, theme.grid, 0.5);
        }
        vline(cr, x, g.top, g.bottom);
        let _ = cr.stroke();
    }
}

/// One right-aligned label column on an axis: tick labels at `(y, text)`,
/// plus a caption on the bottom row (unit name).
pub(crate) fn draw_axis_column<S: AsRef<str>>(
    area: &gtk::DrawingArea,
    cr: &cairo::Context,
    g: &Geometry,
    ticks: impl IntoIterator<Item = (f64, S)>,
    caption: &str,
    right: f64,
    color: Rgba,
) {
    set_color(cr, color, 1.0);
    for (y, text) in ticks {
        let (layout, tw, th) = text_layout(area, text.as_ref());
        show_layout(cr, &layout, (right - tw).max(0.0), y - th / 2.0);
    }
    let (layout, tw, _) = text_layout(area, caption);
    show_layout(cr, &layout, (right - tw).max(0.0), g.caption_y());
}

/// Temperature tick labels for `draw_axis_column`.
pub(crate) fn temp_ticks(g: &Geometry, scale: &TempScale) -> Vec<(f64, String)> {
    scale
        .ticks()
        .map(|t| (scale.y(g, t), format!("{t:.0}")))
        .collect()
}

/// x-axis labels centered under `(x, text)`, kept inside the widget.
pub(crate) fn draw_bottom_labels<S: AsRef<str>>(
    area: &gtk::DrawingArea,
    cr: &cairo::Context,
    g: &Geometry,
    w: f64,
    labels: impl IntoIterator<Item = (f64, S)>,
    color: Rgba,
) {
    set_color(cr, color, 1.0);
    for (x, text) in labels {
        let (layout, tw, _) = text_layout(area, text.as_ref());
        let x = (x - tw / 2.0).clamp(0.0, (w - tw).max(0.0));
        show_layout(cr, &layout, x, g.caption_y());
    }
}

/// Add a flat lead-in/lead-out across the X_INSET margins so a curve
/// runs edge to edge.
pub(crate) fn with_edge_leads(points: &[(f64, f64)], w: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(points.len() + 2);
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        out.push((0.0, first.1));
        out.extend_from_slice(points);
        out.push((w, last.1));
    }
    out
}

/// Stroke a smooth 2px data line.
pub(crate) fn stroke_curve(cr: &cairo::Context, xy: &[(f64, f64)], color: Rgba) {
    if xy.len() < 2 {
        return;
    }
    smooth_path(cr, xy);
    set_color(cr, color, 1.0);
    cr.set_line_width(2.0);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);
    let _ = cr.stroke();
}

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

/// Set a source color, scaling its alpha.
pub(crate) fn set_color(cr: &cairo::Context, c: Rgba, alpha: f64) {
    cr.set_source_rgba(c.0, c.1, c.2, c.3 * alpha);
}

pub(crate) fn hline(cr: &cairo::Context, y: f64, w: f64) {
    let y = y.round() + 0.5; // crisp 1px lines
    cr.move_to(0.0, y);
    cr.line_to(w, y);
}

pub(crate) fn vline(cr: &cairo::Context, x: f64, top: f64, bottom: f64) {
    let x = x.round() + 0.5; // crisp 1px lines
    cr.move_to(x, top);
    cr.line_to(x, bottom);
}

/// New path: Catmull-Rom spline through the points.
pub(crate) fn smooth_path(cr: &cairo::Context, p: &[(f64, f64)]) {
    cr.new_path();
    cr.move_to(p[0].0, p[0].1);
    append_smooth(cr, p);
}

/// Append a Catmull-Rom spline through `p` to the current path, as cubic
/// Béziers. Assumes the current point is already at `p[0]`.
pub(crate) fn append_smooth(cr: &cairo::Context, p: &[(f64, f64)]) {
    for i in 0..p.len().saturating_sub(1) {
        let p0 = p[i.saturating_sub(1)];
        let p1 = p[i];
        let p2 = p[i + 1];
        let p3 = p[(i + 2).min(p.len() - 1)];
        let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        cr.curve_to(c1.0, c1.1, c2.0, c2.1, p2.0, p2.1);
    }
}

pub(crate) fn rounded_top_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::{FRAC_PI_2, PI};
    let r = r.min(w / 2.0).min(h);
    cr.new_path();
    cr.move_to(x, y + h);
    cr.line_to(x, y + r);
    cr.arc(x + r, y + r, r, PI, PI + FRAC_PI_2);
    cr.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    cr.line_to(x + w, y + h);
    cr.close_path();
}

/// Uses the widget's own Pango context, so CSS font-size/family apply.
pub(crate) fn text_layout(area: &gtk::DrawingArea, text: &str) -> (pango::Layout, f64, f64) {
    let layout = area.create_pango_layout(Some(text));
    let (tw, th) = layout.pixel_size();
    (layout, tw as f64, th as f64)
}

pub(crate) fn show_layout(cr: &cairo::Context, layout: &pango::Layout, x: f64, y: f64) {
    cr.move_to(x, y);
    pangocairo::functions::show_layout(cr, layout);
}

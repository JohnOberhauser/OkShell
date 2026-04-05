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
    pub top_left_expander_height: f64,
    pub top_right_expander_height: f64,
    pub bottom_right_expander_height: f64,
    pub bottom_left_expander_height: f64,
    pub left_expander_width: f64,
    pub right_expander_width: f64,
    pub top_revealer_size: (f64, f64),
    pub bottom_revealer_size: (f64, f64),
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
            top_left_expander_height: 0.0,
            top_right_expander_height: 0.0,
            bottom_left_expander_height: 0.0,
            bottom_right_expander_height: 0.0,
            left_expander_width: 0.0,
            right_expander_width: 0.0,
            top_revealer_size: (0.0, 0.0),
            bottom_revealer_size: (0.0, 0.0),
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

/// Resolve frame-specific CSS custom properties from a full vars map
/// and apply them on top of a base FrameStyle.
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

// --- Subclass internals ---

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FrameDrawWidget {
        pub(super) style: RefCell<FrameStyle>,
        /// Cached style with CSS overrides applied.
        pub(super) resolved_style: RefCell<Option<FrameStyle>>,
        /// Hash of the last CSS dump — used to detect changes.
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

            // Invalidate cache + redraw when CSS classes change
            widget.connect_notify(Some("css-classes"), |w, _| {
                w.imp().invalidate_css_cache();
                w.queue_draw();
            });

            // Redraw on theme changes (dark/light switch, theme name change)
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
            let w = widget.width() as f64;
            let h = widget.height() as f64;

            if w <= 0.0 || h <= 0.0 {
                return;
            }

            let style = self.get_resolved_style();

            let left = style.left_thickness;
            let right = style.right_thickness;
            let top = style.top_thickness;
            let bottom = style.bottom_thickness;
            let x = left;
            let y = top;
            let iw = (w - left - right).max(0.0);
            let ih = (h - top - bottom).max(0.0);
            let r = style.border_radius.clamp(0.0, iw.min(ih) / 2.0);

            let lw = style.left_expander_width;
            let rw = style.right_expander_width;
            let tl_h = style.top_left_expander_height;
            let bl_h = style.bottom_left_expander_height;
            let tr_h = style.top_right_expander_height;
            let br_h = style.bottom_right_expander_height;

            let (top_rev_w, top_rev_h) = style.top_revealer_size;
            let (bot_rev_w, bot_rev_h) = style.bottom_revealer_size;

            if style.draw_frame {
                let bounds = gtk::graphene::Rect::new(0.0, 0.0, w as f32, h as f32);
                let cr = snapshot.append_cairo(&bounds);

                // 1) Paint full background
                cr.set_operator(Operator::Over);
                let (fr, fg, fb, fa) = style.background_rgba;
                cr.set_source_rgba(fr, fg, fb, fa);
                cr.rectangle(0.0, 0.0, w, h);
                let _ = cr.fill();

                // 2) Clear the combined hole.  The hole path routes around the
                //    top/bottom inward notches so they stay painted as frame.
                cr.set_operator(Operator::Clear);
                combined_hole_path(
                    &cr, x, y, iw, ih, r,
                    lw, rw, tl_h, tr_h, br_h, bl_h,
                    top_rev_w, top_rev_h, bot_rev_w, bot_rev_h,
                );
                let _ = cr.fill();

                // 3) Border follows the combined shape
                let (border_r, border_g, border_b, border_a) = style.border_rgba;
                if style.border_width > 0.0 && border_a > 0.0 {
                    cr.set_operator(Operator::Over);
                    combined_hole_path(
                        &cr, x, y, iw, ih, r,
                        lw, rw, tl_h, tr_h, br_h, bl_h,
                        top_rev_w, top_rev_h, bot_rev_w, bot_rev_h,
                    );
                    cr.clip();
                    combined_hole_path(
                        &cr, x, y, iw, ih, r,
                        lw, rw, tl_h, tr_h, br_h, bl_h,
                        top_rev_w, top_rev_h, bot_rev_w, bot_rev_h,
                    );
                    cr.set_source_rgba(border_r, border_g, border_b, border_a);
                    cr.set_line_width(style.border_width * 2.0);
                    cr.set_line_join(LineJoin::Round);
                    cr.set_line_cap(LineCap::Round);
                    let _ = cr.stroke();
                    cr.reset_clip();
                }
            }

            self.update_input_region(
                w as i32, h as i32,
                x, y, iw, ih,
                lw, rw, tl_h, tr_h, br_h, bl_h,
                top_rev_w, top_rev_h, bot_rev_w, bot_rev_h,
            );
        }
    }

    impl FrameDrawWidget {
        /// Get the resolved style, re-parsing CSS only if it has changed.
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

        /// Invalidate the cached CSS so the next snapshot re-parses.
        pub(super) fn invalidate_css_cache(&self) {
            self.css_hash.set(0);
            *self.resolved_style.borrow_mut() = None;
        }

        fn update_input_region(
            &self,
            win_w: i32, win_h: i32,
            x: f64, y: f64, iw: f64, ih: f64,
            lw: f64, rw: f64,
            tl_h: f64, tr_h: f64, br_h: f64, bl_h: f64,
            top_rev_w: f64, top_rev_h: f64,
            bot_rev_w: f64, bot_rev_h: f64,
        ) {
            let widget = self.obj();

            let Some(native) = widget.native() else { return };
            let Some(surface) = native.surface() else { return };

            let hole_l = x.floor().max(0.0) as i32;
            let hole_t = y.floor().max(0.0) as i32;
            let hole_r = (x + iw).ceil().min(win_w as f64) as i32;
            let hole_b = (y + ih).ceil().min(win_h as f64) as i32;

            // Start with the full window as clickable
            let region = Region::create();
            region
                .union_rectangle(&RectangleInt::new(0, 0, win_w, win_h))
                .expect("region union");

            // Subtract main hole
            region
                .subtract_rectangle(&RectangleInt::new(
                    hole_l,
                    hole_t,
                    (hole_r - hole_l).max(0),
                    (hole_b - hole_t).max(0),
                ))
                .expect("region subtract hole");

            // Subtract side corner expander areas (extend outward into frame bands)
            let lw_i = lw.ceil() as i32;
            let rw_i = rw.ceil() as i32;

            subtract_rect(&region, hole_l - lw_i, hole_t,
                          lw_i, tl_h.ceil() as i32);
            subtract_rect(&region, hole_l - lw_i, hole_b - bl_h.ceil() as i32,
                          lw_i, bl_h.ceil() as i32);
            subtract_rect(&region, hole_r, hole_t,
                          rw_i, tr_h.ceil() as i32);
            subtract_rect(&region, hole_r, hole_b - br_h.ceil() as i32,
                          rw_i, br_h.ceil() as i32);

            // Top/bottom inward notches live inside the main hole (already
            // subtracted).  Union them back so the frame captures input there.
            if top_rev_w > 0.0 && top_rev_h > 0.0 {
                let notch_cx = x + iw / 2.0;
                let notch_l = (notch_cx - top_rev_w / 2.0).floor() as i32;
                let notch_w = top_rev_w.ceil() as i32;
                let notch_h = top_rev_h.ceil() as i32;
                let _ = region.union_rectangle(&RectangleInt::new(
                    notch_l, hole_t, notch_w, notch_h,
                ));
            }

            if bot_rev_w > 0.0 && bot_rev_h > 0.0 {
                let notch_cx = x + iw / 2.0;
                let notch_l = (notch_cx - bot_rev_w / 2.0).floor() as i32;
                let notch_w = bot_rev_w.ceil() as i32;
                let notch_h = bot_rev_h.ceil() as i32;
                let _ = region.union_rectangle(&RectangleInt::new(
                    notch_l, hole_b - notch_h, notch_w, notch_h,
                ));
            }

            surface.set_input_region(&region);
        }
    }

    fn subtract_rect(region: &Region, x: i32, y: i32, w: i32, h: i32) {
        if w > 0 && h > 0 {
            let _ = region.subtract_rectangle(&RectangleInt::new(x, y, w, h));
        }
    }

    /// Trace the perimeter of the combined hole: the main rounded rectangle
    /// plus rectangular notches extending outward into the left/right frame
    /// bands at each corner where an expander is active, and with inward
    /// indentations at the center top/bottom edges where the revealer menus
    /// occupy space (those regions stay as painted frame).
    ///
    /// Side notches expand the hole outward into the frame bands.
    /// Top/bottom center notches shrink the hole inward — the path detours
    /// around them so the frame background covers those areas.
    ///
    /// All corners are rounded:
    /// - Convex corners use `arc`.
    /// - Concave corners (where a notch meets the main hole edge) use
    ///   `arc_negative`.
    ///
    /// The path is traced clockwise (screen coords, y-down) starting from
    /// the top-left region.
    fn combined_hole_path(
        cr: &Context,
        x: f64, y: f64, iw: f64, ih: f64, r: f64,
        lw: f64, rw: f64,
        tl_h: f64, tr_h: f64, br_h: f64, bl_h: f64,
        top_rev_w: f64, top_rev_h: f64,
        bot_rev_w: f64, bot_rev_h: f64,
    ) {
        let r = r.clamp(0.0, iw.min(ih) / 2.0);
        let pi = std::f64::consts::PI;
        let pi2 = std::f64::consts::FRAC_PI_2;

        let has_tl = tl_h > 0.0 && lw > 0.0;
        let has_tr = tr_h > 0.0 && rw > 0.0;
        let has_br = br_h > 0.0 && rw > 0.0;
        let has_bl = bl_h > 0.0 && lw > 0.0;
        let has_top = top_rev_w > 0.0 && top_rev_h > 0.0;
        let has_bot = bot_rev_w > 0.0 && bot_rev_h > 0.0;

        // Side notch outer corner radii
        let tr_r = if has_tr { r.min(rw / 2.0).min(tr_h / 2.0) } else { 0.0 };
        let br_r = if has_br { r.min(rw / 2.0).min(br_h / 2.0) } else { 0.0 };
        let tl_r = if has_tl { r.min(lw / 2.0).min(tl_h / 2.0) } else { 0.0 };
        let bl_r = if has_bl { r.min(lw / 2.0).min(bl_h / 2.0) } else { 0.0 };

        // Side notch inner (concave) corner radii
        let right_avail = (ih - tr_h - br_h).max(0.0);
        let left_avail  = (ih - tl_h - bl_h).max(0.0);

        let tr_ir = if has_tr { r.min(rw / 2.0).min(tr_h / 2.0).min(right_avail / 2.0) } else { 0.0 };
        let br_ir = if has_br { r.min(rw / 2.0).min(br_h / 2.0).min(right_avail / 2.0) } else { 0.0 };
        let tl_ir = if has_tl { r.min(lw / 2.0).min(tl_h / 2.0).min(left_avail / 2.0) } else { 0.0 };
        let bl_ir = if has_bl { r.min(lw / 2.0).min(bl_h / 2.0).min(left_avail / 2.0) } else { 0.0 };

        // Top/bottom center inward notch geometry
        let top_cx = x + iw / 2.0;
        let top_nl = top_cx - top_rev_w / 2.0;
        let top_nr = top_cx + top_rev_w / 2.0;

        let bot_cx = x + iw / 2.0;
        let bot_nl = bot_cx - bot_rev_w / 2.0;
        let bot_nr = bot_cx + bot_rev_w / 2.0;

        // Convex corner radius for inward notches (the corners at the
        // deepest point of the indentation)
        let top_nr_r = if has_top { r.min(top_rev_w / 2.0).min(top_rev_h / 2.0) } else { 0.0 };
        let bot_nr_r = if has_bot { r.min(bot_rev_w / 2.0).min(bot_rev_h / 2.0) } else { 0.0 };

        // Concave corner radius where inward notch meets the main top/bottom edge
        let top_left_avail = (top_nl - x - if has_tl { 0.0 } else { r }).max(0.0);
        let top_right_avail = (x + iw - top_nr - if has_tr { 0.0 } else { r }).max(0.0);
        let top_ir = if has_top {
            r.min(top_rev_w / 2.0).min(top_rev_h / 2.0)
                .min(top_left_avail / 2.0)
                .min(top_right_avail / 2.0)
        } else { 0.0 };

        let bot_left_avail = (bot_nl - x - if has_bl { 0.0 } else { r }).max(0.0);
        let bot_right_avail = (x + iw - bot_nr - if has_br { 0.0 } else { r }).max(0.0);
        let bot_ir = if has_bot {
            r.min(bot_rev_w / 2.0).min(bot_rev_h / 2.0)
                .min(bot_left_avail / 2.0)
                .min(bot_right_avail / 2.0)
        } else { 0.0 };

        cr.new_path();

        // =================================================================
        // Move to start of top edge
        // =================================================================
        if has_tl {
            cr.move_to(x - lw + tl_r, y);
        } else {
            cr.move_to(x + r, y);
        }

        // =================================================================
        // Top edge, left → right, with optional inward top notch
        // The notch pokes downward into the hole — the path detours down
        // around it so the area stays as frame (not cleared).
        // =================================================================
        if has_top {
            // Left portion of top edge to concave corner
            cr.line_to(top_nl - top_ir, y);
            // Concave corner: top edge turns down into notch left wall
            if top_ir > 0.0 {
                cr.arc(top_nl - top_ir, y + top_ir, top_ir, -pi2, 0.0);
            }
            // Left wall of notch going down
            cr.line_to(top_nl, y + top_rev_h - top_nr_r);
            // Convex bottom-left corner of notch
            cr.arc_negative(top_nl + top_nr_r, y + top_rev_h - top_nr_r, top_nr_r, pi, pi2);
            // Bottom edge of notch (left → right)
            cr.line_to(top_nr - top_nr_r, y + top_rev_h);
            // Convex bottom-right corner of notch
            cr.arc_negative(top_nr - top_nr_r, y + top_rev_h - top_nr_r, top_nr_r, pi2, 0.0);
            // Right wall of notch going up
            cr.line_to(top_nr, y + top_ir);
            // Concave corner: notch right wall turns back to top edge
            if top_ir > 0.0 {
                cr.arc(top_nr + top_ir, y + top_ir, top_ir, pi, 1.5 * pi);
            }
            // Continue top edge rightward
            if has_tr {
                cr.line_to(x + iw + rw - tr_r, y);
            } else {
                cr.line_to(x + iw - r, y);
            }
        } else {
            if has_tr {
                cr.line_to(x + iw + rw - tr_r, y);
            } else {
                cr.line_to(x + iw - r, y);
            }
        }

        // =================================================================
        // Top-right corner (side notch extends outward into right frame band)
        // =================================================================
        if has_tr {
            cr.arc(x + iw + rw - tr_r, y + tr_r, tr_r, -pi2, 0.0);
            cr.arc(x + iw + rw - tr_r, y + tr_h - tr_r, tr_r, 0.0, pi2);
            cr.line_to(x + iw + tr_ir, y + tr_h);
            if tr_ir > 0.0 {
                cr.arc_negative(x + iw + tr_ir, y + tr_h + tr_ir, tr_ir, -pi2, pi);
            }
        } else {
            cr.arc(x + iw - r, y + r, r, -pi2, 0.0);
        }

        // =================================================================
        // Right edge, top → bottom
        // =================================================================
        if has_br {
            cr.line_to(x + iw, y + ih - br_h - br_ir);
            if br_ir > 0.0 {
                cr.arc_negative(x + iw + br_ir, y + ih - br_h - br_ir, br_ir, pi, pi2);
            }
        } else {
            cr.line_to(x + iw, y + ih - r);
        }

        // =================================================================
        // Bottom-right corner (side notch extends outward into right frame band)
        // =================================================================
        if has_br {
            cr.line_to(x + iw + rw - br_r, y + ih - br_h);
            cr.arc(x + iw + rw - br_r, y + ih - br_h + br_r, br_r, -pi2, 0.0);
            cr.arc(x + iw + rw - br_r, y + ih - br_r, br_r, 0.0, pi2);
        } else {
            cr.arc(x + iw - r, y + ih - r, r, 0.0, pi2);
        }

        // =================================================================
        // Bottom edge, right → left, with optional inward bottom notch
        // The notch pokes upward into the hole — the path detours up
        // around it so the area stays as frame.
        // =================================================================
        if has_bot {
            // Right portion of bottom edge to concave corner
            cr.line_to(bot_nr + bot_ir, y + ih);
            // Concave corner: bottom edge turns up into notch right wall
            if bot_ir > 0.0 {
                cr.arc(bot_nr + bot_ir, y + ih - bot_ir, bot_ir, pi2, pi);
            }
            // Right wall of notch going up
            cr.line_to(bot_nr, y + ih - bot_rev_h + bot_nr_r);
            // Convex top-right corner of notch
            cr.arc_negative(bot_nr - bot_nr_r, y + ih - bot_rev_h + bot_nr_r, bot_nr_r, 0.0, -pi2);
            // Top edge of notch (right → left)
            cr.line_to(bot_nl + bot_nr_r, y + ih - bot_rev_h);
            // Convex top-left corner of notch
            cr.arc_negative(bot_nl + bot_nr_r, y + ih - bot_rev_h + bot_nr_r, bot_nr_r, -pi2, pi);
            // Left wall of notch going down
            cr.line_to(bot_nl, y + ih - bot_ir);
            // Concave corner: notch left wall turns back to bottom edge
            if bot_ir > 0.0 {
                cr.arc(bot_nl - bot_ir, y + ih - bot_ir, bot_ir, 0.0, pi2);
            }
            // Continue bottom edge leftward
            if has_bl {
                cr.line_to(x - lw + bl_r, y + ih);
            } else {
                cr.line_to(x + r, y + ih);
            }
        } else {
            if has_bl {
                cr.line_to(x - lw + bl_r, y + ih);
            } else {
                cr.line_to(x + r, y + ih);
            }
        }

        // =================================================================
        // Bottom-left corner (side notch extends outward into left frame band)
        // =================================================================
        if has_bl {
            cr.arc(x - lw + bl_r, y + ih - bl_r, bl_r, pi2, pi);
            cr.arc(x - lw + bl_r, y + ih - bl_h + bl_r, bl_r, pi, 1.5 * pi);
            cr.line_to(x - bl_ir, y + ih - bl_h);
            if bl_ir > 0.0 {
                cr.arc_negative(x - bl_ir, y + ih - bl_h - bl_ir, bl_ir, pi2, 0.0);
            }
        } else {
            cr.arc(x + r, y + ih - r, r, pi2, pi);
        }

        // =================================================================
        // Left edge, bottom → top
        // =================================================================
        if has_tl {
            cr.line_to(x, y + tl_h + tl_ir);
            if tl_ir > 0.0 {
                cr.arc_negative(x - tl_ir, y + tl_h + tl_ir, tl_ir, 0.0, -pi2);
            }
        } else {
            cr.line_to(x, y + r);
        }

        // =================================================================
        // Top-left corner (side notch extends outward into left frame band)
        // =================================================================
        if has_tl {
            cr.line_to(x - lw + tl_r, y + tl_h);
            cr.arc(x - lw + tl_r, y + tl_h - tl_r, tl_r, pi2, pi);
            cr.arc(x - lw + tl_r, y + tl_r, tl_r, pi, 1.5 * pi);
        } else {
            cr.arc(x + r, y + r, r, pi, 1.5 * pi);
        }

        cr.close_path();
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

    /// Update a style property and trigger a redraw.
    /// Also invalidates the CSS cache so changes are picked up.
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
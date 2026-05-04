use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::common::*;
use crate::utils::find_gdk_monitor;
use gtk4::prelude::*;
use gtk4::{cairo, gdk, glib};
use gtk4_layer_shell::LayerShell;

#[derive(Debug, Clone)]
pub struct RegionSelection {
    pub output: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

struct SharedState {
    drag_start: Cell<Option<(f64, f64)>>,
    drag_current: Cell<Option<(f64, f64)>>,
    on_done: RefCell<Option<Box<dyn FnOnce(Result<RegionSelection>)>>>,
    windows: RefCell<Vec<gtk4::Window>>,
}

impl SharedState {
    fn fire(&self, result: Result<RegionSelection>) {
        for window in self.windows.borrow().iter() {
            window.close();
        }
        if let Some(f) = self.on_done.borrow_mut().take() {
            f(result);
        }
    }

    fn is_done(&self) -> bool {
        self.on_done.borrow().is_none()
    }
}

/// Open fullscreen layer-shell overlays on ALL outputs for region selection.
///
/// This returns immediately. When the user drags a region on any monitor
/// (or presses Escape), `on_done` is called with the result. All overlay
/// windows are destroyed automatically.
pub fn select_region<F>(outputs: &[OutputInfo], on_done: F)
where
    F: FnOnce(Result<RegionSelection>) + 'static,
{
    if outputs.is_empty() {
        on_done(Err(ScreenshotError::CaptureFailed(
            "no outputs found".into(),
        )));
        return;
    }

    let state = Rc::new(SharedState {
        drag_start: Cell::new(None),
        drag_current: Cell::new(None),
        on_done: RefCell::new(Some(Box::new(on_done))),
        windows: RefCell::new(Vec::new()),
    });

    let gdk_display = gdk::Display::default().expect("no display");
    let monitors = gdk_display.monitors();

    for output in outputs {
        let gdk_monitor = find_gdk_monitor(&monitors, output);
        let window = create_overlay_window(output, gdk_monitor.as_ref(), &state);
        state.windows.borrow_mut().push(window);
    }

    for window in state.windows.borrow().iter() {
        window.present();
    }
}

/// Create a single overlay window for one output.
fn create_overlay_window(
    output: &OutputInfo,
    gdk_monitor: Option<&gdk::Monitor>,
    state: &Rc<SharedState>,
) -> gtk4::Window {
    let window = gtk4::Window::new();
    window.set_decorated(false);

    // ── Layer shell setup ──
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_exclusive_zone(-1);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    window.set_namespace(Some("okshell-screenshot"));

    if let Some(monitor) = gdk_monitor {
        window.set_monitor(Some(monitor));
    }

    window.add_css_class("screenshot-overlay");
    window.set_cursor_from_name(Some("crosshair"));

    // ── Drawing area ──
    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.add_css_class("screenshot-area-selector");
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    window.set_child(Some(&drawing_area));

    let state_draw = Rc::clone(state);
    let da_for_draw = drawing_area.clone();
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let color = da_for_draw.color();
        draw_overlay(cr, width, height, &state_draw, &color);
    });

    // ── Drag gesture ──
    let drag = gtk4::GestureDrag::new();

    let state_begin = Rc::clone(state);
    drag.connect_drag_begin(move |_, x, y| {
        state_begin.drag_start.set(Some((x, y)));
        state_begin.drag_current.set(Some((x, y)));
        // Redraw all windows so the non-active ones stay dark.
        for w in state_begin.windows.borrow().iter() {
            if let Some(child) = w.child() {
                child.queue_draw();
            }
        }
    });

    let state_update = Rc::clone(state);
    let da_update = drawing_area.clone();
    drag.connect_drag_update(move |gesture, offset_x, offset_y| {
        if let Some((sx, sy)) = gesture.start_point() {
            let max_w = da_update.width() as f64;
            let max_h = da_update.height() as f64;
            let cx = (sx + offset_x).clamp(0.0, max_w);
            let cy = (sy + offset_y).clamp(0.0, max_h);
            state_update.drag_current.set(Some((cx, cy)));
            da_update.queue_draw();
        }
    });

    let state_end = Rc::clone(state);
    let output_name = output.name.clone();
    let da_end = drawing_area.clone();
    drag.connect_drag_end(move |gesture, offset_x, offset_y| {
        if state_end.is_done() {
            return;
        }
        if let Some((sx, sy)) = gesture.start_point() {
            let max_w = da_end.width() as f64;
            let max_h = da_end.height() as f64;
            let ex = (sx + offset_x).clamp(0.0, max_w);
            let ey = (sy + offset_y).clamp(0.0, max_h);

            let rx = sx.min(ex);
            let ry = sy.min(ey);
            let rw = (ex - sx).abs();
            let rh = (ey - sy).abs();

            if rw > 5.0 && rh > 5.0 {
                let region = RegionSelection {
                    output: output_name.clone(),
                    x: rx as i32,
                    y: ry as i32,
                    width: rw as i32,
                    height: rh as i32,
                };
                state_end.fire(Ok(region));
            } else {
                // Too small — reset and let them try again.
                state_end.drag_start.set(None);
                state_end.drag_current.set(None);
            }
        }
    });

    drawing_area.add_controller(drag);

    // ── Keyboard: Escape to cancel ──
    let key_ctrl = gtk4::EventControllerKey::new();
    let state_key = Rc::clone(state);
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gdk::Key::Escape && !state_key.is_done() {
            state_key.fire(Err(ScreenshotError::Cancelled));
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_ctrl);

    // ── Handle unexpected window destruction ──
    let state_destroy = Rc::clone(state);
    window.connect_destroy(move |_| {
        if !state_destroy.is_done() {
            state_destroy.fire(Err(ScreenshotError::Cancelled));
        }
    });

    window
}

/// Draw the dark overlay with a cutout for the current selection.
fn draw_overlay(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    state: &SharedState,
    outline: &gdk::RGBA,
) {
    // Dark overlay.
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
    cr.rectangle(0.0, 0.0, width as f64, height as f64);
    cr.fill().ok();

    if let Some((sx, sy, sw, sh)) = compute_rect(state) {
        // Cut out the selected area.
        cr.set_operator(cairo::Operator::Clear);
        cr.rectangle(sx, sy, sw, sh);
        cr.fill().ok();

        // Border around selection using outline color from CSS.
        cr.set_operator(cairo::Operator::Over);
        cr.set_source_rgba(
            outline.red() as f64,
            outline.green() as f64,
            outline.blue() as f64,
            outline.alpha() as f64,
        );
        cr.set_line_width(2.0);
        cr.rectangle(sx, sy, sw, sh);
        cr.stroke().ok();

        // Dimension label.
        cr.set_font_size(14.0);
        let label = format!("{}×{}", sw as i32, sh as i32);
        if let Ok(extents) = cr.text_extents(&label) {
            let tx = sx + (sw - extents.width()) / 2.0;
            let ty = sy + sh + 20.0;
            cr.move_to(tx, ty);
            cr.show_text(&label).ok();
        }
    }
}

/// Compute the current rectangle from an in-progress drag.
fn compute_rect(state: &SharedState) -> Option<(f64, f64, f64, f64)> {
    let (sx, sy) = state.drag_start.get()?;
    let (cx, cy) = state.drag_current.get()?;
    let x = sx.min(cx);
    let y = sy.min(cy);
    let w = (cx - sx).abs();
    let h = (cy - sy).abs();
    if w > 1.0 && h > 1.0 {
        Some((x, y, w, h))
    } else {
        None
    }
}

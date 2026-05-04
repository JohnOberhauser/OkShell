use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::common::*;
use crate::utils::find_gdk_monitor;
use gtk4::prelude::*;
use gtk4::{cairo, gdk, glib};
use gtk4_layer_shell::LayerShell;
use okshell_services::hyprland_service;

#[derive(Debug, Clone)]
struct WindowRect {
    output: String,
    /// Hyprland window address.
    address: String,
    /// Output-local coordinates.
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    /// Global coordinates (for capture).
    global_x: i32,
    global_y: i32,
    global_w: i32,
    global_h: i32,
}

struct OverlayInfo {
    window: gtk4::Window,
}

struct SharedState {
    /// All window rects across all monitors.
    window_rects: Vec<WindowRect>,
    /// Address of the currently hovered window (if any).
    hovered: RefCell<Option<String>>,
    /// Whether real mouse movement has occurred.
    activated: Cell<bool>,
    on_done: RefCell<Option<Box<dyn FnOnce(Result<HyprlandWindow>)>>>,
    overlays: RefCell<Vec<OverlayInfo>>,
}

impl SharedState {
    fn fire(&self, result: Result<HyprlandWindow>) {
        for info in self.overlays.borrow().iter() {
            info.window.close();
        }
        if let Some(f) = self.on_done.borrow_mut().take() {
            f(result);
        }
    }

    fn is_done(&self) -> bool {
        self.on_done.borrow().is_none()
    }

    fn redraw_all(&self) {
        for info in self.overlays.borrow().iter() {
            if let Some(child) = info.window.child() {
                child.queue_draw();
            }
        }
    }

    /// Find which window rect contains the given output-local point.
    fn hit_test(&self, output: &str, x: f64, y: f64) -> Option<&WindowRect> {
        self.window_rects.iter().find(|r| {
            r.output == output && x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height
        })
    }

    /// Get rects for a specific output.
    fn rects_for_output(&self, output: &str) -> Vec<&WindowRect> {
        self.window_rects
            .iter()
            .filter(|r| r.output == output)
            .collect()
    }
}

/// Open fullscreen overlays on all outputs for window selection.
///
/// Returns immediately. Hover highlights individual windows, click to select.
/// `on_done` receives `Ok(HyprlandWindow)` or `Err(Cancelled)`.
pub fn select_window<F>(outputs: &[OutputInfo], on_done: F)
where
    F: FnOnce(Result<HyprlandWindow>) + 'static,
{
    if outputs.is_empty() {
        on_done(Err(ScreenshotError::CaptureFailed(
            "no outputs found".into(),
        )));
        return;
    }

    // Gather all visible windows and convert to output-local rects.
    let window_rects = build_window_rects();

    // Pre-populate hovered with the focused window.
    let focused_address = {
        let hyprland = hyprland_service();
        hyprland
            .clients
            .get()
            .iter()
            .filter(|c| c.mapped.get() && !c.hidden.get())
            .min_by_key(|c| c.focus_history_id.get())
            .map(|c| format!("{}", c.address.get()))
    };

    let state = Rc::new(SharedState {
        window_rects,
        hovered: RefCell::new(focused_address),
        activated: Cell::new(false),
        on_done: RefCell::new(Some(Box::new(on_done))),
        overlays: RefCell::new(Vec::new()),
    });

    let gdk_display = gdk::Display::default().expect("no display");
    let monitors = gdk_display.monitors();

    for output in outputs {
        let gdk_monitor = find_gdk_monitor(&monitors, output);
        let window = create_window_overlay(output, gdk_monitor.as_ref(), &state);
        state.overlays.borrow_mut().push(OverlayInfo { window });
    }

    for info in state.overlays.borrow().iter() {
        info.window.present();
    }
}

fn build_window_rects() -> Vec<WindowRect> {
    let hyprland = hyprland_service();
    let clients = hyprland.clients.get();
    let monitors = hyprland.monitors.get();

    // Build a set of active workspace IDs (one per monitor).
    let active_workspace_ids: Vec<_> = monitors
        .iter()
        .map(|m| m.active_workspace.get().id)
        .collect();

    let mut rects = Vec::new();

    for client in &clients {
        if !client.mapped.get() || client.hidden.get() {
            continue;
        }

        let size = client.size.get();
        if size.width <= 0 || size.height <= 0 {
            continue;
        }

        // Only include windows on the active workspace of their monitor.
        let ws = client.workspace.get();
        if !active_workspace_ids.contains(&ws.id) {
            continue;
        }

        let at = client.at.get();
        let monitor_id = client.monitor.get();

        let monitor = monitors.iter().find(|m| m.id.get() == monitor_id).cloned();

        if let Some(monitor) = monitor {
            let mon_name = monitor.name.get();
            let mon_x = monitor.x.get();
            let mon_y = monitor.y.get();

            let local_x = (at.x - mon_x) as f64;
            let local_y = (at.y - mon_y) as f64;

            rects.push(WindowRect {
                output: mon_name,
                address: format!("{}", client.address.get()),
                x: local_x,
                y: local_y,
                width: size.width as f64,
                height: size.height as f64,
                global_x: at.x,
                global_y: at.y,
                global_w: size.width,
                global_h: size.height,
            });
        }
    }

    rects
}

fn create_window_overlay(
    output: &OutputInfo,
    gdk_monitor: Option<&gdk::Monitor>,
    state: &Rc<SharedState>,
) -> gtk4::Window {
    let window = gtk4::Window::new();
    window.set_decorated(false);

    // ── Layer shell ──
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
    window.set_cursor_from_name(Some("pointer"));

    // ── Drawing area ──
    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.add_css_class("screenshot-area-selector");
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    window.set_child(Some(&drawing_area));

    let output_name = output.name.clone();

    // ── Draw ──
    let state_draw = Rc::clone(state);
    let output_name_draw = output_name.clone();
    let da_for_draw = drawing_area.clone();
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let color = da_for_draw.color();
        let hovered = state_draw.hovered.borrow().clone();
        draw_window_overlay(
            cr,
            width,
            height,
            &output_name_draw,
            &state_draw,
            hovered.as_deref(),
            &color,
        );
    });

    // ── Motion: hit-test windows ──
    let motion = gtk4::EventControllerMotion::new();

    let state_motion = Rc::clone(state);
    let output_name_motion = output_name.clone();
    motion.connect_motion(move |_, x, y| {
        state_motion.activated.set(true);

        let new_hovered = state_motion
            .hit_test(&output_name_motion, x, y)
            .map(|r| r.address.clone());

        if *state_motion.hovered.borrow() != new_hovered {
            *state_motion.hovered.borrow_mut() = new_hovered;
            state_motion.redraw_all();
        }
    });

    let state_leave = Rc::clone(state);
    motion.connect_leave(move |_| {
        if !state_leave.activated.get() {
            return;
        }
        *state_leave.hovered.borrow_mut() = None;
        state_leave.redraw_all();
    });

    drawing_area.add_controller(motion);

    // ── Click: select window ──
    let click = gtk4::GestureClick::new();
    let state_click = Rc::clone(state);
    let output_name_click = output_name.clone();
    click.connect_released(move |_, _, x, y| {
        if state_click.is_done() {
            return;
        }
        if let Some(rect) = state_click.hit_test(&output_name_click, x, y) {
            let win = HyprlandWindow {
                address: rect.address.clone(),
                output: rect.output.clone(),
                x: rect.global_x,
                y: rect.global_y,
                width: rect.global_w,
                height: rect.global_h,
            };
            state_click.fire(Ok(win));
        }
    });

    drawing_area.add_controller(click);

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

    // ── Unexpected destruction ──
    let state_destroy = Rc::clone(state);
    window.connect_destroy(move |_| {
        if !state_destroy.is_done() {
            state_destroy.fire(Err(ScreenshotError::Cancelled));
        }
    });

    window
}

fn draw_window_overlay(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    output_name: &str,
    state: &SharedState,
    hovered_address: Option<&str>,
    outline: &gdk::RGBA,
) {
    // Dark overlay covering the entire monitor.
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
    cr.rectangle(0.0, 0.0, width as f64, height as f64);
    cr.fill().ok();

    let rects = state.rects_for_output(output_name);

    for rect in &rects {
        let is_hovered = hovered_address == Some(rect.address.as_str());

        if is_hovered {
            // Clear the hovered window area to show it through.
            cr.set_operator(cairo::Operator::Clear);
            cr.rectangle(rect.x, rect.y, rect.width, rect.height);
            cr.fill().ok();

            // Draw outline border.
            cr.set_operator(cairo::Operator::Over);
            cr.set_source_rgba(
                outline.red() as f64,
                outline.green() as f64,
                outline.blue() as f64,
                outline.alpha() as f64,
            );
            cr.set_line_width(3.0);
            cr.rectangle(
                rect.x - 1.5,
                rect.y - 1.5,
                rect.width + 3.0,
                rect.height + 3.0,
            );
            cr.stroke().ok();
        }
    }
}

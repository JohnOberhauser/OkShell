use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::common::*;
use crate::utils::find_gdk_monitor;
use gtk4::prelude::*;
use gtk4::{cairo, gdk, glib};
use gtk4_layer_shell::LayerShell;

struct OverlayInfo {
    window: gtk4::Window,
}

struct SharedState {
    /// Which output name the mouse is currently over (if any).
    hovered: RefCell<Option<String>>,
    /// Whether we've seen real mouse movement yet.
    /// Until this is true, motion/leave events don't update hovered.
    activated: Cell<bool>,
    on_done: RefCell<Option<Box<dyn FnOnce(Result<String>)>>>,
    overlays: RefCell<Vec<OverlayInfo>>,
}

impl SharedState {
    fn fire(&self, result: Result<String>) {
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
}

/// Open fullscreen layer-shell overlays on all outputs for monitor selection.
///
/// Returns immediately. The hovered monitor is shown clear (unhighlighted),
/// all others are darkened. Click to select, Escape to cancel.
///
/// `on_done` receives `Ok(output_name)` or `Err(Cancelled)`.
pub fn select_monitor<F>(outputs: &[OutputInfo], on_done: F)
where
    F: FnOnce(Result<String>) + 'static,
{
    if outputs.is_empty() {
        on_done(Err(ScreenshotError::CaptureFailed(
            "no outputs found".into(),
        )));
        return;
    }

    // Pre-populate hovered with the focused monitor so the initial draw
    // shows it unhighlighted. activated stays false until real mouse movement.
    let focused = {
        use okshell_services::hyprland_service;
        let hyprland = hyprland_service();
        hyprland
            .monitors
            .get()
            .iter()
            .find(|m| m.focused.get())
            .map(|m| m.name.get())
    };

    let state = Rc::new(SharedState {
        hovered: RefCell::new(focused),
        activated: Cell::new(false),
        on_done: RefCell::new(Some(Box::new(on_done))),
        overlays: RefCell::new(Vec::new()),
    });

    let gdk_display = gdk::Display::default().expect("no display");
    let monitors = gdk_display.monitors();

    for output in outputs {
        let gdk_monitor = find_gdk_monitor(&monitors, output);
        let (window, _) = create_monitor_overlay(output, gdk_monitor.as_ref(), &state);
        state.overlays.borrow_mut().push(OverlayInfo { window });
    }

    for info in state.overlays.borrow().iter() {
        info.window.present();
    }
}

fn create_monitor_overlay(
    output: &OutputInfo,
    gdk_monitor: Option<&gdk::Monitor>,
    state: &Rc<SharedState>,
) -> (gtk4::Window, gtk4::EventControllerMotion) {
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

    // Draw: dark if not hovered, clear if hovered, with outline border.
    let state_draw = Rc::clone(state);
    let output_name_draw = output_name.clone();
    let da_for_draw = drawing_area.clone();
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let color = da_for_draw.color();
        let is_hovered = state_draw.hovered.borrow().as_deref() == Some(output_name_draw.as_str());
        draw_monitor_overlay(cr, width, height, is_hovered, &color);
    });

    // ── Motion: track which monitor the mouse is on ──
    let motion = gtk4::EventControllerMotion::new();

    let state_motion = Rc::clone(state);
    let output_name_motion = output_name.clone();
    motion.connect_motion(move |_, _x, _y| {
        // First real mouse movement activates the motion tracking.
        state_motion.activated.set(true);

        let needs_update =
            state_motion.hovered.borrow().as_deref() != Some(output_name_motion.as_str());
        if needs_update {
            *state_motion.hovered.borrow_mut() = Some(output_name_motion.clone());
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

    drawing_area.add_controller(motion.clone());

    // ── Click: select this monitor ──
    let click = gtk4::GestureClick::new();
    let state_click = Rc::clone(state);
    let output_name_click = output_name.clone();
    click.connect_released(move |_, _, _, _| {
        if !state_click.is_done() {
            state_click.fire(Ok(output_name_click.clone()));
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

    (window, motion)
}

fn draw_monitor_overlay(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    is_hovered: bool,
    outline: &gdk::RGBA,
) {
    if is_hovered {
        // Hovered monitor: mostly clear with a subtle border.
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.05);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        cr.fill().ok();

        // Outline border.
        cr.set_source_rgba(
            outline.red() as f64,
            outline.green() as f64,
            outline.blue() as f64,
            outline.alpha() as f64,
        );
        cr.set_line_width(3.0);
        cr.rectangle(1.5, 1.5, width as f64 - 3.0, height as f64 - 3.0);
        cr.stroke().ok();
    } else {
        // Non-hovered: dark overlay.
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        cr.fill().ok();
    }
}

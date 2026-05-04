use relm4::gtk::{self, gdk, glib, prelude::*};

pub struct HoverScrollHandle {
    _motion: gtk::EventControllerMotion,
    _scroll: gtk::EventControllerScroll,
}

/// Attach "scroll while hovered" behavior to any widget (e.g. gtk::Box).
///
/// Returns handles so you can keep them alive / remove later if you want.
/// (GTK also keeps them alive while attached to the widget, but returning them
/// is convenient if you want to disconnect/remove.)
pub fn attach_hover_scroll<W, F>(widget: &W, on_scroll: F) -> HoverScrollHandle
where
    W: IsA<gtk::Widget>,
    F: Fn(f64, f64, bool, bool) + 'static, // (dx, dy, hovered, shift)
{
    // Track hover state
    let hovered = std::rc::Rc::new(std::cell::Cell::new(false));

    let motion = gtk::EventControllerMotion::new();
    {
        let hovered = hovered.clone();
        motion.connect_enter(move |_, _, _| hovered.set(true));
    }
    {
        let hovered = hovered.clone();
        motion.connect_leave(move |_| hovered.set(false));
    }
    widget.add_controller(motion.clone());

    // Scroll controller (VERTICAL + DISCRETE; add SMOOTH if you want touchpads)
    let scroll = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE,
    );
    scroll.set_propagation_phase(gtk::PropagationPhase::Bubble);

    {
        let hovered = hovered.clone();
        scroll.connect_scroll(move |ctrl, dx, dy| {
            if !hovered.get() {
                return glib::Propagation::Proceed; // = propagate
            }

            // Modifier keys
            let state = ctrl.current_event_state();
            let shift = state.contains(gdk::ModifierType::SHIFT_MASK);

            on_scroll(dx, dy, true, shift);

            glib::Propagation::Stop // stop propagation so parent doesn't consume it
        });
    }

    widget.add_controller(scroll.clone());

    HoverScrollHandle {
        _motion: motion,
        _scroll: scroll,
    }
}

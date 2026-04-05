use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::gtk::glib;
use std::cell::Cell;
use std::rc::Rc;

pub fn wire_vertical_to_horizontal(scroll_window: &gtk::ScrolledWindow, step: f64) {
    let controller = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL,
    );

    let hadj = scroll_window.hadjustment();
    let target = Rc::new(Cell::new(hadj.value()));
    let animating = Rc::new(Cell::new(false));

    let target_clone = target.clone();
    let animating_clone = animating.clone();
    let hadj_clone = hadj.clone();
    let widget = scroll_window.clone();

    controller.connect_scroll(move |_, dx, dy| {
        let delta = if dx.abs() > dy.abs() { dx } else { dy };
        let new_target = (target_clone.get() + delta * step)
            .clamp(hadj_clone.lower(), hadj_clone.upper() - hadj_clone.page_size());
        target_clone.set(new_target);

        if !animating_clone.get() {
            animating_clone.set(true);

            let hadj = hadj_clone.clone();
            let target = target_clone.clone();
            let animating = animating_clone.clone();

            widget.add_tick_callback(move |_, _| {
                let current = hadj.value();
                let goal = target.get();
                let diff = goal - current;

                if diff.abs() < 0.5 {
                    hadj.set_value(goal);
                    animating.set(false);
                    return glib::ControlFlow::Break;
                }

                // Lerp toward target — 0.15 = snappy, 0.08 = buttery
                hadj.set_value(current + diff * 0.15);
                glib::ControlFlow::Continue
            });
        }

        glib::Propagation::Stop
    });

    scroll_window.add_controller(controller);
}
use std::cell::Cell;
use std::rc::Rc;

use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;

const ANIM_DURATION_US: f64 = 400_000.0; // 400ms

/// A horizontal LevelBar whose value eases to each new target.
/// Values are 0.0..=100.0.
#[derive(Debug)]
pub(crate) struct LevelAnim {
    bar: gtk::LevelBar,
    current: Cell<f64>,
    from: Cell<f64>,
    to: Cell<f64>,
    start_us: Cell<i64>,
    animating: Cell<bool>,
}

impl LevelAnim {
    pub(crate) fn new() -> Rc<Self> {
        let bar = gtk::LevelBar::builder()
            .orientation(gtk::Orientation::Horizontal)
            .min_value(0.0)
            .max_value(100.0)
            .hexpand(true)
            .build();

        // GTK's default offsets (low/high/full) are meant for 0..1 ranges like
        // battery levels and would tag the bar as "full". Drop them so the
        // filled block is styled only by CSS.
        for name in ["low", "high", "full"] {
            bar.remove_offset_value(Some(name));
        }

        Rc::new(Self {
            bar,
            current: Cell::new(0.0),
            from: Cell::new(0.0),
            to: Cell::new(0.0),
            start_us: Cell::new(0),
            animating: Cell::new(false),
        })
    }

    pub(crate) fn widget(&self) -> &gtk::LevelBar {
        &self.bar
    }

    pub(crate) fn animate_to(self: &Rc<Self>, target: f64) {
        let target = target.clamp(0.0, 100.0);
        if (target - self.to.get()).abs() < f64::EPSILON {
            return;
        }

        // Start from wherever the bar is right now, even mid-animation.
        self.from.set(self.current.get());
        self.to.set(target);
        self.start_us.set(glib::monotonic_time());

        if self.animating.replace(true) {
            return; // tick callback already running; it'll pick up the new target
        }

        let anim = self.clone();
        self.bar.add_tick_callback(move |bar, clock| {
            let elapsed = (clock.frame_time() - anim.start_us.get()) as f64;
            let t = (elapsed / ANIM_DURATION_US).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - t).powi(3); // ease-out cubic

            let from = anim.from.get();
            let value = from + (anim.to.get() - from) * eased;
            anim.current.set(value);
            bar.set_value(value);

            if t >= 1.0 {
                anim.animating.set(false);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }
}

pub(crate) fn percent_of(value: f64, total: f64) -> f64 {
    if total > 0.0 {
        value / total * 100.0
    } else {
        0.0
    }
}

use std::cell::Cell;
use std::f64::consts::PI;
use std::rc::Rc;

use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;

const RING_SIZE: i32 = 28;
const RING_WIDTH: f64 = 2.5;
const ANIM_DURATION_US: f64 = 400_000.0; // 400ms

/// An icon with an animated circular percentage ring around it.
#[derive(Debug, Clone)]
pub(crate) struct UsageRing {
    overlay: gtk::Overlay,
    fill: gtk::DrawingArea,
    anim: Rc<RingAnim>,
}

impl UsageRing {
    pub(crate) fn new(icon_name: &str) -> Self {
        let anim = Rc::new(RingAnim::default());

        let track = ring_area("usage-ring-track");
        track.set_draw_func(|area, cr, w, h| draw_arc(area, cr, w, h, 1.0));

        let fill = ring_area("usage-ring");
        let draw_anim = anim.clone();
        fill.set_draw_func(move |area, cr, w, h| {
            draw_arc(area, cr, w, h, draw_anim.current.get() / 100.0)
        });

        let image = gtk::Image::builder()
            .icon_name(icon_name)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&track));
        overlay.add_overlay(&fill);
        overlay.add_overlay(&image);

        Self {
            overlay,
            fill,
            anim,
        }
    }

    pub(crate) fn widget(&self) -> &gtk::Overlay {
        &self.overlay
    }

    /// Animate the ring to `percent` (0.0..=100.0).
    pub(crate) fn set_value(&self, percent: f64) {
        self.anim.animate_to(&self.fill, percent);
    }
}

fn ring_area(css_class: &str) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::builder()
        .content_width(RING_SIZE)
        .content_height(RING_SIZE)
        .build();
    area.add_css_class(css_class);
    area
}

#[derive(Debug, Default)]
struct RingAnim {
    current: Cell<f64>,
    from: Cell<f64>,
    to: Cell<f64>,
    start_us: Cell<i64>,
    animating: Cell<bool>,
}

impl RingAnim {
    fn animate_to(self: &Rc<Self>, area: &gtk::DrawingArea, target: f64) {
        let target = target.clamp(0.0, 100.0);
        if (target - self.to.get()).abs() < f64::EPSILON {
            return;
        }

        self.from.set(self.current.get());
        self.to.set(target);
        self.start_us.set(glib::monotonic_time());

        if self.animating.replace(true) {
            return;
        }

        let anim = self.clone();
        area.add_tick_callback(move |area, clock| {
            let elapsed = (clock.frame_time() - anim.start_us.get()) as f64;
            let t = (elapsed / ANIM_DURATION_US).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - t).powi(3);

            let from = anim.from.get();
            anim.current.set(from + (anim.to.get() - from) * eased);
            area.queue_draw();

            if t >= 1.0 {
                anim.animating.set(false);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }
}

fn draw_arc(area: &gtk::DrawingArea, cr: &gtk::cairo::Context, w: i32, h: i32, fraction: f64) {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction <= 0.0 {
        return;
    }

    let size = w.min(h) as f64;
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let radius = (size - RING_WIDTH) / 2.0;

    let c = area.color();
    cr.set_source_rgba(
        c.red() as f64,
        c.green() as f64,
        c.blue() as f64,
        c.alpha() as f64,
    );
    cr.set_line_width(RING_WIDTH);
    cr.set_line_cap(gtk::cairo::LineCap::Round);

    let start = -PI / 2.0;
    cr.arc(cx, cy, radius, start, start + fraction * 2.0 * PI);
    let _ = cr.stroke();
}

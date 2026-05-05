use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::graphene;
use relm4::gtk::prelude::*;
use relm4::gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    pub struct DiagonalRevealer {
        pub child: glib::WeakRef<gtk::Widget>,
        pub revealed: Cell<bool>,
        pub current_pos: Cell<f64>, // 0.0 = hidden, 1.0 = revealed
        pub tick_id: Cell<Option<gtk::TickCallbackId>>,
        pub anim_start: Cell<Option<i64>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DiagonalRevealer {
        const NAME: &'static str = "OkShellDiagonalRevealer";
        type Type = super::DiagonalRevealer;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for DiagonalRevealer {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(None::<gtk::LayoutManager>);
            self.obj().set_overflow(gtk::Overflow::Hidden);
        }

        fn dispose(&self) {
            if let Some(child) = self.child.upgrade() {
                child.unparent();
            }
            if let Some(tick_id) = self.tick_id.take() {
                tick_id.remove();
            }
        }
    }

    impl WidgetImpl for DiagonalRevealer {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let Some(child) = self.child.upgrade() else {
                return (0, 0, -1, -1);
            };
            let (_, nat, _, _) = child.measure(orientation, for_size);
            let pos = self.current_pos.get();
            let scaled = (nat as f64 * pos) as i32;
            (0, scaled, -1, -1)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let Some(child) = self.child.upgrade() else {
                return;
            };
            // Allocate child its full natural size; we clip via overflow
            let (_, nat_w, _, _) = child.measure(gtk::Orientation::Horizontal, -1);
            let (_, nat_h, _, _) = child.measure(gtk::Orientation::Vertical, nat_w);
            child.allocate(nat_w, nat_h, baseline, None);
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let Some(child) = self.child.upgrade() else {
                return;
            };
            let w = self.obj().width() as f32;
            let h = self.obj().height() as f32;
            snapshot.push_clip(&graphene::Rect::new(0.0, 0.0, w, h));
            self.obj().snapshot_child(&child, snapshot);
            snapshot.pop();
        }
    }
}

glib::wrapper! {
    pub struct DiagonalRevealer(ObjectSubclass<imp::DiagonalRevealer>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DiagonalRevealer {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_child(&self, child: Option<&impl IsA<gtk::Widget>>) {
        if let Some(old) = self.imp().child.upgrade() {
            old.unparent();
        }
        if let Some(child) = child {
            let widget = child.upcast_ref::<gtk::Widget>();
            widget.set_parent(self);
            self.imp().child.set(Some(widget));
        }
    }

    pub fn set_revealed(&self, revealed: bool) {
        if self.imp().revealed.get() == revealed {
            return;
        }
        self.imp().revealed.set(revealed);
        self.start_animation();
    }

    fn start_animation(&self) {
        let target: f64 = if self.imp().revealed.get() { 1.0 } else { 0.0 };
        let start_pos = self.imp().current_pos.get();
        let duration_ms = 200.0;
        let widget = self.clone();
        let start_time = std::cell::Cell::new(None::<i64>);

        if let Some(old) = self.imp().tick_id.take() {
            old.remove();
        }

        let id = self.add_tick_callback(move |w, clock| {
            let now = clock.frame_time();
            let started = start_time.get().unwrap_or_else(|| {
                start_time.set(Some(now));
                now
            });
            let elapsed = (now - started) as f64 / 1000.0; // microseconds → ms
            let t = (elapsed / duration_ms).clamp(0.0, 1.0);
            // Ease-in-out
            let eased = if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            };
            let pos = start_pos + (target - start_pos) * eased;
            widget.imp().current_pos.set(pos);
            w.queue_resize(); // crucial: re-measure

            if t >= 1.0 {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
        self.imp().tick_id.set(Some(id));
    }
}

use relm4::gtk;
use gtk::{glib, prelude::*, subclass::prelude::*};

mod notifying_layout {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    pub struct NotifyingLayout {
        // Delegated real box layout
        inner: std::cell::RefCell<Option<gtk::BoxLayout>>,
        last_w: Cell<i32>,
        last_h: Cell<i32>,
    }

    impl NotifyingLayout {
        pub fn configure_like(&self, old: &gtk::BoxLayout) {
            let binding = self.inner.borrow();
            let inner = binding
                .as_ref()
                .expect("inner BoxLayout must exist");

            inner.set_orientation(old.orientation());
            inner.set_spacing(old.spacing());
            inner.set_homogeneous(old.is_homogeneous());
            inner.set_baseline_position(old.baseline_position());
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotifyingLayout {
        const NAME: &'static str = "OkNotifyingLayout";
        type Type = super::NotifyingLayout;
        type ParentType = gtk::LayoutManager;
    }

    impl ObjectImpl for NotifyingLayout {
        fn constructed(&self) {
            self.parent_constructed();

            // Default inner layout; you can overwrite properties after constructing the LM.
            let inner = gtk::BoxLayout::new(gtk::Orientation::Horizontal);
            self.inner.replace(Some(inner));
        }
    }

    impl LayoutManagerImpl for NotifyingLayout {
        fn measure(
            &self,
            widget: &gtk::Widget,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let binding = self.inner.borrow();
            let inner = binding
                .as_ref()
                .expect("inner BoxLayout must exist");

            // Delegate measurement to the inner BoxLayout
            inner.measure(widget, orientation, for_size)
        }

        fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
            let binding = self.inner.borrow();
            let inner = binding
                .as_ref()
                .expect("inner BoxLayout must exist");

            // Delegate allocation to the inner BoxLayout
            inner.allocate(widget, width, height, baseline);

            // Notify on change
            if width != self.last_w.get() || height != self.last_h.get() {
                self.last_w.set(width);
                self.last_h.set(height);

                // Emit "resized" on the widget being allocated
                widget.emit_by_name::<()>("resized", &[&width, &height]);
            }
        }
    }
}

glib::wrapper! {
    pub struct NotifyingLayout(ObjectSubclass<notifying_layout::NotifyingLayout>)
        @extends gtk::LayoutManager;
}

impl NotifyingLayout {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Configure the delegated BoxLayout to match an existing BoxLayout.
    pub fn configure_like(&self, old: &gtk::BoxLayout) {
        self.imp().configure_like(old);
    }
}

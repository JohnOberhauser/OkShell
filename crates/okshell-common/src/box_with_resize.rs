use relm4::gtk;
use gtk::{glib, prelude::*, subclass::prelude::*};
use crate::notifying_layout::NotifyingLayout;

mod box_with_resize {
    use super::*;
    use std::sync::OnceLock;

    #[derive(Default)]
    pub struct BoxWithResize;

    #[glib::object_subclass]
    impl ObjectSubclass for BoxWithResize {
        const NAME: &'static str = "BoxWithResize";
        type Type = super::BoxWithResize;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for BoxWithResize {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Copy the existing BoxLayout settings into our delegating LM
            let old = obj
                .layout_manager()
                .and_downcast::<gtk::BoxLayout>()
                .expect("gtk::Box should have a BoxLayout");

            let lm = NotifyingLayout::new();
            lm.configure_like(&old);

            obj.set_layout_manager(Some(lm));
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![glib::subclass::Signal::builder("resized")
                    .param_types([i32::static_type(), i32::static_type()])
                    .build()]
            })
        }
    }

    impl WidgetImpl for BoxWithResize {}
    impl BoxImpl for BoxWithResize {}
}

glib::wrapper! {
    pub struct BoxWithResize(ObjectSubclass<box_with_resize::BoxWithResize>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl BoxWithResize {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

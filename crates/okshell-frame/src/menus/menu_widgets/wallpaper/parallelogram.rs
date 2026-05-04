use relm4::gtk::prelude::*;
use relm4::gtk::subclass::prelude::*;
use relm4::gtk::{gdk, glib, graphene, gsk};

pub(crate) const SKEW_FRACTION: f32 = 0.12;

mod imp {
    use super::*;
    use relm4::gtk;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct ParallelogramPaintable {
        pub(super) texture: RefCell<Option<gdk::Texture>>,
        pub(super) width: Cell<i32>,
        pub(super) height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ParallelogramPaintable {
        const NAME: &'static str = "ParallelogramPaintable";
        type Type = super::ParallelogramPaintable;
        type Interfaces = (gdk::Paintable,);
    }

    impl ObjectImpl for ParallelogramPaintable {}

    impl PaintableImpl for ParallelogramPaintable {
        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let Some(texture) = self.texture.borrow().clone() else {
                return;
            };

            let w = width as f32;
            let h = height as f32;
            let skew = w * SKEW_FRACTION;

            let gtk_snapshot = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();

            gtk_snapshot.push_mask(gsk::MaskMode::Alpha);

            let path_builder = gsk::PathBuilder::new();
            path_builder.move_to(skew, 0.0);
            path_builder.line_to(w, 0.0);
            path_builder.line_to(w - skew, h);
            path_builder.line_to(0.0, h);
            path_builder.close();
            let path = path_builder.to_path();

            gtk_snapshot.append_fill(&path, gsk::FillRule::Winding, &gdk::RGBA::WHITE);

            gtk_snapshot.pop();

            let tex_w = texture.intrinsic_width() as f64;
            let tex_h = texture.intrinsic_height() as f64;

            if tex_w > 0.0 && tex_h > 0.0 {
                let scale_x = width / tex_w;
                let scale_y = height / tex_h;
                let scale = scale_x.max(scale_y);

                let draw_w = tex_w * scale;
                let draw_h = tex_h * scale;
                let offset_x = (width - draw_w) / 2.0;
                let offset_y = (height - draw_h) / 2.0;

                gtk_snapshot.save();
                gtk_snapshot.translate(&graphene::Point::new(offset_x as f32, offset_y as f32));
                texture.snapshot(snapshot, draw_w, draw_h);
                gtk_snapshot.restore();
            } else {
                texture.snapshot(snapshot, width, height);
            }

            gtk_snapshot.pop();
        }

        fn intrinsic_width(&self) -> i32 {
            self.width.get()
        }

        fn intrinsic_height(&self) -> i32 {
            self.height.get()
        }

        fn intrinsic_aspect_ratio(&self) -> f64 {
            let w = self.width.get();
            let h = self.height.get();
            if h > 0 { w as f64 / h as f64 } else { 0.0 }
        }
    }
}

glib::wrapper! {
    pub struct ParallelogramPaintable(ObjectSubclass<imp::ParallelogramPaintable>)
        @implements gdk::Paintable;
}

impl ParallelogramPaintable {
    pub fn new(width: i32, height: i32) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().width.set(width);
        obj.imp().height.set(height);
        obj
    }

    pub fn set_texture(&self, texture: Option<&gdk::Texture>) {
        let imp = self.imp();
        *imp.texture.borrow_mut() = texture.cloned();
        self.invalidate_contents();
    }
}

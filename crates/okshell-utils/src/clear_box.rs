use relm4::gtk::{
    self,
    prelude::{BoxExt, WidgetExt},
};

pub fn clear_box(b: &gtk::Box) {
    while let Some(child) = b.first_child() {
        b.remove(&child);
        child.unparent(); // optional; remove already unparents, but harmless
    }
}
use std::any::Any;
use relm4::{
    gtk::{
        self,
    }
};
use crate::dynamic_box::generic_widget_controller::GenericWidgetController;

pub struct SimpleWidgetController {
    root: gtk::Widget,
}
impl SimpleWidgetController {
    pub fn new(root: gtk::Widget) -> Self { Self { root } }
}
impl GenericWidgetController for SimpleWidgetController {
    fn root_widget(&self) -> gtk::Widget { self.root.clone() }
    fn as_any(&self) -> &dyn Any { self }
}
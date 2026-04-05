use std::any::Any;
use relm4::{gtk::{
    self,
    prelude::*,
}, Component, Controller, ComponentController};

pub trait GenericWidgetController {
    fn root_widget(&self) -> gtk::Widget;
    fn as_any(&self) -> &dyn Any;
}

impl<T> GenericWidgetController for Controller<T>
where
    T: Component + 'static,
    T::Root: IsA<gtk::Widget>,
{
    fn root_widget(&self) -> gtk::Widget {
        self.widget().clone().upcast::<gtk::Widget>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait GenericWidgetControllerExtSafe {
    fn downcast_ref<T: 'static>(&self) -> Option<&T>;
}

impl GenericWidgetControllerExtSafe for dyn GenericWidgetController {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}
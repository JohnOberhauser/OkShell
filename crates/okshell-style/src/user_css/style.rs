use reactive_stores::{Patch, Store};

#[derive(Debug, Clone, PartialEq, Eq, Store, Patch, Default)]
pub struct Style {
    pub css: String,
}

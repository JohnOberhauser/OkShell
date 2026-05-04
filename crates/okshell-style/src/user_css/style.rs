use reactive_stores::{Patch, Store};

#[derive(Debug, Default, Clone, PartialEq, Eq, Store, Patch)]
pub struct Style {
    pub css: String,
}

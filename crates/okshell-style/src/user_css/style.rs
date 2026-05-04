use reactive_stores::{Patch, Store};

#[derive(Debug, Clone, PartialEq, Eq, Store, Patch)]
pub struct Style {
    pub css: String,
}

impl Default for Style {
    fn default() -> Self {
        Self { css: String::new() }
    }
}

use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum ContentFit {
    Contain,
    Cover,
    Fill,
    ScaleDown,
}

impl PatchField for ContentFit {
    fn patch_field(
        &mut self,
        new: Self,
        path: &StorePath,
        notify: &mut dyn FnMut(&StorePath),
        _keys: Option<&KeyMap>,
    ) {
        if *self != new {
            *self = new;
            notify(path);
        }
    }
}

impl ContentFit {
    pub fn to_index(&self) -> u32 {
        match self {
            ContentFit::Contain => 0,
            ContentFit::Cover => 1,
            ContentFit::Fill => 2,
            ContentFit::ScaleDown => 3,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => ContentFit::Contain,
            1 => ContentFit::Cover,
            2 => ContentFit::Fill,
            _ => ContentFit::ScaleDown,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ContentFit::Contain => "Contain",
            ContentFit::Cover => "Cover",
            ContentFit::Fill => "Fill",
            ContentFit::ScaleDown => "Scale Down",
        }
    }

    pub fn display_names() -> Vec<&'static str> {
        Self::all().iter().map(|p| p.display_name()).collect()
    }

    pub fn all() -> &'static [ContentFit] {
        &[
            ContentFit::Contain,
            ContentFit::Cover,
            ContentFit::Fill,
            ContentFit::ScaleDown,
        ]
    }
}
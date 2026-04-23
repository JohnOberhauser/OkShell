use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Store, JsonSchema)]
pub struct ThemeFilterStrength(f64);

impl ThemeFilterStrength {
    pub fn new(v: f64) -> Self {
        Self(v.clamp(0.0, 1.0))
    }
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl PatchField for ThemeFilterStrength {
    fn patch_field(
        &mut self,
        new: Self,
        path: &StorePath,
        notify: &mut dyn FnMut(&StorePath),
        _keys: Option<&KeyMap>,
    ) {
        let clamped = ThemeFilterStrength::new(new.0);
        if self.0 != clamped.0 {
            *self = clamped;
            notify(path);
        }
    }
}

impl PartialEq for ThemeFilterStrength {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for ThemeFilterStrength {}

#[derive(Clone, Debug, Serialize, Deserialize, Store, JsonSchema)]
pub struct ContrastFilterStrength(f64);

impl ContrastFilterStrength {
    pub fn new(v: f64) -> Self {
        Self(v.clamp(0.0, 2.0))
    }
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl PatchField for ContrastFilterStrength {
    fn patch_field(
        &mut self,
        new: Self,
        path: &StorePath,
        notify: &mut dyn FnMut(&StorePath),
        _keys: Option<&KeyMap>,
    ) {
        let clamped = ContrastFilterStrength::new(new.0);
        if self.0 != clamped.0 {
            *self = clamped;
            notify(path);
        }
    }
}

impl PartialEq for ContrastFilterStrength {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for ContrastFilterStrength {}
use std::hash::{Hash, Hasher};
use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use serde::{Deserialize, Serialize};
use wayle_weather::LocationQuery;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
#[serde(tag = "type")]
pub enum LocationQueryConfig {
    Coordinates { lat: OrdF64, lon: OrdF64 },
    City { name: String, country: String },
}

impl From<LocationQueryConfig> for LocationQuery {
    fn from(c: LocationQueryConfig) -> Self {
        match c {
            LocationQueryConfig::Coordinates { lat, lon } => {
                LocationQuery::Coordinates { lat: lat.0, lon: lon.0 }
            }
            LocationQueryConfig::City { name, country } => {
                LocationQuery::City { name, country: Some(country) }
            }
        }
    }
}

impl PatchField for LocationQueryConfig {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationQueryType {
    Coordinates,
    City,
}

impl LocationQueryType {
    pub fn all() -> &'static [Self] {
        &[Self::Coordinates, Self::City]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Coordinates => "Coordinates",
            Self::City => "City",
        }
    }
}

impl LocationQueryConfig {
    pub fn kind(&self) -> LocationQueryType {
        match self {
            Self::Coordinates { .. } => LocationQueryType::Coordinates,
            Self::City { .. } => LocationQueryType::City,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrdF64(pub f64);

impl PartialEq for OrdF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrdF64 {}

impl Hash for OrdF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Default for OrdF64 {
    fn default() -> Self {
        Self(0.0)
    }
}
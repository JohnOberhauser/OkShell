use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use wayle_weather::TemperatureUnit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Store, JsonSchema)]
#[serde()]
pub enum TemperatureUnitConfig {
    #[default]
    Metric,
    Imperial,
}

impl PatchField for TemperatureUnitConfig {
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

impl From<TemperatureUnitConfig> for TemperatureUnit {
    fn from(u: TemperatureUnitConfig) -> Self {
        match u {
            TemperatureUnitConfig::Metric => TemperatureUnit::Metric,
            TemperatureUnitConfig::Imperial => TemperatureUnit::Imperial,
        }
    }
}

impl TemperatureUnitConfig {
    pub fn all() -> &'static [Self] {
        &[
            Self::Metric,
            Self::Imperial,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Metric => "Metric",
            Self::Imperial => "Imperial",
        }
    }
}
use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Store, JsonSchema)]
#[serde()]
pub enum QuickSettingsIcon {
    #[default]
    Arch,
    Nix,
    Fedora,
    Hyprland,
}

impl PatchField for QuickSettingsIcon {
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

impl QuickSettingsIcon {
    pub fn display_name(&self) -> &'static str {
        match self {
            QuickSettingsIcon::Arch => "Arch",
            QuickSettingsIcon::Fedora => "Fedora",
            QuickSettingsIcon::Hyprland => "Hyprland",
            QuickSettingsIcon::Nix => "Nix",
        }
    }

    pub fn display_names() -> Vec<&'static str> {
        Self::all().iter().map(|p| p.display_name()).collect()
    }

    pub fn all() -> &'static [QuickSettingsIcon] {
        &[
            QuickSettingsIcon::Arch,
            QuickSettingsIcon::Fedora,
            QuickSettingsIcon::Hyprland,
            QuickSettingsIcon::Nix,
        ]
    }

    pub fn to_index(&self) -> u32 {
        match self {
            QuickSettingsIcon::Arch => 0,
            QuickSettingsIcon::Fedora => 1,
            QuickSettingsIcon::Hyprland => 2,
            QuickSettingsIcon::Nix => 3,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => QuickSettingsIcon::Arch,
            1 => QuickSettingsIcon::Fedora,
            2 => QuickSettingsIcon::Hyprland,
            _ => QuickSettingsIcon::Nix,
        }
    }
}

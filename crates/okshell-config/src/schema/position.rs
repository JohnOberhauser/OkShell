use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

impl PatchField for Position {
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

impl Position {
    pub fn to_index(&self) -> u32 {
        match self {
            Position::Left => 0,
            Position::Right => 1,
            Position::Top => 2,
            Position::Bottom => 3,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => Position::Left,
            1 => Position::Right,
            2 => Position::Top,
            _ => Position::Bottom,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Position::Left => "Left",
            Position::Right => "Right",
            Position::Top => "Top",
            Position::Bottom => "Bottom",
        }
    }

    pub fn display_names() -> Vec<&'static str> {
        Self::all().iter().map(|p| p.display_name()).collect()
    }

    pub fn all() -> &'static [Position] {
        &[
            Position::Left,
            Position::Right,
            Position::Top,
            Position::Bottom,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum NotificationPosition {
    Left,
    Right,
    Center,
}

impl PatchField for NotificationPosition {
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

impl NotificationPosition {
    pub fn to_index(&self) -> u32 {
        match self {
            NotificationPosition::Left => 0,
            NotificationPosition::Right => 1,
            NotificationPosition::Center => 2,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => NotificationPosition::Left,
            1 => NotificationPosition::Right,
            _ => NotificationPosition::Center,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            NotificationPosition::Left => "Left",
            NotificationPosition::Right => "Right",
            NotificationPosition::Center => "Center",
        }
    }

    pub fn display_names() -> Vec<&'static str> {
        Self::all().iter().map(|p| p.display_name()).collect()
    }

    pub fn all() -> &'static [NotificationPosition] {
        &[
            NotificationPosition::Left,
            NotificationPosition::Right,
            NotificationPosition::Center,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl PatchField for Orientation {
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
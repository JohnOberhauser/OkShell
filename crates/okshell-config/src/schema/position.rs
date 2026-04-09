use reactive_stores::{KeyMap, PatchField, Store, StorePath};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store, JsonSchema)]
pub enum Position {
    Left,
    Right,
    Top,
    TopLeft,
    TopRight,
    Bottom,
    BottomLeft,
    BottomRight,
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
            Position::TopLeft => 3,
            Position::TopRight => 4,
            Position::Bottom => 5,
            Position::BottomLeft => 6,
            Position::BottomRight => 7,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => Position::Left,
            1 => Position::Right,
            2 => Position::Top,
            3 => Position::TopLeft,
            4 => Position::TopRight,
            5 => Position::Bottom,
            6 => Position::BottomLeft,
            _ => Position::BottomRight,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Position::Left => "Left",
            Position::Right => "Right",
            Position::Top => "Top",
            Position::TopLeft => "Top Left",
            Position::TopRight => "Top Right",
            Position::Bottom => "Bottom",
            Position::BottomLeft => "Bottom Left",
            Position::BottomRight => "Bottom Right",
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
            Position::TopLeft,
            Position::TopRight,
            Position::Bottom,
            Position::BottomLeft,
            Position::BottomRight,
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
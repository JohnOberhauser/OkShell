use std::env::home_dir;
use std::path::PathBuf;

pub(crate) fn styles_dir() -> PathBuf {
    home_dir()
        .expect("HOME not set")
        .join(".config/okshell/styles")
}

pub(crate) fn style_path(name: &str) -> PathBuf {
    styles_dir().join(format!("{name}.css"))
}

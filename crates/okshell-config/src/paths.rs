use dirs::home_dir;
use std::path::PathBuf;

pub(crate) const DEFAULT_PROFILE_NAME: &str = "default";

pub(crate) fn default_config_path() -> PathBuf {
    home_dir().expect("HOME not set").join(format!(
        ".config/okshell/profiles/{}.yaml",
        DEFAULT_PROFILE_NAME
    ))
}

pub(crate) fn profiles_dir() -> PathBuf {
    home_dir()
        .expect("HOME not set")
        .join(".config/okshell/profiles")
}

pub(crate) fn active_profile_cache_path() -> PathBuf {
    home_dir()
        .expect("HOME not set")
        .join(".cache/okshell/active_profile")
}

pub(crate) fn profile_path(name: &str) -> PathBuf {
    profiles_dir().join(format!("{name}.yaml"))
}

#[derive(Debug)]
pub enum ProfileDeleteError {
    NotFound,
    Io(std::io::Error),
}

impl std::fmt::Display for ProfileDeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "profile not found"),
            Self::Io(e) => write!(f, "failed to delete profile: {e}"),
        }
    }
}

impl std::error::Error for ProfileDeleteError {}

#[derive(Debug)]
pub enum ProfileCreateError {
    AlreadyExists,
    Io(Box<dyn std::error::Error>),
}

impl std::fmt::Display for ProfileCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExists => write!(f, "profile already exists"),
            Self::Io(e) => write!(f, "failed to create profile: {e}"),
        }
    }
}

impl std::error::Error for ProfileCreateError {}
mod ramp;
mod service;
mod wayland;

pub use service::GammaService;
pub use wayland::GammaManager;

use std::sync::OnceLock;

pub const TEMP_NEUTRAL: u32 = 6600;
pub const TEMP_MIN: u32 = 1000;
pub const TEMP_MAX: u32 = 10000;

#[derive(Debug, Clone, PartialEq)]
pub struct GammaState {
    pub enabled: bool,
    pub night_temp: u32,
}

impl Default for GammaState {
    fn default() -> Self {
        Self {
            enabled: false,
            night_temp: 5500,
        }
    }
}

static GAMMA: OnceLock<GammaService> = OnceLock::new();

pub fn gamma_service() -> &'static GammaService {
    GAMMA.get_or_init(|| GammaService::start().expect("failed to start gamma service"))
}

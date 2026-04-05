use std::sync::Arc;

use anyhow::Result;
use tokio::sync::watch;

use crate::{GammaState, wayland::GammaManager, TEMP_NEUTRAL};

const TRANSITION_MS: u64 = 500;
/// Interval between ramp updates — ~60fps.
const STEP_MS: u64 = 16;
/// Number of steps over the full transition.
const STEPS: u64 = TRANSITION_MS / STEP_MS;

#[derive(Clone)]
pub struct GammaService {
    inner: Arc<Inner>,
}

struct Inner {
    tx: watch::Sender<GammaState>,
}

impl GammaService {
    pub fn start() -> Result<Self> {
        let initial = GammaState::default();
        let (tx, rx) = watch::channel(initial);

        std::thread::Builder::new()
            .name("okshell-gamma".into())
            .spawn(move || {
                if let Err(e) = run_wayland_thread(rx) {
                    eprintln!("okshell-gamma: wayland thread exited: {e:#}");
                }
            })?;

        Ok(Self { inner: Arc::new(Inner { tx }) })
    }

    pub fn state(&self) -> GammaState {
        self.inner.tx.borrow().clone()
    }

    pub fn enabled(&self) -> bool {
        self.inner.tx.borrow().enabled
    }

    pub fn night_temp(&self) -> u32 {
        self.inner.tx.borrow().night_temp
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.inner.tx.send_modify(|s| s.enabled = enabled);
    }

    pub fn set_night_temp(&self, temp_k: u32) {
        let clamped = temp_k.clamp(crate::TEMP_MIN, crate::TEMP_MAX);
        self.inner.tx.send_modify(|s| s.night_temp = clamped);
    }

    // ── Observation ───────────────────────────────────────────────────────────

    /// Subscribe to state changes.
    ///
    /// The returned receiver yields whenever `enabled` or `night_temp` changes.
    /// Use `receiver.changed().await` in async contexts.
    ///
    /// # GTK / Relm4 usage
    ///
    /// ```no_run
    /// # use okshell_gamma::gamma_service;
    /// let mut rx = gamma_service().subscribe();
    /// glib::spawn_future_local(async move {
    ///     while rx.changed().await.is_ok() {
    ///         let state = rx.borrow_and_update().clone();
    ///         // update your toggle / slider here
    ///     }
    /// });
    /// ```
    pub fn subscribe(&self) -> watch::Receiver<GammaState> {
        self.inner.tx.subscribe()
    }
}

fn run_wayland_thread(mut rx: watch::Receiver<GammaState>) -> Result<()> {
    let mut mgr = GammaManager::connect()?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()?;

    // The temperature currently applied to the compositor.
    let initial = rx.borrow_and_update().clone();
    let start_temp = target_temp(&initial);
    mgr.apply_temp(start_temp)?;
    let mut current_temp = start_temp;

    loop {
        // Wait for a state change.
        match rt.block_on(rx.changed()) {
            Err(_) => break, // sender dropped — shut down
            Ok(()) => {}
        }

        // Transition loop: step current_temp toward target, 16ms per step.
        // Exits early if a new change arrives (rx.has_changed()), so the
        // outer loop can restart from the updated current_temp.
        loop {
            let state = rx.borrow_and_update().clone();
            let target = target_temp(&state);

            if current_temp == target {
                // Already there, nothing to do.
                break;
            }

            for step in 1..=STEPS {
                let t = step as f64 / STEPS as f64;
                let interpolated = lerp(current_temp, target, t);
                if let Err(e) = mgr.apply_temp(interpolated) {
                    eprintln!("okshell-gamma: apply failed: {e:#}");
                }
                current_temp = interpolated;

                std::thread::sleep(std::time::Duration::from_millis(STEP_MS));
                if rx.has_changed().unwrap_or(false) {
                    break;
                }
            }

            // If no new change arrived, we're done.
            if !rx.has_changed().unwrap_or(false) {
                // Snap to exact target to avoid any rounding drift.
                if current_temp != target {
                    let _ = mgr.apply_temp(target);
                    current_temp = target;
                }
                break;
            }
        }
    }

    Ok(())
}

fn target_temp(state: &GammaState) -> u32 {
    if state.enabled { state.night_temp } else { TEMP_NEUTRAL }
}

/// Linear interpolation between two Kelvin values.
fn lerp(from: u32, to: u32, t: f64) -> u32 {
    (from as f64 + (to as f64 - from as f64) * t).round() as u32
}
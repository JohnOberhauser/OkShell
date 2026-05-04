use std::os::fd::{AsFd, FromRawFd, OwnedFd};

use anyhow::{Context, Result, bail};
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, delegate_noop,
    protocol::{wl_output, wl_registry},
};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1, zwlr_gamma_control_v1,
};

use crate::{GammaState, TEMP_NEUTRAL, ramp::build_ramp};

struct Output {
    wl: wl_output::WlOutput,
    ctrl: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
    ramp_size: usize,
}

struct AppData {
    manager: Option<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1>,
    outputs: Vec<Output>,
}

impl AppData {
    fn new() -> Self {
        Self {
            manager: None,
            outputs: Vec::new(),
        }
    }
}

// ── Dispatch: registry ────────────────────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };

        match interface.as_str() {
            "zwlr_gamma_control_manager_v1" => {
                let mgr = registry
                    .bind::<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    );
                state.manager = Some(mgr);
            }
            "wl_output" => {
                let wl = registry.bind::<wl_output::WlOutput, _, _>(name, version.min(4), qh, ());
                state.outputs.push(Output {
                    wl,
                    ctrl: None,
                    ramp_size: 0,
                });
            }
            _ => {}
        }
    }
}

// ── Dispatch: gamma control manager (no events) ───────────────────────────────

impl Dispatch<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
        _: zwlr_gamma_control_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

// ── Dispatch: per-output gamma control ───────────────────────────────────────

impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, usize> for AppData {
    fn event(
        state: &mut Self,
        _: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        event: zwlr_gamma_control_v1::Event,
        idx: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_gamma_control_v1::Event::GammaSize { size } => {
                if let Some(o) = state.outputs.get_mut(*idx) {
                    o.ramp_size = size as usize;
                }
            }
            zwlr_gamma_control_v1::Event::Failed => {
                eprintln!("okshell-gamma: compositor rejected gamma control for output {idx}");
            }
            _ => {}
        }
    }
}

// wl_output: we don't need any events (name is nice-to-have but not required).
delegate_noop!(AppData: ignore wl_output::WlOutput);

pub struct GammaManager {
    _conn: Connection,
    queue: EventQueue<AppData>,
    data: AppData,
}

impl GammaManager {
    /// Connect to the Wayland display and prepare gamma controls for all outputs.
    ///
    /// Returns an error if the compositor does not advertise
    /// `zwlr_gamma_control_manager_v1`
    pub fn connect() -> Result<Self> {
        let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
        let mut queue = conn.new_event_queue();
        let qh = queue.handle();

        conn.display().get_registry(&qh, ());

        let mut data = AppData::new();

        // Roundtrip 1: discover globals.
        queue
            .roundtrip(&mut data)
            .context("Wayland roundtrip 1 failed")?;

        let Some(ref mgr) = data.manager else {
            bail!(
                "compositor does not support zwlr_gamma_control_manager_v1\n\
                 (this protocol is available on wlroots compositors like Hyprland and Sway,\n\
                  but not on GNOME or KDE which manage gamma internally)"
            );
        };

        // Bind a gamma control for each output found in roundtrip 1.
        for (idx, output) in data.outputs.iter_mut().enumerate() {
            let ctrl = mgr.get_gamma_control(&output.wl, &qh, idx);
            output.ctrl = Some(ctrl);
        }

        // Roundtrip 2: collect gamma_size events.
        queue
            .roundtrip(&mut data)
            .context("Wayland roundtrip 2 failed")?;

        Ok(Self {
            _conn: conn,
            queue,
            data,
        })
    }

    /// Apply `state` to every output.
    pub fn apply(&mut self, state: &GammaState) -> Result<()> {
        let temp = if state.enabled {
            state.night_temp
        } else {
            TEMP_NEUTRAL
        };

        // Keep all fds alive in this Vec until after the roundtrip.
        // set_gamma() only queues the request; the compositor reads the fd
        // during the flush/roundtrip, so closing early causes a size mismatch.
        let mut _fds: Vec<OwnedFd> = Vec::new();

        for output in &self.data.outputs {
            if output.ramp_size == 0 {
                // gamma_size not yet received — skip; shouldn't happen after connect().
                continue;
            }
            let Some(ref ctrl) = output.ctrl else {
                continue;
            };

            let ramp = build_ramp(temp, 1.0, output.ramp_size);
            let fd = ramp_to_memfd(&ramp).context("failed to create memfd for gamma ramp")?;
            ctrl.set_gamma(fd.as_fd());
            _fds.push(fd);
        }

        self.queue
            .roundtrip(&mut self.data)
            .context("Wayland flush after set_gamma failed")?;

        // _fds drops here, after the roundtrip has completed.
        Ok(())
    }

    /// Apply a raw temperature in Kelvin directly to all outputs.
    pub fn apply_temp(&mut self, temp_k: u32) -> Result<()> {
        let mut _fds: Vec<OwnedFd> = Vec::new();

        for output in &self.data.outputs {
            if output.ramp_size == 0 {
                continue;
            }
            let Some(ref ctrl) = output.ctrl else {
                continue;
            };

            let ramp = build_ramp(temp_k, 1.0, output.ramp_size);
            let fd = ramp_to_memfd(&ramp).context("failed to create memfd for gamma ramp")?;
            ctrl.set_gamma(fd.as_fd());
            _fds.push(fd);
        }

        self.queue
            .roundtrip(&mut self.data)
            .context("Wayland flush after set_gamma failed")?;

        Ok(())
    }
}

// ── fd helper ────────────────────────────────────────────────────────────────

/// Write `ramp` into an anonymous temp file and return its fd.
///
/// mkstemp → ftruncate to exact byte size → mmap → write → munmap.
///
/// The compositor validates the fd via fstat against
/// `ramp_size * 3 * sizeof(u16)`. ftruncate is what sets that size;
/// a plain write_all does not, which is why the previous memfd approach
/// produced a size mismatch error.
fn ramp_to_memfd(ramp: &[u16]) -> Result<OwnedFd> {
    let byte_len = std::mem::size_of_val(ramp);

    // mkstemp + immediate unlink: anonymous temp file, cleaned up on close.
    let raw_fd = unsafe {
        let mut template: Vec<u8> = b"/tmp/okshell-gamma-XXXXXX".to_vec();
        template.push(0u8);
        let fd = libc::mkstemp(template.as_mut_ptr() as *mut libc::c_char);
        if fd < 0 {
            return Err(std::io::Error::last_os_error()).context("mkstemp");
        }
        libc::unlink(template.as_ptr() as *const libc::c_char);
        fd
    };

    // ftruncate to the exact expected size — this is what the compositor checks.
    if unsafe { libc::ftruncate(raw_fd, byte_len as libc::off_t) } < 0 {
        unsafe { libc::close(raw_fd) };
        return Err(std::io::Error::last_os_error()).context("ftruncate");
    }

    // mmap and copy the ramp data in.
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            byte_len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            raw_fd,
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        unsafe { libc::close(raw_fd) };
        return Err(std::io::Error::last_os_error()).context("mmap");
    }

    unsafe {
        std::ptr::copy_nonoverlapping(ramp.as_ptr() as *const u8, ptr as *mut u8, byte_len);
        libc::munmap(ptr, byte_len);
    }

    Ok(unsafe { OwnedFd::from_raw_fd(raw_fd) })
}

use std::collections::HashMap;
use std::os::fd::AsFd;
use std::sync::{Arc, Mutex};

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};
use crate::selectors::area_selector::RegionSelection;
use crate::utils::query_outputs;
use super::{HyprlandWindow, ScreenshotError, Result};

#[derive(Debug, Clone)]
pub struct ScreenshotResult {
    pub saved_path: Option<std::path::PathBuf>,
    pub in_clipboard: bool,
}

/// Low-level capture backend wrapping wlr-screencopy.
pub struct CaptureBackend;

/// Per-frame capture state driven by wlr-screencopy events.
#[derive(Debug)]
struct FrameState {
    /// Format info received from the compositor.
    format: Option<FrameFormat>,
    /// Whether the frame is ready to copy.
    ready: bool,
    /// Whether the copy completed.
    done: bool,
    /// Whether the copy failed.
    failed: bool,
}

#[derive(Debug, Clone)]
struct FrameFormat {
    format: wl_shm::Format,
    width: u32,
    height: u32,
    stride: u32,
}

/// Aggregated state for the wayland event loop.
struct CaptureState {
    outputs: HashMap<String, wl_output::WlOutput>,
    _shm: Option<wl_shm::WlShm>,
    _screencopy_mgr: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    frame_state: Arc<Mutex<FrameState>>,
}

impl CaptureBackend {
    pub fn new() -> Result<Self> {
        // Quick check that we can connect to wayland.
        Connection::connect_to_env()
            .map_err(|e| ScreenshotError::WaylandConnect(e.to_string()))?;
        Ok(Self)
    }

    /// Capture all outputs and stitch them together.
    pub fn capture_all(&self) -> Result<image::RgbaImage> {
        let outputs = query_outputs()?;
        if outputs.is_empty() {
            return Err(ScreenshotError::CaptureFailed("no outputs found".into()));
        }

        // Determine bounding box across all outputs.
        let min_x = outputs.iter().map(|o| o.x).min().unwrap();
        let min_y = outputs.iter().map(|o| o.y).min().unwrap();
        let max_x = outputs.iter().map(|o| o.x + o.width).max().unwrap();
        let max_y = outputs.iter().map(|o| o.y + o.height).max().unwrap();
        let total_w = (max_x - min_x) as u32;
        let total_h = (max_y - min_y) as u32;

        let mut canvas = image::RgbaImage::new(total_w, total_h);

        for output_info in &outputs {
            let frame = self.capture_output_raw(&output_info.name)?;
            let offset_x = (output_info.x - min_x) as u32;
            let offset_y = (output_info.y - min_y) as u32;
            image::imageops::overlay(&mut canvas, &frame, offset_x as i64, offset_y as i64);
        }

        Ok(canvas)
    }

    /// Capture a single output by name.
    pub fn capture_output(&self, name: &str) -> Result<image::RgbaImage> {
        self.capture_output_raw(name)
    }

    /// Capture a window by cropping the relevant output.
    pub fn capture_window(&self, win: &HyprlandWindow) -> Result<image::RgbaImage> {
        let outputs = query_outputs()?;
        let output_info = outputs
            .iter()
            .find(|o| o.name == win.output)
            .ok_or_else(|| ScreenshotError::OutputNotFound(win.output.clone()))?;

        let full = self.capture_output_raw(&win.output)?;

        // Window coords are global; convert to output-local.
        let local_x = (win.x - output_info.x).max(0) as u32;
        let local_y = (win.y - output_info.y).max(0) as u32;
        let w = (win.width as u32).min(full.width().saturating_sub(local_x));
        let h = (win.height as u32).min(full.height().saturating_sub(local_y));

        if w == 0 || h == 0 {
            return Err(ScreenshotError::CaptureFailed("window has zero size".into()));
        }

        Ok(image::imageops::crop_imm(&full, local_x, local_y, w, h).to_image())
    }

    /// Capture a user-selected region by cropping the relevant output.
    pub fn capture_region(&self, region: &RegionSelection) -> Result<image::RgbaImage> {
        let full = self.capture_output_raw(&region.output)?;

        let x = (region.x as u32).min(full.width());
        let y = (region.y as u32).min(full.height());
        let w = (region.width as u32).min(full.width().saturating_sub(x));
        let h = (region.height as u32).min(full.height().saturating_sub(y));

        if w == 0 || h == 0 {
            return Err(ScreenshotError::CaptureFailed("region has zero size".into()));
        }

        Ok(image::imageops::crop_imm(&full, x, y, w, h).to_image())
    }

    /// Core capture logic for a single output using wlr-screencopy.
    fn capture_output_raw(&self, output_name: &str) -> Result<image::RgbaImage> {
        // Fresh connection per capture to avoid stale state when
        // capturing multiple outputs sequentially.
        let conn = Connection::connect_to_env()
            .map_err(|e| ScreenshotError::WaylandConnect(e.to_string()))?;

        let (globals, mut queue) = registry_queue_init::<CaptureState>(&conn)
            .map_err(|e| ScreenshotError::WaylandConnect(e.to_string()))?;

        let qh = queue.handle();

        // Bind globals.
        let shm: wl_shm::WlShm = globals
            .bind(&qh, 1..=1, ())
            .map_err(|_| ScreenshotError::ProtocolNotSupported)?;

        let screencopy_mgr: zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1 = globals
            .bind(&qh, 3..=3, ())
            .map_err(|_| ScreenshotError::ProtocolNotSupported)?;

        // Collect outputs.
        let mut state = CaptureState {
            outputs: HashMap::new(),
            _shm: Some(shm.clone()),
            _screencopy_mgr: Some(screencopy_mgr.clone()),
            frame_state: Arc::new(Mutex::new(FrameState {
                format: None,
                ready: false,
                done: false,
                failed: false,
            })),
        };

        // We need a roundtrip to discover outputs.
        // Bind all wl_output globals.
        let output_list = globals.contents().clone_list();
        for global in output_list.iter() {
            if global.interface == "wl_output" {
                let _: wl_output::WlOutput = globals
                    .registry()
                    .bind(global.name, global.version.min(4), &qh, global.name);
            }
        }

        // Roundtrip to receive output name events.
        queue
            .roundtrip(&mut state)
            .map_err(|e| ScreenshotError::CaptureFailed(e.to_string()))?;

        let wl_output = state
            .outputs
            .get(output_name)
            .cloned()
            .ok_or_else(|| ScreenshotError::OutputNotFound(output_name.to_string()))?;

        // Request a screencopy frame.
        let frame = screencopy_mgr.capture_output(0, &wl_output, &qh, ());

        // Roundtrip to receive buffer format info.
        queue
            .roundtrip(&mut state)
            .map_err(|e| ScreenshotError::CaptureFailed(e.to_string()))?;

        let fmt = {
            let fs = state.frame_state.lock().unwrap();
            fs.format
                .clone()
                .ok_or_else(|| ScreenshotError::CaptureFailed("no format received".into()))?
        };

        // Create shm buffer.
        let buf_size = (fmt.stride * fmt.height) as usize;
        let file = create_shm_file(buf_size)?;
        let pool = shm.create_pool(file.as_fd(), buf_size as i32, &qh, ());
        let buffer = pool.create_buffer(
            0,
            fmt.width as i32,
            fmt.height as i32,
            fmt.stride as i32,
            fmt.format,
            &qh,
            (),
        );

        // Tell screencopy to copy into our buffer.
        frame.copy(&buffer);

        // Pump events until done or failed.
        loop {
            queue
                .blocking_dispatch(&mut state)
                .map_err(|e| ScreenshotError::CaptureFailed(e.to_string()))?;

            let fs = state.frame_state.lock().unwrap();
            if fs.failed {
                return Err(ScreenshotError::CaptureFailed("compositor rejected copy".into()));
            }
            if fs.done {
                break;
            }
        }

        // Read pixels from shared memory.
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .len(buf_size)
                .map(&file)
                .map_err(|e| ScreenshotError::CaptureFailed(e.to_string()))?
        };

        let image = convert_to_rgba(&mmap, fmt.width, fmt.height, fmt.stride, fmt.format)?;

        // Cleanup protocol objects.
        buffer.destroy();
        pool.destroy();
        frame.destroy();
        screencopy_mgr.destroy();

        Ok(image)
    }
}

/// Convert raw pixel data from the compositor's format to RGBA8.
fn convert_to_rgba(
    data: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    format: wl_shm::Format,
) -> Result<image::RgbaImage> {
    let mut img = image::RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x * 4) as usize;
            if offset + 3 >= data.len() {
                break;
            }
            let pixel = match format {
                // ARGB8888 — most common from compositors.
                wl_shm::Format::Argb8888 => {
                    let b = data[offset];
                    let g = data[offset + 1];
                    let r = data[offset + 2];
                    let a = data[offset + 3];
                    image::Rgba([r, g, b, a])
                }
                // XRGB8888 — like ARGB but alpha is unused (opaque).
                wl_shm::Format::Xrgb8888 => {
                    let b = data[offset];
                    let g = data[offset + 1];
                    let r = data[offset + 2];
                    image::Rgba([r, g, b, 255])
                }
                // ABGR8888
                wl_shm::Format::Abgr8888 => {
                    let r = data[offset];
                    let g = data[offset + 1];
                    let b = data[offset + 2];
                    let a = data[offset + 3];
                    image::Rgba([r, g, b, a])
                }
                // XBGR8888
                wl_shm::Format::Xbgr8888 => {
                    let r = data[offset];
                    let g = data[offset + 1];
                    let b = data[offset + 2];
                    image::Rgba([r, g, b, 255])
                }
                other => {
                    return Err(ScreenshotError::CaptureFailed(format!(
                        "unsupported shm format: {:?}",
                        other
                    )));
                }
            };
            img.put_pixel(x, y, pixel);
        }
    }

    Ok(img)
}

/// Create an anonymous shared memory file for the wl_shm_pool.
fn create_shm_file(size: usize) -> Result<std::fs::File> {
    use rustix::shm;

    let name = format!("/okshell-screenshot-{}", std::process::id());
    let fd = shm::open(
        &name,
        shm::OFlags::CREATE | shm::OFlags::EXCL | shm::OFlags::RDWR,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
        .map_err(|e| ScreenshotError::CaptureFailed(format!("shm_open: {e}")))?;

    // Immediately unlink so it's cleaned up.
    let _ = shm::unlink(&name);

    rustix::fs::ftruncate(&fd, size as u64)
        .map_err(|e| ScreenshotError::CaptureFailed(format!("ftruncate: {e}")))?;

    Ok(std::fs::File::from(fd))
}

// ── Wayland dispatch implementations ──────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Handled by GlobalListContents
    }
}

impl Dispatch<wl_shm::WlShm, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, u32> for CaptureState {
    fn event(
        state: &mut Self,
        proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        _data: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name } = event {
            state.outputs.insert(name, proxy.clone());
        }
    }
}

impl Dispatch<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, ()> for CaptureState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        _event: zwlr_screencopy_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, ()> for CaptureState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let mut fs = state.frame_state.lock().unwrap();
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                fs.format = Some(FrameFormat {
                    format: format.into_result().unwrap_or(wl_shm::Format::Argb8888),
                    width,
                    height,
                    stride,
                });
            }
            zwlr_screencopy_frame_v1::Event::BufferDone => {
                fs.ready = true;
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                fs.done = true;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                fs.failed = true;
            }
            _ => {}
        }
    }
}
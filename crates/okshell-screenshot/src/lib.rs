mod capture;
mod common;
pub mod record;
mod selectors;
mod utils;

pub use capture::{CaptureBackend, ScreenshotResult};
pub use selectors::area_selector::select_region;

use crate::common::*;
use crate::record::{RecordHandle, RecordResult, WfRecorderArgs, start_recording};
use crate::utils::{default_screenshot_path, query_outputs};
use gtk4::glib;
use image::ImageEncoder;
use selectors::area_selector::RegionSelection;
use selectors::{monitor_selector, window_selector};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum CaptureArea {
    All,
    SelectRegion,
    SelectMonitor,
    SelectWindow,
}

#[derive(Debug, Clone)]
pub enum OutputTarget {
    FileAndClipboard,
    File,
    Clipboard,
}

#[derive(Debug, Clone)]
pub struct ScreenshotRequest {
    pub area: CaptureArea,
    pub target: OutputTarget,
}

#[derive(Debug, Clone)]
pub struct ScreenRecordRequest {
    pub area: CaptureArea,
    pub audio: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ScreenSelectAreaRequest {
    SelectRegion,
    SelectMonitor,
}

#[derive(Debug, Clone)]
pub enum ScreenSelection {
    Region(RegionSelection),
    Monitor(String),
}

pub fn select_screen<F>(request: ScreenSelectAreaRequest, on_done: F)
where
    F: FnOnce(Result<ScreenSelection>) + Send + 'static,
{
    match request {
        ScreenSelectAreaRequest::SelectRegion => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_done(Err(e)),
            };
            select_region(&outputs, move |region_result| {
                on_done(region_result.map(ScreenSelection::Region));
            });
        }
        ScreenSelectAreaRequest::SelectMonitor => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_done(Err(e)),
            };
            if outputs.len() == 1 {
                on_done(Ok(ScreenSelection::Monitor(outputs[0].name.clone())));
            } else {
                monitor_selector::select_monitor(&outputs, move |monitor_result| {
                    on_done(monitor_result.map(ScreenSelection::Monitor));
                });
            }
        }
    }
}

pub fn record_screen<S, D>(
    request: ScreenRecordRequest,
    _delay: Duration,
    on_started: S,
    on_done: D,
) where
    S: FnOnce(anyhow::Result<RecordHandle>) + Send + 'static,
    D: FnOnce(anyhow::Result<RecordResult>) + Send + 'static,
{
    match request.area {
        CaptureArea::SelectRegion => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_started(Err(e.into())),
            };
            select_region(&outputs, move |region_result| match region_result {
                Ok(region) => {
                    let args = WfRecorderArgs::Region {
                        x: region.x,
                        y: region.y,
                        width: region.width,
                        height: region.height,
                    };
                    on_started(Ok(start_recording(request.audio, args, on_done)));
                }
                Err(e) => on_started(Err(e.into())),
            });
        }
        CaptureArea::SelectMonitor => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_started(Err(e.into())),
            };
            monitor_selector::select_monitor(
                &outputs,
                move |monitor_result| match monitor_result {
                    Ok(name) => {
                        let args = WfRecorderArgs::Monitor { name };
                        on_started(Ok(start_recording(request.audio, args, on_done)));
                    }
                    Err(e) => on_started(Err(e.into())),
                },
            );
        }
        CaptureArea::SelectWindow => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_started(Err(e.into())),
            };
            window_selector::select_window(&outputs, move |window_result| match window_result {
                Ok(win) => {
                    let args = WfRecorderArgs::Window {
                        x: win.x,
                        y: win.y,
                        width: win.width,
                        height: win.height,
                    };
                    on_started(Ok(start_recording(request.audio, args, on_done)));
                }
                Err(e) => on_started(Err(e.into())),
            });
        }
        CaptureArea::All => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_started(Err(e.into())),
            };
            if outputs.len() > 1 {
                monitor_selector::select_monitor(&outputs, move |monitor_result| {
                    match monitor_result {
                        Ok(name) => {
                            let args = WfRecorderArgs::Monitor { name };
                            on_started(Ok(start_recording(request.audio, args, on_done)));
                        }
                        Err(e) => on_started(Err(e.into())),
                    }
                });
            } else {
                on_started(Ok(start_recording(
                    request.audio,
                    WfRecorderArgs::All,
                    on_done,
                )));
            }
        }
    }
}

pub fn take_screenshot<F>(request: ScreenshotRequest, delay: Duration, on_done: F)
where
    F: FnOnce(Result<ScreenshotResult>) + Send + 'static,
{
    match request.area {
        CaptureArea::SelectRegion => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_done(Err(e)),
            };
            let target = request.target.clone();
            select_region(&outputs, move |region_result| match region_result {
                Ok(region) => capture_and_finish_async(
                    move || CaptureBackend::new()?.capture_region(&region),
                    target,
                    delay,
                    on_done,
                ),
                Err(e) => on_done(Err(e)),
            });
        }
        CaptureArea::SelectMonitor => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_done(Err(e)),
            };
            let target = request.target.clone();
            monitor_selector::select_monitor(
                &outputs,
                move |monitor_result| match monitor_result {
                    Ok(name) => capture_and_finish_async(
                        move || CaptureBackend::new()?.capture_output(&name),
                        target,
                        delay,
                        on_done,
                    ),
                    Err(e) => on_done(Err(e)),
                },
            );
        }
        CaptureArea::SelectWindow => {
            let outputs = match query_outputs() {
                Ok(o) => o,
                Err(e) => return on_done(Err(e)),
            };
            let target = request.target.clone();
            window_selector::select_window(&outputs, move |window_result| match window_result {
                Ok(win) => capture_and_finish_async(
                    move || CaptureBackend::new()?.capture_window(&win),
                    target,
                    delay,
                    on_done,
                ),
                Err(e) => on_done(Err(e)),
            });
        }
        CaptureArea::All => {
            capture_and_finish_async(
                || CaptureBackend::new()?.capture_all(),
                request.target,
                delay,
                on_done,
            );
        }
    }
}

fn capture_and_finish_async<C, F>(capture: C, target: OutputTarget, delay: Duration, on_done: F)
where
    C: FnOnce() -> Result<image::RgbaImage> + Send + 'static,
    F: FnOnce(Result<ScreenshotResult>) + Send + 'static,
{
    std::thread::spawn(move || {
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
        let result = capture().and_then(|image| finish_capture(image, &target));
        glib::idle_add_once(move || {
            on_done(result);
        });
    });
}

fn finish_capture(image: image::RgbaImage, target: &OutputTarget) -> Result<ScreenshotResult> {
    match target {
        OutputTarget::FileAndClipboard => {
            let path = default_screenshot_path();
            save_to_file(&image, &path)?;
            copy_to_clipboard(&image)?;
            Ok(ScreenshotResult {
                saved_path: Some(path),
                in_clipboard: true,
            })
        }
        OutputTarget::File => {
            let path = default_screenshot_path();
            save_to_file(&image, &path)?;
            Ok(ScreenshotResult {
                saved_path: Some(path),
                in_clipboard: false,
            })
        }
        OutputTarget::Clipboard => {
            copy_to_clipboard(&image)?;
            Ok(ScreenshotResult {
                saved_path: None,
                in_clipboard: true,
            })
        }
    }
}

fn save_to_file(image: &image::RgbaImage, path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);

    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        writer,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::Sub,
    );

    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| ScreenshotError::EncodingFailed(e.to_string()))
}

fn copy_to_clipboard(image: &image::RgbaImage) -> Result<()> {
    let mut png_data: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut png_data,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::Sub,
    );
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e: image::ImageError| ScreenshotError::EncodingFailed(e.to_string()))?;

    use std::process::{Command, Stdio};
    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| ScreenshotError::ClipboardFailed(format!("failed to spawn wl-copy: {e}")))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(&png_data)
            .map_err(|e| ScreenshotError::ClipboardFailed(e.to_string()))?;
    }

    let status = child
        .wait()
        .map_err(|e| ScreenshotError::ClipboardFailed(e.to_string()))?;

    if !status.success() {
        return Err(ScreenshotError::ClipboardFailed(
            "wl-copy exited with non-zero status".into(),
        ));
    }

    Ok(())
}

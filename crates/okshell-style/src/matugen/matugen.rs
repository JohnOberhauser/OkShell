use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex};
use relm4::gtk::glib;
use tracing::{debug, info};
use okshell_config::schema::config::Matugen;
use crate::matugen::json_struct::{MatugenTheme, MatugenThemeCustomOnly};
use crate::matugen::css_mapping::to_css;

// Stores the pending timeout handle so we can cancel it on the next call
static PENDING: Mutex<Option<glib::JoinHandle<()>>> = Mutex::new(None);

pub fn apply_matugen_debounced(
    wallpaper: PathBuf,
    matugen: Matugen,
    theme: MatugenThemeCustomOnly,
    on_done: impl Fn(anyhow::Result<String>) + 'static,
) {
    // Cancel any pending call
    let mut pending = PENDING.lock().unwrap();
    if let Some(handle) = pending.take() {
        handle.abort();
    }

    *pending = Some(glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(300)).await;
        let result = apply_matugen(&wallpaper, matugen, theme);
        on_done(result);
    }));
}

pub fn apply_matugen(
    wallpaper: &std::path::Path,
    matugen: Matugen,
    theme: MatugenThemeCustomOnly,
) -> anyhow::Result<String> {
    info!("Calling matugen: {:?}, {:?}", wallpaper.display(), matugen);
    let child = Command::new("matugen")
        .args([
            "image", wallpaper.to_str().unwrap(),
            "--quiet",
            "--json", "hex",
            "--prefer", matugen.preference.to_string().as_str(),
            "--type", matugen.scheme_type.to_string().as_str(),
            "--mode", matugen.mode.to_string().as_str(),
            "--contrast", matugen.contrast.to_string().as_str(),
            "--import-json-string", serde_json::to_string(&theme)?.as_str(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let json_str = read_json_from_child(child)?;
    let theme: MatugenTheme = serde_json::from_str(&json_str)?;
    Ok(to_css(&theme))
}

pub fn apply_matugen_from_theme_debounced(
    theme: MatugenTheme,
    on_done: impl Fn(anyhow::Result<String>) + 'static,
) {
    // Cancel any pending call
    let mut pending = PENDING.lock().unwrap();
    if let Some(handle) = pending.take() {
        handle.abort();
    }

    *pending = Some(glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(300)).await;
        let result = apply_matugen_from_theme(&theme);
        on_done(result);
    }));
}

pub fn apply_matugen_from_theme(
    theme: &MatugenTheme,
) -> anyhow::Result<String> {
    info!("Calling matugen with static theme");
    let child = Command::new("matugen")
        .args([
            "color", "hex", "000000",
            "--quiet",
            "--json", "hex",
            "--import-json-string", serde_json::to_string(&theme)?.as_str(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let json_str = read_json_from_child(child)?;
    let theme: MatugenTheme = serde_json::from_str(&json_str)?;
    Ok(to_css(&theme))
}

fn read_json_from_child(mut child: std::process::Child) -> anyhow::Result<String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture matugen stdout"))?;

    let stderr = child.stderr.take();

    let mut reader = BufReader::new(stdout);
    let mut json_buf = String::new();
    let mut depth: i32 = 0;
    let mut started = false;
    let mut ended = false;
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 { break; }

        let mut log_line = true;

        for ch in line.chars() {
            if ch == '{' {
                if !started {
                    started = true;
                    log_line = false;
                }
                depth += 1;
                json_buf.push(ch);
            } else if ch == '}' && started {
                depth -= 1;
                json_buf.push(ch);
                if depth == 0 {
                    ended = true;
                    break;
                }
            } else if started {
                json_buf.push(ch);
            }
        }

        if log_line && !started && !line.trim().is_empty() {
            debug!("matugen: {}", line.trim());
        }

        if ended { break; }
    }

    if !ended {
        anyhow::bail!("matugen stdout ended before JSON was complete");
    }

    // Drain remaining output and reap the child in the background
    std::thread::spawn(move || {
        for line in reader.lines().flatten() {
            if !line.trim().is_empty() {
                debug!("matugen: {}", line.trim());
            }
        }
        if let Some(stderr) = stderr {
            for line in BufReader::new(stderr).lines().flatten() {
                if !line.trim().is_empty() {
                    debug!("matugen stderr: {}", line.trim());
                }
            }
        }
        let _ = child.wait();
    });

    Ok(json_buf)
}
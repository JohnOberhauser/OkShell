use crate::css_mapping::to_css;
use crate::json_struct::{MatugenTheme, MatugenThemeCustomOnly};
use okshell_config::schema::config::Matugen;
use relm4::gtk::glib;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

enum MatugenJob {
    Image {
        wallpaper: PathBuf,
        matugen: Matugen,
        theme: MatugenThemeCustomOnly,
    },
    Static {
        theme: MatugenTheme,
    },
}

struct MatugenResult {
    css: anyhow::Result<String>,
    waiter: Option<ChildWaiter>,
}

struct ChildWaiter {
    reader: BufReader<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
    child: std::process::Child,
}

impl ChildWaiter {
    fn wait(self) {
        for line in self.reader.lines().flatten() {
            if !line.trim().is_empty() {
                debug!("matugen: {}", line.trim());
            }
        }
        if let Some(stderr) = self.stderr {
            for line in BufReader::new(stderr).lines().flatten() {
                if !line.trim().is_empty() {
                    debug!("matugen stderr: {}", line.trim());
                }
            }
        }
        let mut child = self.child;
        let _ = child.wait();
    }
}

impl MatugenJob {
    fn kind(&self) -> &'static str {
        match self {
            MatugenJob::Image { .. } => "image",
            MatugenJob::Static { .. } => "static",
        }
    }

    fn run(self) -> MatugenResult {
        match self {
            MatugenJob::Image {
                wallpaper,
                matugen,
                theme,
            } => apply_matugen(&wallpaper, matugen, theme),
            MatugenJob::Static { theme } => apply_matugen_from_theme(&theme),
        }
    }
}

struct RunnerState {
    running: bool,
    waiting: Option<(
        MatugenJob,
        Box<dyn FnOnce(anyhow::Result<String>) + Send + 'static>,
    )>,
}

static RUNNER: Mutex<Option<Arc<Mutex<RunnerState>>>> = Mutex::new(None);

fn get_runner() -> Arc<Mutex<RunnerState>> {
    let mut guard = RUNNER.lock().unwrap();
    guard
        .get_or_insert_with(|| {
            Arc::new(Mutex::new(RunnerState {
                running: false,
                waiting: None,
            }))
        })
        .clone()
}

fn submit_job(job: MatugenJob, on_done: impl FnOnce(anyhow::Result<String>) + Send + 'static) {
    let runner = get_runner();
    let mut state = runner.lock().unwrap();

    if state.running {
        if state.waiting.is_some() {
            info!("Matugen: replacing queued job with new one");
        } else {
            info!("Matugen: job queued, waiting for current job to finish");
        }
        state.waiting = Some((job, Box::new(on_done)));
        return;
    }

    state.running = true;
    drop(state);

    let runner_clone = runner.clone();
    std::thread::spawn(move || {
        info!("Matugen: {} job started", job.kind());
        let MatugenResult { css, waiter } = job.run();
        info!("Matugen: json received, dispatching css");

        glib::idle_add_once(move || {
            on_done(css);
        });
        if let Some(waiter) = waiter {
            waiter.wait();
        }

        loop {
            let next = {
                let mut state = runner_clone.lock().unwrap();
                if let Some((job, cb)) = state.waiting.take() {
                    info!("Matugen: job finished, starting queued {} job", job.kind());
                    Some((job, cb))
                } else {
                    state.running = false;
                    info!("Matugen: job finished, queue empty");
                    None
                }
            };

            match next {
                Some((job, cb)) => {
                    let MatugenResult { css, waiter } = job.run();
                    info!("Matugen: json received, dispatching css");
                    glib::idle_add_once(move || {
                        cb(css);
                    });
                    if let Some(waiter) = waiter {
                        waiter.wait();
                    }
                }
                None => break,
            }
        }
    });
}

pub fn apply_matugen_from_image_queued(
    wallpaper: PathBuf,
    matugen: Matugen,
    theme: MatugenThemeCustomOnly,
    on_done: impl FnOnce(anyhow::Result<String>) + Send + 'static,
) {
    submit_job(
        MatugenJob::Image {
            wallpaper,
            matugen,
            theme,
        },
        on_done,
    );
}

pub fn apply_matugen_from_theme_queued(
    theme: MatugenTheme,
    on_done: impl FnOnce(anyhow::Result<String>) + Send + 'static,
) {
    submit_job(MatugenJob::Static { theme }, on_done);
}

fn apply_matugen(
    wallpaper: &std::path::Path,
    matugen: Matugen,
    theme: MatugenThemeCustomOnly,
) -> MatugenResult {
    let child = Command::new("matugen")
        .args([
            "image",
            wallpaper.to_str().unwrap(),
            "--quiet",
            "--json",
            "hex",
            "--prefer",
            matugen.preference.to_string().as_str(),
            "--type",
            matugen.scheme_type.to_string().as_str(),
            "--mode",
            matugen.mode.to_string().as_str(),
            "--contrast",
            matugen.contrast.to_string().as_str(),
            "--import-json-string",
            serde_json::to_string(&theme).unwrap().as_str(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(child) => read_json_from_child(child),
        Err(e) => MatugenResult {
            css: Err(e.into()),
            waiter: None,
        },
    }
}

fn apply_matugen_from_theme(theme: &MatugenTheme) -> MatugenResult {
    let child = Command::new("matugen")
        .args([
            "color",
            "hex",
            "000000",
            "--quiet",
            "--json",
            "hex",
            "--import-json-string",
            serde_json::to_string(&theme).unwrap().as_str(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(child) => read_json_from_child(child),
        Err(e) => MatugenResult {
            css: Err(e.into()),
            waiter: None,
        },
    }
}

fn read_json_from_child(mut child: std::process::Child) -> MatugenResult {
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            return MatugenResult {
                css: Err(anyhow::anyhow!("failed to capture matugen stdout")),
                waiter: None,
            };
        }
    };

    let stderr = child.stderr.take();

    let mut reader = BufReader::new(stdout);
    let mut json_buf = String::new();
    let mut depth: i32 = 0;
    let mut started = false;
    let mut ended = false;
    let mut line = String::new();

    loop {
        line.clear();
        let n = match reader.read_line(&mut line) {
            Ok(n) => n,
            Err(e) => {
                return MatugenResult {
                    css: Err(e.into()),
                    waiter: None,
                };
            }
        };
        if n == 0 {
            break;
        }

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

        if ended {
            break;
        }
    }

    if !ended {
        return MatugenResult {
            css: Err(anyhow::anyhow!(
                "matugen stdout ended before JSON was complete"
            )),
            waiter: None,
        };
    }

    let css = match serde_json::from_str::<MatugenTheme>(&json_buf) {
        Ok(theme) => Ok(to_css(&theme)),
        Err(e) => Err(e.into()),
    };

    MatugenResult {
        css,
        waiter: Some(ChildWaiter {
            reader,
            stderr,
            child,
        }),
    }
}

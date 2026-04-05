use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::process::{Command, Child};
use anyhow::Result;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use crate::utils::default_recording_path;

#[derive(Debug, Clone)]
pub struct RecordResult {
    pub saved_path: Option<PathBuf>,
}

/// Returned to the caller immediately after recording starts.
/// Call `.stop()` to terminate wf-recorder and trigger `on_done`.
#[derive(Debug, Clone)]
pub struct RecordHandle {
    stopped: Arc<AtomicBool>,
}

impl RecordHandle {
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
}

pub(crate) fn start_recording(
    audio: Option<String>,
    args: WfRecorderArgs,
    on_done: impl FnOnce(Result<RecordResult>) + Send + 'static,
) -> RecordHandle {
    let stopped = Arc::new(AtomicBool::new(false));
    let handle = RecordHandle { stopped: stopped.clone() };

    std::thread::spawn(move || {
        let path = default_recording_path();
        // ensure dir exists
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut cmd = Command::new("wf-recorder");
        match args {
            WfRecorderArgs::Region { x, y, width, height } => {
                cmd.args(["--geometry", &format!("{x},{y} {width}x{height}")]);
            }
            WfRecorderArgs::Monitor { name } => {
                cmd.args(["-o", &name]);
            }
            WfRecorderArgs::Window { x, y, width, height } => {
                cmd.args(["--geometry", &format!("{x},{y} {width}x{height}")]);
            }
            WfRecorderArgs::All => {}
        }
        cmd.arg("-f").arg(&path);
        cmd.args(["-c", "libx265"]);
        cmd.args(["-p", "crf=18"]);
        cmd.args(["-p", "preset=fast"]);
        cmd.args(["-x", "yuv444p"]);
        if let Some(audio) = audio {
            cmd.arg(format!("--audio={}", audio));
        }

        let mut child: Child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return on_done(Err(e.into())),
        };

        // Poll the stop flag, then kill the child
        loop {
            if stopped.load(Ordering::SeqCst) {
                let pid = Pid::from_raw(child.id() as i32);
                let _ = kill(pid, Signal::SIGINT);
                let _ = child.wait(); // wait for it to finish cleanly
                break;
            }
            if let Ok(Some(_)) = child.try_wait() {
                // process exited on its own
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        on_done(Ok(RecordResult { saved_path: Some(path) }));
    });

    handle
}

pub(crate) enum WfRecorderArgs {
    Region { x: i32, y: i32, width: i32, height: i32 },
    Monitor { name: String },
    Window { x: i32, y: i32, width: i32, height: i32 },
    All,
}
use std::process::Stdio;
use tokio::process::Command;

pub fn reboot() {
    tokio::spawn(async {
        let result = Command::new("systemctl")
            .args(["reboot"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(out) if out.status.success() => {}
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                tracing::error!(
                    status = %out.status,
                    stderr = %stderr.trim(),
                    "systemctl reboot failed"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to execute systemctl reboot");
            }
        }
    });
}
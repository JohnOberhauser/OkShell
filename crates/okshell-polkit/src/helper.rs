use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error};

const HELPER_SOCKET: &str = "/run/polkit/agent-helper.socket";

#[derive(Debug, Clone)]
pub enum HelperEvent {
    Request { prompt: String, echo: bool },
    Info(String),
    Error(String),
    Completed { success: bool },
}

pub struct PolkitAgentHelper {
    writer: tokio::io::WriteHalf<UnixStream>,
}

impl PolkitAgentHelper {
    pub async fn connect(
        username: &str,
        cookie: &str,
    ) -> std::io::Result<(Self, tokio::sync::mpsc::Receiver<HelperEvent>)> {
        let stream = UnixStream::connect(HELPER_SOCKET).await?;
        let (reader, mut writer) = tokio::io::split(stream);

        // Write username and cookie — helper reads these first
        writer.write_all(format!("{username}\n").as_bytes()).await?;
        writer.write_all(format!("{cookie}\n").as_bytes()).await?;
        writer.flush().await?;

        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            let buf_reader = BufReader::new(reader);
            let mut lines = buf_reader.lines();
            let mut completed = false;

            while let Ok(Some(line)) = lines.next_line().await {
                debug!("[polkit-helper] {line}");

                let event = parse_helper_line(&line);
                if matches!(event, HelperEvent::Completed { .. }) {
                    completed = true;
                }
                if tx.send(event).await.is_err() {
                    break;
                }
                if completed {
                    break;
                }
            }

            if !completed {
                let _ = tx.send(HelperEvent::Completed { success: false }).await;
            }
        });

        Ok((Self { writer }, rx))
    }

    pub async fn respond(&mut self, password: &str) -> std::io::Result<()> {
        self.writer.write_all(password.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await
    }
}

fn parse_helper_line(line: &str) -> HelperEvent {
    if line == "SUCCESS" {
        return HelperEvent::Completed { success: true };
    }
    if line == "FAILURE" {
        return HelperEvent::Completed { success: false };
    }
    if let Some(rest) = line.strip_prefix("PAM_PROMPT_ECHO_OFF ") {
        return HelperEvent::Request {
            prompt: rest.to_string(),
            echo: false,
        };
    }
    if let Some(rest) = line.strip_prefix("PAM_PROMPT_ECHO_ON ") {
        return HelperEvent::Request {
            prompt: rest.to_string(),
            echo: true,
        };
    }
    if let Some(rest) = line.strip_prefix("PAM_TEXT_INFO ") {
        return HelperEvent::Info(rest.to_string());
    }
    if let Some(rest) = line.strip_prefix("PAM_ERROR_MSG ") {
        return HelperEvent::Error(rest.to_string());
    }

    error!("[polkit-helper] unrecognized line: {line}");
    HelperEvent::Info(line.to_string())
}
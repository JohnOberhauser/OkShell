use std::collections::HashMap;
use tracing::{debug, error, info};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::OwnedValue;
use zbus::{Connection, fdo, interface};

use crate::helper::{HelperEvent, PolkitAgentHelper};
use crate::prompt::PolkitPromptInput;

const AGENT_OBJECT_PATH: &str = "/com/okshell/PolkitAgent";

#[derive(Debug)]
pub enum PasswordAction {
    Submit(String),
    Cancel,
}

pub struct PolkitAgentObject {
    prompt_sender: relm4::ComponentSender<crate::prompt::PolkitPromptModel>,
}

#[interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]
impl PolkitAgentObject {
    async fn begin_authentication(
        &self,
        #[zbus(signal_emitter)] _emitter: SignalEmitter<'_>,
        action_id: &str,
        message: &str,
        icon_name: &str,
        _details: HashMap<String, String>,
        cookie: &str,
        identities: Vec<(String, HashMap<String, OwnedValue>)>,
    ) -> fdo::Result<()> {
        info!("[polkit] BeginAuthentication action={action_id} cookie={cookie}");

        let username = pick_username(&identities)
            .ok_or_else(|| fdo::Error::Failed("no unix-user identity found".into()))?;

        debug!("[polkit] authenticating user: {username}");

        // We may need multiple attempts (wrong password → retry)
        let max_attempts = 3;

        for attempt in 1..=max_attempts {
            debug!("[polkit] attempt {attempt}/{max_attempts}");

            // Spawn the helper process
            let (mut helper, mut helper_rx) =
                PolkitAgentHelper::connect(&username, cookie)
                    .await
                    .map_err(|e| fdo::Error::Failed(format!("failed to connect to helper: {e}")))?;

            debug!("[polkit] helper spawned");
            // Channel for the UI to send password/cancel back to us
            let (password_tx, mut password_rx) = tokio::sync::mpsc::channel::<PasswordAction>(4);

            self.prompt_sender.input(PolkitPromptInput::Show {
                message: message.to_string(),
                icon_name: icon_name.to_string(),
                password_tx: password_tx.clone(),
            });

            let mut success = false;

            // Event loop: shuttle between helper events and UI responses
            loop {
                tokio::select! {
                    Some(event) = helper_rx.recv() => {
                        match event {
                            HelperEvent::Request { prompt, echo } => {
                                // Tell UI a password is needed (with prompt text)
                                self.prompt_sender.input(PolkitPromptInput::PromptReady {
                                    prompt,
                                    echo,
                                });

                                // Wait for the user to submit or cancel
                                match password_rx.recv().await {
                                    Some(PasswordAction::Submit(password)) => {
                                        if let Err(e) = helper.respond(&password).await {
                                            error!("[polkit] failed to send password to helper: {e}");
                                            break;
                                        }
                                    }
                                    Some(PasswordAction::Cancel) | None => {
                                        drop(helper);
                                        self.prompt_sender.input(PolkitPromptInput::Hide);
                                        return Err(fdo::Error::Failed("cancelled by user".into()));
                                    }
                                }
                            }
                            HelperEvent::Info(text) => {
                                self.prompt_sender.input(PolkitPromptInput::InfoMessage(text));
                            }
                            HelperEvent::Error(text) => {
                                self.prompt_sender.input(PolkitPromptInput::ErrorMessage(text));
                            }
                            HelperEvent::Completed { success: s } => {
                                success = s;
                                break;
                            }
                        }
                    }
                    // If the password channel closes unexpectedly, bail
                    else => {
                        drop(helper);
                        break;
                    }
                }
            }

            if success {
                self.prompt_sender.input(PolkitPromptInput::Hide);
                info!("[polkit] authentication succeeded");
                return Ok(());
            }

            // Failed — show error, clear entry, let loop retry with a fresh helper
            if attempt < max_attempts {
                self.prompt_sender.input(PolkitPromptInput::ErrorMessage(
                    "Authentication failed. Try again.".into(),
                ));
                self.prompt_sender.input(PolkitPromptInput::ClearEntry);
            }
        }

        // All attempts exhausted
        self.prompt_sender.input(PolkitPromptInput::Hide);
        Err(fdo::Error::Failed("authentication failed".into()))
    }

    async fn cancel_authentication(&self, cookie: &str) -> fdo::Result<()> {
        info!("[polkit] CancelAuthentication cookie={cookie}");
        self.prompt_sender.input(PolkitPromptInput::Hide);
        Ok(())
    }
}

fn pick_username(identities: &[(String, HashMap<String, OwnedValue>)]) -> Option<String> {
    for (kind, props) in identities {
        if kind == "unix-user" {
            if let Some(uid_val) = props.get("uid") {
                // uid comes as a variant; try to extract u32
                let uid: u32 = uid_val.try_into().ok().or_else(|| {
                    // Fallback: try downcast to u32 directly
                    TryInto::<u32>::try_into(uid_val).ok()
                })?;

                // Resolve uid → username via nix or libc
                #[cfg(target_os = "linux")]
                {
                    use nix::unistd::{Uid, User};
                    if let Ok(Some(user)) = User::from_uid(Uid::from_raw(uid)) {
                        return Some(user.name);
                    }
                }
            }
        }
    }
    None
}

/// Register the polkit agent on the system bus.
///
/// Call this once at startup, passing the Relm4 component sender for the prompt UI.
pub async fn register_polkit_agent(
    prompt_sender: relm4::ComponentSender<crate::prompt::PolkitPromptModel>,
) -> anyhow::Result<Connection> {
    let connection = Connection::system().await?;

    let agent = PolkitAgentObject { prompt_sender };

    // Export the agent object
    connection
        .object_server()
        .at(AGENT_OBJECT_PATH, agent)
        .await?;

    // Register with the polkit authority
    let session_id = std::env::var("XDG_SESSION_ID").unwrap_or_default();

    // Call org.freedesktop.PolicyKit1.Authority.RegisterAuthenticationAgent
    connection
        .call_method(
            Some("org.freedesktop.PolicyKit1"),
            "/org/freedesktop/PolicyKit1/Authority",
            Some("org.freedesktop.PolicyKit1.Authority"),
            "RegisterAuthenticationAgent",
            // subject: (kind, details) — a PolkitSubject
            // For a unix-session: ("unix-session", {"session-id": Variant(session_id)})
            &(
                (
                    "unix-session",
                    HashMap::from([(
                        "session-id",
                        zbus::zvariant::Value::from(session_id.as_str()),
                    )]),
                ),
                "en_US.UTF-8",     // locale
                AGENT_OBJECT_PATH, // object path
            ),
        )
        .await?;

    info!("[polkit] agent registered at {AGENT_OBJECT_PATH}");
    Ok(connection)
}

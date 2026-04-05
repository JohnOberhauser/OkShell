use std::time::Duration;
use gtk4::glib;
use gtk4::glib::SignalHandlerId;
use gtk4::prelude::{GtkWindowExt, MonitorExt, WidgetExt};
use gtk4_layer_shell::{Layer, LayerShell};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use tracing::info;
use okshell_auth::fingerprint::{FingerprintAuth, FingerprintEvent};
use okshell_session::session_lock::session_lock;
use crate::lock_screen::{LockScreenInit, LockScreenInput, LockScreenModel, LockScreenOutput, LOCK_SCREEN_REVEALER_TRANSITION_DURATION};
use crate::utils::username::current_username;

pub struct LockScreenManagerModel {
    lock_screens: Vec<Controller<LockScreenModel>>,
    fingerprint_active: bool,
    fingerprint_cancel: Option<tokio::sync::oneshot::Sender<()>>,
    _monitor_added_lock_signal_handler_id: SignalHandlerId,
    _lock_signal_handler_id: SignalHandlerId,
    _unlock_signal_handler_id: SignalHandlerId,
}

#[derive(Debug)]
pub enum LockScreenManagerInput {
    LockScreenCreated(Controller<LockScreenModel>),
    SessionLocked,
    SessionUnlocked,
    CancelFingerprint,
    AuthSuccess,
}

#[derive(Debug)]
pub enum LockScreenManagerOutput {}

pub struct LockScreenManagerInit {}

#[derive(Debug)]
pub enum LockScreenManagerCommandOutput {
    FingerprintEvent(FingerprintEvent),
}

#[relm4::component(pub)]
impl Component for LockScreenManagerModel {
    type CommandOutput = LockScreenManagerCommandOutput;
    type Input = LockScreenManagerInput;
    type Output = LockScreenManagerOutput;
    type Init = LockScreenManagerInit;

    view! {
        #[root]
        gtk::Window {
            set_decorated: false,
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_layer(Layer::Background);
        root.set_default_size(1, 1);
        root.set_visible(false);

        let sender_clone = sender.clone();
        let monitor_added_lock_signal_handler_id = session_lock().connect_monitor(move |_instance, monitor| {
            info!("Lock monitor signal for {:?}", monitor.connector());
            let controller = LockScreenModel::builder()
                .launch(LockScreenInit {
                    monitor: monitor.clone(),
                })
                .forward(sender_clone.input_sender(), |msg| {
                    match msg {
                        LockScreenOutput::CancelFingerprint => {
                            LockScreenManagerInput::CancelFingerprint
                        }
                        LockScreenOutput::PasswordAuthSuccess => {
                            LockScreenManagerInput::AuthSuccess
                        }
                    }
                });
            sender_clone.input(LockScreenManagerInput::LockScreenCreated(controller));
        });

        let sender_clone = sender.clone();
        let lock_signal_handler_id = session_lock().connect_locked(move |_| {
            sender_clone.input(LockScreenManagerInput::SessionLocked);
        });

        let sender_clone = sender.clone();
        let unlock_signal_handler_id = session_lock().connect_unlocked(move |_| {
            sender_clone.input(LockScreenManagerInput::SessionUnlocked);
        });

        let model = LockScreenManagerModel {
            lock_screens: Vec::new(),
            fingerprint_active: false,
            fingerprint_cancel: None,
            _monitor_added_lock_signal_handler_id: monitor_added_lock_signal_handler_id,
            _lock_signal_handler_id: lock_signal_handler_id,
            _unlock_signal_handler_id: unlock_signal_handler_id,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            LockScreenManagerInput::LockScreenCreated(controller) => {
                if self.fingerprint_active {
                    controller.sender().emit(LockScreenInput::FingerprintReady);
                }
                self.lock_screens.push(controller);
            }
            LockScreenManagerInput::SessionLocked => {
                self.fingerprint_active = true;
                self.start_fingerprint_auth(&sender);
            }
            LockScreenManagerInput::SessionUnlocked => {
                info!("Session unlocked");
                self.fingerprint_active = false;
                if let Some(cancel) = self.fingerprint_cancel.take() {
                    let _ = cancel.send(());
                }
                self.lock_screens.clear();
            }
            LockScreenManagerInput::CancelFingerprint => {
                self.fingerprint_active = false;
                if let Some(cancel) = self.fingerprint_cancel.take() {
                    let _ = cancel.send(());
                }
                // Lock screens switch to password-only mode
                for ls in &self.lock_screens {
                    ls.emit(LockScreenInput::ShowPasswordEntry);
                }
            }
            LockScreenManagerInput::AuthSuccess => {
                for ls in &self.lock_screens {
                    ls.emit(LockScreenInput::HideScreen);
                }
                glib::timeout_add_local_once(Duration::from_millis(LOCK_SCREEN_REVEALER_TRANSITION_DURATION as u64), || {
                    session_lock().unlock();
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            LockScreenManagerCommandOutput::FingerprintEvent(event) => {
                match event {
                    FingerprintEvent::Ready => {
                        for ls in &self.lock_screens {
                            ls.emit(LockScreenInput::FingerprintReady);
                        }
                    }
                    FingerprintEvent::Scanning => {
                        for ls in &self.lock_screens {
                            ls.emit(LockScreenInput::FingerprintScanning);
                        }
                    }
                    FingerprintEvent::Match => {
                        sender.input(LockScreenManagerInput::AuthSuccess)
                    }
                    FingerprintEvent::NoMatch => {
                        for ls in &self.lock_screens {
                            ls.emit(LockScreenInput::FingerprintFailed);
                        }
                    }
                    FingerprintEvent::UnknownError => {
                        // shouldn't happen
                    }
                    FingerprintEvent::Error(e) => {
                        info!("Fingerprint error: {e}");
                        // Fingerprint unavailable, lock screens show password only
                        for ls in &self.lock_screens {
                            ls.emit(LockScreenInput::ShowPasswordEntry);
                        }
                    }
                }
            }
        }
    }
}

impl LockScreenManagerModel {
    fn start_fingerprint_auth(&mut self, sender: &ComponentSender<Self>) {
        let username = current_username();
        let cmd_sender = sender.command_sender().clone();
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();
        self.fingerprint_cancel = Some(cancel_tx);

        tokio::spawn(async move {
            let auth = match FingerprintAuth::new().await {
                Ok(a) => a,
                Err(e) => {
                    info!("fprintd not available: {e}");
                    let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(
                        FingerprintEvent::Error(e.to_string()),
                    ));
                    return;
                }
            };

            if let Err(e) = auth.start(&username).await {
                info!("fprintd start failed: {e}");
                let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(
                    FingerprintEvent::Error(e.to_string()),
                ));
                return;
            }

            let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(
                FingerprintEvent::Ready,
            ));

            let mut unknown_error_count = 0u32;

            loop {
                tokio::select! {
                _ = &mut cancel_rx => {
                    info!("Fingerprint cancelled");
                    let _ = auth.stop().await;
                    return;
                }
                result = auth.wait_for_result() => {
                    match result {
                        Ok(FingerprintEvent::Match) => {
                            let _ = auth.stop().await;
                            let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(
                                FingerprintEvent::Match,
                            ));
                            return;
                        }
                        Ok(FingerprintEvent::NoMatch) => {
                            unknown_error_count = 0;
                            let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(
                                FingerprintEvent::NoMatch,
                            ));
                            let _ = auth.device.verify_stop().await;
                            let _ = auth.device.verify_start("any").await;
                        }
                        Ok(FingerprintEvent::UnknownError) => {
                            unknown_error_count += 1;
                            if unknown_error_count > 3 {
                                let _ = auth.stop().await;
                                let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(
                                    FingerprintEvent::Error("Device error".into()),
                                ));
                                return;
                            }
                            info!("Unknown error, retrying ({unknown_error_count}/3)");
                            let _ = auth.device.verify_stop().await;
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            let _ = auth.device.verify_start("any").await;
                        }
                        Ok(event) => {
                            let _ = auth.stop().await;
                            let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(event));
                            return;
                        }
                        Err(e) => {
                            let _ = auth.stop().await;
                            let _ = cmd_sender.send(LockScreenManagerCommandOutput::FingerprintEvent(
                                FingerprintEvent::Error(e.to_string()),
                            ));
                            return;
                        }
                    }
                }
            }
            }
        });
    }
}
use crate::utils::username::current_username;
use gtk4::glib;
use gtk4::glib::SourceId;
use okshell_cache::wallpaper::{
    WallpaperStateStoreFields, current_wallpaper_image, wallpaper_store,
};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::schema::config::{ConfigStoreFields, GeneralStoreFields};
use okshell_session::session_lock::session_lock;
use pam::Client;
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::gtk::gdk;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk, once_cell};
use time::OffsetDateTime;
use time::format_description::parse;
use tracing::info;

static TIME_FORMAT_24: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:24 padding:zero]:[minute padding:zero]").unwrap()
    });

static TIME_FORMAT_12: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[hour repr:12 padding:zero]:[minute padding:zero] [period case:lower]").unwrap()
    });

static DAY_FORMAT: once_cell::sync::Lazy<Vec<time::format_description::FormatItem<'static>>> =
    once_cell::sync::Lazy::new(|| {
        parse("[weekday repr:long], [month repr:long] [day padding:none]").unwrap()
    });

pub static LOCK_SCREEN_REVEALER_TRANSITION_DURATION: u32 = 300;

#[derive(Debug)]
enum StackState {
    Nothing,
    Fingerprint,
    Password,
    Authenticating,
}

#[derive(Debug)]
enum FingerprintState {
    Normal,
    Scanning,
    Fail,
    Success,
}

#[derive(Debug)]
pub struct LockScreenModel {
    show_password: bool,
    place_holder_text: String,
    format_24_h: bool,
    time_label: String,
    day_label: String,
    timer_id: Option<SourceId>,
    revealed: bool,
    stack_state: StackState,
    fingerprint_state: FingerprintState,
    _effects: EffectScope,
}

#[derive(Debug)]
pub enum LockScreenInput {
    OnShow,
    TogglePasswordVisibility,
    AttemptLogin,
    UpdateTime,
    FingerprintReady,
    FingerprintScanning,
    FingerprintFailed,
    ShowPasswordEntry,
    UsePasswordClicked,
    HideScreen,
    PasswordSuccess,
    PasswordFailed,
    WallpaperChanged,
}

#[derive(Debug)]
pub enum LockScreenOutput {
    CancelFingerprint,
    PasswordAuthSuccess,
}

pub struct LockScreenInit {
    pub monitor: gdk::Monitor,
}

#[derive(Debug)]
pub enum LockScreenCommandOutput {}

#[relm4::component(pub)]
impl Component for LockScreenModel {
    type CommandOutput = LockScreenCommandOutput;
    type Input = LockScreenInput;
    type Output = LockScreenOutput;
    type Init = LockScreenInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Window {
            add_css_class: "lock-screen-window",
            set_decorated: false,
            set_visible: false,

            #[name = "content"]
            gtk::Overlay {
                #[watch]
                set_css_classes: if model.revealed {
                    &["lockscreen-content", "revealed"]
                } else {
                    &["lockscreen-content"]
                },

                add_overlay = &gtk::Box {
                    set_vexpand: true,
                    set_hexpand: false,
                    set_margin_top: 200,
                    set_margin_bottom: 200,
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,

                        gtk::Label {
                            set_css_classes: &[
                                "label-xxxl-bold",
                                "lockscreen-label"
                            ],
                            #[watch]
                            set_label: &model.day_label,
                        },

                        gtk::Label {
                            set_css_classes: &[
                                "label-xxl-bold",
                                "lockscreen-label"
                            ],
                            #[watch]
                            set_label: &model.time_label,
                        },
                    },

                    gtk::Box {
                        set_vexpand: true,
                    },

                    gtk::Stack {
                        set_transition_type: gtk::StackTransitionType::Crossfade,
                        set_transition_duration: 300,
                        set_vhomogeneous: true,
                        #[watch]
                        set_visible_child_name: match model.stack_state {
                            StackState::Nothing => "none",
                            StackState::Fingerprint => "fingerprint",
                            StackState::Password => "password",
                            StackState::Authenticating => "authenticating",
                        },

                        add_named[Some("none")] = &gtk::Box {

                        },

                        add_named[Some("fingerprint")] = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,
                            set_valign: gtk::Align::Center,

                            gtk::Label {
                            set_css_classes: &[
                                    "label-xxl-bold",
                                    "lockscreen-label"
                                ],
                                set_label: "Scan fingerprint",
                            },

                            gtk::Image {
                                #[watch]
                                set_css_classes: match model.fingerprint_state {
                                    FingerprintState::Normal => {
                                        &["lockscreen-fingerprint-icon"]
                                    }
                                    FingerprintState::Scanning => {
                                        &["lockscreen-fingerprint-icon", "scanning"]
                                    }
                                    FingerprintState::Fail => {
                                        &["lockscreen-fingerprint-icon", "fail"]
                                    }
                                    FingerprintState::Success => {
                                        &["lockscreen-fingerprint-icon", "success"]
                                    }
                                },
                                set_icon_name: Some("fingerprint-symbolic"),
                            },

                            #[name = "use_password_button"]
                            gtk::Button {
                                add_css_class: "unset",
                                set_hexpand: false,
                                set_halign: gtk::Align::Center,
                                connect_clicked[sender] => move |_| {
                                    sender.input(LockScreenInput::UsePasswordClicked);
                                },

                                gtk::Label {
                                    set_css_classes: &["label-small", "lockscreen-label"],
                                    set_label: "Use password instead",
                                },
                            },
                        },

                        add_named[Some("password")] = &gtk::Box {
                            add_css_class: "lockscreen-entry-wrapper",
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: false,
                            set_vexpand: false,
                            set_valign: gtk::Align::Center,

                            #[name = "password_entry"]
                            gtk::Entry {
                                add_css_class: "lockscreen-entry",
                                set_width_request: 400,
                                set_hexpand: false,
                                #[watch]
                                set_placeholder_text: Some(model.place_holder_text.as_str()),
                                #[watch]
                                set_visibility: model.show_password,
                                #[watch]
                                set_icon_from_icon_name: (
                                    gtk::EntryIconPosition::Secondary,
                                    Some(
                                        if model.show_password {
                                            "eye-symbolic"
                                        } else {
                                            "eye-off-symbolic"
                                        }
                                    )
                                ),
                                set_icon_activatable: (gtk::EntryIconPosition::Secondary, true),
                                connect_icon_press[sender] => move |_, pos| {
                                    if pos == gtk::EntryIconPosition::Secondary {
                                        sender.input(LockScreenInput::TogglePasswordVisibility);
                                    }
                                },
                                connect_activate[sender] => move |_| {
                                    sender.input(LockScreenInput::AttemptLogin);
                                },
                            },
                        },

                        add_named[Some("authenticating")] = &gtk::Box {
                            set_valign: gtk::Align::Center,
                            set_halign: gtk::Align::Center,

                            gtk::Spinner {
                                add_css_class: "lockscreen-spinner",
                                set_spinning: true,
                                set_width_request: 40,
                                set_height_request: 40,
                            }
                        },
                    },


                },

                #[name = "wallpaper"]
                gtk::Picture {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_content_fit: gtk::ContentFit::Cover,
                },
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let base_config = okshell_config::config_manager::config_manager().config();

        let sender_clone = sender.clone();
        let id = glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            sender_clone.input(LockScreenInput::UpdateTime);
            glib::ControlFlow::Continue
        });

        let format_24_h = base_config
            .clone()
            .general()
            .clock_format_24_h()
            .get_untracked();

        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

        let time = if format_24_h {
            now.format(&TIME_FORMAT_24)
        } else {
            now.format(&TIME_FORMAT_12)
        }
        .unwrap();

        let day = now.format(&DAY_FORMAT).unwrap();

        let mut effects = EffectScope::new();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let _revision = wallpaper_store().revision().get();
            sender_clone.input(LockScreenInput::WallpaperChanged);
        });

        let model = LockScreenModel {
            show_password: false,
            place_holder_text: String::new(),
            format_24_h,
            time_label: time,
            day_label: day,
            timer_id: Some(id),
            revealed: false,
            stack_state: StackState::Nothing,
            fingerprint_state: FingerprintState::Normal,
            _effects: effects,
        };

        let widgets = view_output!();

        // Set initial wallpaper
        set_wallpaper_picture(&widgets.wallpaper);

        session_lock().assign_window_to_monitor(&widgets.root, &params.monitor);

        widgets.root.set_visible(true);

        sender.input(LockScreenInput::OnShow);

        EditableExt::set_alignment(&widgets.password_entry, 0.5);

        widgets
            .use_password_button
            .set_cursor_from_name(Some("pointer"));

        info!("Lock screen created");

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            LockScreenInput::OnShow => {
                self.revealed = true;
            }
            LockScreenInput::TogglePasswordVisibility => {
                self.show_password = !self.show_password;
            }
            LockScreenInput::AttemptLogin => {
                info!("Attempt password login");
                self.stack_state = StackState::Authenticating;
                let password = widgets.password_entry.text().to_string();
                let username = current_username();
                let sender = sender.clone();

                tokio::task::spawn_blocking(move || {
                    let success = (|| {
                        let mut client = Client::with_password("system-login").ok()?;
                        client
                            .conversation_mut()
                            .set_credentials(username, password);
                        client.authenticate().ok()
                    })();
                    if success.is_some() {
                        sender.input(LockScreenInput::PasswordSuccess);
                    } else {
                        sender.input(LockScreenInput::PasswordFailed);
                    }
                });
            }
            LockScreenInput::PasswordSuccess => {
                sender.input(LockScreenInput::HideScreen);
                glib::timeout_add_local_once(
                    std::time::Duration::from_millis(
                        LOCK_SCREEN_REVEALER_TRANSITION_DURATION as u64,
                    ),
                    || {
                        session_lock().unlock();
                    },
                );
            }

            LockScreenInput::PasswordFailed => {
                info!("Password authentication failed");
                widgets.password_entry.set_text("");
                self.place_holder_text = "Authentication Failed".to_string();
                self.stack_state = StackState::Password;
                let entry = widgets.password_entry.clone();
                glib::idle_add_local_once(move || {
                    entry.grab_focus();
                });
            }
            LockScreenInput::FingerprintReady => {
                info!("fingerprint ready");
                self.stack_state = StackState::Fingerprint;
                let btn = widgets.use_password_button.clone();
                glib::idle_add_local_once(move || {
                    btn.grab_focus();
                });
            }
            LockScreenInput::FingerprintScanning => {
                info!("fingerprint scanning");
                self.fingerprint_state = FingerprintState::Scanning;
            }
            LockScreenInput::FingerprintFailed => {
                info!("fingerprint failed");
                self.fingerprint_state = FingerprintState::Fail;
            }
            LockScreenInput::ShowPasswordEntry => {
                info!("show password entry");
                self.stack_state = StackState::Password;
                let entry = widgets.password_entry.clone();
                glib::idle_add_local_once(move || {
                    entry.grab_focus();
                });
            }
            LockScreenInput::UsePasswordClicked => {
                let _ = sender.output(LockScreenOutput::CancelFingerprint);
            }
            LockScreenInput::HideScreen => {
                self.fingerprint_state = FingerprintState::Success;
                self.revealed = false
            }
            LockScreenInput::UpdateTime => {
                let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

                let time = if self.format_24_h {
                    now.format(&TIME_FORMAT_24)
                } else {
                    now.format(&TIME_FORMAT_12)
                }
                .unwrap();

                let day = now.format(&DAY_FORMAT).unwrap();

                self.day_label = day;
                self.time_label = time;
            }
            LockScreenInput::WallpaperChanged => {
                set_wallpaper_picture(&widgets.wallpaper);
            }
        }

        self.update_view(widgets, sender);
    }
}

/// Set the lock screen wallpaper picture from the in-memory buffer.
fn set_wallpaper_picture(picture: &gtk::Picture) {
    if let Some(image) = current_wallpaper_image() {
        let bytes = glib::Bytes::from(&*image.buf);
        let texture = gdk::MemoryTexture::new(
            image.width as i32,
            image.height as i32,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            (image.width * 4) as usize,
        );
        picture.set_paintable(Some(&texture));
    } else {
        picture.set_paintable(None::<&gdk::Texture>);
    }
}

impl Drop for LockScreenModel {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.take() {
            id.remove();
        }
    }
}

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use relm4::gtk::glib;
use relm4::gtk::pango;
use relm4::gtk::prelude::*;
use okshell_common::WatcherToken;
use wayle_media::core::player::Player;
use wayle_media::types::{LoopMode, PlaybackState, ShuffleMode};
use okshell_utils::media::spawn_media_player_watcher;

#[derive(Clone, Copy)]
enum ScrollState {
    PauseStart(u32),
    Scrolling,
    PauseEnd(u32),
}

pub(crate) struct MediaPlayerModel {
    pub player: Arc<Player>,
    track_name: String,
    track_name_scroll_source: Option<glib::SourceId>,
    artist_name: String,
    artist_name_scroll_source: Option<glib::SourceId>,
    current_track_time: String,
    track_length: String,
    scale_value_changed_signal: Option<glib::SignalHandlerId>,
    position: Duration,
    pending_seek: Option<(Duration, Instant)>, // (target, when_we_sent_it)
    can_loop: bool,
    can_shuffle: bool,
    can_go_next: bool,
    can_go_previous: bool,
    can_play: bool,
    can_seek: bool,
    playback_state: PlaybackState,
    shuffle_mode: ShuffleMode,
    loop_mode: LoopMode,
}

#[derive(Debug)]
pub(crate) enum MediaPlayerInput {
    ScaleChanged(f64),  // fires continuously while dragging, only updates pending_seek display
    ScaleClicked(f64),  // fires once on mouse up, triggers actual seek
    ShuffleClicked,
    LoopClicked,
    PreviousClicked,
    NextClicked,
    PlayPauseClicked,
}

#[derive(Debug)]
pub(crate) enum MediaPlayerOutput {}

pub(crate) struct MediaPlayerInit {
    pub player: Arc<Player>,
}

#[derive(Debug)]
pub(crate) enum MediaPlayerCommandOutput {
    PlaybackStateChanged,
    MetaDataChanged,
    LoopModeChanged,
    ShuffleModeChanged,
    CapabilitiesChanged,
    PositionChanged(Duration),
}

#[relm4::component(pub)]
impl Component for MediaPlayerModel {
    type CommandOutput = MediaPlayerCommandOutput;
    type Input = MediaPlayerInput;
    type Output = MediaPlayerOutput;
    type Init = MediaPlayerInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "media-player-menu-widget",
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,

            #[name = "track_scroll_window"]
            gtk::ScrolledWindow {
                set_policy: (gtk::PolicyType::External, gtk::PolicyType::Never),
                set_overflow: gtk::Overflow::Hidden,
                set_hexpand: true,

                #[name = "track"]
                gtk::Label {
                    add_css_class: "label-small-bold-variant",
                    #[watch]
                    set_label: model.track_name.as_str(),
                    set_xalign: 0.5,
                    set_wrap: false,
                    set_max_width_chars: -1,
                    set_ellipsize: pango::EllipsizeMode::None,
                },
            },

            #[name = "artist_scroll_window"]
            gtk::ScrolledWindow {
                set_policy: (gtk::PolicyType::External, gtk::PolicyType::Never),
                set_overflow: gtk::Overflow::Hidden,
                set_hexpand: true,

                #[name = "artist"]
                gtk::Label {
                    add_css_class: "label-small-bold",
                    #[watch]
                    set_label: model.artist_name.as_str(),
                    set_xalign: 0.5,
                    set_wrap: false,
                    set_max_width_chars: -1,
                    set_ellipsize: pango::EllipsizeMode::None,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "label-small",
                    #[watch]
                    set_label: model.current_track_time.as_str(),
                },

                #[name = "scale"]
                gtk::Scale {
                    add_css_class: "ok-progress-bar",
                    set_hexpand: true,
                    set_can_focus: false,
                    set_focus_on_click: false,
                    set_range: (0.0, 1.0),
                    set_margin_start: 20,
                    set_margin_end: 20,
                    #[watch]
                    set_sensitive: model.can_seek,
                },

                gtk::Label {
                    add_css_class: "label-small",
                    #[watch]
                    set_label: model.track_length.as_str(),
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_align: gtk::Align::Center,

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    #[watch]
                    set_sensitive: model.can_shuffle,
                    connect_clicked[sender] => move |_| {
                        sender.input(MediaPlayerInput::ShuffleClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_icon_name: match model.shuffle_mode {
                            ShuffleMode::On => Some("media-shuffle-symbolic"),
                            ShuffleMode::Off => Some("media-shuffle-off-symbolic"),
                            ShuffleMode::Unsupported => Some("media-shuffle-off-symbolic"),
                        },
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    #[watch]
                    set_sensitive: model.can_go_previous,
                    connect_clicked[sender] => move |_| {
                        sender.input(MediaPlayerInput::PreviousClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: Some("media-skip-previous-symbolic"),
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    #[watch]
                    set_sensitive: model.can_play,
                    connect_clicked[sender] => move |_| {
                        sender.input(MediaPlayerInput::PlayPauseClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_icon_name: match model.playback_state {
                            PlaybackState::Playing => Some("media-pause-symbolic"),
                            PlaybackState::Paused => Some("media-play-symbolic"),
                            PlaybackState::Stopped => Some("media-play-symbolic"),
                        },
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    #[watch]
                    set_sensitive: model.can_go_next,
                    connect_clicked[sender] => move |_| {
                        sender.input(MediaPlayerInput::NextClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: Some("media-skip-next-symbolic"),
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    #[watch]
                    set_sensitive: model.can_loop,
                    connect_clicked[sender] => move |_| {
                        sender.input(MediaPlayerInput::LoopClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_icon_name: match model.loop_mode {
                            LoopMode::None => Some("media-repeat-off-symbolic"),
                            LoopMode::Track => Some("media-repeat-once-symbolic"),
                            LoopMode::Playlist => Some("media-repeat-symbolic"),
                            LoopMode::Unsupported => Some("media-repeat-off-symbolic"),
                        },
                    },
                },
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut watcher_token = WatcherToken::new();

        let token = watcher_token.reset();

        spawn_media_player_watcher(
            &params.player,
            &sender,
            token,
            ||MediaPlayerCommandOutput::PlaybackStateChanged,
            ||MediaPlayerCommandOutput::MetaDataChanged,
            ||MediaPlayerCommandOutput::LoopModeChanged,
            ||MediaPlayerCommandOutput::ShuffleModeChanged,
            ||MediaPlayerCommandOutput::CapabilitiesChanged,
            |d|MediaPlayerCommandOutput::PositionChanged(d),
        );

        let position = params.player.position.get();
        let current_track_time = format_duration(position);
        let track_length = format_duration(
            params.player.metadata.length.get().unwrap_or(Duration::new(0, 0))
        );

        let can_shuffle = params.player.can_shuffle.get();
        let can_loop = params.player.can_loop.get();
        let can_go_next = params.player.can_go_next.get();
        let can_go_previous = params.player.can_go_previous.get();
        let can_play = params.player.can_play.get();
        let can_seek = params.player.can_seek.get();
        let playback_state = params.player.playback_state.get();
        let shuffle_mode = params.player.shuffle_mode.get();
        let loop_mode = params.player.loop_mode.get();

        let mut model = MediaPlayerModel {
            player: params.player,
            track_name: "".to_string(),
            track_name_scroll_source: None,
            artist_name: "".to_string(),
            artist_name_scroll_source: None,
            current_track_time,
            track_length,
            scale_value_changed_signal: None,
            position,
            pending_seek: None,
            can_shuffle,
            can_loop,
            can_go_next,
            can_go_previous,
            can_play,
            can_seek,
            playback_state,
            shuffle_mode,
            loop_mode,
        };

        let widgets = view_output!();

        model.track_name_scroll_source = Some(start_scroll(&widgets.track_scroll_window));
        model.artist_name_scroll_source = Some(start_scroll(&widgets.artist_scroll_window));

        model.scale_value_changed_signal = Some(setup_scale_seek(&widgets.scale, &sender));

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
            MediaPlayerInput::ScaleChanged(value) => {
                if let Some(length) = self.player.metadata.length.get() {
                    let position = length.mul_f64(value);
                    let now = Instant::now();
                    self.pending_seek = Some((position, now));
                    self.current_track_time = format_duration(position);
                    if let Some(scale_signal) = &self.scale_value_changed_signal {
                        widgets.scale.block_signal(scale_signal);
                        widgets.scale.set_value(value);
                        widgets.scale.unblock_signal(scale_signal);
                    }
                }
            }

            MediaPlayerInput::ScaleClicked(value) => {
                let player = self.player.clone();
                if let Some(length) = player.metadata.length.get() {
                    let new_position = length.mul_f64(value);
                    let now = Instant::now();
                    self.pending_seek = Some((new_position, now));
                    tokio::spawn(async move {
                        let _ = player.set_position(new_position).await;
                    });
                }
            }
            MediaPlayerInput::ShuffleClicked => {
                let current_mode = self.player.shuffle_mode.get();
                let new_mode = match current_mode {
                    ShuffleMode::On => ShuffleMode::Off,
                    ShuffleMode::Off => ShuffleMode::On,
                    ShuffleMode::Unsupported => ShuffleMode::Off,
                };

                self.shuffle_mode = new_mode;

                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.set_shuffle_mode(new_mode).await;
                });
            }
            MediaPlayerInput::LoopClicked => {
                let current_mode = self.player.loop_mode.get();
                let new_mode = match current_mode {
                    LoopMode::None => LoopMode::Playlist,
                    LoopMode::Track => LoopMode::None,
                    LoopMode::Playlist => LoopMode::Track,
                    LoopMode::Unsupported => LoopMode::None,
                };

                self.loop_mode = new_mode;

                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.set_loop_mode(new_mode).await;
                });
            }
            MediaPlayerInput::NextClicked => {
                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.next().await;
                });
            }
            MediaPlayerInput::PreviousClicked => {
                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.previous().await;
                });
            }
            MediaPlayerInput::PlayPauseClicked => {
                let current_mode = self.player.playback_state.get();
                let new_mode = match current_mode {
                    PlaybackState::Playing => PlaybackState::Paused,
                    PlaybackState::Paused => PlaybackState::Playing,
                    PlaybackState::Stopped => PlaybackState::Playing,
                };

                self.playback_state = new_mode;

                let player = self.player.clone();
                tokio::spawn(async move {
                    let _ = player.play_pause().await;
                });
            }
        }

        self.update_view(widgets, sender);
    }


    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            MediaPlayerCommandOutput::PlaybackStateChanged => {
                self.playback_state = self.player.playback_state.get();
            }
            MediaPlayerCommandOutput::MetaDataChanged => {
                let title = self.player.metadata.title.get();
                let artist = self.player.metadata.artist.get();

                if self.track_name != title {
                    self.track_name = title;
                    widgets.track_scroll_window.hadjustment().set_value(0.0);
                }

                if self.artist_name != artist {
                    self.artist_name = artist;
                    widgets.artist_scroll_window.hadjustment().set_value(0.0);
                }
            }
            MediaPlayerCommandOutput::LoopModeChanged => {
                self.loop_mode = self.player.loop_mode.get();
            }
            MediaPlayerCommandOutput::ShuffleModeChanged => {
                self.shuffle_mode = self.player.shuffle_mode.get();
            }
            MediaPlayerCommandOutput::CapabilitiesChanged => {
                self.can_loop = self.player.can_loop.get();
                self.can_shuffle = self.player.can_shuffle.get();
                self.can_go_next = self.player.can_go_next.get();
                self.can_go_previous = self.player.can_go_previous.get();
                self.can_play = self.player.can_play.get();
                self.can_seek = self.player.can_seek.get();
            }
            MediaPlayerCommandOutput::PositionChanged(position) => {
                self.position = position;

                if let Some((_, sent_at)) = self.pending_seek {
                    let timed_out = sent_at.elapsed() > Duration::from_secs(3);
                    if timed_out {
                        self.pending_seek = None;
                    } else {
                        // Still in the ignore window, don't update scale
                        self.update_view(widgets, sender);
                        return;
                    }
                }

                self.current_track_time = format_duration(position);
                if let Some(length) = self.player.metadata.length.get() {
                    self.track_length = format_duration(length);
                    if let Some(scale_signal) = &self.scale_value_changed_signal {
                        if !length.is_zero() {
                            widgets.scale.block_signal(scale_signal);
                            widgets.scale.set_value(position.as_secs_f64() / length.as_secs_f64());
                            widgets.scale.unblock_signal(scale_signal);
                        } else {
                            widgets.scale.set_value(0.0);
                        }
                    } else {
                        widgets.scale.set_value(0.0);
                    }
                } else {
                    widgets.scale.set_value(0.0);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

fn setup_scale_seek(
    scale: &gtk::Scale,
    sender: &ComponentSender<MediaPlayerModel>,
) -> glib::SignalHandlerId {
    let pending_source: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

    let sender = sender.clone();
    scale.connect_value_changed(move |scale| {
        // Cancel previous debounced seek
        if let Some(source_id) = pending_source.take() {
            source_id.remove();
        }

        let value = scale.value();

        // Immediately update display so it doesn't jump while dragging
        let _ = sender.input(MediaPlayerInput::ScaleChanged(value));

        let seek_sender = sender.clone();
        let pending = pending_source.clone();

        // Only actually seek after dragging stops
        let source_id = glib::timeout_add_local_once(Duration::from_millis(300), move || {
            pending.set(None);
            let _ = seek_sender.input(MediaPlayerInput::ScaleClicked(value));
        });

        pending_source.set(Some(source_id));
    })
}


fn start_scroll(scrolled_window: &gtk::ScrolledWindow) -> glib::SourceId {
    let state = Rc::new(Cell::new(ScrollState::PauseStart(0)));
    let scroll = scrolled_window.clone();
    glib::timeout_add_local(Duration::from_millis(30), move || {
        let adj = scroll.hadjustment();
        let max = adj.upper() - adj.page_size();
        if max <= 0.0 {
            return glib::ControlFlow::Continue;
        }
        match state.get() {
            ScrollState::PauseStart(n) => {
                if n >= 40 {
                    state.set(ScrollState::Scrolling);
                } else {
                    state.set(ScrollState::PauseStart(n + 1));
                }
            }
            ScrollState::Scrolling => {
                let current = adj.value();
                if current >= max {
                    state.set(ScrollState::PauseEnd(0));
                } else {
                    adj.set_value(current + 2.0);
                }
            }
            ScrollState::PauseEnd(n) => {
                if n >= 40 {
                    adj.set_value(0.0);
                    state.set(ScrollState::PauseStart(0));
                } else {
                    state.set(ScrollState::PauseEnd(n + 1));
                }
            }
        }
        glib::ControlFlow::Continue
    })
}

fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}
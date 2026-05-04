use crate::menus::menu_widgets::media_player::media_player::{MediaPlayerInit, MediaPlayerModel};
use okshell_services::media_service;
use okshell_utils::media::spawn_media_players_watcher;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};

pub(crate) struct MediaPlayersModel {
    player_controllers: Vec<Controller<MediaPlayerModel>>,
    active_player_name: String,
    previous_button_sensitive: bool,
    next_button_sensitive: bool,
    players_visible: bool,
}

#[derive(Debug)]
pub(crate) enum MediaPlayersInput {
    PreviousClicked,
    NextClicked,
    UpdateState,
}

#[derive(Debug)]
pub(crate) enum MediaPlayersOutput {}

pub(crate) struct MediaPlayersInit {}

#[derive(Debug)]
pub(crate) enum MediaPlayersCommandOutput {
    PlayersChanged,
    ActivePlayerChanged,
}

#[relm4::component(pub)]
impl Component for MediaPlayersModel {
    type CommandOutput = MediaPlayersCommandOutput;
    type Input = MediaPlayersInput;
    type Output = MediaPlayersOutput;
    type Init = MediaPlayersInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            #[watch]
            set_visible: model.players_visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "label-small-bold-variant",
                    #[watch]
                    set_label: model.active_player_name.as_str(),
                    set_hexpand: true,
                    set_xalign: 0.0
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    #[watch]
                    set_sensitive: model.previous_button_sensitive,
                    connect_clicked[sender] => move |_| {
                        sender.input(MediaPlayersInput::PreviousClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: Some("menu-left-symbolic"),
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    #[watch]
                    set_sensitive: model.next_button_sensitive,
                    connect_clicked[sender] => move |_| {
                        sender.input(MediaPlayersInput::NextClicked);
                    },

                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: Some("menu-right-symbolic"),
                    },
                },
            },

            #[name = "player_container"]
            gtk::Stack {
                set_transition_type: gtk::StackTransitionType::SlideLeftRight,
                set_transition_duration: 200,
                set_hexpand: true,
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_media_players_watcher(
            &sender,
            || MediaPlayersCommandOutput::PlayersChanged,
            || MediaPlayersCommandOutput::ActivePlayerChanged,
        );

        let players = media_service().player_list.get();

        let model = MediaPlayersModel {
            player_controllers: Vec::new(),
            active_player_name: "".to_string(),
            previous_button_sensitive: false,
            next_button_sensitive: false,
            players_visible: !players.is_empty(),
        };

        let widgets = view_output!();

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
            MediaPlayersInput::PreviousClicked => {
                let service = media_service();
                let players = service.player_list.get();
                let active = service.active_player.get();
                if let Some(active) = active
                    && let Some(idx) = players.iter().position(|p| p.id == active.id)
                    && idx > 0
                {
                    let prev_id = players[idx - 1].id.clone();
                    tokio::spawn(async move {
                        let _ = service.set_active_player(Some(prev_id)).await;
                    });
                }
            }
            MediaPlayersInput::NextClicked => {
                let service = media_service();
                let players = service.player_list.get();
                let active = service.active_player.get();
                if let Some(active) = active
                    && let Some(idx) = players.iter().position(|p| p.id == active.id)
                    && idx + 1 < players.len()
                {
                    let next_id = players[idx + 1].id.clone();
                    tokio::spawn(async move {
                        let _ = service.set_active_player(Some(next_id)).await;
                    });
                }
            }
            MediaPlayersInput::UpdateState => {
                let service = media_service();
                let players = service.player_list.get();
                let active_player = service.active_player.get();
                let active_id = active_player.as_ref().map(|p| &p.id);

                // Update button sensitivity
                if let Some(active) = &active_player {
                    self.active_player_name = active.identity.get();
                    if let Some(idx) = players.iter().position(|p| p.id == active.id) {
                        self.previous_button_sensitive = idx > 0;
                        self.next_button_sensitive = idx + 1 < players.len();
                    }
                } else {
                    self.previous_button_sensitive = false;
                    self.next_button_sensitive = false;
                }

                // Reveal active player, hide others
                for controller in &self.player_controllers {
                    if Some(&controller.model().player.id) == active_id {
                        widgets
                            .player_container
                            .set_visible_child(controller.widget());
                    }
                }
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MediaPlayersCommandOutput::PlayersChanged => {
                let service = media_service();
                let players = service.player_list.get();

                self.players_visible = !players.is_empty();

                // Remove controllers for players no longer in the list
                self.player_controllers.retain(|controller| {
                    let still_exists = players.iter().any(|p| p.id == controller.model().player.id);
                    if !still_exists {
                        widgets.player_container.remove(controller.widget());
                    }
                    still_exists
                });

                // Add controllers for new players
                for player in &players {
                    let already_exists = self
                        .player_controllers
                        .iter()
                        .any(|c| c.model().player.id == player.id);

                    if !already_exists {
                        let player_clone = player.clone();
                        let controller = MediaPlayerModel::builder()
                            .launch(MediaPlayerInit {
                                player: player_clone,
                            })
                            .detach();
                        widgets.player_container.add_child(controller.widget());
                        self.player_controllers.push(controller);
                    }
                }

                sender.input(MediaPlayersInput::UpdateState);
            }
            MediaPlayersCommandOutput::ActivePlayerChanged => {
                sender.input(MediaPlayersInput::UpdateState);
            }
        }

        self.update_view(widgets, sender);
    }
}

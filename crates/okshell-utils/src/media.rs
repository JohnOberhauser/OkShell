use futures::StreamExt;
use okshell_common::{watch, watch_cancellable};
use okshell_services::media_service;
use relm4::{Component, ComponentSender};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use wayle_media::core::player::Player;

pub fn spawn_media_players_watcher<C>(
    sender: &ComponentSender<C>,
    players_changed: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    active_player_changed: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = media_service();

    let players = service.player_list.clone();
    watch!(sender, [players.watch()], |out| {
        let _ = out.send(players_changed());
    });

    let active_player = service.active_player.clone();
    watch!(sender, [active_player.watch()], |out| {
        let _ = out.send(active_player_changed());
    });
}

pub fn spawn_media_player_watcher<C>(
    player: &Player,
    sender: &ComponentSender<C>,
    cancellation_token: CancellationToken,
    playback_state_changed: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    metadata_changed: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    loop_mode_changed: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    shuffle_mode_changed: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    capabilities_changed: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
    position_changed: impl Fn(Duration) -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let playback_state = player.playback_state.clone();
    let token = cancellation_token.clone();
    watch_cancellable!(sender, token, [playback_state.watch()], |out| {
        let _ = out.send(playback_state_changed());
    });

    let meta_data = player.metadata.clone();
    let token = cancellation_token.clone();
    watch_cancellable!(sender, token, [meta_data.watch()], |out| {
        let _ = out.send(metadata_changed());
    });

    let loop_mode = player.loop_mode.clone();
    let token = cancellation_token.clone();
    watch_cancellable!(sender, token, [loop_mode.watch()], |out| {
        let _ = out.send(loop_mode_changed());
    });

    let shuffle_mode = player.shuffle_mode.clone();
    let token = cancellation_token.clone();
    watch_cancellable!(sender, token, [shuffle_mode.watch()], |out| {
        let _ = out.send(shuffle_mode_changed());
    });

    let can_shuffle = player.can_shuffle.clone();
    let can_loop = player.can_loop.clone();
    let can_go_next = player.can_go_next.clone();
    let can_go_previous = player.can_go_previous.clone();
    let can_play = player.can_play.clone();
    let can_seek = player.can_seek.clone();
    let token = cancellation_token.clone();
    watch_cancellable!(
        sender,
        token,
        [
            can_shuffle.watch(),
            can_loop.watch(),
            can_go_next.watch(),
            can_go_previous.watch(),
            can_play.watch(),
            can_seek.watch(),
        ],
        |out| {
            let _ = out.send(capabilities_changed());
        }
    );

    let position_player = player.clone();
    let token = cancellation_token.clone();
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);
        let mut stream = Box::pin(position_player.position.watch());
        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                _ = token.cancelled() => break,
                Some(position) = stream.next() => {
                    let _ = out.send(position_changed(position));
                }
                else => break,
            }
        }
    });
}

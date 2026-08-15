use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;
use mpris_server::{
    Metadata, PlaybackStatus, Player, TrackId, Volume, zbus::Result,
};
use std::future::Future;
use std::sync::mpsc;

const BUS_NAME_SUFFIX: &str = "cosmicradio";
const TRACK_ID: &str = "/com/system76/CosmicRadio/Track";

/// Commands received from external MPRIS clients, forwarded to the app.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Play,
    Pause,
    PlayPause,
    Stop,
    SetVolume(f64),
    Raise,
}

/// Exposes COSMIC Radio to other applications via the MPRIS2 D-Bus interface
/// (`org.mpris.MediaPlayer2.cosmicradio`).
///
/// The `Player` handle from `mpris-server` is not `Send`, so it stays on the
/// main thread alongside the iced app. Its event loop is driven by calling
/// [`Mpris::tick`] from the app's message loop.
pub struct Mpris {
    player: Player,
    pool: LocalPool,
}

impl Mpris {
    /// Creates the MPRIS server on the session bus and starts its event loop.
    ///
    /// Must be called from within a Tokio runtime context (the iced app runs
    /// `init` and `update` inside one). Returns `None` if the session bus is
    /// unavailable or the bus name is already taken.
    pub fn new(commands: mpsc::Sender<Command>) -> Option<Self> {
        let player = tokio::runtime::Handle::current()
            .block_on(
                Player::builder(BUS_NAME_SUFFIX)
                    .identity("COSMIC Radio")
                    .desktop_entry("com.system76.CosmicRadio")
                    .supported_uri_schemes(["http", "https"])
                    .can_play(true)
                    .can_pause(true)
                    .can_go_next(false)
                    .can_go_previous(false)
                    .can_seek(false)
                    .can_control(true)
                    .playback_status(PlaybackStatus::Stopped)
                    .build(),
            )
            .ok()?;

        player.connect_play({
            let commands = commands.clone();
            move |_| {
                let _ = commands.send(Command::Play);
            }
        });
        player.connect_pause({
            let commands = commands.clone();
            move |_| {
                let _ = commands.send(Command::Pause);
            }
        });
        player.connect_play_pause({
            let commands = commands.clone();
            move |_| {
                let _ = commands.send(Command::PlayPause);
            }
        });
        player.connect_stop({
            let commands = commands.clone();
            move |_| {
                let _ = commands.send(Command::Stop);
            }
        });
        player.connect_set_volume({
            let commands = commands.clone();
            move |_, volume: Volume| {
                let _ = commands.send(Command::SetVolume(volume));
            }
        });
        player.connect_raise(move |_| {
            let _ = commands.send(Command::Raise);
        });

        let mut pool = LocalPool::new();
        let _ = pool.spawner().spawn_local(player.run());
        pool.run_until_stalled();

        Some(Self { player, pool })
    }

    /// Drives the MPRIS event loop so external method calls are handled.
    pub fn tick(&mut self) {
        self.pool.run_until_stalled();
    }

    /// Publishes the now-playing metadata and marks playback as active.
    pub fn set_playing(
        &self,
        title: String,
        artist: Option<String>,
        url: String,
        art_url: Option<String>,
    ) {
        let mut builder = Metadata::builder()
            .title(title)
            .url(url)
            .trackid(TrackId::try_from(TRACK_ID).unwrap());
        if let Some(artist) = artist {
            builder = builder.artist([artist]);
        }
        if let Some(art_url) = art_url {
            builder = builder.art_url(art_url);
        }

        self.run_async(self.player.set_metadata(builder.build()));
        self.run_async(self.player.set_playback_status(PlaybackStatus::Playing));
    }

    /// Marks playback as stopped.
    pub fn set_stopped(&self) {
        self.run_async(self.player.set_playback_status(PlaybackStatus::Stopped));
    }

    /// Mirrors the app volume to the MPRIS `Volume` property.
    pub fn set_volume(&self, volume: f64) {
        self.run_async(self.player.set_volume(volume));
    }

    /// Polls an async MPRIS operation to completion on the current Tokio
    /// runtime, if one is available.
    fn run_async(&self, future: impl Future<Output = Result<()>>) {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let _ = handle.block_on(future);
        }
    }
}

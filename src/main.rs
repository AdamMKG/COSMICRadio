use cosmic::{
    Element, Task, app,
    iced::{
        self, Alignment,
        platform_specific::shell::commands::popup::{destroy_popup, get_popup},
        widget::{column, row, image},
        window, Subscription,
    },
    widget::{text, scrollable, button, divider, slider},
};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use serde::Deserialize;
use gstreamer_play::Play;
use sha2::{Sha256, Digest};

const APP_ID: &str = "com.system76.CosmicRadio";

#[derive(Debug, Clone, Deserialize)]
struct Station {
    name: String,
    url: String,
    artwork: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    stations: Vec<Station>,
}

impl Default for Config {
    fn default() -> Self {
        Self { stations: vec![] }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("cosmic-radio")
        .join("stations.toml")
}

fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        }
    }
    Config::default()
}

fn ensure_config() -> PathBuf {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if !path.exists() {
        let default_config = r#"[[stations]]
name = "SomaFM Groove Salad"
url = "https://somafm.com/groovesalad256.mp3"
artwork = "https://somafm.com/img3/groovesalad-400.png"

[[stations]]
name = "SomaFM Drone Zone"
url = "https://somafm.com/dronezone256.mp3"
artwork = "https://somafm.com/img3/dronezone-400.png"
"#;
        let _ = fs::write(&path, default_config);
    }
    path
}

struct RadioApp {
    core: cosmic::app::Core,
    popup: Option<window::Id>,
    stations: Vec<Station>,
    current_station: Option<usize>,
    is_playing: bool,
    volume: f64,
    player: Option<Play>,
    station_artwork: HashMap<usize, PathBuf>,
}

#[derive(Debug, Clone)]
enum Message {
    TogglePopup,
    Closed(window::Id),
    SelectStation(usize),
    TogglePlayback,
    SetVolume(f64),
    EditStations,
}

impl cosmic::Application for RadioApp {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(core: cosmic::app::Core, _flags: ()) -> (Self, app::Task<Self::Message>) {
        let _path = ensure_config();
        let config = load_config();
        let stations = config.stations;

        gstreamer::init().expect("Failed to initialize GStreamer");

        let player: Play = Play::new(None::<gstreamer_play::PlayVideoRenderer>);

        (
            Self {
                core,
                popup: None,
                stations,
                current_station: None,
                is_playing: false,
                volume: 0.5,
                player: Some(player),
                station_artwork: HashMap::new(),
            },
            Task::none(),
        )
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::Closed(id))
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                } else {
                    let new_id = window::Id::unique();
                    self.popup.replace(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    // Preload artwork for all stations
                    for i in 0..self.stations.len() {
                        self.load_artwork(i);
                    }

                    return get_popup(popup_settings);
                }
            }
            Message::Closed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::SelectStation(index) => {
                self.current_station = Some(index);
                self.is_playing = true;
                self.start_playback();
                self.load_artwork(index);
            }
            Message::TogglePlayback => {
                self.is_playing = !self.is_playing;
                if self.is_playing {
                    self.start_playback();
                } else {
                    self.stop_playback();
                }
            }
            Message::SetVolume(volume) => {
                self.volume = volume;
                if let Some(player) = &self.player {
                    player.set_volume(volume);
                }
            }
            Message::EditStations => {
                let path = config_path();
                let _ = std::process::Command::new("xdg-open")
                    .arg(path)
                    .spawn();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.core
            .applet
            .icon_button("radio_icon")
            .on_press_down(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        if matches!(self.popup, Some(p) if p == id) {
            let current_station_name = self.current_station
                .and_then(|i| self.stations.get(i))
                .map(|s| s.name.as_str())
                .unwrap_or("No station selected");

            let current_artwork = self.current_station
                .and_then(|i| self.station_artwork.get(&i));

            let play_button = if self.is_playing {
                button::text("Stop").on_press(Message::TogglePlayback)
            } else {
                button::text("Play").on_press(Message::TogglePlayback)
            };

            let stations_list = self.stations.iter().enumerate().map(|(i, station)| {
                let station_art = self.station_artwork.get(&i);
                let btn_content = row![
                    Self::artwork_image(station_art, 32),
                    text::body(&station.name),
                ].spacing(8).align_y(Alignment::Center);

                button::custom(btn_content)
                    .on_press(Message::SelectStation(i))
                    .width(iced::Length::Fill)
                    .padding(8)
                    .into()
            });

            let title_row = row![
                text::title3("COSMIC Radio"),
                iced::widget::Space::new().width(iced::Length::Fill),
                Self::artwork_image(current_artwork, 32),
            ].align_y(Alignment::Center);

            let now_playing_row = row![
                Self::artwork_image(current_artwork, 48),
                text::body(current_station_name).size(14),
            ].spacing(8).align_y(Alignment::Center);

            let content = column![
                title_row,
                now_playing_row,
                divider::horizontal::default(),
                row![
                    play_button.width(iced::Length::Fill),
                ].spacing(8),
                text::body("Volume").size(12),
                slider(0.0..=1.0, self.volume, Message::SetVolume).step(0.01),
                divider::horizontal::default(),
                scrollable(
                    column(stations_list)
                        .spacing(4)
                        .padding(8)
                )
                .height(iced::Length::Fixed(300.0)),
                divider::horizontal::default(),
                button::text("Edit Stations")
                    .on_press(Message::EditStations)
                    .width(iced::Length::Fill),
            ]
            .align_x(Alignment::Start)
            .padding(8)
            .spacing(8);

            self.core.applet.popup_container(content).into()
        } else {
            column![].into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn style(&self) -> Option<iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl RadioApp {
    fn start_playback(&mut self) {
        if let Some(index) = self.current_station {
            if let Some(station) = self.stations.get(index) {
                if let Some(player) = &self.player {
                    player.set_uri(Some(&station.url));
                    player.play();
                    player.set_volume(self.volume);
                    eprintln!("Playing: {}", station.name);
                }
            }
        }
    }

    fn stop_playback(&mut self) {
        if let Some(player) = &self.player {
            player.stop();
        }
    }

    fn artwork_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("cosmic-radio")
            .join("artwork")
    }

    fn cache_filename(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let result = hasher.finalize();
        format!("{:x}.png", result)
    }

    fn is_cache_fresh(path: &PathBuf) -> bool {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed < Duration::from_secs(5 * 24 * 60 * 60);
                }
            }
        }
        false
    }

    fn artwork_image(artwork_path: Option<&PathBuf>, size: u16) -> Element<'static, Message> {
        if let Some(path) = artwork_path {
            if path.exists() {
                image(image::Handle::from_path(path.clone()))
                    .width(iced::Length::Fixed(size as f32))
                    .height(iced::Length::Fixed(size as f32))
                    .into()
            } else {
                iced::widget::Space::new().width(iced::Length::Fixed(size as f32)).into()
            }
        } else {
            iced::widget::Space::new().width(iced::Length::Fixed(size as f32)).into()
        }
    }

    fn load_artwork(&mut self, index: usize) {
        if let Some(station) = self.stations.get(index) {
            let artwork_url = station.artwork.clone().unwrap_or_else(|| {
                Self::derive_artwork_url(&station.url)
            });

            if artwork_url.is_empty() {
                return;
            }

            let cache_dir = Self::artwork_cache_dir();
            let cache_file = cache_dir.join(Self::cache_filename(&artwork_url));

            // Check cache first
            if cache_file.exists() && Self::is_cache_fresh(&cache_file) {
                if !self.station_artwork.contains_key(&index) {
                    self.station_artwork.insert(index, cache_file);
                }
                return;
            }

            // Download artwork in background
            let cache_dir_clone = cache_dir.clone();
            let cache_file_clone = cache_file.clone();
            tokio::spawn(async move {
                if let Err(e) = std::fs::create_dir_all(&cache_dir_clone) {
                    eprintln!("Failed to create cache dir: {}", e);
                    return;
                }

                let client = reqwest::Client::new();
                match client.get(&artwork_url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.bytes().await {
                                Ok(bytes) => {
                                    if let Err(e) = std::fs::write(&cache_file_clone, &bytes) {
                                        eprintln!("Failed to write cache: {}", e);
                                    }
                                }
                                Err(e) => eprintln!("Failed to read response: {}", e),
                            }
                        } else {
                            eprintln!("HTTP {}", response.status());
                        }
                    }
                    Err(e) => eprintln!("Request failed: {}", e),
                }
            });

            // Store the path immediately (file will be downloaded)
            self.station_artwork.insert(index, cache_file);
        }
    }

    fn derive_artwork_url(stream_url: &str) -> String {
        if stream_url.contains("somafm.com") {
            let station_name = stream_url
                .split('/')
                .next_back()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("");
            if !station_name.is_empty() {
                return format!("https://somafm.com/img3/{}-400.png", station_name);
            }
        }
        String::new()
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<RadioApp>(())
}

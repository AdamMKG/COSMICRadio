use cosmic::{
    app,
    iced::{
        self,
        platform_specific::shell::commands::popup::{destroy_popup, get_popup},
        widget::{column, container, image, row},
        window, Alignment, Subscription,
    },
    widget::{button, divider, scrollable, slider, text},
    Element, Task,
};
use gstreamer::{tags::Artist, tags::Title, BusSyncReply, MessageView};
use gstreamer::prelude::*;
use gstreamer_play::Play;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

const APP_ID: &str = "com.system76.CosmicRadio";
const MARQUEE_MAX_CHARS: usize = 29;
const MARQUEE_END_PAUSE_TICKS: f64 = 20.0;

#[derive(Debug, Clone, Deserialize)]
struct Station {
    name: String,
    url: String,
    artwork: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct StationGroup {
    name: String,
    stations: Vec<Station>,
}

#[derive(Debug, Deserialize)]
struct OldConfig {
    stations: Vec<Station>,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    groups: Vec<StationGroup>,
}

impl Default for Config {
    fn default() -> Self {
        Self { groups: vec![] }
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
            if let Ok(value) = content.parse::<toml::Value>() {
                if value.get("groups").is_some() {
                    if let Ok(config) = toml::from_str::<Config>(&content) {
                        return config;
                    }
                } else if value.get("stations").is_some() {
                    if let Ok(old_config) = toml::from_str::<OldConfig>(&content) {
                        return Config {
                            groups: vec![StationGroup {
                                name: "Ungrouped".to_string(),
                                stations: old_config.stations,
                            }],
                        };
                    }
                }
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
        let default_path = PathBuf::from("/usr/share/cosmic-radio/stations.toml");
        if default_path.exists() {
            let _ = fs::copy(&default_path, &path);
        }
    }
    path
}

fn resolve_pls(content: &str) -> Option<(String, String)> {
    let mut url = None;
    let mut title = None;
    for line in content.lines() {
        let line = line.trim();
        let lower = line.to_lowercase();
        if lower.starts_with("file1=") {
            url = Some(line[6..].to_string());
        } else if lower.starts_with("title1=") {
            title = Some(line[7..].to_string());
        }
        if url.is_some() && title.is_some() {
            break;
        }
    }
    url.map(|u| (u, title.unwrap_or_default()))
}

struct RadioApp {
    core: cosmic::app::Core,
    popup: Option<window::Id>,
    groups: Vec<StationGroup>,
    flat_stations: Vec<Station>,
    current_station: Option<usize>,
    is_playing: bool,
    volume: f64,
    player: Option<Play>,
    station_artwork: HashMap<usize, PathBuf>,
    group_collapsed: Vec<bool>,
    now_playing: String,
    scroll_offset: f64,
    latest_metadata: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
enum Message {
    TogglePopup,
    Closed(window::Id),
    SelectStation(usize),
    TogglePlayback,
    SetVolume(f64),
    EditStations,
    ToggleGroupCollapse(usize),
    MarqueeTick,
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
        let groups = config.groups;
        let flat_stations: Vec<Station> = groups.iter().flat_map(|g| g.stations.clone()).collect();
        let group_collapsed = vec![false; groups.len()];

        gstreamer::init().expect("Failed to initialize GStreamer");

        let player: Play = Play::new(None::<gstreamer_play::PlayVideoRenderer>);
        let latest_metadata = Arc::new(Mutex::new(None::<String>));

        let pipeline = player.pipeline();
        if let Some(bus) = pipeline.bus() {
            let md = latest_metadata.clone();
            bus.set_sync_handler(move |_bus: &gstreamer::Bus, msg: &gstreamer::Message| {
                if let MessageView::Tag(tag_msg) = msg.view() {
                    let tags = tag_msg.tags();
                    let artist = tags.get::<Artist>().map(|v| v.get().to_string());
                    let title = tags.get::<Title>().map(|v| v.get().to_string());

                    let display = match (artist, title) {
                        (Some(a), Some(t)) => Some(format!("{} - {}", a, t)),
                        (None, Some(t)) => Some(t),
                        (Some(a), None) => Some(a),
                        _ => None,
                    };

                    if let Some(s) = display {
                        if let Ok(mut guard) = md.lock() {
                            *guard = Some(s);
                        }
                    }
                }
                BusSyncReply::Pass
            });
        }

        (
            Self {
                core,
                popup: None,
                groups,
                flat_stations,
                current_station: None,
                is_playing: false,
                volume: 0.5,
                player: Some(player),
                station_artwork: HashMap::new(),
                group_collapsed,
                now_playing: String::new(),
                scroll_offset: 0.0,
                latest_metadata,
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

                    for i in 0..self.flat_stations.len() {
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
                self.scroll_offset = 0.0;
                self.start_playback();
                self.load_artwork(index);
            }
            Message::TogglePlayback => {
                self.is_playing = !self.is_playing;
                if self.is_playing {
                    self.scroll_offset = 0.0;
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

                let editor_cmd = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR"));

                match editor_cmd {
                    Ok(cmd) => {
                        let _ = std::process::Command::new(cmd).arg(path).spawn();
                    }
                    Err(_) => {
                        let status = std::process::Command::new("cosmic-edit").arg(&path).spawn();
                        if status.is_err() {
                            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                        }
                    }
                }
            }
            Message::ToggleGroupCollapse(index) => {
                if index < self.group_collapsed.len() {
                    self.group_collapsed[index] = !self.group_collapsed[index];
                }
            }
            Message::MarqueeTick => {
                if let Ok(mut md) = self.latest_metadata.lock() {
                    if let Some(new_title) = md.take() {
                        self.now_playing = new_title;
                        self.scroll_offset = 0.0;
                    }
                }

                if !self.now_playing.is_empty() {
                    let char_count = self.now_playing.chars().count();
                    if char_count > MARQUEE_MAX_CHARS {
                        let max_offset = (char_count - MARQUEE_MAX_CHARS) as f64;
                        let total_cycle = max_offset + MARQUEE_END_PAUSE_TICKS;
                        if self.scroll_offset >= total_cycle {
                            self.scroll_offset = 0.0;
                        } else if self.scroll_offset < max_offset {
                            self.scroll_offset += 1.0;
                        } else {
                            self.scroll_offset += 1.0;
                        }
                    }
                }
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
            let current_station_name = self
                .current_station
                .and_then(|i| self.flat_stations.get(i))
                .map(|s| s.name.as_str())
                .unwrap_or("No station selected");

            let current_artwork = self
                .current_station
                .and_then(|i| self.station_artwork.get(&i));

            let play_button = if self.is_playing {
                button::text("Stop").on_press(Message::TogglePlayback)
            } else {
                button::text("Play").on_press(Message::TogglePlayback)
            };

            let mut now_playing_elements: Vec<Element<'_, Message>> = Vec::new();

            if let Some(art_path) = current_artwork {
                now_playing_elements.push(Self::artwork_image(Some(art_path), 48));
            }

            let display_text = if self.now_playing.is_empty() {
                current_station_name.to_string()
            } else {
                self.now_playing.clone()
            };

            let char_count = display_text.chars().count();
            let marquee_text: String = if char_count > MARQUEE_MAX_CHARS {
                let offset = (self.scroll_offset as usize).min(char_count - MARQUEE_MAX_CHARS);
                display_text.chars().skip(offset).take(MARQUEE_MAX_CHARS).collect()
            } else {
                display_text
            };

            let now_playing_element: Element<'_, Message> = container(
                text::body(marquee_text).size(14),
            )
            .width(iced::Length::Fixed(220.0))
            .clip(true)
            .into();

            now_playing_elements.push(now_playing_element);

            let now_playing_row = row(now_playing_elements)
                .spacing(8)
                .align_y(Alignment::Center);

            let mut ui_elements: Vec<Element<'_, Message>> = Vec::new();
            let mut flat_idx: usize = 0;

            for (group_idx, group) in self.groups.iter().enumerate() {
                let collapse_icon = if self.group_collapsed[group_idx] {
                    "▶"
                } else {
                    "▼"
                };
                let group_header = row![
                    text::body(&group.name).size(12),
                    iced::widget::Space::new().width(iced::Length::Fill),
                    button::custom(text::body(collapse_icon))
                        .on_press(Message::ToggleGroupCollapse(group_idx))
                        .padding(4)
                        .width(iced::Length::Shrink),
                ]
                .align_y(Alignment::Center)
                .spacing(8);
                ui_elements.push(group_header.into());

                if !self.group_collapsed[group_idx] {
                    for station in &group.stations {
                        let station_art = self.station_artwork.get(&flat_idx);
                        let station_btn = button::custom(
                            row![
                                Self::artwork_image(station_art, 32),
                                text::body(&station.name),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::SelectStation(flat_idx))
                        .width(iced::Length::Fill)
                        .padding(8);
                        ui_elements.push(station_btn.into());
                        flat_idx += 1;
                    }
                } else {
                    flat_idx += group.stations.len();
                }
            }

            let content = column![
                now_playing_row,
                divider::horizontal::default(),
                row![play_button.width(iced::Length::Fill),].spacing(8),
                text::body("Volume").size(12),
                slider(0.0..=1.0, self.volume, Message::SetVolume).step(0.01),
                divider::horizontal::default(),
                scrollable(column(ui_elements).spacing(4).padding([4, 12, 4, 8]))
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
        iced::time::every(Duration::from_millis(50)).map(|_| Message::MarqueeTick)
    }

    fn style(&self) -> Option<iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl RadioApp {
    fn resolve_pls_url(url: &str) -> Option<(String, String)> {
        let content = if url.starts_with("http://") || url.starts_with("https://") {
            reqwest::blocking::get(url).ok()?.text().ok()?
        } else {
            fs::read_to_string(url).ok()?
        };
        resolve_pls(&content)
    }

    fn start_playback(&mut self) {
        if let Some(index) = self.current_station {
            if let Some(station) = self.flat_stations.get(index) {
                if let Some(player) = &self.player {
                    let (stream_url, display_name) = if station.url.ends_with(".pls") {
                        Self::resolve_pls_url(&station.url)
                            .map(|(u, n)| (u, n))
                            .unwrap_or_else(|| (station.url.clone(), station.name.clone()))
                    } else {
                        (station.url.clone(), station.name.clone())
                    };

                    player.set_uri(Some(&stream_url));
                    player.play();
                    player.set_volume(self.volume);
                    self.now_playing = display_name;
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
                iced::widget::Space::new()
                    .width(iced::Length::Fixed(size as f32))
                    .into()
            }
        } else {
            iced::widget::Space::new()
                .width(iced::Length::Fixed(size as f32))
                .into()
        }
    }

    fn load_artwork(&mut self, index: usize) {
        if let Some(station) = self.flat_stations.get(index) {
            let artwork_url = match &station.artwork {
                Some(url) if !url.is_empty() => url.clone(),
                _ => return,
            };

            let cache_dir = Self::artwork_cache_dir();
            let cache_file = cache_dir.join(Self::cache_filename(&artwork_url));

            if cache_file.exists() && Self::is_cache_fresh(&cache_file) {
                if !self.station_artwork.contains_key(&index) {
                    self.station_artwork.insert(index, cache_file);
                }
                return;
            }

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

            self.station_artwork.insert(index, cache_file);
        }
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<RadioApp>(())
}

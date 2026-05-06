use cosmic::{
    app,
    iced::{
        self,
        platform_specific::shell::commands::popup::{destroy_popup, get_popup},
        widget::{column, image, row},
        window, Alignment, Subscription,
    },
    widget::{button, divider, scrollable, slider, text},
    Element, Task,
};
use gstreamer_play::Play;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const APP_ID: &str = "com.system76.CosmicRadio";

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
            // Try parsing as TOML value to check structure
            if let Ok(value) = content.parse::<toml::Value>() {
                // New format: has "groups" key
                if value.get("groups").is_some() {
                    if let Ok(config) = toml::from_str::<Config>(&content) {
                        return config;
                    }
                }
                // Old format: has "stations" key (flat list)
                else if value.get("stations").is_some() {
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
        let default_config = r#"[[groups]]
name = "SomaFM"

[[groups.stations]]
name = "Beat Blender"
url = "https://somafm.com/beatblender256.mp3"
artwork = "https://somafm.com/img3/beatblender-400.png"

[[groups.stations]]
name = "Black Rock FM"
url = "https://somafm.com/blackrockfm256.mp3"
artwork = "https://somafm.com/img3/blackrockfm-400.png"

[[groups.stations]]
name = "Boot Liquor"
url = "https://somafm.com/bootliquor256.mp3"
artwork = "https://somafm.com/img3/bootliquor-400.png"

[[groups.stations]]
name = "Bossa Beyond"
url = "https://somafm.com/bossabeyond256.mp3"
artwork = "https://somafm.com/img3/bossabeyond-400.png"

[[groups.stations]]
name = "Chillits"
url = "https://somafm.com/chillits256.mp3"
artwork = "https://somafm.com/img3/chillits-400.png"

[[groups.stations]]
name = "Cliqhop IDM"
url = "https://somafm.com/cliqhopidm256.mp3"
artwork = "https://somafm.com/img3/cliqhopidm-400.png"

[[groups.stations]]
name = "Covers"
url = "https://somafm.com/covers256.mp3"
artwork = "https://somafm.com/img3/covers-400.png"

[[groups.stations]]
name = "Dark Zone"
url = "https://somafm.com/darkzone256.mp3"
artwork = "https://somafm.com/img3/darkzone-400.png"

[[groups.stations]]
name = "Deep Space One"
url = "https://somafm.com/deepspaceone256.mp3"
artwork = "https://somafm.com/img3/deepspaceone-400.png"

[[groups.stations]]
name = "DEF CON Radio"
url = "https://somafm.com/defconradio256.mp3"
artwork = "https://somafm.com/img3/defconradio-400.png"

[[groups.stations]]
name = "Department Store Christmas"
url = "https://somafm.com/departmentstorechristmas256.mp3"
artwork = "https://somafm.com/img3/departmentstorechristmas-400.png"

[[groups.stations]]
name = "Digitalis"
url = "https://somafm.com/digitalis256.mp3"
artwork = "https://somafm.com/img3/digitalis-400.png"

[[groups.stations]]
name = "Doomed"
url = "https://somafm.com/doomed256.mp3"
artwork = "https://somafm.com/img3/doomed-400.png"

[[groups.stations]]
name = "Drone Zone"
url = "https://somafm.com/dronezone256.mp3"
artwork = "https://somafm.com/img3/dronezone-400.png"

[[groups.stations]]
name = "Dubstep Beyond"
url = "https://somafm.com/dubstepbeyond256.mp3"
artwork = "https://somafm.com/img3/dubstepbeyond-400.png"

[[groups.stations]]
name = "Fluid"
url = "https://somafm.com/fluid256.mp3"
artwork = "https://somafm.com/img3/fluid-400.png"

[[groups.stations]]
name = "Folk Forward"
url = "https://somafm.com/folkforward256.mp3"
artwork = "https://somafm.com/img3/folkforward-400.png"

[[groups.stations]]
name = "Groove Salad"
url = "https://somafm.com/groovesalad256.mp3"
artwork = "https://somafm.com/img3/groovesalad-400.png"

[[groups.stations]]
name = "Groove Salad Classic"
url = "https://somafm.com/groovesaladclassic256.mp3"
artwork = "https://somafm.com/img3/groovesaladclassic-400.png"

[[groups.stations]]
name = "Heavyweight Reggae"
url = "https://somafm.com/heavyweightreggae256.mp3"
artwork = "https://somafm.com/img3/heavyweightreggae-400.png"

[[groups.stations]]
name = "Iceland Airwaves"
url = "https://somafm.com/icelandairwaves256.mp3"
artwork = "https://somafm.com/img3/icelandairwaves-400.png"

[[groups.stations]]
name = "Illinois Street Lounge"
url = "https://somafm.com/illinoisstreetlounge256.mp3"
artwork = "https://somafm.com/img3/illinoisstreetlounge-400.png"

[[groups.stations]]
name = "Indie Pop Rocks"
url = "https://somafm.com/indiepoprocks256.mp3"
artwork = "https://somafm.com/img3/indiepoprocks-400.png"

[[groups.stations]]
name = "In Sound"
url = "https://somafm.com/insound256.mp3"
artwork = "https://somafm.com/img3/insound-400.png"

[[groups.stations]]
name = "Jolly Ol' Soul"
url = "https://somafm.com/jollyolsoul256.mp3"
artwork = "https://somafm.com/img3/jollyolsoul-400.png"

[[groups.stations]]
name = "Left Coast 70s"
url = "https://somafm.com/leftcoast70s256.mp3"
artwork = "https://somafm.com/img3/leftcoast70s-400.png"

[[groups.stations]]
name = "Lush"
url = "https://somafm.com/lush256.mp3"
artwork = "https://somafm.com/img3/lush-400.png"

[[groups.stations]]
name = "Metal Detector"
url = "https://somafm.com/metaldetector256.mp3"
artwork = "https://somafm.com/img3/metaldetector-400.png"

[[groups.stations]]
name = "Mission Control"
url = "https://somafm.com/missioncontrol256.mp3"
artwork = "https://somafm.com/img3/missioncontrol-400.png"

[[groups.stations]]
name = "n5MD Radio"
url = "https://somafm.com/n5mdradio256.mp3"
artwork = "https://somafm.com/img3/n5mdradio-400.png"

[[groups.stations]]
name = "PopTron"
url = "https://somafm.com/poptron256.mp3"
artwork = "https://somafm.com/img3/poptron-400.png"

[[groups.stations]]
name = "Secret Agent"
url = "https://somafm.com/secretagent256.mp3"
artwork = "https://somafm.com/img3/secretagent-400.png"

[[groups.stations]]
name = "Seven Inch Soul"
url = "https://somafm.com/seveninchsoul256.mp3"
artwork = "https://somafm.com/img3/seveninchsoul-400.png"

[[groups.stations]]
name = "SF 10-33"
url = "https://somafm.com/sf1033.mp3"
artwork = "https://somafm.com/img3/sf1033-400.png"

[[groups.stations]]
name = "SF Police Scanner"
url = "https://somafm.com/sfpolicescanner256.mp3"
artwork = "https://somafm.com/img3/sfpolicescanner-400.png"

[[groups.stations]]
name = "SomaFM Live"
url = "https://somafm.com/somafmlive256.mp3"
artwork = "https://somafm.com/img3/somafmlive-400.png"

[[groups.stations]]
name = "Sonic Universe"
url = "https://somafm.com/sonicuniverse256.mp3"
artwork = "https://somafm.com/img3/sonicuniverse-400.png"

[[groups.stations]]
name = "Space Station Soma"
url = "https://somafm.com/spacestationsoma256.mp3"
artwork = "https://somafm.com/img3/spacestationsoma-400.png"

[[groups.stations]]
name = "Suburbs of Goa"
url = "https://somafm.com/suburbsofgoa256.mp3"
artwork = "https://somafm.com/img3/suburbsofgoa-400.png"

[[groups.stations]]
name = "Synphaera"
url = "https://somafm.com/synphaera256.mp3"
artwork = "https://somafm.com/img3/synphaera-400.png"

[[groups.stations]]
name = "The Trip"
url = "https://somafm.com/thetrip256.mp3"
artwork = "https://somafm.com/img3/thetrip-400.png"

[[groups.stations]]
name = "Thistle Radio"
url = "https://somafm.com/thistleradio256.mp3"
artwork = "https://somafm.com/img3/thistleradio-400.png"

[[groups.stations]]
name = "Tiki Time"
url = "https://somafm.com/tikitime256.mp3"
artwork = "https://somafm.com/img3/tikitime-400.png"

[[groups.stations]]
name = "Underground 80s"
url = "https://somafm.com/underground80s256.mp3"
artwork = "https://somafm.com/img3/underground80s-400.png"

[[groups.stations]]
name = "Vaporwaves"
url = "https://somafm.com/vaporwaves256.mp3"
artwork = "https://somafm.com/img3/vaporwaves-400.png"

[[groups.stations]]
name = "Xmas in Frisko"
url = "https://somafm.com/xmasinfrisko256.mp3"
artwork = "https://somafm.com/img3/xmasinfrisko-400.png"

[[groups.stations]]
name = "Xmas Lounge"
url = "https://somafm.com/xmaslounge256.mp3"
artwork = "https://somafm.com/img3/xmaslounge-400.png"

[[groups.stations]]
name = "Xmas Rocks"
url = "https://somafm.com/xmasrocks256.mp3"
artwork = "https://somafm.com/img3/xmasrocks-400.png"
"#;
        let _ = fs::write(&path, default_config);
    }
    path
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

        // Flatten stations for playback indexing
        let flat_stations: Vec<Station> = groups
            .iter()
            .flat_map(|g| g.stations.clone())
            .collect();

        // Initialize all groups as expanded (not collapsed)
        let group_collapsed = vec![false; groups.len()];

        gstreamer::init().expect("Failed to initialize GStreamer");

        let player: Play = Play::new(None::<gstreamer_play::PlayVideoRenderer>);

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
                let _ = std::process::Command::new("xdg-open").arg(path).spawn();
            }
            Message::ToggleGroupCollapse(index) => {
                if index < self.group_collapsed.len() {
                    self.group_collapsed[index] = !self.group_collapsed[index];
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

            // Build now_playing row elements conditionally
            let mut now_playing_elements: Vec<Element<'_, Message>> = Vec::new();

            // Only add artwork if we have a valid cached path
            if let Some(art_path) = current_artwork {
                now_playing_elements.push(Self::artwork_image(Some(art_path), 48));
            }

            // Always add the station name text
            now_playing_elements.push(text::body(current_station_name).size(14).into());

            let now_playing_row = row(now_playing_elements)
                .spacing(8)
                .align_y(Alignment::Center);

            let title_row = row![
                text::title3("COSMIC Radio"),
                iced::widget::Space::new().width(iced::Length::Fill),
            ]
            .align_y(Alignment::Center);

            // Build UI elements for groups and stations
            let mut ui_elements: Vec<Element<'_, Message>> = Vec::new();
            let mut flat_idx: usize = 0; // Tracks index into flat_stations

            for (group_idx, group) in self.groups.iter().enumerate() {
                // Group header with collapse toggle
                let collapse_icon = if self.group_collapsed[group_idx] { "▶" } else { "▼" };
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

                // Add stations if group is expanded
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
                    flat_idx += group.stations.len(); // Skip stations in collapsed group
                }
            }

            let content = column![
                title_row,
                now_playing_row,
                divider::horizontal::default(),
                row![play_button.width(iced::Length::Fill),].spacing(8),
                text::body("Volume").size(12),
                slider(0.0..=1.0, self.volume, Message::SetVolume).step(0.01),
                divider::horizontal::default(),
                scrollable(column(ui_elements).spacing(4).padding(8))
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
            if let Some(station) = self.flat_stations.get(index) {
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
            let artwork_url = station
                .artwork
                .clone()
                .unwrap_or_else(|| Self::derive_artwork_url(&station.url));

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

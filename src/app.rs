use crate::artwork::ArtworkCache;
use crate::audio::AudioBackend;
use crate::config::{ConfigManager, Station};
use cosmic::{
    app,
    iced::{
        self,
        platform_specific::shell::commands::popup::{destroy_popup, get_popup},
        widget::{column, container, image, row, svg, text_input, tooltip},
        window, Alignment, Subscription,
    },
    widget::{button, divider, scrollable, slider, text},
    Element, Task,
};
use std::path::PathBuf;
use std::time::Duration;

const MARQUEE_MAX_CHARS: usize = 33;
const MARQUEE_SCROLL_INTERVAL: u32 = 3;
const MARQUEE_START_PAUSE_TICKS: f64 = 60.0;
const MARQUEE_END_PAUSE_TICKS: f64 = 60.0;

const PLAY_SVG: &[u8] = include_bytes!("../data/play_button.svg");
const STOP_SVG: &[u8] = include_bytes!("../data/stop_button.svg");
const ADD_SVG: &[u8] = include_bytes!("../data/add_station.svg");
const ARTWORK_PLACEHOLDER: &[u8] = include_bytes!("../data/cosmic-broadcast-mic-symbolic.svg");

pub struct RadioApp {
    core: cosmic::app::Core,
    popup: Option<window::Id>,
    config: ConfigManager,
    current_station: Option<usize>,
    is_playing: bool,
    volume: f64,
    audio: AudioBackend,
    artwork: ArtworkCache,
    group_collapsed: Vec<bool>,
    now_playing: String,
    scroll_offset: f64,
    scroll_tick_counter: u32,
    show_url_input: bool,
    url_input: String,
    temp_stream_url: Option<String>,
    temp_stream_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    Closed(window::Id),
    SelectStation(usize),
    TogglePlayback,
    SetVolume(f64),
    ToggleUrlInput,
    EditStationsToml,
    ToggleGroupCollapse(usize),
    UrlInputChanged(String),
    SubmitUrl,
    MarqueeTick,
}

impl cosmic::Application for RadioApp {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.system76.CosmicRadio";

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(core: cosmic::app::Core, _flags: ()) -> (Self, app::Task<Self::Message>) {
        let config = ConfigManager::load();
        let group_collapsed = vec![false; config.group_count()];
        let audio = AudioBackend::new();

        (
            Self {
                core,
                popup: None,
                config,
                current_station: None,
                is_playing: false,
                volume: 0.5,
                audio,
                artwork: ArtworkCache::new(),
                group_collapsed,
                now_playing: String::new(),
                scroll_offset: 0.0,
                scroll_tick_counter: 0,
                show_url_input: false,
                url_input: String::new(),
                temp_stream_url: None,
                temp_stream_name: None,
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

                    for i in 0..self.config.flat_stations().len() {
                        if let Some(station) = self.config.flat_stations().get(i) {
                            if let Some(ref url) = station.artwork {
                                if !url.is_empty() {
                                    self.artwork.load_artwork(url, i);
                                }
                            }
                        }
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
                self.temp_stream_url = None;
                self.temp_stream_name = None;
                self.is_playing = true;
                self.scroll_offset = 0.0;
                self.scroll_tick_counter = 0;

                if let Some(station) = self.config.flat_stations().get(index) {
                    let (_, display_name) = self.audio.play(&station.url, &station.name);
                    self.now_playing = display_name;
                    self.audio.set_volume(self.volume);

                    if let Some(ref url) = station.artwork {
                        if !url.is_empty() {
                            self.artwork.load_artwork(url, index);
                        }
                    }
                }
            }
            Message::TogglePlayback => {
                self.is_playing = !self.is_playing;
                if self.is_playing {
                    self.scroll_offset = 0.0;
                    self.scroll_tick_counter = 0;
                    if let Some(index) = self.current_station {
                        if let Some(station) = self.config.flat_stations().get(index) {
                            let (_, display_name) = self.audio.play(&station.url, &station.name);
                            self.now_playing = display_name;
                            self.audio.set_volume(self.volume);
                        }
                    } else if let Some(ref url) = self.temp_stream_url {
                        let name = self.temp_stream_name.clone().unwrap_or_default();
                        let (_, display_name) = self.audio.play(url, &name);
                        self.now_playing = display_name;
                        self.audio.set_volume(self.volume);
                    }
                } else {
                    self.audio.stop();
                }
            }
            Message::SetVolume(volume) => {
                self.volume = volume;
                self.audio.set_volume(volume);
            }
            Message::ToggleUrlInput => {
                self.show_url_input = !self.show_url_input;
            }
            Message::UrlInputChanged(input) => {
                self.url_input = input;
            }
            Message::SubmitUrl => {
                let mut url = self.url_input.trim().to_string();
                if url.is_empty() {
                    return Task::none();
                }
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    url = format!("https://{}", url);
                }
                self.show_url_input = false;
                self.url_input.clear();
                self.current_station = None;
                self.is_playing = true;
                self.scroll_offset = 0.0;
                self.scroll_tick_counter = 0;

                let fallback_name = Self::derive_name_from_url(&url);
                let (stream_url, display_name) = self.audio.play(&url, &fallback_name);
                let station_url = stream_url.clone();
                self.temp_stream_url = Some(stream_url);
                self.temp_stream_name = Some(display_name.clone());
                self.now_playing = display_name;
                self.audio.set_volume(self.volume);

                let station_name = self
                    .temp_stream_name
                    .clone()
                    .unwrap_or_else(|| Self::derive_name_from_url(&url));
                let exists = self
                    .config
                    .groups()
                    .iter()
                    .flat_map(|g| &g.stations)
                    .any(|s| s.name == station_name || s.url == station_url);
                if !exists {
                    let new_station = Station {
                        name: station_name,
                        url: station_url,
                        artwork: None,
                        auto_add: Some(true),
                    };
                    self.config.add_to_group(new_station, "Uncategorised");
                    while self.group_collapsed.len() < self.config.group_count() {
                        self.group_collapsed.push(false);
                    }
                }
            }
            Message::EditStationsToml => {
                let path = self.config.path().clone();

                let editor_cmd = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR"));

                match editor_cmd {
                    Ok(cmd) => {
                        let _ = std::process::Command::new(cmd).arg(&path).spawn();
                    }
                    Err(_) => {
                        let status = std::process::Command::new("cosmic-edit").arg(&path).spawn();
                        if status.is_err() {
                            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
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
                if let Some(new_title) = self.audio.take_metadata() {
                    self.now_playing = new_title;
                    self.scroll_offset = 0.0;
                    self.scroll_tick_counter = 0;
                }

                if !self.now_playing.is_empty() {
                    let char_count = self.now_playing.chars().count();
                    if char_count > MARQUEE_MAX_CHARS {
                        let max_offset = (char_count - MARQUEE_MAX_CHARS) as u32;
                        let scroll_phase_ticks = (max_offset + 1) * MARQUEE_SCROLL_INTERVAL;
                        let total_cycle = MARQUEE_START_PAUSE_TICKS as u32
                            + scroll_phase_ticks
                            + MARQUEE_END_PAUSE_TICKS as u32;

                        if self.scroll_tick_counter < MARQUEE_START_PAUSE_TICKS as u32 {
                            self.scroll_tick_counter += 1;
                        } else if self.scroll_tick_counter
                            < MARQUEE_START_PAUSE_TICKS as u32 + scroll_phase_ticks
                        {
                            let elapsed =
                                self.scroll_tick_counter - MARQUEE_START_PAUSE_TICKS as u32;
                            let new_offset = (elapsed / MARQUEE_SCROLL_INTERVAL) as f64;
                            if new_offset > self.scroll_offset {
                                self.scroll_offset = new_offset;
                            }
                            self.scroll_tick_counter += 1;
                        } else if self.scroll_tick_counter < total_cycle {
                            self.scroll_tick_counter += 1;
                        } else {
                            self.scroll_offset = 0.0;
                            self.scroll_tick_counter = 0;
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
            let flat_stations = self.config.flat_stations();

            let current_station_name = self
                .current_station
                .and_then(|i| flat_stations.get(i))
                .map(|s| s.name.as_str())
                .unwrap_or("No station selected");

            let current_artwork = self.current_station.and_then(|i| self.artwork.get(&i));

            let show_play_icon = (self.current_station.is_none() && self.temp_stream_url.is_none())
                || !self.is_playing;
            let icon_bytes = if show_play_icon { PLAY_SVG } else { STOP_SVG };
            let icon_label = if show_play_icon { "Play" } else { "Stop" };
            let icon_handle = svg::Handle::from_memory(icon_bytes);
            let play_button = tooltip::Tooltip::new(
                button::custom(
                    svg(icon_handle)
                        .width(iced::Length::Fixed(24.0))
                        .height(iced::Length::Fixed(24.0)),
                )
                .on_press(Message::TogglePlayback)
                .padding(4),
                icon_label,
                tooltip::Position::Bottom,
            );

            let mut now_playing_elements: Vec<Element<'_, Message>> = Vec::new();

            now_playing_elements.push(Self::artwork_image(current_artwork, 48));

            let display_text = if self.now_playing.is_empty() {
                current_station_name.to_string()
            } else {
                self.now_playing.clone()
            };

            let char_count = display_text.chars().count();
            let marquee_text: String = if char_count > MARQUEE_MAX_CHARS {
                let offset = (self.scroll_offset as usize).min(char_count - MARQUEE_MAX_CHARS);
                display_text
                    .chars()
                    .skip(offset)
                    .take(MARQUEE_MAX_CHARS)
                    .collect()
            } else {
                display_text
            };

            let now_playing_element: Element<'_, Message> =
                container(text::body(marquee_text).size(14))
                    .width(iced::Length::Fill)
                    .clip(true)
                    .into();

            now_playing_elements.push(now_playing_element);
            now_playing_elements.push(play_button.into());
            now_playing_elements.push(
                iced::widget::Space::new()
                    .width(iced::Length::Fixed(4.0))
                    .into(),
            );

            let now_playing_row = row(now_playing_elements)
                .spacing(8)
                .align_y(Alignment::Center);

            let mut ui_elements: Vec<Element<'_, Message>> = Vec::new();
            let mut flat_idx: usize = 0;

            for (group_idx, group) in self.config.groups().iter().enumerate() {
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
                        let station_art = self.artwork.get(&flat_idx);
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

            let url_input_section: Element<'_, Message> = if self.show_url_input {
                column![
                    text_input::TextInput::new("https://...", &self.url_input)
                        .on_input(Message::UrlInputChanged)
                        .on_submit(Message::SubmitUrl),
                    button::text("Play").on_press(Message::SubmitUrl),
                ]
                .spacing(4)
                .into()
            } else {
                column![].into()
            };

            let content = column![
                now_playing_row,
                divider::horizontal::default(),
                text::body("Volume").size(12),
                slider(0.0..=1.0, self.volume, Message::SetVolume).step(0.01),
                divider::horizontal::default(),
                scrollable(column(ui_elements).spacing(4).padding([4, 12, 4, 8]))
                    .height(iced::Length::Fixed(300.0)),
                divider::horizontal::default(),
                row![
                    button::custom(
                        svg(svg::Handle::from_memory(ADD_SVG))
                            .width(iced::Length::Fixed(24.0))
                            .height(iced::Length::Fixed(24.0)),
                    )
                    .on_press(Message::ToggleUrlInput)
                    .padding(4),
                    iced::widget::Space::new().width(iced::Length::Fill),
                    button::text("Edit stations.toml")
                        .on_press(Message::EditStationsToml)
                        .width(iced::Length::Shrink),
                ]
                .align_y(Alignment::Center)
                .spacing(8),
                url_input_section,
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
    fn artwork_image(artwork_path: Option<&PathBuf>, size: u16) -> Element<'static, Message> {
        if let Some(path) = artwork_path {
            if path.exists() {
                return image(image::Handle::from_path(path.clone()))
                    .width(iced::Length::Fixed(size as f32))
                    .height(iced::Length::Fixed(size as f32))
                    .into();
            }
        }
        svg(svg::Handle::from_memory(ARTWORK_PLACEHOLDER))
            .width(iced::Length::Fixed(size as f32))
            .height(iced::Length::Fixed(size as f32))
            .into()
    }

    fn derive_name_from_url(url: &str) -> String {
        url.split('?')
            .next()
            .unwrap_or(url)
            .split('/')
            .filter(|s| !s.is_empty())
            .last()
            .and_then(|s| {
                let stem = s.rsplit_once('.').map(|(name, _)| name).unwrap_or(s);
                let cleaned = stem.replace(['-', '_'], " ").trim().to_string();
                if cleaned.is_empty() {
                    None
                } else {
                    Some(cleaned)
                }
            })
            .unwrap_or_else(|| url.split('/').nth(2).unwrap_or("Unknown").to_string())
    }
}

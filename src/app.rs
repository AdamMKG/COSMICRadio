use crate::artwork::ArtworkCache;
use crate::audio::AudioBackend;
use crate::config::{ConfigManager, Station};
use cosmic::{
    app,
    iced::{
        self,
        platform_specific::shell::commands::popup::{destroy_popup, get_popup},
        widget::{column, container, image, row, svg, tooltip},
        window, Alignment, Subscription,
    },
    widget::{button, divider, scrollable, slider, text},
    Element, Task,
};
use std::path::PathBuf;
use std::time::Duration;

const MARQUEE_MAX_CHARS: usize = 29;
const MARQUEE_END_PAUSE_TICKS: f64 = 20.0;

const PLAY_SVG: &[u8] = include_bytes!("../data/play_button.svg");
const STOP_SVG: &[u8] = include_bytes!("../data/stop_button.svg");

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
    show_add_menu: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    Closed(window::Id),
    SelectStation(usize),
    TogglePlayback,
    SetVolume(f64),
    ToggleAddMenu,
    AddCurrentlyPlaying,
    AddViaUrl,
    EditStationsToml,
    ToggleGroupCollapse(usize),
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
                show_add_menu: false,
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
                self.is_playing = true;
                self.scroll_offset = 0.0;

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
                    if let Some(index) = self.current_station {
                        if let Some(station) = self.config.flat_stations().get(index) {
                            let (_, display_name) = self.audio.play(&station.url, &station.name);
                            self.now_playing = display_name;
                            self.audio.set_volume(self.volume);
                        }
                    }
                } else {
                    self.audio.stop();
                }
            }
            Message::SetVolume(volume) => {
                self.volume = volume;
                self.audio.set_volume(volume);
            }
            Message::ToggleAddMenu => {
                self.show_add_menu = !self.show_add_menu;
            }
            Message::AddCurrentlyPlaying => {
                self.show_add_menu = false;

                if let Some(index) = self.current_station {
                    if let Some(station) = self.config.flat_stations().get(index) {
                        let exists = self
                            .config
                            .groups()
                            .iter()
                            .flat_map(|g| &g.stations)
                            .any(|s| s.name == station.name || s.url == station.url);

                        if !exists {
                            let new_station = Station {
                                name: station.name.clone(),
                                url: station.url.clone(),
                                artwork: station.artwork.clone(),
                                auto_add: Some(true),
                            };
                            self.config.add_station(new_station);
                            while self.group_collapsed.len() < self.config.group_count() {
                                self.group_collapsed.push(false);
                            }
                        }
                    }
                }
            }
            Message::AddViaUrl => {
                self.show_add_menu = false;
            }
            Message::EditStationsToml => {
                self.show_add_menu = false;

                let path = self.config.path().clone();

                let editor_cmd = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR"));

                match editor_cmd {
                    Ok(cmd) => {
                        let _ = std::process::Command::new(cmd).arg(&path).spawn();
                    }
                    Err(_) => {
                        let status = std::process::Command::new("cosmic-edit")
                            .arg(&path)
                            .spawn();
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
            let flat_stations = self.config.flat_stations();

            let current_station_name = self
                .current_station
                .and_then(|i| flat_stations.get(i))
                .map(|s| s.name.as_str())
                .unwrap_or("No station selected");

            let current_artwork = self.current_station.and_then(|i| self.artwork.get(&i));

            let show_play_icon = self.current_station.is_none() || !self.is_playing;
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
                let offset =
                    (self.scroll_offset as usize).min(char_count - MARQUEE_MAX_CHARS);
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

            let now_playing_row =
                row(now_playing_elements).spacing(8).align_y(Alignment::Center);

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

            let add_menu: Element<'_, Message> = if self.show_add_menu {
                column![
                    button::text("Add currently playing")
                        .on_press(Message::AddCurrentlyPlaying)
                        .width(iced::Length::Fill),
                    button::text("Add via URL")
                        .on_press(Message::AddViaUrl)
                        .width(iced::Length::Fill),
                    button::text("Edit stations.toml")
                        .on_press(Message::EditStationsToml)
                        .width(iced::Length::Fill),
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
                    button::custom(text::body("+"))
                        .on_press(Message::ToggleAddMenu)
                        .padding(4),
                ]
                .spacing(8),
                add_menu,
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
}

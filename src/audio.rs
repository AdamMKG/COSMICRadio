use gstreamer::prelude::*;
use gstreamer::{tags::Artist, tags::Title, BusSyncReply, MessageView};
use gstreamer_play::Play;
use std::sync::{Arc, Mutex};

pub struct AudioBackend {
    player: Play,
    latest_metadata: Arc<Mutex<Option<String>>>,
}

impl AudioBackend {
    pub fn new() -> Self {
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

        Self {
            player,
            latest_metadata,
        }
    }

    pub fn play(&self, url: &str, fallback_name: &str) -> (String, String) {
        let (stream_url, display_name) = if url.ends_with(".pls") {
            Self::resolve_pls_url(url)
                .unwrap_or_else(|| (url.to_string(), fallback_name.to_string()))
        } else {
            (url.to_string(), fallback_name.to_string())
        };

        self.player.set_uri(Some(&stream_url));
        self.player.play();
        (stream_url, display_name)
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn set_volume(&self, volume: f64) {
        self.player.set_volume(volume);
    }

    pub fn take_metadata(&self) -> Option<String> {
        self.latest_metadata.lock().ok().and_then(|mut g| g.take())
    }

    pub fn resolve_pls(content: &str) -> Option<(String, String)> {
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

    fn resolve_pls_url(url: &str) -> Option<(String, String)> {
        let content = if url.starts_with("http://") || url.starts_with("https://") {
            reqwest::blocking::get(url).ok()?.text().ok()?
        } else {
            std::fs::read_to_string(url).ok()?
        };
        Self::resolve_pls(&content)
    }
}

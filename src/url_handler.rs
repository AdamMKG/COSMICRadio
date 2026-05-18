use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ResolvedChannel {
    pub name: String,
    pub stream_url: String,
    pub artwork_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSource {
    pub group_name: String,
    pub channels: Vec<ResolvedChannel>,
}

pub fn resolve_url(url: &str) -> Result<ResolvedSource, String> {
    let response = reqwest::blocking::get(url)
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let body = response
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))?;
    let trimmed = body.trim();

    if trimmed.starts_with("[playlist]") {
        if let Some((pls_url, pls_name, _)) = resolve_pls(&body) {
            return Ok(ResolvedSource {
                group_name: "Uncategorised".to_string(),
                channels: vec![ResolvedChannel {
                    name: pls_name,
                    stream_url: pls_url,
                    artwork_url: None,
                }],
            });
        }
        return Err("Failed to parse PLS file".to_string());
    }

    if trimmed.starts_with('{') {
        if let Some(result) = try_parse_somafm(&body) {
            return Ok(result);
        }
        return Err("Unrecognised JSON format".to_string());
    }

    if trimmed.starts_with('[') {
        if let Some(result) = try_parse_radio_browser(&body) {
            return Ok(result);
        }
        return Err("Unrecognised JSON format".to_string());
    }

    if content_type.starts_with("text/html")
        || content_type.starts_with("application/xhtml")
    {
        return Err("URL returned an HTML page, expected an audio stream, PLS playlist, or JSON API. Try https://api.somafm.com/channels.json for SomaFM.".to_string());
    }

    if content_type.starts_with("audio/") || content_type.is_empty() {
        let name = derive_name_from_url(url);
        return Ok(ResolvedSource {
            group_name: "Uncategorised".to_string(),
            channels: vec![ResolvedChannel {
                name,
                stream_url: url.to_string(),
                artwork_url: None,
            }],
        });
    }

    let name = derive_name_from_url(url);
    Ok(ResolvedSource {
        group_name: "Uncategorised".to_string(),
        channels: vec![ResolvedChannel {
            name,
            stream_url: url.to_string(),
            artwork_url: None,
        }],
    })
}

fn resolve_pls(content: &str) -> Option<(String, String, Option<u32>)> {
    let entries = parse_pls_entries(content);
    select_best_entry(&entries).map(|e| {
        (
            e.file.clone(),
            e.title.clone().unwrap_or_default(),
            e.bitrate,
        )
    })
}

#[derive(Debug)]
struct PlsEntry {
    file: String,
    title: Option<String>,
    bitrate: Option<u32>,
}

fn parse_pls_entries(content: &str) -> Vec<PlsEntry> {
    let mut entries: HashMap<u32, PlsEntry> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        let lower = line.to_lowercase();

        if let Some(rest) = lower.strip_prefix("file") {
            if let Some((num_str, _value)) = rest.split_once('=') {
                if let Ok(num) = num_str.parse::<u32>() {
                    let entry = entries.entry(num).or_insert_with(|| PlsEntry {
                        file: String::new(),
                        title: None,
                        bitrate: None,
                    });
                    entry.file = line[4 + num_str.len() + 1..].trim().to_string();
                }
            }
        } else if let Some(rest) = lower.strip_prefix("title") {
            if let Some((num_str, _)) = rest.split_once('=') {
                if let Ok(num) = num_str.parse::<u32>() {
                    let entry = entries.entry(num).or_insert_with(|| PlsEntry {
                        file: String::new(),
                        title: None,
                        bitrate: None,
                    });
                    entry.title = Some(line[5 + num_str.len() + 1..].trim().to_string());
                }
            }
        } else if let Some(rest) = lower.strip_prefix("bitrate") {
            if let Some((num_str, value)) = rest.split_once('=') {
                if let Ok(num) = num_str.parse::<u32>() {
                    if let Ok(br) = value.trim().parse::<u32>() {
                        let entry = entries.entry(num).or_insert_with(|| PlsEntry {
                            file: String::new(),
                            title: None,
                            bitrate: None,
                        });
                        entry.bitrate = Some(br);
                    }
                }
            }
        }
    }

    entries.into_values().collect()
}

fn select_best_entry(entries: &[PlsEntry]) -> Option<&PlsEntry> {
    entries
        .iter()
        .max_by_key(|e| e.bitrate.unwrap_or(0))
        .or_else(|| entries.first())
}

fn try_parse_somafm(body: &str) -> Option<ResolvedSource> {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct SomaFmChannel {
        title: String,
        description: Option<String>,
        image: Option<String>,
        largeimage: Option<String>,
        xlimage: Option<String>,
        playlists: Vec<SomaFmPlaylist>,
    }

    #[derive(Deserialize)]
    struct SomaFmPlaylist {
        url: String,
        format: Option<String>,
        quality: Option<String>,
    }

    #[derive(Deserialize)]
    struct SomaFmResponse {
        channels: Vec<SomaFmChannel>,
    }

    let response: SomaFmResponse = serde_json::from_str(body).ok()?;

    let mut channels = Vec::new();
    for ch in &response.channels {
        let best_playlist = ch
            .playlists
            .iter()
            .filter(|p| p.quality.as_deref() == Some("highest"))
            .max_by_key(|p| {
                match p.format.as_deref() {
                    Some("mp3") => 2,
                    Some("aac") => 1,
                    _ => 0,
                }
            })
            .or_else(|| ch.playlists.first())?;

        let stream_url = if best_playlist.url.to_lowercase().ends_with(".pls") {
            let pls_content =
                reqwest::blocking::get(&best_playlist.url).ok()?.text().ok()?;
            resolve_pls(&pls_content).map(|(u, _, _)| u).unwrap_or_else(|| best_playlist.url.clone())
        } else {
            best_playlist.url.clone()
        };

        let artwork = ch
            .largeimage
            .as_ref()
            .or(ch.image.as_ref())
            .cloned();

        channels.push(ResolvedChannel {
            name: ch.title.clone(),
            stream_url,
            artwork_url: artwork,
        });
    }

    Some(ResolvedSource {
        group_name: "SomaFM".to_string(),
        channels,
    })
}

fn try_parse_radio_browser(body: &str) -> Option<ResolvedSource> {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct RadioBrowserStation {
        name: Option<String>,
        url: Option<String>,
        url_resolved: Option<String>,
        bitrate: Option<u32>,
        codec: Option<String>,
        favicon: Option<String>,
        tags: Option<String>,
        country: Option<String>,
        language: Option<String>,
    }

    let stations: Vec<RadioBrowserStation> = serde_json::from_str(body).ok()?;

    if stations.is_empty() || stations[0].name.is_none() && stations[0].url.is_none() {
        return None;
    }

    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut channels = Vec::new();

    for station in &stations {
        let stream_url = station
            .url_resolved
            .as_ref()
            .or(station.url.as_ref())?;

        if !seen_urls.insert(stream_url.clone()) {
            continue;
        }

        let name = station
            .name
            .as_deref()
            .unwrap_or("Unknown Radio")
            .trim()
            .to_string();

        channels.push(ResolvedChannel {
            name,
            stream_url: stream_url.clone(),
            artwork_url: station.favicon.clone(),
        });
    }

    if channels.is_empty() {
        return None;
    }

    channels.sort_by(|a, b| {
        let a_br = stations
            .iter()
            .find(|s| {
                s.url_resolved.as_deref() == Some(&a.stream_url)
                    || s.url.as_deref() == Some(&a.stream_url)
            })
            .and_then(|s| s.bitrate)
            .unwrap_or(0);
        let b_br = stations
            .iter()
            .find(|s| {
                s.url_resolved.as_deref() == Some(&b.stream_url)
                    || s.url.as_deref() == Some(&b.stream_url)
            })
            .and_then(|s| s.bitrate)
            .unwrap_or(0);
        b_br.cmp(&a_br)
    });

    Some(ResolvedSource {
        group_name: "Radio Browser".to_string(),
        channels,
    })
}

pub fn derive_name_from_url(url: &str) -> String {
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

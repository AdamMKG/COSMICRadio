use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub name: String,
    pub url: String,
    pub artwork: Option<String>,
    #[serde(rename = "auto-add", skip_serializing_if = "Option::is_none")]
    pub auto_add: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationGroup {
    pub name: String,
    pub stations: Vec<Station>,
}

#[derive(Debug, Deserialize)]
struct OldConfig {
    stations: Vec<Station>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct ConfigManager {
    path: PathBuf,
    groups: Vec<StationGroup>,
    flat: Vec<Station>,
}

impl ConfigManager {
    pub fn load() -> Self {
        let path = ensure_config();
        let config = load_config();
        let flat = config.groups.iter().flat_map(|g| g.stations.clone()).collect();
        Self {
            path,
            groups: config.groups,
            flat,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn groups(&self) -> &[StationGroup] {
        &self.groups
    }

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    pub fn flat_stations(&self) -> &[Station] {
        &self.flat
    }

    pub fn add_station(&mut self, station: Station) {
        if let Some(fav_group) = self.groups.iter_mut().find(|g| g.name == "Favourites") {
            fav_group.stations.push(station);
        } else {
            self.groups.push(StationGroup {
                name: "Favourites".to_string(),
                stations: vec![station],
            });
        }
        self.flat = self.groups.iter().flat_map(|g| g.stations.clone()).collect();
        self.save();
    }

    fn save(&self) {
        let config = Config {
            groups: self.groups.clone(),
        };
        if let Ok(toml_str) = toml::to_string_pretty(&config) {
            let _ = fs::write(&self.path, toml_str);
        }
    }
}

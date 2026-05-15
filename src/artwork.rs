use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub struct ArtworkCache {
    cache: HashMap<usize, PathBuf>,
}

impl ArtworkCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, index: &usize) -> Option<&PathBuf> {
        self.cache.get(index)
    }

    pub fn load_artwork(&mut self, artwork_url: &str, index: usize) {
        let cache_dir = Self::artwork_cache_dir();
        let cache_file = cache_dir.join(Self::cache_filename(artwork_url));

        if cache_file.exists() && Self::is_cache_fresh(&cache_file) {
            if !self.cache.contains_key(&index) {
                self.cache.insert(index, cache_file);
            }
            return;
        }

        let cache_dir_clone = cache_dir.clone();
        let cache_file_clone = cache_file.clone();
        let url = artwork_url.to_string();
        tokio::spawn(async move {
            if let Err(e) = fs::create_dir_all(&cache_dir_clone) {
                eprintln!("Failed to create cache dir: {}", e);
                return;
            }

            let client = reqwest::Client::new();
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.bytes().await {
                            Ok(bytes) => {
                                if let Err(e) = fs::write(&cache_file_clone, &bytes) {
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

        self.cache.insert(index, cache_file);
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
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed < Duration::from_secs(5 * 24 * 60 * 60);
                }
            }
        }
        false
    }
}

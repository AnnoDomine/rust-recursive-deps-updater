use std::{
    collections::HashMap,
    env,
    fmt::{Debug, Display},
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::{constants::*, enums::*, meta_status, simple_status, status_codes::Module};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheRegistry<T> {
    pub cache: HashMap<String, CacheEntry<T>>,
    pub cache_type: CacheType,
}

impl Display for CacheType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheType::CratesIOApi => write!(f, "{:}", RRDU_REGISTRY_CACHE_CRATES_IO_API),
            CacheType::CratesIOIndex => write!(f, "{:}", RRDU_REGISTRY_CACHE_CRATES_IO_INDEX),
            CacheType::RustsecJsonResponse => write!(f, "{:}", RRDU_REGISTRY_CACHE_RUSTSEC_API_IDS),
        }
    }
}

#[allow(clippy::result_unit_err)]
impl CacheType {
    pub fn get_cache_type(name: &str) -> Result<CacheType, ()> {
        match name {
            RRDU_REGISTRY_CACHE_CRATES_IO_API => Ok(CacheType::CratesIOApi),
            RRDU_REGISTRY_CACHE_CRATES_IO_INDEX => Ok(CacheType::CratesIOIndex),
            RRDU_REGISTRY_CACHE_RUSTSEC_API_IDS => Ok(CacheType::RustsecJsonResponse),
            _ => {
                simple_status!(
                    log::LevelFilter::Error,
                    Module::CACHE,
                    404,
                    format!("Cache '{:}' unknown.", name)
                );
                Err(())
            }
        }
    }

    pub fn get_ttl(&self) -> Duration {
        match self {
            CacheType::CratesIOIndex => CACHE_ENTRY_CRATES_IO_INDEX_TTL,
            CacheType::CratesIOApi => CACHE_ENTRY_CRATES_IO_API_TTL,
            CacheType::RustsecJsonResponse => CACHE_ENTRY_RUSTSEC_API_TTL,
        }
    }
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de> + Debug> Display for CacheRegistry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.fmt_cache())
    }
}

#[allow(clippy::result_unit_err)]
impl<T: Clone + Serialize + for<'de> Deserialize<'de> + Debug> CacheRegistry<T> {
    pub fn new(cache_type: CacheType) -> Self {
        let mut cache_registry = Self {
            cache: HashMap::new(),
            cache_type,
        };
        let _ = cache_registry.load_cache();
        cache_registry
    }

    pub fn fmt_cache(&self) -> Vec<String> {
        let cache = self.cache.clone();
        let mut map: Vec<String> = Vec::new();
        for (ca_k, ca_v) in cache {
            map.push(format!("{:} [{:?}]:", ca_k, ca_v.timestamp));
            map.push(format!("{:#?}", ca_v.response));
            map.push("".to_string())
        }
        map
    }

    pub fn add_cache_entry(&mut self, key: &str, entry: T) {
        let timestamp = SystemTime::now();
        simple_status!(
            log::LevelFilter::Debug,
            Module::CACHE,
            200,
            format!("{:} cached. Type: '{:?}'", key, self.cache_type)
        );
        self.cache.insert(
            key.to_string(),
            CacheEntry {
                timestamp,
                response: entry,
            },
        );
    }

    pub fn use_cache(timestamp: SystemTime, ttl: Duration) -> bool {
        let now = SystemTime::now();
        if let Ok(diff) = now.duration_since(timestamp) {
            simple_status!(
                log::LevelFilter::Debug,
                Module::CACHE,
                200,
                format!("Use cached entry. {:?}", ttl > diff)
            );
            return ttl > diff;
        }
        false
    }

    pub fn get_cache_entry(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.cache.get(key)
            && Self::use_cache(entry.timestamp, self.cache_type.get_ttl())
        {
            simple_status!(
                log::LevelFilter::Debug,
                Module::CACHE,
                200,
                format!(
                    "Cache entry found and valid for '{key}'. Type: '{:?}'",
                    self.cache_type
                )
            );
            Some(entry.response.clone())
        } else {
            simple_status!(
                log::LevelFilter::Debug,
                Module::CACHE,
                404,
                format!(
                    "No cache entry found or valid for '{key}'. Type: '{:?}'",
                    self.cache_type
                )
            );
            None
        }
    }

    pub fn get_cache_path(&self) -> Result<PathBuf, ()> {
        let mut cache_path = match env::home_dir() {
            Some(h) => h,
            None => {
                simple_status!(
                    log::LevelFilter::Error,
                    Module::CACHE,
                    404,
                    "Could not retreive the home folder.".to_string()
                );
                return Err(());
            }
        };
        cache_path.push(".rrdu");
        cache_path.push("cache");
        if let Ok(false) = fs::exists(&cache_path) {
            simple_status!(
                log::LevelFilter::Error,
                Module::CACHE,
                404,
                format!(
                    "Could not find cache folder or cache file '{:}'. Creating one.",
                    self.cache_type
                )
            );
            if let Err(create_err) = fs::create_dir_all(&cache_path) {
                meta_status!(
                    log::LevelFilter::Error,
                    Module::CACHE,
                    500,
                    format!(
                        "Unexpected error while creating cache file '{:}'.",
                        self.cache_type
                    ),
                    create_err
                );
                return Err(());
            }
        };
        cache_path.push(self.cache_type.to_string());
        Ok(cache_path)
    }

    pub fn save_cache(&self, file: Option<&PathBuf>) {
        let cache_file = match file {
            Some(p) => p.clone(),
            None => match self.get_cache_path() {
                Ok(p) => p,
                _ => {
                    return;
                }
            },
        };
        match File::create(&cache_file) {
            Ok(buf) => {
                if let Err(save_err) = serde_json::to_writer(buf, &self.cache) {
                    meta_status!(
                        log::LevelFilter::Error,
                        Module::CACHE,
                        400,
                        format!("Error while write cache file. Type: '{:}'", self.cache_type),
                        save_err
                    );
                } else {
                    simple_status!(
                        log::LevelFilter::Info,
                        Module::CACHE,
                        200,
                        format!("Cache saved. Type: '{:?}'", self.cache_type)
                    );
                }
            }
            Err(create_err) => {
                meta_status!(
                    log::LevelFilter::Error,
                    Module::CACHE,
                    500,
                    format!(
                        "Error while create cache file. Type: '{:}'",
                        self.cache_type
                    ),
                    create_err
                );
            }
        };
    }

    /// Returns the cache for rrdu (<user-home-folder>/.rrdu/cache/)
    pub fn load_cache(&mut self) -> Result<(), ()> {
        let file_path = self.get_cache_path()?;

        if !file_path.exists() {
            self.clone().save_cache(Some(&file_path));
            return Ok(());
        }

        let loaded = self.get_reponse_deserialised(&file_path)?;
        self.cache = loaded;
        simple_status!(
            log::LevelFilter::Info,
            Module::CACHE,
            200,
            format!("Cache loaded. '{:}'", self.cache_type)
        );
        Ok(())
    }

    pub fn get_reponse_deserialised(
        &self,
        path: &PathBuf,
    ) -> Result<HashMap<String, CacheEntry<T>>, ()> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(read_err) => {
                meta_status!(
                    log::LevelFilter::Error,
                    Module::CACHE,
                    500,
                    "Unexpected error while reading file.",
                    read_err
                );
                return Err(());
            }
        };
        let reader = BufReader::new(file);

        match serde_json::from_reader::<_, HashMap<String, CacheEntry<T>>>(reader) {
            Ok(map) => Ok(map),
            Err(err) => {
                meta_status!(
                    log::LevelFilter::Error,
                    Module::CACHE,
                    500,
                    "Unexpected error while reading file content.",
                    err
                );
                Err(())
            }
        }
    }
}

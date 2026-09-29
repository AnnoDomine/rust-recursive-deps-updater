use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::*,
    enums::*,
    registry::{
        crates_index_response_structs::CratesIndexResponseParsed,
        crates_io_response_structs::CratesIOResponse,
        rust_sec_json_response_structs::RustsecJsonResponse,
    },
    status_codes::{Module, StatusCodeSchema},
};

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

impl CacheType {
    pub fn get_cache_type(name: &str) -> Result<CacheType, StatusCodeSchema> {
        match name {
            RRDU_REGISTRY_CACHE_CRATES_IO_API => Ok(CacheType::CratesIOApi),
            RRDU_REGISTRY_CACHE_CRATES_IO_INDEX => Ok(CacheType::CratesIOIndex),
            RRDU_REGISTRY_CACHE_RUSTSEC_API_IDS => Ok(CacheType::RustsecJsonResponse),
            _ => Err(StatusCodeSchema::simple(
                Module::CACHE,
                404,
                format!("Cache '{:}' unknown.", name),
            )),
        }
    }

    pub fn get_reponse_deserialised(
        &self,
        path: &PathBuf,
    ) -> Result<CacheResponse, StatusCodeSchema<std::io::Error>> {
        {
            let file = match File::open(path) {
                Ok(f) => f,
                Err(read_err) => {
                    return Err(StatusCodeSchema::with_meta(
                        Module::CACHE,
                        500,
                        format!("Unexpected error while reading file '{:}'.", self),
                        read_err,
                    ));
                }
            };
            let reader = BufReader::new(file);

            let map_serde_err = |err: serde_json::Error| {
                StatusCodeSchema::with_meta(
                    Module::CACHE,
                    500,
                    format!("Unexpected error while reading file content '{:}'.", self),
                    std::io::Error::new(std::io::ErrorKind::InvalidData, err),
                )
            };

            match self {
                CacheType::CratesIOIndex => {
                    let map: HashMap<String, CacheEntry<CratesIndexResponseParsed>> =
                        serde_json::from_reader(reader).map_err(map_serde_err)?;
                    Ok(CacheResponse::CratesIOIndexResponse(map))
                }
                CacheType::CratesIOApi => {
                    let map: HashMap<String, CacheEntry<Box<CratesIOResponse>>> =
                        serde_json::from_reader(reader).map_err(map_serde_err)?;
                    Ok(CacheResponse::CratesIOAPIResponse(map))
                }
                CacheType::RustsecJsonResponse => {
                    let map: HashMap<String, CacheEntry<RustsecJsonResponse>> =
                        serde_json::from_reader(reader).map_err(map_serde_err)?;
                    Ok(CacheResponse::RustSecJsonResponse(map))
                }
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

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> CacheRegistry<T> {
    pub fn new(cache_type: CacheType) -> Self {
        let mut cache_registry = Self {
            cache: HashMap::new(),
            cache_type,
        };
        let _ = cache_registry.load_cache();
        cache_registry
    }

    pub fn add_cache_entry(&mut self, key: &str, entry: T) {
        let timestamp = SystemTime::now();
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
            return ttl > diff;
        }
        false
    }

    pub fn get_cache_entry(&self, key: &str) -> Option<T> {
        let entry = self.cache.get(key)?;
        if Self::use_cache(entry.timestamp, self.cache_type.get_ttl()) {
            Some(entry.response.clone())
        } else {
            None
        }
    }

    pub fn get_cache_path(&self) -> Result<PathBuf, StatusCodeSchema<std::io::Error>> {
        let mut cache_path = match env::home_dir() {
            Some(h) => h,
            None => {
                return Err(StatusCodeSchema::simple(
                    Module::CACHE,
                    404,
                    "Could not retreive the home folder.".to_string(),
                ));
            }
        };
        cache_path.push(".rrdu");
        cache_path.push("cache");
        if let Err(err) = fs::exists(&cache_path) {
            println!(
                "{:}",
                StatusCodeSchema::with_meta(
                    Module::CACHE,
                    404,
                    format!(
                        "Could not find cache folder or cache file '{:}'. Creating one.",
                        self.cache_type
                    ),
                    err
                )
            );
            if let Err(create_err) = fs::create_dir_all(&cache_path) {
                return Err(StatusCodeSchema::with_meta(
                    Module::CACHE,
                    500,
                    format!(
                        "Unexpected error while creating cache file '{:}'.",
                        self.cache_type
                    ),
                    create_err,
                ));
            }
        };
        Ok(cache_path)
    }

    pub fn save_cache(
        self,
        file: Option<&PathBuf>,
    ) -> Result<(), StatusCodeSchema<std::io::Error>> {
        let cache_file = match file {
            Some(p) => p.clone(),
            None => self.get_cache_path()?,
        };
        match File::create(cache_file) {
            Ok(buf) => {
                if let Err(save_err) = serde_json::to_writer(buf, &self.cache) {
                    return Err(StatusCodeSchema::with_meta(
                        Module::CACHE,
                        400,
                        format!("Error while write cache file {:}", self.cache_type),
                        save_err.into(),
                    ));
                };
                Ok(())
            }
            Err(create_err) => Err(StatusCodeSchema::with_meta(
                Module::CACHE,
                500,
                format!("Error while create cache file {:}", self.cache_type),
                create_err,
            )),
        }
    }

    /// Returns the cache for rrdu (<user-home-folder>/.rrdu/cache/)
    pub fn load_cache(&mut self) -> Result<(), StatusCodeSchema<std::io::Error>> {
        let mut file_path = self.get_cache_path()?;
        file_path.push(self.cache_type.to_string());

        if !file_path.exists() {
            self.clone().save_cache(Some(&file_path))?;
            return Ok(());
        }

        let loaded = self.get_reponse_deserialised(&file_path)?;
        self.cache = loaded;
        Ok(())
    }

    pub fn get_reponse_deserialised(
        &self,
        path: &PathBuf,
    ) -> Result<HashMap<String, CacheEntry<T>>, StatusCodeSchema<std::io::Error>> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(read_err) => {
                return Err(StatusCodeSchema::with_meta(
                    Module::CACHE,
                    500,
                    format!("Unexpected error while reading file '{:?}'.", path),
                    read_err,
                ));
            }
        };
        let reader = BufReader::new(file);

        match serde_json::from_reader::<_, HashMap<String, CacheEntry<T>>>(reader) {
            Ok(map) => Ok(map),
            Err(err) => Err(StatusCodeSchema::with_meta(
                Module::CACHE,
                500,
                format!("Unexpected error while reading file content '{:?}'.", path),
                std::io::Error::new(std::io::ErrorKind::InvalidData, err),
            )),
        }
    }
}

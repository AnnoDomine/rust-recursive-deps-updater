use std::{collections::HashMap, thread::sleep};

use serde::{Deserialize, Serialize};
use ureq::{
    Body,
    http::{Response, Uri},
};

use crate::{
    constants::*,
    enums::*,
    registry::{
        cache_registry::CacheRegistry,
        crates_index_response_structs::{CratesIndexItem, CratesIndexResponseParsed},
        crates_io_response_structs::CratesIOResponse,
        rust_sec_json_response_structs::RustsecJsonResponse,
    },
    status_codes::StatusCodeSchema,
};

pub struct CratesIOResponseItem {
    pub name: String,
    pub version: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    #[serde(default)]
    pub crates: HashMap<String, RegistryClientState>,
    pub crates_io_index_cache: CacheRegistry<CratesIndexResponseParsed>,
    pub crates_io_api_cache: CacheRegistry<Box<CratesIOResponse>>,
    pub rustsec_json_cache: CacheRegistry<RustsecJsonResponse>,
}

impl Default for CrateResponses {
    fn default() -> Self {
        Self::new()
    }
}

impl CrateResponses {
    pub fn new() -> Self {
        Self {
            index: None,
            api: None,
            audit: None,
        }
    }

    pub fn set_index(&mut self, value: CratesIndexResponseParsed) {
        self.index = Some(value);
    }

    pub fn set_api(&mut self, value: Box<CratesIOResponse>) {
        self.api = Some(value);
    }

    pub fn set_audit(&mut self, value: super::rust_sec_json_response_structs::RustsecJsonResponse) {
        self.audit = Some(value);
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            crates: HashMap::new(),
            crates_io_index_cache: CacheRegistry::new(CacheType::CratesIOIndex),
            crates_io_api_cache: CacheRegistry::new(CacheType::CratesIOApi),
            rustsec_json_cache: CacheRegistry::new(CacheType::RustsecJsonResponse),
        }
    }

    pub fn map_crates(&mut self, crates: HashMap<String, DependencyCollectionVersion>) {
        for (dep, status) in crates {
            if let DependencyCollectionVersion::None = status {
                self.crates
                    .insert(dep.clone(), RegistryClientState::Uninitialised);
            }
        }
    }

    pub fn start_calls(&mut self) -> Result<(), ureq::Error> {
        for (k, state) in self.clone().crates {
            if state == RegistryClientState::Uninitialised && self.get_mut_crate(&k).is_some() {
                let mut response_map = CrateResponses::new();
                self.set_crate_state(k.clone(), RegistryClientState::Loading);
                if let Err(e) = self.clone().call_index(&mut response_map, &k) {
                    self.set_crate_state(
                        k,
                        RegistryClientState::Errored(crate::errors::CollectionVersionError::Other(
                            e.to_string(),
                        )),
                    );
                    return Err(e);
                };
                if let Err(e) = self.clone().call_api(&mut response_map, &k) {
                    self.set_crate_state(
                        k,
                        RegistryClientState::Errored(crate::errors::CollectionVersionError::Other(
                            e.to_string(),
                        )),
                    );
                    return Err(e);
                };
                if let Err(e) = self.clone().call_audit(&mut response_map, &k) {
                    self.set_crate_state(
                        k,
                        RegistryClientState::Errored(crate::errors::CollectionVersionError::Other(
                            e.to_string(),
                        )),
                    );
                    return Err(e);
                };
                self.set_crate_state(k, RegistryClientState::Succesed(response_map));
                sleep(TIME_BETWEEN_REQUESTS);
            }
        }
        Ok(())
    }

    pub fn set_crate_state(&mut self, c: String, new_state: RegistryClientState) {
        self.crates.insert(c, new_state);
    }

    pub fn get_mut_crate(&mut self, c: &str) -> Option<&mut RegistryClientState> {
        self.crates.get_mut(c)
    }

    pub fn create_user_agent_value(self, reason: &str) -> String {
        format!(
            "{:}{:}{:} < crates.io: {:} - GitHub: {:} >",
            DEFAULT_USER_AGENT_HEADER_APPLICATION,
            env!("CARGO_PKG_VERSION"),
            reason,
            DEFAULT_USER_AGENT_HEADER_CRATES_IO_URL,
            DEFAULT_USER_AGENT_HEADER_GITHUB_URL,
        )
    }

    fn add_path_slash(url: &mut Vec<String>) {
        url.push("/".to_string());
    }

    pub fn call(self, url: Uri, reason: &str) -> Result<Response<Body>, ureq::Error> {
        ureq::get(url)
            .header("User-Agent", self.create_user_agent_value(reason))
            .call()
    }

    /// URL:
    /// - 0 chars -> Invalid (Return None)
    /// - 1 char -> https://index.crates.io/1/f
    /// - 2 chars -> https://index.crates.io/2/fo
    /// - 3 chars -> https://index.crates.io/3/foo
    /// - 4+ chars -> https://index.crates.io/fo/ob/foobar
    pub fn parse_index_url(&self, c: &str) -> Option<Uri> {
        let mut url_path: Vec<String> = Vec::new();
        url_path.push(CRATE_IO_INDEX_URL.to_string());
        Self::add_path_slash(&mut url_path);
        match c.len() {
            0 => return None,
            1 => url_path.push("1".to_string()),
            2 => url_path.push("2".to_string()),
            3 => {
                url_path.push("3".to_string());
                Self::add_path_slash(&mut url_path);
                url_path.push(c.split_at(2).0.to_string());
            }
            _ => {
                let splitted_c = c.split_at(2);
                url_path.push(splitted_c.0.to_string());
                Self::add_path_slash(&mut url_path);
                url_path.push(splitted_c.1.split_at(2).0.to_string());
            }
        };
        Self::add_path_slash(&mut url_path);
        url_path.push(c.to_string());
        Some(url_path.join("").parse::<Uri>().unwrap())
    }
    pub fn call_index(&self, map: &mut CrateResponses, c: &str) -> Result<(), ureq::Error> {
        match self.crates_io_index_cache.get_cache_entry(c) {
            Some(cached) => {
                map.set_index(cached);
            }
            None => {
                if let Some(url) = self.clone().parse_audit_url(c) {
                    let body = self
                        .clone()
                        .call(url, DEFAULT_USER_AGENT_HEADER_TYPE_AVAILABILITY)?
                        .body_mut()
                        .read_to_string()?;
                    map.set_index(CratesIndexItem::parse_response(body));
                };
            }
        };
        Ok(())
    }

    pub fn parse_api_url(&self, c: &str) -> Option<Uri> {
        let mut url_path: Vec<String> = Vec::new();
        url_path.push(CRATE_IO_API_URL.to_string());
        Self::add_path_slash(&mut url_path);
        url_path.push(c.to_string());
        url_path.push("?keywords=full".to_string());
        Some(url_path.join("").parse::<Uri>().unwrap())
    }
    pub fn call_api(self, map: &mut CrateResponses, c: &str) -> Result<(), ureq::Error> {
        match self.crates_io_api_cache.get_cache_entry(c) {
            Some(cached) => map.set_api(cached),
            None => {
                if let Some(url) = self.clone().parse_audit_url(c) {
                    let body = self
                        .clone()
                        .call(url, DEFAULT_USER_AGENT_HEADER_TYPE_AVAILABILITY)?
                        .body_mut()
                        .read_json::<Box<CratesIOResponse>>()?;
                    map.set_api(body);
                };
            }
        }
        Ok(())
    }

    pub fn parse_audit_url(self, c: &str) -> Option<Uri> {
        let mut url_path: Vec<String> = Vec::new();
        url_path.push(RUSTSEC_API_URL.to_string());
        Self::add_path_slash(&mut url_path);
        url_path.push(c.to_string());
        // The uri requests specified <crate>.json
        url_path.push(".json".to_string());
        Some(url_path.join("").parse::<Uri>().unwrap())
    }
    pub fn call_audit(self, map: &mut CrateResponses, c: &str) -> Result<(), ureq::Error> {
        match self.rustsec_json_cache.get_cache_entry(c) {
            Some(cached) => map.set_audit(cached),
            None => {
                if let Some(url) = self.clone().parse_audit_url(c) {
                    let body = self
                        .clone()
                        .call(url, DEFAULT_USER_AGENT_HEADER_TYPE_AUDIT)?
                        .body_mut()
                        .read_json::<RustsecJsonResponse>()?;
                    map.set_audit(body);
                };
            }
        }
        Ok(())
    }

    pub fn call_crate_information(&mut self) -> Result<(), StatusCodeSchema<std::io::Error>> {
        Ok(())
    }
}

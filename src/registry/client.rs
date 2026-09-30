use std::{collections::HashMap, thread::sleep};

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use ureq::{
    Body,
    http::{Response, Uri},
};

use crate::{
    constants::*,
    enums::*,
    meta_status,
    registry::{
        cache_registry::CacheRegistry,
        crates_index_response_structs::{CratesIndexItem, CratesIndexResponseParsed},
        crates_io_response_structs::CratesIOResponse,
        rust_sec_json_response_structs::RustsecJsonResponse,
    },
    simple_status,
    status_codes::Module,
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

    pub fn save_cache(&self) {
        self.crates_io_index_cache.clone().save_cache(None);
        self.crates_io_api_cache.clone().save_cache(None);
        self.rustsec_json_cache.clone().save_cache(None);
    }

    pub fn start_calls(&mut self) {
        let uninitialised: Vec<String> = self
            .crates
            .iter()
            .filter(|(_, state)| **state == RegistryClientState::Uninitialised)
            .map(|(k, _)| k.clone())
            .collect();
        simple_status!(
            LevelFilter::Info,
            Module::REGISTRYCLIENT,
            100,
            format!(
                "Starting registry queries for {} uninitialised crate(s)...",
                uninitialised.len()
            )
        );
        for k in uninitialised {
            let mut response_map = CrateResponses::new();
            self.set_crate_state(k.clone(), RegistryClientState::Loading);
            simple_status!(
                LevelFilter::Info,
                Module::REGISTRYCLIENT,
                100,
                format!("Fetching metadata for '{k}'...")
            );
            if let Err(e) = self.call_index(&mut response_map, &k) {
                meta_status!(
                    LevelFilter::Error,
                    Module::REGISTRYCLIENT,
                    500,
                    format!("Failed to query sparse index for '{k}'"),
                    &e
                );
                self.set_crate_state(
                    k,
                    RegistryClientState::Errored(crate::errors::CollectionVersionError::Other(
                        e.to_string(),
                    )),
                );
            } else if let Err(e) = self.call_api(&mut response_map, &k) {
                meta_status!(
                    LevelFilter::Error,
                    Module::REGISTRYCLIENT,
                    500,
                    format!("Failed to query API metadata for '{k}'"),
                    &e
                );
                self.set_crate_state(
                    k,
                    RegistryClientState::Errored(crate::errors::CollectionVersionError::Other(
                        e.to_string(),
                    )),
                );
            } else {
                let _ = self.call_audit(&mut response_map, &k);
                self.set_crate_state(k, RegistryClientState::Succesed(response_map));
            }
            simple_status!(
                LevelFilter::Debug,
                Module::REGISTRYCLIENT,
                100,
                "Rate limit cooldown: waiting 1s for next request."
            );
            sleep(TIME_BETWEEN_REQUESTS);
        }
        simple_status!(
            LevelFilter::Info,
            Module::REGISTRYCLIENT,
            200,
            "Completed registry queries for all crates."
        );
        self.save_cache();
    }

    pub fn set_crate_state(&mut self, c: String, new_state: RegistryClientState) {
        self.crates.insert(c, new_state);
    }

    pub fn get_mut_crate(&mut self, c: &str) -> Option<&mut RegistryClientState> {
        self.crates.get_mut(c)
    }

    pub fn create_user_agent_value(&self, reason: &str) -> String {
        format!(
            "{:}{:}{:} (crates.io: {:} - GitHub: {:})",
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

    pub fn call(
        &self,
        url: Uri,
        reason: &str,
        accept: &str,
    ) -> Result<Response<Body>, ureq::Error> {
        ureq::get(url)
            .header("User-Agent", self.create_user_agent_value(reason))
            .header("accept", accept)
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
                url_path.push(c.split_at(1).0.to_string());
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
        url_path.join("").parse::<Uri>().ok()
    }
    pub fn call_index(&mut self, map: &mut CrateResponses, c: &str) -> Result<(), ureq::Error> {
        match self.crates_io_index_cache.get_cache_entry(c) {
            Some(cached) => {
                simple_status!(
                    LevelFilter::Trace,
                    Module::REGISTRYCLIENT,
                    200,
                    format!("Index for '{c}' retrieved from cache")
                );
                map.set_index(cached);
            }
            None => {
                if let Some(url) = self.parse_index_url(c) {
                    simple_status!(
                        LevelFilter::Trace,
                        Module::REGISTRYCLIENT,
                        100,
                        format!("Querying sparse index for '{c}' from {url}")
                    );
                    let body = self
                        .call(
                            url,
                            DEFAULT_USER_AGENT_HEADER_TYPE_AVAILABILITY,
                            "text/plain",
                        )?
                        .body_mut()
                        .read_to_string()?;
                    let parsed = CratesIndexItem::parse_response(&body);
                    map.set_index(parsed.clone());
                    self.crates_io_index_cache.add_cache_entry(c, parsed);
                };
            }
        };
        simple_status!(
            LevelFilter::Debug,
            Module::REGISTRYCLIENT,
            200,
            format!("Index entry for '{c}' is available.")
        );
        Ok(())
    }

    pub fn parse_api_url(&self, c: &str) -> Option<Uri> {
        let mut url_path: Vec<String> = Vec::new();
        url_path.push(CRATE_IO_API_URL.to_string());
        Self::add_path_slash(&mut url_path);
        url_path.push(c.to_string());
        url_path.push("?keywords=full".to_string());
        url_path.join("").parse::<Uri>().ok()
    }
    pub fn call_api(&mut self, map: &mut CrateResponses, c: &str) -> Result<(), ureq::Error> {
        match self.crates_io_api_cache.get_cache_entry(c) {
            Some(cached) => {
                simple_status!(
                    LevelFilter::Trace,
                    Module::REGISTRYCLIENT,
                    200,
                    format!("API metadata for '{c}' retrieved from cache")
                );
                map.set_api(cached);
            }
            None => {
                if let Some(url) = self.parse_api_url(c) {
                    simple_status!(
                        LevelFilter::Trace,
                        Module::REGISTRYCLIENT,
                        100,
                        format!("Querying crates.io API metadata for '{c}' from {url}")
                    );
                    let body = self
                        .call(
                            url,
                            DEFAULT_USER_AGENT_HEADER_TYPE_FULL_INFORMATION,
                            "application/json",
                        )?
                        .body_mut()
                        .read_json::<Box<CratesIOResponse>>()?;
                    map.set_api(body.clone());
                    self.crates_io_api_cache.add_cache_entry(c, body);
                };
            }
        }
        simple_status!(
            LevelFilter::Debug,
            Module::REGISTRYCLIENT,
            200,
            format!("API details for '{c}' fetched successfully.")
        );
        Ok(())
    }

    pub fn parse_audit_url(&self, c: &str) -> Option<Uri> {
        let mut url_path: Vec<String> = Vec::new();
        url_path.push(RUSTSEC_API_URL.to_string());
        Self::add_path_slash(&mut url_path);
        url_path.push(c.to_string());
        // The uri requests specified <crate>.json
        url_path.push(".json".to_string());
        url_path.join("").parse::<Uri>().ok()
    }
    pub fn call_audit(&mut self, map: &mut CrateResponses, c: &str) -> Result<(), ureq::Error> {
        match self.rustsec_json_cache.get_cache_entry(c) {
            Some(cached) => {
                simple_status!(
                    LevelFilter::Trace,
                    Module::REGISTRYCLIENT,
                    200,
                    format!("RustSec audit reports for '{c}' retrieved from cache")
                );
                map.set_audit(cached);
            }
            None => {
                if let Some(url) = self.parse_audit_url(c) {
                    simple_status!(
                        LevelFilter::Trace,
                        Module::REGISTRYCLIENT,
                        100,
                        format!("Querying RustSec advisory report for '{c}' from {url}")
                    );
                    let res = self.call(
                        url,
                        DEFAULT_USER_AGENT_HEADER_TYPE_AUDIT,
                        "application/json",
                    );

                    let body = match res {
                        Ok(mut response) => {
                            response.body_mut().read_json::<RustsecJsonResponse>()?
                        }
                        // We catch 404 as it means there are no reports on rustsec and is a valid response like an empty array.
                        Err(ureq::Error::StatusCode(404)) => {
                            simple_status!(
                                LevelFilter::Trace,
                                Module::REGISTRYCLIENT,
                                404,
                                format!(
                                    "No security advisories on rustsec.org for '{c}' (404 Not Found)"
                                )
                            );
                            Vec::new()
                        }
                        Err(err) => return Err(err),
                    };

                    map.set_audit(body.clone());
                    self.rustsec_json_cache.add_cache_entry(c, body);
                };
            }
        }
        simple_status!(
            LevelFilter::Debug,
            Module::REGISTRYCLIENT,
            200,
            format!("RustSec audit reports for '{c}' parsed.")
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_url_generation() {
        let client = Client::new();

        let test_cases = [
            ("c", "https://index.crates.io/1/c"),
            ("p", "https://index.crates.io/1/p"),
            ("cc", "https://index.crates.io/2/cc"),
            ("log", "https://index.crates.io/3/l/log"),
            ("num", "https://index.crates.io/3/n/num"),
            ("wyz", "https://index.crates.io/3/w/wyz"),
            ("tokio", "https://index.crates.io/to/ki/tokio"),
            ("serde", "https://index.crates.io/se/rd/serde"),
            ("hashbrown", "https://index.crates.io/ha/sh/hashbrown"),
            ("rand", "https://index.crates.io/ra/nd/rand"),
            ("base64", "https://index.crates.io/ba/se/base64"),
            ("thiserror", "https://index.crates.io/th/is/thiserror"),
            ("regex", "https://index.crates.io/re/ge/regex"),
            ("clap", "https://index.crates.io/cl/ap/clap"),
            ("semver", "https://index.crates.io/se/mv/semver"),
            ("anyhow", "https://index.crates.io/an/yh/anyhow"),
            ("time", "https://index.crates.io/ti/me/time"),
            (
                "rust-recursive-deps-updater",
                "https://index.crates.io/ru/st/rust-recursive-deps-updater",
            ),
        ];

        for (crate_name, expected_index_url) in test_cases {
            let index_uri = client
                .parse_index_url(crate_name)
                .expect("Valid index URL expected");
            assert_eq!(
                index_uri.to_string(),
                expected_index_url,
                "Failed index URL for {}",
                crate_name
            );

            let api_uri = client
                .parse_api_url(crate_name)
                .expect("Valid API URL expected");
            let expected_api_url = format!(
                "https://crates.io/api/v1/crates/{}?keywords=full",
                crate_name
            );
            assert_eq!(
                api_uri.to_string(),
                expected_api_url,
                "Failed API URL for {}",
                crate_name
            );

            let audit_uri = client
                .parse_audit_url(crate_name)
                .expect("Valid audit URL expected");
            let expected_audit_url = format!("https://rustsec.org/packages/{}.json", crate_name);
            assert_eq!(
                audit_uri.to_string(),
                expected_audit_url,
                "Failed audit URL for {}",
                crate_name
            );
        }
    }

    #[test]
    fn test_fixtures_validation() {
        let crates = [
            "tokio",
            "serde",
            "hashbrown",
            "rand",
            "base64",
            "thiserror",
            "log",
            "cc",
            "regex",
            "clap",
            "semver",
            "anyhow",
            "time",
            "num",
        ];

        for crate_name in crates {
            // 1. Index fixture validation
            let index_path = format!("tests/fixtures/{}_index.txt", crate_name);
            let mut index_file =
                File::open(&index_path).unwrap_or_else(|_| panic!("Failed to open {}", index_path));
            let mut index_content = String::new();
            index_file
                .read_to_string(&mut index_content)
                .unwrap_or_else(|_| panic!("Failed to read {}", index_path));
            let index_parsed = CratesIndexItem::parse_response(&index_content);
            assert!(
                !index_parsed.is_empty(),
                "Index parsed items should not be empty for {}",
                crate_name
            );

            // 2. API fixture validation
            let api_path = format!("tests/fixtures/{}_api.json", crate_name);
            let api_file =
                File::open(&api_path).unwrap_or_else(|_| panic!("Failed to open {}", api_path));
            let api_parsed: Result<Box<CratesIOResponse>, _> = serde_json::from_reader(api_file);
            assert!(
                api_parsed.is_ok(),
                "API parsing failed for {}: {:?}",
                crate_name,
                api_parsed.err()
            );
            let api_data = api_parsed.unwrap();
            assert_eq!(
                api_data.crate_field.name, crate_name,
                "Crate name mismatch in API fixture for {}",
                crate_name
            );

            // 3. Rustsec fixture validation
            let rustsec_path = format!("tests/fixtures/{}_rustsec.json", crate_name);
            let rustsec_file = File::open(&rustsec_path)
                .unwrap_or_else(|_| panic!("Failed to open {}", rustsec_path));
            let rustsec_parsed: Result<RustsecJsonResponse, _> =
                serde_json::from_reader(rustsec_file);
            assert!(
                rustsec_parsed.is_ok(),
                "Rustsec parsing failed for {}: {:?}",
                crate_name,
                rustsec_parsed.err()
            );
        }
    }
}

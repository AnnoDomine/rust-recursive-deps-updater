//! HTTP client for crates.io registry and RustSec advisory endpoints.
//!
//! Handles HTTPS requests to the crates.io sparse index, crates.io web API,
//! and RustSec advisories with rate limiting, user-agent generation, and cache integration.

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
    errors::CollectionVersionError,
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

/// Lightweight summary record of a crates.io crate version.
pub struct CratesIOResponseItem {
    /// Name of the crate.
    pub name: String,
    /// Latest version string.
    pub version: String,
    /// Unique identifier.
    pub id: String,
}

/// Registry HTTP client managing connection pools, query execution, rate limits, and caches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    /// Map tracking query state for each target crate.
    #[serde(default)]
    pub crates: HashMap<String, RegistryClientState>,
    /// Cache registry for sparse index responses.
    pub crates_io_index_cache: CacheRegistry<CratesIndexResponseParsed>,
    /// Cache registry for web API metadata responses.
    pub crates_io_api_cache: CacheRegistry<Box<CratesIOResponse>>,
    /// Cache registry for RustSec advisory responses.
    pub rustsec_json_cache: CacheRegistry<RustsecJsonResponse>,
}

impl Default for CrateResponses {
    fn default() -> Self {
        Self::new()
    }
}

impl CrateResponses {
    /// Creates a new empty response aggregate.
    pub fn new() -> Self {
        Self {
            index: None,
            api: None,
            audit: None,
        }
    }

    /// Sets the parsed sparse index response.
    ///
    /// # Arguments
    /// * `value` - Parsed index response.
    pub fn set_index(&mut self, value: CratesIndexResponseParsed) {
        self.index = Some(value);
    }

    /// Sets the crates.io API metadata response.
    ///
    /// # Arguments
    /// * `value` - Boxed API response.
    pub fn set_api(&mut self, value: Box<CratesIOResponse>) {
        self.api = Some(value);
    }

    /// Sets the RustSec advisory response.
    ///
    /// # Arguments
    /// * `value` - RustSec JSON advisory response.
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
    /// Creates a new registry client instance with initialized disk caches.
    pub fn new() -> Self {
        Self {
            crates: HashMap::new(),
            crates_io_index_cache: CacheRegistry::new(CacheType::CratesIOIndex),
            crates_io_api_cache: CacheRegistry::new(CacheType::CratesIOApi),
            rustsec_json_cache: CacheRegistry::new(CacheType::RustsecJsonResponse),
        }
    }

    /// Populates client state with collected workspace dependencies.
    ///
    /// # Arguments
    /// * `crates` - Map of crate names and their current collection status.
    pub fn map_crates(&mut self, crates: HashMap<String, DependencyCollectionVersion>) {
        for (dep, status) in crates {
            if let DependencyCollectionVersion::None = status {
                self.crates
                    .insert(dep.clone(), RegistryClientState::Uninitialised);
            }
        }
    }

    /// Maps registry responses to the collected dependencies map with their latest resolved version or error.
    ///
    /// # Arguments
    /// * `collected_deps` - Mutable map collecting resolved dependency versions.
    pub fn map_latest_by_api(
        &self,
        collected_deps: &mut HashMap<String, DependencyCollectionVersion>,
    ) {
        for (dep, entry) in &self.crates {
            if let RegistryClientState::Succesed(c) = entry
                && let Some(api) = &c.api
            {
                collected_deps.insert(
                    dep.to_string(),
                    DependencyCollectionVersion::Latest(api.crate_field.newest_version.to_string()),
                );
            } else if let RegistryClientState::Errored(e) = entry {
                collected_deps.insert(
                    dep.to_string(),
                    DependencyCollectionVersion::Error(e.clone()),
                );
            } else {
                collected_deps.insert(
                    dep.to_string(),
                    DependencyCollectionVersion::Error(CollectionVersionError::Other(
                        "Unexpected error".to_string(),
                    )),
                );
            }
        }
    }

    fn get_newest_index(&self, index: &[CratesIndexItem]) -> Option<CratesIndexItem> {
        let unyanked = index.iter().filter(|e| !e.yanked);
        let stable = unyanked
            .clone()
            .filter(|e| semver::Version::parse(&e.vers).is_ok_and(|v| v.pre.is_empty()));
        stable
            .max_by_key(|e| semver::Version::parse(&e.vers).ok())
            .or_else(|| unyanked.max_by_key(|e| semver::Version::parse(&e.vers).ok()))
            .cloned()
    }

    /// Maps sparse index responses to the collected dependencies map with their latest resolved version or error.
    ///
    /// # Arguments
    /// * `collected_deps` - Mutable map collecting resolved dependency versions.
    pub fn map_latest_by_index(
        &self,
        collected_deps: &mut HashMap<String, DependencyCollectionVersion>,
    ) {
        for (dep, entry) in &self.crates {
            if let RegistryClientState::Succesed(c) = entry
                && let Some(index) = &c.index
                && let Some(newest) = self.get_newest_index(index)
            {
                collected_deps.insert(
                    dep.to_string(),
                    DependencyCollectionVersion::Latest(newest.vers),
                );
            } else if let RegistryClientState::Errored(e) = entry {
                collected_deps.insert(
                    dep.to_string(),
                    DependencyCollectionVersion::Error(e.clone()),
                );
            } else {
                collected_deps.insert(
                    dep.to_string(),
                    DependencyCollectionVersion::Error(CollectionVersionError::Other(
                        "Unexpected error".to_string(),
                    )),
                );
            }
        }
    }

    /// Executes registry queries sequentially for all uninitialized crates, respecting rate limits.
    pub fn start_calls(&mut self, is_headless: bool) {
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
            match self.call_index(&mut response_map, &k) {
                Err(e) => {
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
                }
                Ok(_) => {
                    if !is_headless {
                        if let Err(e) = self.call_api(&mut response_map, &k) {
                            meta_status!(
                                LevelFilter::Error,
                                Module::REGISTRYCLIENT,
                                500,
                                format!("Failed to query API metadata for '{k}'"),
                                &e
                            );
                            self.set_crate_state(
                                k.clone(),
                                RegistryClientState::Errored(
                                    crate::errors::CollectionVersionError::Other(e.to_string()),
                                ),
                            );
                        } else {
                            let _ = self.call_audit(&mut response_map, &k);
                            self.set_crate_state(k, RegistryClientState::Succesed(response_map));
                        }
                    } else {
                        self.set_crate_state(k, RegistryClientState::Succesed(response_map));
                    }
                }
            }
        }
        simple_status!(
            LevelFilter::Info,
            Module::REGISTRYCLIENT,
            200,
            "Completed registry queries for all crates."
        );
    }

    /// Updates the query state for a specific crate.
    ///
    /// # Arguments
    /// * `c` - Crate name.
    /// * `new_state` - New state to assign.
    pub fn set_crate_state(&mut self, c: String, new_state: RegistryClientState) {
        self.crates.insert(c, new_state);
    }

    /// Retrieves a mutable reference to a crate's query state.
    ///
    /// # Arguments
    /// * `c` - Crate name.
    pub fn get_mut_crate(&mut self, c: &str) -> Option<&mut RegistryClientState> {
        self.crates.get_mut(c)
    }

    /// Formats a policy-compliant User-Agent header value.
    ///
    /// # Arguments
    /// * `reason` - Descriptive purpose tag for this query.
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

    /// Dispatches an HTTPS GET request with User-Agent and Accept headers.
    ///
    /// # Arguments
    /// * `url` - Target endpoint URI.
    /// * `reason` - Operation identifier for User-Agent header.
    /// * `accept` - Expected MIME type for the Accept header.
    ///
    /// # Returns
    /// The HTTP [`Response`] from `ureq`.
    ///
    /// # Errors
    /// Returns [`ureq::Error`] on network or HTTP protocol failure.
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

    /// Constructs the sparse index URL for a crate name according to crates.io prefixing rules.
    ///
    /// - 0 chars -> Invalid (returns `None`)
    /// - 1 char -> <https://index.crates.io/1/f>
    /// - 2 chars -> <https://index.crates.io/2/fo>
    /// - 3 chars -> <https://index.crates.io/3/foo>
    /// - 4+ chars -> <https://index.crates.io/fo/ob/foobar>
    ///
    /// # Arguments
    /// * `c` - Crate name.
    ///
    /// # Returns
    /// Parsed [`Uri`], or `None` if invalid.
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

    /// Queries or loads from cache the sparse index for the specified crate.
    ///
    /// # Arguments
    /// * `map` - Target aggregate response container.
    /// * `c` - Crate name.
    ///
    /// # Errors
    /// Returns [`ureq::Error`] if network fetch fails.
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
                    self.crates_io_index_cache.save_cache(None);
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

    /// Constructs the crates.io REST API URL for a crate.
    ///
    /// # Arguments
    /// * `c` - Crate name.
    ///
    /// # Returns
    /// Parsed [`Uri`], or `None` if formatting fails.
    pub fn parse_api_url(&self, c: &str) -> Option<Uri> {
        let mut url_path: Vec<String> = Vec::new();
        url_path.push(CRATE_IO_API_URL.to_string());
        Self::add_path_slash(&mut url_path);
        url_path.push(c.to_string());
        url_path.push("?keywords=full".to_string());
        url_path.join("").parse::<Uri>().ok()
    }

    /// Queries or loads from cache the crates.io REST API metadata for the specified crate.
    ///
    /// # Arguments
    /// * `map` - Target aggregate response container.
    /// * `c` - Crate name.
    ///
    /// # Errors
    /// Returns [`ureq::Error`] if network fetch fails.
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
                    self.crates_io_api_cache.save_cache(None);
                };
                simple_status!(
                    LevelFilter::Debug,
                    Module::REGISTRYCLIENT,
                    100,
                    "Rate limit cooldown: waiting 1s for next request."
                );
                sleep(TIME_BETWEEN_REQUESTS);
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

    /// Constructs the RustSec vulnerability database URL for a crate.
    ///
    /// # Arguments
    /// * `c` - Crate name.
    ///
    /// # Returns
    /// Parsed [`Uri`], or `None` if formatting fails.
    pub fn parse_audit_url(&self, c: &str) -> Option<Uri> {
        let mut url_path: Vec<String> = Vec::new();
        url_path.push(RUSTSEC_API_URL.to_string());
        Self::add_path_slash(&mut url_path);
        url_path.push(c.to_string());
        // The uri requests specified <crate>.json
        url_path.push(".json".to_string());
        url_path.join("").parse::<Uri>().ok()
    }

    /// Queries or loads from cache the RustSec vulnerability reports for the specified crate.
    ///
    /// # Arguments
    /// * `map` - Target aggregate response container.
    /// * `c` - Crate name.
    ///
    /// # Errors
    /// Returns [`ureq::Error`] on non-404 network failure.
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
                    self.rustsec_json_cache.save_cache(None);
                };
                simple_status!(
                    LevelFilter::Debug,
                    Module::REGISTRYCLIENT,
                    100,
                    "Rate limit cooldown: waiting 1s for next request."
                );
                sleep(TIME_BETWEEN_REQUESTS);
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

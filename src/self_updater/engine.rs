use std::process;

use log::LevelFilter;
use self_update::{
    Release, ReleaseAsset, ReleaseStatus,
    backends::github::{Update, UpdateBuilder},
};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        BIN_NAME, CARGO_INSTALL_BIN_NAME, CRATE_IO_API_URL, DEFAULT_USER_AGENT_HEADER_APPLICATION,
        DEFAULT_USER_AGENT_HEADER_CRATES_IO_URL, DEFAULT_USER_AGENT_HEADER_GITHUB_URL,
        DEFAULT_USER_AGENT_HEADER_TYPE_SELF_UPDATE,
    },
    errors::RrduConfigResult,
    logger::status_codes::Module,
    registry::{
        cache_registry::CacheRegistry,
        crates_io_response_structs::{Crate, CratesIOResponse},
    },
    simple_status,
};

const HTTP: &str = "http://";
const HTTPS: &str = "https://";
const DOMAIN: &str = "github.com/";

/// Represents an asset attached to a GitHub release.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RrduReleaseAsset {
    /// Asset filename.
    pub name: String,
    /// Direct download URL for the asset.
    pub download_url: String,
    /// Optional cryptographic digest/hash of the asset.
    pub digest: Option<String>,
}

impl RrduReleaseAsset {
    /// Creates a new `RrduReleaseAsset` from a `self_update::ReleaseAsset`.
    ///
    /// # Arguments
    /// * `asset` - The upstream release asset to convert.
    ///
    /// # Returns
    /// A converted `RrduReleaseAsset` instance.
    pub fn new(asset: ReleaseAsset) -> Self {
        Self {
            name: asset.name().to_string(),
            download_url: asset.download_url().to_string(),
            digest: asset.digest().map(|d| d.to_string()),
        }
    }
}

/// Represents a GitHub release metadata record.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RrduRelease {
    /// Display name of the release.
    pub name: String,
    /// SemVer or tag version string of the release.
    pub version: String,
    /// Publication timestamp or date string.
    pub date: String,
    /// Markdown description or changelog body of the release.
    pub body: Option<String>,
    /// Web URL pointing to the release notes on GitHub.
    pub release_notes_url: Option<String>,
    /// List of assets associated with this release.
    pub assets: Vec<RrduReleaseAsset>,
}

impl RrduRelease {
    /// Creates a new `RrduRelease` from a `self_update::Release`.
    ///
    /// # Arguments
    /// * `release` - The upstream release metadata to convert.
    ///
    /// # Returns
    /// A converted `RrduRelease` instance.
    pub fn new(release: Release) -> Self {
        let assets: Vec<RrduReleaseAsset> = release
            .assets()
            .iter()
            .map(|a| RrduReleaseAsset::new(a.clone()))
            .collect();
        Self {
            name: release.name().to_string(),
            version: release.version().to_string(),
            date: release.date().to_string(),
            body: release.body().map(|b| b.to_string()),
            release_notes_url: release.release_notes_url().map(|rn| rn.to_string()),
            assets,
        }
    }
}

/// Status of release updates comparing local version with remote releases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RrduReleaseStatus {
    /// Local binary is fully up to date.
    UpToDate,
    /// A newer release is available with accompanying release details.
    Updated(RrduRelease),
}

/// Aggregated release notes and update status from GitHub Releases and crates.io.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseNotes {
    /// Release status and metadata retrieved from GitHub.
    pub gh: Option<RrduReleaseStatus>,
    /// Crate metadata retrieved from crates.io.
    pub ci: Option<Crate>,
}

impl PartialEq for ReleaseNotes {
    fn eq(&self, other: &Self) -> bool {
        let is_gh_none = self.gh.is_none() && other.gh.is_none();
        let is_ci_none = self.ci.is_none() && other.ci.is_none();
        let is_gh_some_eq = if let Some(s_gh) = self.gh.clone()
            && let Some(o_gh) = other.gh.clone()
        {
            matches!(
                (s_gh, o_gh),
                (RrduReleaseStatus::UpToDate, RrduReleaseStatus::UpToDate)
                    | (
                        RrduReleaseStatus::Updated(..),
                        RrduReleaseStatus::Updated(..)
                    )
            )
        } else {
            false
        };
        let is_ci_some_eq = if let Some(s_ci) = self.ci.clone()
            && let Some(o_ci) = other.ci.clone()
        {
            s_ci == o_ci
        } else {
            false
        };

        is_gh_none || is_ci_none || is_gh_some_eq || is_ci_some_eq
    }
}

impl Eq for ReleaseNotes {}

impl ReleaseNotes {
    /// Creates an empty `ReleaseNotes` container.
    ///
    /// # Returns
    /// A default `ReleaseNotes` instance with uninitialized release sources.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records GitHub release metadata as an updated release.
    ///
    /// # Arguments
    /// * `release` - The GitHub release information to associate.
    pub fn set_git_hub(&mut self, release: Release) {
        self.gh = Some(RrduReleaseStatus::Updated(RrduRelease::new(release)));
    }

    /// Records crates.io release metadata.
    ///
    /// # Arguments
    /// * `crate_info` - Crate details fetched from crates.io.
    pub fn set_crates_io(&mut self, crate_info: Crate) {
        self.ci = Some(crate_info.clone());
    }
}

/// Self-update engine managing version checks against crates.io and GitHub Releases.
///
/// Cached in user space with a 1-day TTL to avoid redundant network requests.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateEngine {
    /// Whether a newer version is available on crates.io.
    pub crates_io: bool,
    /// Whether a newer version is available on GitHub Releases.
    pub git_hub: bool,
    /// Aggregated release notes from both upstream sources.
    pub releases: ReleaseNotes,
}

impl SelfUpdateEngine {
    /// Initializes or loads a cached `SelfUpdateEngine` instance.
    ///
    /// If a valid cache entry exists, it is returned directly. Otherwise, fresh
    /// update states are queried from GitHub and crates.io, and written to cache.
    ///
    /// # Returns
    /// An initialized `SelfUpdateEngine` instance.
    ///
    /// # Errors
    /// Returns `RrduConfigResult::Err` if remote API requests or cache serialization fails.
    pub fn new() -> RrduConfigResult<Self> {
        let cache: CacheRegistry<SelfUpdateEngine> =
            CacheRegistry::new(crate::enums::CacheType::SelfUpdate);

        if let Some(entry) = cache.get_cache_entry("self_update") {
            return Ok(entry.clone());
        }

        let mut engine = Self {
            crates_io: false,
            git_hub: false,
            releases: ReleaseNotes::new(),
        };

        engine.request_update_states()?;

        engine.cache();

        Ok(engine)
    }

    fn get_user_agent(&self) -> String {
        format!(
            "{:}{:}{:} (crates.io: {:} - GitHub: {:})",
            DEFAULT_USER_AGENT_HEADER_APPLICATION,
            env!("CARGO_PKG_VERSION"),
            DEFAULT_USER_AGENT_HEADER_TYPE_SELF_UPDATE,
            DEFAULT_USER_AGENT_HEADER_CRATES_IO_URL,
            DEFAULT_USER_AGENT_HEADER_GITHUB_URL,
        )
    }

    fn get_repo(&self) -> String {
        env!("CARGO_PKG_REPOSITORY")
            .replace(HTTP, "")
            .replace(HTTPS, "")
            .replace(DOMAIN, "")
    }

    fn get_update_builder(&self) -> UpdateBuilder {
        let repo_binding = self.get_repo();
        let owner_repo = repo_binding.split("/").collect::<Vec<&str>>();
        let user_agent = self.get_user_agent();
        let mut binding = Update::configure();
        let builder = binding
            .repo_owner(owner_repo[0])
            .repo_name(owner_repo[1])
            .bin_name(BIN_NAME)
            .show_download_progress(true)
            .request_header("User-Agent", user_agent)
            .check_install_path_writable(true)
            .current_version(env!("CARGO_PKG_VERSION"));
        builder.clone()
    }

    fn get_git_hub(&mut self) -> RrduConfigResult<()> {
        let status = self.get_update_builder().build()?;

        // Based on the docs of 'is_update_available'
        // **returns the newest strictly-newer Release, or None when already up to date.**
        // So if github responded and the method returns 'some' a new version is available.
        self.git_hub = status.is_update_available()?.is_some();
        simple_status!(
            LevelFilter::Trace,
            Module::UPDATER,
            202,
            format!("GitHub response. Is outdaten: {}", self.git_hub)
        );

        let release = status.get_release_version(env!("CARGO_PKG_VERSION"))?;
        self.releases.set_git_hub(release.clone());
        simple_status!(
            LevelFilter::Trace,
            Module::UPDATER,
            202,
            format!(
                "GitHub response. Release notes of current version: {:#?}",
                release
            )
        );
        Ok(())
    }

    fn compare_crates_io_versions(&mut self, res: Box<CratesIOResponse>) {
        let default_version = res.clone().crate_field.default_version;
        let current_version = env!("CARGO_PKG_VERSION");

        if let (Ok(mut current_ver), Ok(mut latest_ver)) = (
            semver::Version::parse(current_version),
            semver::Version::parse(&default_version),
        ) {
            // SemVer 2.0 needs to cut by the build-metadate
            current_ver.build = semver::BuildMetadata::EMPTY;
            latest_ver.build = semver::BuildMetadata::EMPTY;

            if current_ver <= latest_ver {
                self.crates_io = true;
                self.releases.set_crates_io(res.crate_field.clone());
            }
        };
    }

    fn get_creates_io(&mut self) -> RrduConfigResult<()> {
        let url: Vec<String> = vec![
            CRATE_IO_API_URL.to_string(),
            "/".to_string(),
            CARGO_INSTALL_BIN_NAME.to_string(),
            "?keywords=full".to_string(),
        ];

        let res = ureq::get(url.join(""))
            .header("User-Agent", self.get_user_agent())
            .header("accept", "application/json")
            .call()?
            .body_mut()
            .read_json::<Box<CratesIOResponse>>()?;

        self.compare_crates_io_versions(res.clone());

        Ok(())
    }

    fn cache(&self) {
        let mut cache: CacheRegistry<SelfUpdateEngine> =
            CacheRegistry::new(crate::enums::CacheType::SelfUpdate);
        cache.add_cache_entry("self_update", self.clone());
        cache.save_cache(None);
    }

    /// Run `self_update` Update with GitHub process.
    pub fn update_with_github(&mut self) -> RrduConfigResult<()> {
        if let ReleaseStatus::Updated(release) =
            self.get_update_builder().build()?.update_extended()?
        {
            self.releases.set_git_hub(release);
            self.cache();
        };

        Ok(())
    }

    /// Starts update with cargo binary.
    /// # WIP
    /// Executes `cargo install rust-recursive-deps-updater@latest` in a `same-privileged` process.
    pub fn update_with_cargo(&mut self) -> RrduConfigResult<()> {
        let mut cmd = process::Command::new("cargo");
        cmd.args(["install", CARGO_INSTALL_BIN_NAME]);
        let sdtout = cmd.output()?;
        simple_status!(
            LevelFilter::Info,
            Module::UPDATER,
            200,
            format!("RRDU updated with cargo for user!\n{:#?}", &sdtout)
        );
        self.cache();

        Ok(())
    }

    fn request_update_states(&mut self) -> RrduConfigResult<()> {
        self.get_git_hub()?;
        self.get_creates_io()?;

        Ok(())
    }
}

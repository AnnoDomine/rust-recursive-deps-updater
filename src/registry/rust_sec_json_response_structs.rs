//! RustSec vulnerability and advisory response models conforming to OSV format.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Collection of RustSec security advisory records for a crate.
pub type RustsecJsonResponse = Vec<RustsecItem>;

/// Represents a single RustSec advisory record (OSV format).
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustsecItem {
    /// Unique advisory identifier (e.g. `RUSTSEC-2021-0001`).
    pub id: String,
    /// Last modification timestamp.
    pub modified: String,
    /// Original publication timestamp.
    pub published: String,
    /// Common Vulnerabilities and Exposures (CVE) aliases.
    pub aliases: Vec<String>,
    /// Related advisory identifiers.
    pub related: Vec<Value>,
    /// Brief summary of the vulnerability.
    pub summary: String,
    /// Detailed description and impact analysis.
    pub details: String,
    /// Severity ratings (CVSS).
    pub severity: Vec<Severity>,
    /// Affected packages, versions, and architectures.
    pub affected: Vec<Affected>,
    /// External references and documentation links.
    pub references: Vec<Reference>,
    /// Database-specific metadata including license.
    #[serde(rename = "database_specific")]
    pub database_specific: DatabaseSpecific2,
}

/// Vulnerability severity scoring information.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Severity {
    /// Scoring methodology (e.g. `CVSS_V3`).
    #[serde(rename = "type")]
    pub type_field: String,
    /// Numeric score or CVSS vector string.
    pub score: String,
}

/// Details of a package affected by an advisory.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Affected {
    /// Target package identification.
    pub package: Package,
    /// Ecosystem-specific affected functions and architectures.
    #[serde(rename = "ecosystem_specific")]
    pub ecosystem_specific: EcosystemSpecific,
    /// RustSec-specific advisory categorizations.
    #[serde(rename = "database_specific")]
    pub database_specific: DatabaseSpecific,
    /// SemVer version ranges affected by the advisory.
    pub ranges: Vec<Range>,
    /// Specific version release values affected.
    pub versions: Vec<Value>,
}

/// Package identity within a software ecosystem.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    /// Ecosystem identifier (e.g. `crates.io`).
    pub ecosystem: String,
    /// Package or crate name.
    pub name: String,
    /// Package URL (purl) identifier.
    pub purl: String,
}

/// Ecosystem-specific details of affected components.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcosystemSpecific {
    /// Target environments, architectures, and functions affected.
    pub affects: Affects,
    /// Specific affected function paths.
    #[serde(rename = "affected_functions")]
    pub affected_functions: Value,
}

/// Architecture, operating system, and function impact scope.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Affects {
    /// Hardware architectures affected (e.g. `x86_64`).
    pub arch: Vec<Value>,
    /// Operating systems affected (e.g. `linux`, `windows`).
    pub os: Vec<String>,
    /// Vulnerable function or method names.
    pub functions: Vec<String>,
}

/// Database-specific advisory categories and CVSS metrics.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSpecific {
    /// Vulnerability category tags (e.g. `memory-corruption`).
    pub categories: Vec<String>,
    /// Optional CVSS score string.
    pub cvss: Option<String>,
    /// Informational flags or notices.
    pub informational: Value,
}

/// Version range event specification.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    /// Version range format type (e.g. `SEMVER`).
    #[serde(rename = "type")]
    pub type_field: String,
    /// Sequence of introduction and fix events.
    pub events: Vec<Event>,
}

/// Version milestone event (introduction or patch).
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Version where the vulnerability was first introduced.
    pub introduced: Option<String>,
    /// Version containing the patch or fix.
    pub fixed: Option<String>,
}

/// External reference or URL link.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    /// Reference type tag (e.g. `WEB`, `ADVISORY`).
    #[serde(rename = "type")]
    pub type_field: String,
    /// Target URL string.
    pub url: String,
}

/// Database license metadata.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSpecific2 {
    /// Database license string (e.g. `CC0-1.0`).
    pub license: String,
}

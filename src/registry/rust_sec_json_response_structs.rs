use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type RustsecJsonResponse = Vec<RustsecItem>;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustsecItem {
    pub id: String,
    pub modified: String,
    pub published: String,
    pub aliases: Vec<String>,
    pub related: Vec<Value>,
    pub summary: String,
    pub details: String,
    pub severity: Vec<Severity>,
    pub affected: Vec<Affected>,
    pub references: Vec<Reference>,
    #[serde(rename = "database_specific")]
    pub database_specific: DatabaseSpecific2,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Severity {
    #[serde(rename = "type")]
    pub type_field: String,
    pub score: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Affected {
    pub package: Package,
    #[serde(rename = "ecosystem_specific")]
    pub ecosystem_specific: EcosystemSpecific,
    #[serde(rename = "database_specific")]
    pub database_specific: DatabaseSpecific,
    pub ranges: Vec<Range>,
    pub versions: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub ecosystem: String,
    pub name: String,
    pub purl: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcosystemSpecific {
    pub affects: Affects,
    #[serde(rename = "affected_functions")]
    pub affected_functions: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Affects {
    pub arch: Vec<Value>,
    pub os: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSpecific {
    pub categories: Vec<String>,
    pub cvss: String,
    pub informational: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    #[serde(rename = "type")]
    pub type_field: String,
    pub events: Vec<Event>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub introduced: Option<String>,
    pub fixed: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSpecific2 {
    pub license: String,
}

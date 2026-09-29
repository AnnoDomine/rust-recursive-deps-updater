use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::status_codes::StatusCodeSchema;

pub type CratesIndexResponseParsed = Vec<CratesIndexItem>;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CratesIndexItem {
    pub name: String,
    pub vers: String,
    pub deps: Vec<Dep>,
    pub cksum: String,
    pub features: Features,
    pub yanked: bool,
    pub pubtime: String,
}

impl CratesIndexItem {
    fn parse_item(item: &str) -> Option<CratesIndexItem> {
        match serde_json::de::from_str::<CratesIndexItem>(item) {
            Ok(parsed_item) => Some(parsed_item),
            Err(serde_json_error) => {
                println!(
                    "{:}",
                    StatusCodeSchema::with_meta(
                        crate::status_codes::Module::REGISTRYCLIENT,
                        400,
                        "Error while parsing index item.".to_string(),
                        serde_json_error
                    )
                );
                None
            }
        }
    }
    pub fn parse_response(response: String) -> CratesIndexResponseParsed {
        response
            .split("\n")
            .flat_map(CratesIndexItem::parse_item)
            .collect()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dep {
    pub name: String,
    pub req: String,
    pub features: Vec<String>,
    pub optional: bool,
    #[serde(rename = "default_features")]
    pub default_features: bool,
    pub target: Value,
    pub kind: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Features {}

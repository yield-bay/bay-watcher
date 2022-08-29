use serde::{Deserialize, Serialize};
use serde_json::Value;

// assets

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetsRoot {
    pub name: String,
    pub timestamp: String,
    pub version: Version,
    pub tags: Tags,
    #[serde(rename = "logoURL")]
    pub logo_url: String,
    pub keywords: Vec<String>,
    pub tokens: Vec<Token>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tags {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub chain_id: i64,
    pub address: String,
    pub symbol: String,
    pub decimals: i64,
    #[serde(rename = "logoURL")]
    pub logo_url: String,
}

// pairs

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    #[serde(rename = "base_id")]
    pub base_id: String,
    #[serde(rename = "base_name")]
    pub base_name: String,
    #[serde(rename = "base_symbol")]
    pub base_symbol: String,
    #[serde(rename = "quote_id")]
    pub quote_id: String,
    #[serde(rename = "quote_name")]
    pub quote_name: String,
    #[serde(rename = "quote_symbol")]
    pub quote_symbol: String,
    #[serde(rename = "last_price")]
    pub last_price: String,
    #[serde(rename = "base_volume")]
    pub base_volume: String,
    #[serde(rename = "quote_volume")]
    pub quote_volume: String,
}

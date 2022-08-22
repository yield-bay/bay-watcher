use serde::{Deserialize, Serialize};

use std::fmt;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub address: String,
    pub chain: String,
    pub protocol: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub logos: Vec<String>,
    pub price: f64,
    pub liquidity: f64,
    pub total_supply: f64,
    #[serde(rename = "isLP")]
    pub is_lp: bool,
    #[serde(rename = "feesAPR")]
    pub fees_apr: f64,
    pub underlying_assets: Vec<String>,
    pub underlying_assets_alloc: Vec<f64>,
    #[serde(rename = "lastUpdatedAtUTC")]
    pub last_updated_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Farm {
    pub id: i32,
    pub chef: String,
    pub chain: String,
    pub protocol: String,
    pub farm_type: String,
    pub farm_impl: String,
    pub asset: FarmAsset,
    pub tvl: f64,
    pub apr: APR,
    pub rewards: Vec<Reward>,
    pub alloc_point: u32,
    #[serde(rename = "lastUpdatedAtUTC")]
    pub last_updated_at_utc: String,
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct UnderlyingAsset {
//     pub symbol: String,
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmAsset {
    pub symbol: String,
    pub address: String,
    pub price: f64,
    pub logos: Vec<String>,
    // pub underlying_assets: Vec<FarmAsset>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APR {
    pub reward: f64,
    pub base: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub amount: f64,
    pub asset: String,
    #[serde(rename = "valueUSD")]
    pub value_usd: f64,
    pub freq: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FarmType {
    StandardAmm,
    StableAmm,
    SingleStaking,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FarmImplementation {
    Solidity,
    Vyper,
    Ink,
    Pallet,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Annually,
}

impl fmt::Display for FarmType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for FarmImplementation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Freq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Value;

// GetFactoGauges

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFactoGaugesRoot {
    pub success: bool,
    pub data: Option<GetFactoGaugesData>,
    pub generated_time_ms: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFactoGaugesData {
    pub gauges: Vec<Gauge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gauge {
    pub gauge: String,
    #[serde(rename = "swap_token")]
    pub swap_token: String,
    pub name: String,
    pub symbol: String,
    pub has_crv: bool,
    #[serde(rename = "side_chain")]
    pub side_chain: bool,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "gauge_data")]
    pub gauge_data: GaugeData,
    #[serde(rename = "swap_data")]
    pub swap_data: SwapData,
    pub lp_token_price: f64,
    pub swap: String,
    pub rewards_need_nudging: bool,
    pub are_crv_rewards_stuck_in_bridge: bool,
    pub extra_rewards: Vec<ExtraReward>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeData {
    #[serde(rename = "working_supply")]
    pub working_supply: String,
    pub total_supply: String,
    #[serde(rename = "gauge_relative_weight")]
    pub gauge_relative_weight: String,
    #[serde(rename = "get_gauge_weight")]
    pub get_gauge_weight: String,
    #[serde(rename = "inflation_rate")]
    pub inflation_rate: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    #[serde(rename = "virtual_price")]
    pub virtual_price: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraReward {
    pub gauge_address: String,
    pub token_address: String,
    pub token_price: f64,
    pub name: String,
    pub symbol: String,
    pub decimals: String,
    pub apy_data: ApyData,
    pub apy: f64,
    pub meta_data: MetaData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApyData {
    pub is_reward_still_active: bool,
    pub token_price: f64,
    pub rate: f64,
    pub total_supply: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    pub rate: String,
    pub period_finish: i64,
}

// GetFactoryAPYs

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFactoryAPYsRoot {
    pub success: bool,
    pub data: Option<GetFactoryAPYsData>,
    pub generated_time_ms: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFactoryAPYsData {
    pub pool_details: Vec<PoolDetail>,
    pub total_volume: f64,
    // pub latest: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolDetail {
    pub index: i64,
    pub pool_address: String,
    pub pool_symbol: String,
    pub apy_formatted: String,
    pub apy: f64,
    pub virtual_price: Value,
    pub volume: f64,
}

// GetFactoryV2Pools

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFactoryV2PoolsRoot {
    pub success: bool,
    pub data: Option<GetFactoryV2PoolsData>,
    pub generated_time_ms: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFactoryV2PoolsData {
    pub pool_data: Vec<PoolDaum>,
    pub tvl_all: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolDaum {
    pub id: String,
    pub address: String,
    pub coins_addresses: Vec<String>,
    pub decimals: Vec<String>,
    #[serde(deserialize_with = "float_or_string")]
    pub virtual_price: f64,
    pub underlying_decimals: Option<Vec<String>>,
    pub asset_type: String,
    pub total_supply: String,
    pub implementation_address: String,
    pub name: String,
    pub symbol: String,
    pub implementation: String,
    pub asset_type_name: String,
    pub coins: Vec<Coin>,
    pub usd_total: f64,
    pub is_meta_pool: bool,
    pub usd_total_excluding_base_pool: f64,
}

fn float_or_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(num) => {
            if let Some(float) = num.as_f64() {
                Ok(float)
            } else {
                Err(serde::de::Error::custom("Failed to parse float"))
            }
        }
        serde_json::Value::String(string) => {
            if let Ok(float) = string.parse::<f64>() {
                Ok(float)
            } else {
                Err(serde::de::Error::custom("Failed to parse float"))
            }
        }
        _ => Err(serde::de::Error::custom("Expected a number or a string")),
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coin {
    pub address: String,
    pub usd_price: Option<f64>,
    pub decimals: String,
    pub is_base_pool_lp_token: bool,
    pub symbol: String,
    pub pool_balance: String,
}

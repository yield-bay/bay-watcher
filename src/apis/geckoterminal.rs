use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub attributes: Attributes,
    pub relationships: Relationships,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub address: String,
    pub name: String,
    #[serde(rename = "fully_diluted_valuation")]
    pub fully_diluted_valuation: String,
    #[serde(rename = "from_volume_in_usd")]
    pub from_volume_in_usd: String,
    #[serde(rename = "to_volume_in_usd")]
    pub to_volume_in_usd: String,
    #[serde(rename = "swap_count_24h")]
    pub swap_count_24h: i64,
    #[serde(rename = "reserve_threshold_met")]
    pub reserve_threshold_met: bool,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: String,
    #[serde(rename = "base_token_id")]
    pub base_token_id: String,
    #[serde(rename = "reserve_in_usd")]
    pub reserve_in_usd: String,
    #[serde(rename = "pool_fee")]
    pub pool_fee: Value,
    #[serde(rename = "swap_url")]
    pub swap_url: String,
    #[serde(rename = "historical_data")]
    pub historical_data: HistoricalData,
    #[serde(rename = "price_percent_change")]
    pub price_percent_change: String,
    #[serde(rename = "price_percent_changes")]
    pub price_percent_changes: PricePercentChanges,
    #[serde(rename = "sentiment_votes")]
    pub sentiment_votes: SentimentVotes,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalData {
    #[serde(rename = "last_5m")]
    pub last_5m: Last5m,
    #[serde(rename = "last_15m")]
    pub last_15m: Last15m,
    #[serde(rename = "last_30m")]
    pub last_30m: Last30m,
    #[serde(rename = "last_1h")]
    pub last_1h: Last1h,
    #[serde(rename = "last_6h")]
    pub last_6h: Last6h,
    #[serde(rename = "last_24h")]
    pub last_24h: Last24h,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Last5m {
    #[serde(rename = "swaps_count")]
    pub swaps_count: i64,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: String,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: String,
    #[serde(rename = "buy_swaps_count")]
    pub buy_swaps_count: i64,
    #[serde(rename = "sell_swaps_count")]
    pub sell_swaps_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Last15m {
    #[serde(rename = "swaps_count")]
    pub swaps_count: i64,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: String,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: String,
    #[serde(rename = "buy_swaps_count")]
    pub buy_swaps_count: i64,
    #[serde(rename = "sell_swaps_count")]
    pub sell_swaps_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Last30m {
    #[serde(rename = "swaps_count")]
    pub swaps_count: i64,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: String,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: String,
    #[serde(rename = "buy_swaps_count")]
    pub buy_swaps_count: i64,
    #[serde(rename = "sell_swaps_count")]
    pub sell_swaps_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Last1h {
    #[serde(rename = "swaps_count")]
    pub swaps_count: i64,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: String,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: String,
    #[serde(rename = "buy_swaps_count")]
    pub buy_swaps_count: i64,
    #[serde(rename = "sell_swaps_count")]
    pub sell_swaps_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Last6h {
    #[serde(rename = "swaps_count")]
    pub swaps_count: i64,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: String,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: String,
    #[serde(rename = "buy_swaps_count")]
    pub buy_swaps_count: i64,
    #[serde(rename = "sell_swaps_count")]
    pub sell_swaps_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Last24h {
    #[serde(rename = "swaps_count")]
    pub swaps_count: i64,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: String,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: String,
    #[serde(rename = "buy_swaps_count")]
    pub buy_swaps_count: i64,
    #[serde(rename = "sell_swaps_count")]
    pub sell_swaps_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricePercentChanges {
    #[serde(rename = "last_5m")]
    pub last_5m: String,
    #[serde(rename = "last_15m")]
    pub last_15m: String,
    #[serde(rename = "last_30m")]
    pub last_30m: String,
    #[serde(rename = "last_1h")]
    pub last_1h: String,
    #[serde(rename = "last_6h")]
    pub last_6h: String,
    #[serde(rename = "last_24h")]
    pub last_24h: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentimentVotes {
    pub total: f64,
    #[serde(rename = "up_percentage")]
    pub up_percentage: f64,
    #[serde(rename = "down_percentage")]
    pub down_percentage: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationships {
    pub dex: Dex,
    pub tokens: Tokens,
    #[serde(rename = "pool_metric")]
    pub pool_metric: PoolMetric,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dex {
    pub data: Data2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data2 {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub data: Vec<Daum>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMetric {
    pub data: Data3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data3 {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

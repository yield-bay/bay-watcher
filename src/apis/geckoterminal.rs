use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub attributes: Attributes,
    pub relationships: Relationships,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub address: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "fully_diluted_valuation")]
    pub fully_diluted_valuation: Option<String>,
    #[serde(rename = "from_volume_in_usd")]
    pub from_volume_in_usd: Option<String>,
    #[serde(rename = "to_volume_in_usd")]
    pub to_volume_in_usd: Option<String>,
    #[serde(rename = "swap_count_24h")]
    pub swap_count_24h: i64,
    #[serde(rename = "reserve_threshold_met")]
    pub reserve_threshold_met: bool,
    #[serde(rename = "price_in_usd")]
    pub price_in_usd: Option<String>,
    #[serde(rename = "base_token_id")]
    pub base_token_id: Option<String>,
    #[serde(rename = "reserve_in_usd")]
    pub reserve_in_usd: Option<String>,
    #[serde(rename = "pool_fee")]
    pub pool_fee: Option<String>,
    #[serde(rename = "swap_url")]
    pub swap_url: Option<String>,
    #[serde(rename = "historical_data")]
    pub historical_data: HistoricalData,
    #[serde(rename = "price_percent_change")]
    pub price_percent_change: Option<String>,
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
    pub price_in_usd: Option<String>,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: Option<String>,
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
    pub price_in_usd: Option<String>,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: Option<String>,
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
    pub price_in_usd: Option<String>,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: Option<String>,
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
    pub price_in_usd: Option<String>,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: Option<String>,
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
    pub price_in_usd: Option<String>,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: Option<String>,
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
    pub price_in_usd: Option<String>,
    #[serde(rename = "volume_in_usd")]
    pub volume_in_usd: Option<String>,
    #[serde(rename = "buy_swaps_count")]
    pub buy_swaps_count: i64,
    #[serde(rename = "sell_swaps_count")]
    pub sell_swaps_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricePercentChanges {
    #[serde(rename = "last_5m")]
    pub last_5m: Option<String>,
    #[serde(rename = "last_15m")]
    pub last_15m: Option<String>,
    #[serde(rename = "last_30m")]
    pub last_30m: Option<String>,
    #[serde(rename = "last_1h")]
    pub last_1h: Option<String>,
    #[serde(rename = "last_6h")]
    pub last_6h: Option<String>,
    #[serde(rename = "last_24h")]
    pub last_24h: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentimentVotes {
    pub total: Option<f64>,
    #[serde(rename = "up_percentage")]
    pub up_percentage: Option<f64>,
    #[serde(rename = "down_percentage")]
    pub down_percentage: Option<f64>,
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
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub data: Vec<Daum>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMetric {
    pub data: Data3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data3 {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

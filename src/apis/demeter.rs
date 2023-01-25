use serde::{Deserialize, Serialize};

pub type DeoFarms = Vec<DeoFarm>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeoFarm {
    pub underlying_asset_name: String,
    pub tvl: i64,
    pub apr: f64,
    pub deposit_fee_in_percent: i64,
    pub reward_token: String,
    pub reward_token_per_day: f64,
    pub reward_token_price: f64,
}

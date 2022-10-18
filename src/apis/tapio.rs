use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub tdot: Option<Tdot>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tdot {
    #[serde(rename = "tdot_fee")]
    pub tdot_fee: TdotFee,
    #[serde(rename = "tdot_yield")]
    pub tdot_yield: TdotYield,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TdotFee {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TdotYield {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

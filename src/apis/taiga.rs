use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub taiksm: Option<Taiksm>,
    #[serde(rename = "3usd")]
    pub n3usd: Option<n3usd>,
    pub lksm: Option<Lksm>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taiksm {
    #[serde(rename = "taiksm_fee")]
    pub taiksm_fee: TaiksmFee,
    #[serde(rename = "taiksm_yield")]
    pub taiksm_yield: TaiksmYield,
    #[serde(rename = "tai_reward")]
    pub tai_reward: TaiReward,
    #[serde(rename = "kar_reward")]
    pub kar_reward: KarReward,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiksmFee {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiksmYield {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiReward {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KarReward {
    pub apr: i64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct n3usd {
    #[serde(rename = "3usd_fee")]
    pub n3usd_fee: n3usdFee,
    #[serde(rename = "tai_reward")]
    pub tai_reward: TaiReward2,
    #[serde(rename = "taiksm_reward")]
    pub taiksm_reward: TaiksmReward,
    #[serde(rename = "lksm_reward")]
    pub lksm_reward: LksmReward,
    #[serde(rename = "kar_reward")]
    pub kar_reward: KarReward2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct n3usdFee {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiReward2 {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiksmReward {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LksmReward {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KarReward2 {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lksm {
    #[serde(rename = "lksm_yield")]
    pub lksm_yield: LksmYield,
    #[serde(rename = "kar_reward")]
    pub kar_reward: KarReward3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LksmYield {
    pub apr: f64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KarReward3 {
    pub apr: i64,
    pub token: String,
    pub decimals: i64,
    pub cumulative: i64,
    pub claimable: i64,
    pub reserved: i64,
}

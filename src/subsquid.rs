use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenlinkStableSwaps {
    pub stable_swaps: Vec<ZenlinkStableSwap>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenlinkStableSwap {
    pub id: String,
    pub lp_token: String,
    pub stable_swap_day_data: Vec<StableSwapDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StableSwapDayData {
    pub id: String,
    pub date: String,
    #[serde(rename = "dailyVolumeUSD")]
    pub daily_volume_usd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenlinkPairDayDatas {
    pub pair_day_data: Vec<PairDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairDayData {
    pub date: String,
    #[serde(rename = "dailyVolumeUSD")]
    pub daily_volume_usd: String,
    pub pair_address: String,
    pub id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairsData {
    pub pairs: Vec<Pair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub id: String,
    #[serde(rename = "reserveUSD")]
    pub reserve_usd: String,
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
    #[serde(rename = "untrackedVolumeUSD")]
    pub untracked_volume_usd: String,
    pub total_supply: String,
    pub reserve0: String,
    pub reserve1: String,
    #[serde(rename = "token0Price")]
    pub token0price: String,
    #[serde(rename = "token1Price")]
    pub token1price: String,
    pub token0: PToken,
    pub token1: PToken,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i64,
    pub total_liquidity: String,
    pub token_day_data: Vec<TokenDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensData {
    pub tokens: Vec<Token>,
    pub bundles: Vec<Bundle>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i64,
    pub total_liquidity: String,
    #[serde(rename = "derivedETH")]
    pub derived_eth: String,
    pub token_day_data: Vec<TokenDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDayData {
    #[serde(rename = "priceUSD")]
    pub price_usd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub eth_price: String,
}

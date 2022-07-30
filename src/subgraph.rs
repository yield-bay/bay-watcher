use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairDayDatas {
    pub pair_day_datas: Vec<PairDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairDayData {
    pub date: i64,
    #[serde(rename = "dailyVolumeUSD")]
    pub daily_volume_usd: String,
    pub pair_address: String,
    pub id: String,
    pub token0: Token0,
    pub token1: Token1,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token0 {
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token1 {
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlocksData {
    pub blocks: Vec<Block>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: String,
    pub number: String,
    pub timestamp: String,
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
    pub decimals: String,
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
    pub decimals: String,
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

// ////////

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiTokensData {
    pub tokens: Vec<SushiToken>,
    pub bundles: Vec<Bundle>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: String,
    #[serde(rename = "liquidity")]
    pub liquidity: String,
    // pub total_liquidity: String,
    #[serde(rename = "derivedETH")]
    pub derived_eth: String,
    #[serde(rename = "dayData")]
    pub day_data: Vec<TokenDayData>,
    // pub token_day_data: Vec<TokenDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiPairsData {
    pub pairs: Vec<SushiPair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiPair {
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
    pub token0: SushiPToken,
    pub token1: SushiPToken,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiPToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: String,
    pub liquidity: String,
    pub day_data: Vec<TokenDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiPairDayDatas {
    pub pair_day_datas: Vec<SushiPairDayData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiPairDayData {
    pub date: i64,
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
    pub id: String,
    pub pair: SushiPDDPair,
    pub token0: Token0,
    pub token1: Token1,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SushiPDDPair {
    pub id: String,
}

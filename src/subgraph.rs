use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarbeamStableRoot {
    pub data: SolarbeamStableData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarbeamStableData {
    pub swap: SolarbeamStableSwap,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarbeamStableSwap {
    pub id: String,
    pub address: String,
    pub tokens: Vec<TokenSymbol>,
    pub daily_data: Vec<DailyDaum>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyDaum {
    pub id: String,
    pub timestamp: String,
    pub volume: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StellaStableRoot {
    pub data: StellaStableData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StellaStableData {
    pub swap: StellaStableSwap,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StellaStableSwap {
    pub id: String,
    pub address: String,
    pub tokens: Vec<TokenSymbol>,
    pub daily_volumes: Vec<DailyVolume>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSymbol {
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyVolume {
    pub id: String,
    pub timestamp: String,
    pub volume: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZenlinkPairDayDatas {
    pub pair_day_data: Vec<PairDayData>,
}

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
    pub daily_volume_token0: String,
    pub daily_volume_token1: String,
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

// Tapio/Taiga

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaruraTokenPriceHistoryData {
    pub token: KaruraDexToken,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaruraDexToken {
    pub daily_data: KaruraDailyData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaruraDailyData {
    pub nodes: Vec<KaruraDailyDataNode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaruraDailyDataNode {
    pub price: String,
    pub timestamp: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TapioDD {
    pub daily_data: TapioDailyData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TapioDailyData {
    pub nodes: Vec<TapioDailyDataNode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TapioDailyDataNode {
    pub yield_volume: f64,
    pub fee_volume: f64,
    pub total_supply: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulsarRoot {
    pub data: PulsarData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulsarData {
    pub pools: Vec<Pool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub id: String,
    pub token0: PulsarToken,
    pub token1: PulsarToken,
    #[serde(rename = "totalValueLockedUSD")]
    pub total_value_locked_usd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulsarToken {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EternalFarmingRoot {
    pub data: EternalFarmingData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EternalFarmingData {
    pub eternal_farmings: Vec<EternalFarming>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EternalFarming {
    pub id: String,
    pub pool: String,
    pub reward_token: String,
    pub reward: String,
    pub bonus_reward_token: String,
    pub bonus_reward: String,
    pub start_time: String,
    pub end_time: String,
    pub reward_rate: String,
    pub bonus_reward_rate: String,
    pub token_amount_for_tier1: String,
    pub token_amount_for_tier2: String,
    pub token_amount_for_tier3: String,
    #[serde(rename = "tier1Multiplier")]
    pub tier1multiplier: String,
    #[serde(rename = "tier2Multiplier")]
    pub tier2multiplier: String,
    #[serde(rename = "tier3Multiplier")]
    pub tier3multiplier: String,
    pub multiplier_token: String,
    pub min_range_length: String,
}

use serde::{Deserialize, Serialize};
use serde_json::Value;

// pub type Root = Vec<Root2>;

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Root2 {
//     #[serde(rename = "_id")]
//     pub id: String,
//     #[serde(rename = "id")]
//     pub id2: String,
//     pub exchange: String,
//     pub created_at: String,
//     pub updated_at: String,
//     #[serde(rename = "__v")]
//     pub v: i64,
//     pub created_at_block_number: i64,
//     pub created_at_timestamp: i64,
//     pub token0: Token0,
//     pub token1: Token1,
//     pub token_index: i64,
//     #[serde(rename = "type")]
//     pub type_field: String,
//     pub creation: Creation,
//     pub team: Team,
//     pub info: Info,
//     pub initial_reserve0: i64,
//     pub initial_reserve1: i64,
//     pub initial_liquidity: f64,
//     pub initial_liquidity_updated_at: String,
//     pub liquidity: f64,
//     pub reserve0: f64,
//     pub reserve1: f64,
//     pub reserve_updated_at: String,
//     pub tx_count: i64,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Token0 {
//     #[serde(rename = "_id")]
//     pub id: String,
//     #[serde(rename = "id")]
//     pub id2: String,
//     pub decimals: i64,
//     pub name: String,
//     pub symbol: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Token1 {
//     #[serde(rename = "_id")]
//     pub id: String,
//     #[serde(rename = "id")]
//     pub id2: String,
//     pub name: String,
//     pub symbol: String,
//     pub decimals: String,
//     pub total_supply: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Creation {
//     pub block_number: String,
//     pub block_hash: String,
//     pub time_stamp: String,
//     pub hash: String,
//     pub nonce: String,
//     pub transaction_index: String,
//     pub from: String,
//     pub to: String,
//     pub value: String,
//     pub gas: String,
//     pub gas_price: String,
//     pub input: String,
//     pub method_id: String,
//     pub function_name: String,
//     pub contract_address: String,
//     pub cumulative_gas_used: String,
//     #[serde(rename = "txreceipt_status")]
//     pub txreceipt_status: String,
//     pub gas_used: String,
//     pub confirmations: String,
//     pub is_error: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Team {
//     pub wallet: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Info {
//     pub locks: Vec<Value>,
//     pub address: String,
//     pub holders: i64,
//     pub decimals: i64,
//     pub name: String,
//     pub symbol: String,
//     pub total_supply: String,
//     pub max_supply_formatted: i64,
//     pub total_supply_formatted: i64,
//     pub total_supply_formatted_updated_at: String,
// }

pub type Root = Vec<Root2>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root2 {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "id")]
    pub id2: Option<String>,
    pub exchange: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(rename = "__v")]
    pub v: Option<i64>,
    pub created_at_block_number: Option<i64>,
    pub created_at_timestamp: i64,
    pub token0: Option<Token0>,
    pub token1: Option<Token1>,
    pub token_index: Option<i64>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub creation: Option<Creation>,
    pub team: Option<Team>,
    pub info: Option<Info>,
    pub initial_reserve0: Option<f64>,
    pub initial_reserve1: Option<f64>,
    pub initial_liquidity: Option<f64>,
    pub initial_liquidity_updated_at: Option<String>,
    pub liquidity: Option<f64>,
    pub reserve0: Option<f64>,
    pub reserve1: Option<f64>,
    pub reserve_updated_at: Option<String>,
    pub tx_count: Option<i64>,
    pub custom: Option<Custom>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token0 {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "id")]
    pub id2: String,
    pub decimals: Value,
    pub name: String,
    pub symbol: String,
    pub audit: Option<Audit>,
    pub total_supply: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    pub code_verified: bool,
    pub date: String,
    pub lock_transactions: bool,
    pub mint: bool,
    pub proxy: bool,
    pub status: String,
    pub unlimited_fees: bool,
    pub version: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token1 {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "id")]
    pub id2: String,
    pub name: String,
    pub symbol: String,
    pub decimals: Value,
    pub total_supply: Option<String>,
    pub audit: Option<Audit2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit2 {
    pub code_verified: bool,
    pub date: String,
    pub lock_transactions: bool,
    pub mint: bool,
    pub proxy: bool,
    pub status: String,
    pub unlimited_fees: bool,
    pub version: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creation {
    pub block_number: String,
    pub block_hash: String,
    pub time_stamp: String,
    pub hash: String,
    pub nonce: String,
    pub transaction_index: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub input: String,
    pub method_id: Option<String>,
    pub function_name: Option<String>,
    pub contract_address: String,
    pub cumulative_gas_used: String,
    #[serde(rename = "txreceipt_status")]
    pub txreceipt_status: String,
    pub gas_used: String,
    pub confirmations: String,
    pub is_error: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub wallet: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub locks: Vec<Value>,
    pub address: String,
    pub holders: i64,
    pub decimals: Option<i64>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub total_supply: Option<String>,
    pub max_supply_formatted: Option<f64>,
    pub total_supply_formatted: Option<f64>,
    pub total_supply_formatted_updated_at: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Custom {
    pub info: Info2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info2 {
    pub website: String,
    pub description: String,
    pub github: String,
    pub twitter: String,
    pub telegram: String,
    pub discord: String,
    pub logo: String,
    pub updated_at: String,
}

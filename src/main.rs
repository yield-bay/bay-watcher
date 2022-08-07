use std::{collections::HashMap, str::FromStr, sync::Arc, thread, time};

use chrono::prelude::Utc;
use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::{abigen, Address, H160, U256},
    providers::{Http, Provider},
    signers::LocalWallet,
    utils::to_checksum,
};
use gql_client::Client;
use mongodb::{
    bson::{bson, doc},
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};
use serde::Serialize;

mod models;
mod subgraph;

abigen!(
    IChefV2,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (address, uint256, uint256, uint256, uint16, uint256, uint256)
        function poolTotalLp(uint256) external view returns (uint256)
        function poolRewarders(uint256) external view returns (address [])
        function poolRewardsPerSec(uint256) external view returns (address[], string[], uint256[], uint256[])
    ]"#,
);

abigen!(
    IStellaDistributorV1,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (address, uint256, uint256, uint256, uint16, uint256, uint256)
        function stellaPerBlock() external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
    ]"#,
);

abigen!(
    IMiniChefV2,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (uint128, uint64, uint64)
        function sushiPerSecond() external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
        function lpToken(uint256) external view returns (address)
    ]"#,
);

abigen!(
    IComplexRewarderTime,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (uint128, uint64, uint64)
        function rewardPerSecond() external view returns (uint256)
    ]"#,
);

abigen!(
    IStandardLpToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
    ]"#,
);

abigen!(
    IStableLpToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function owner() external view returns (address)
        function totalSupply() external view returns (uint256)
        function balanceOf(address) external view returns (uint256)
    ]"#,
);

abigen!(
    IVestedToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function owner() external view returns (address)
    ]"#,
);

abigen!(
    IStableLpTokenOwner,
    r#"[
        function getNumberOfTokens() external view returns (uint256)
        function getToken(uint8) external view returns (address)
        function getTokenBalance(uint8) external view returns (uint256)
        function getTokenBalances() external view returns (uint256[])
        function getTokenIndex(address) external view returns (uint256)
        function getTokenPrecisionMultipliers() external view returns (uint256[])
        function getTokens() external view returns (address[])
        function getVirtualPrice() external view returns (uint256)
    ]"#,
);

abigen!(
    IAnyswapV5ERC20,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
        function balanceOf(address) external view returns (uint256)
    ]"#,
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let delay = time::Duration::from_secs(60 * 2);
    loop {
        run_jobs().await.unwrap();
        thread::sleep(delay);
    }
}

async fn run_jobs() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Parse a connection string into an options struct.
    let mongo_uri = dotenv::var("DB_CONN_STRING").unwrap();
    println!("mongo_uri: {}", mongo_uri);

    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(mongo_uri).await?;

    // Manually set an option.
    client_options.app_name = Some("Bay Watcher".to_string());

    // Get a handle to the deployment.
    let client = MongoClient::with_options(client_options)?;

    // Get a handle to a database.
    let db = client.database("bayCave");

    let assets_collection = db.collection::<models::Asset>("assets");
    let farms_collection = db.collection::<models::Farm>("farms");

    let mut headers = HashMap::new();
    headers.insert("content-type", "application/json");

    let daily_data_tai_ksm_query = r#"
        query {
            dailyData(first:30, orderBy: TIMESTAMP_DESC, filter: {poolId: {equalTo: 0}}) {
                nodes {
                    yieldVolume
                    feeVolume
                    totalSupply
                }
            }
        }
    "#;

    let daily_data_3_usd_query = r#"
        query DD3USD($days: Int) {
            dailyData(first: $days, orderBy: TIMESTAMP_DESC, filter: {poolId: {equalTo: 1}}) {
                nodes {
                    yieldVolume
                    feeVolume
                    totalSupply
                }
            }
        }
    "#;

    let token_price_history_query = r#"
        query TPHQ($asset: String!, $days: Int!) {
            token(id: $asset) {
                dailyData(first: $days, orderBy: TIMESTAMP_DESC) {
                    nodes {
                        price
                        timestamp
                    }
                }
            }
        }
    "#;

    let _tai_ksm = fetch_tai_ksm(
        daily_data_tai_ksm_query.to_owned(),
        token_price_history_query.to_owned(),
    )
    .await;
    let _3usd = fetch_3usd(
        daily_data_3_usd_query.to_owned(),
        token_price_history_query.to_owned(),
    )
    .await;

    println!("_tai_ksm:\n{:?}\n_3usd:\n{:?}", _tai_ksm, _3usd);

    let mut tai_ksm_rewards = vec![];
    for r in _tai_ksm.1.clone() {
        tai_ksm_rewards.push(bson!({
            "amount": r.0 as f64,
            "asset":  r.1.clone(),
            "valueUSD": r.2 as f64,
            "freq": r.3.clone(),
        }));
    }

    let timestamp = Utc::now().to_string();

    println!("taiKSM farm lastUpdatedAtUTC {}", timestamp.clone());

    let tai_ksm_ff = doc! {
        "id": 0,
        "chef": "taiKSM".to_string(),
        "chain": "karura".to_string(),
        "protocol": "taiga".to_string(),
    };
    let tai_ksm_fu = doc! {
        "$set" : {
            "id": 0,
            "chef": "taiKSM".to_string(),
            "chain": "karura".to_string(),
            "protocol": "taiga".to_string(),
            "farmType": models::FarmType::SingleStaking.to_string(),
            "farmImpl": models::FarmImplementation::Pallet.to_string(),
            "asset": {
                "symbol": "taiKSM".to_string(),
                "address": "taiKSM".to_string(),
                "price": 0 as f64,
                "logos": ["https://raw.githubusercontent.com/yield-bay/assets/main/karura/taiga/taiKSM.png".to_string()],
            },
            "tvl": _tai_ksm.0 as f64,
            "apr.reward": _tai_ksm.2.1 as f64 * 100.0,
            "apr.base": _tai_ksm.2.0 as f64 * 100.0,
            "rewards": tai_ksm_rewards,
            "allocPoint": 1,
            "lastUpdatedAtUTC": timestamp.clone(),
        }
    };
    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .build();
    farms_collection
        .find_one_and_update(tai_ksm_ff, tai_ksm_fu, Some(options))
        .await?;

    let mut _3usd_rewards = vec![];
    for r in _3usd.1.clone() {
        _3usd_rewards.push(bson!({
            "amount": r.0 as f64,
            "asset":  r.1.clone(),
            "valueUSD": r.2 as f64,
            "freq": r.3.clone(),
        }));
    }

    let timestamp = Utc::now().to_string();

    println!("3USD farm lastUpdatedAtUTC {}", timestamp.clone());

    let _3usd_ff = doc! {
        "id": 1,
        "chef": "3USD".to_string(),
        "chain": "karura".to_string(),
        "protocol": "taiga".to_string(),
    };
    let _3usd_fu = doc! {
        "$set" : {
            "id": 1,
            "chef": "3USD".to_string(),
            "chain": "karura".to_string(),
            "protocol": "taiga".to_string(),
            "farmType": models::FarmType::SingleStaking.to_string(),
            "farmImpl": models::FarmImplementation::Pallet.to_string(),
            "asset": {
                "symbol": "3USD".to_string(),
                "address": "3USD".to_string(),
                "price": 0 as f64,
                "logos": ["https://raw.githubusercontent.com/yield-bay/assets/main/karura/taiga/3USD.png".to_string()],
            },
            "tvl": _3usd.0 as f64,
            "apr.reward": _3usd.2.1 as f64 * 100.0,
            "apr.base": _3usd.2.0 as f64 * 100.0,
            "rewards": _3usd_rewards,
            "allocPoint": 1,
            "lastUpdatedAtUTC": timestamp.clone(),
        }
    };
    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .build();
    farms_collection
        .find_one_and_update(_3usd_ff, _3usd_fu, Some(options))
        .await?;

    // let delay = time::Duration::from_secs(60 * 10);
    // thread::sleep(delay);

    let solarbeam_subgraph = "https://api.thegraph.com/subgraphs/name/solar-ape/solarbeam";
    let stellaswap_subgraph = "https://api.thegraph.com/subgraphs/name/stellaswap/stella-swap";
    let beamswap_subgraph = "https://api.thegraph.com/subgraphs/name/beamswap/beamswap-dex";
    let sushi_subgraph = "https://api.thegraph.com/subgraphs/name/sushiswap/exchange-moonriver";

    let solarbeam_blocklytics_subgraph =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/blocklytics";
    let solarflare_blocklytics_subgraph =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/solarflare-blocklytics";

    let tokens_query = r#"
        query {
            tokens(orderBy: tradeVolumeUSD, orderDirection: desc, first: 1000) {
                id
                symbol
                name
                decimals
                totalLiquidity
                derivedETH
                tokenDayData(first: 1, orderBy: date, orderDirection: desc) {
                    priceUSD
                }
            }
            bundles(first: 1) {
                ethPrice
            }
        }
    "#;

    let sushi_tokens_query = r#"
        query {
            tokens(orderBy: volumeUSD, orderDirection: desc, first: 1000) {
                id
                symbol
                name
                decimals
                liquidity
                derivedETH
                dayData(first: 1, orderBy: date, orderDirection: desc) {
                    priceUSD
                }
            }
            bundles(first: 1) {
                ethPrice
            }
        }
    "#;

    let pairs_query = r#"
        query {
            pairs(orderBy: reserveUSD, orderDirection: desc, first: 1000) {
                id
                reserveUSD
                volumeUSD
                untrackedVolumeUSD
                totalSupply
                reserve0
                reserve1
                token0Price
                token1Price
                token0 {
                    id
                    symbol
                    name
                    decimals
                    totalLiquidity
                    tokenDayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
                token1 {
                    id
                    symbol
                    name
                    decimals
                    totalLiquidity
                    tokenDayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
            }
        }
    "#;

    let sushi_pairs_query = r#"
        query {
            pairs(orderBy: reserveUSD, orderDirection: desc, first: 1000) {
                id
                reserveUSD
                volumeUSD
                untrackedVolumeUSD
                totalSupply
                reserve0
                reserve1
                token0Price
                token1Price
                token0 {
                    id
                    symbol
                    name
                    decimals
                    liquidity
                    dayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
                token1 {
                    id
                    symbol
                    name
                    decimals
                    liquidity
                    dayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
            }
        }
    "#;

    let one_day_blocks_query = r#"
        query OneDayBlocks($start: Int!, $end: Int!) {
            blocks(
                first: 1
                orderBy: timestamp
                orderDirection: asc
                where: { timestamp_gt: $start, timestamp_lt: $end }
            ) {
                id
                number
                timestamp
            }
        }
    "#;

    let one_day_pools_query = r#"
        query OneDayPools($blocknum: Int!) {
            pairs(orderBy: reserveUSD, orderDirection: desc, first: 1000, block: { number: $blocknum }) {
                id
                reserveUSD
                volumeUSD
                untrackedVolumeUSD
            }
        }
    "#;

    let pair_day_datas_query = r#"
        query PairDayDatas($addr: String) {
            pairDayDatas(
                orderDirection: desc
                orderBy: date
                first: 7
                where: {pairAddress: $addr}
            ) {
                date
                dailyVolumeUSD
                pairAddress
                id
                token0 {
                    symbol
                }
                token1 {
                    symbol
                }
            }
        }
    "#;

    let sushi_pair_day_datas_query = r#"
        query PairDayDatas($addr: String) {
            pairDayDatas(
                orderDirection: desc
                orderBy: date
                first: 1
                where: {pair: $addr}
            ) {
                date
                volumeUSD
                id
                pair {
                    id
                }
                token0 {
                    symbol
                }
                token1 {
                    symbol
                }
            }
        }
    "#;

    let solarbeam_subgraph_client =
        Client::new_with_headers(solarbeam_subgraph.clone(), headers.clone());
    let stellaswap_subgraph_client =
        Client::new_with_headers(stellaswap_subgraph.clone(), headers.clone());
    let beamswap_subgraph_client =
        Client::new_with_headers(beamswap_subgraph.clone(), headers.clone());
    let sushi_subgraph_client = Client::new_with_headers(sushi_subgraph.clone(), headers.clone());

    let _moonriver_blocklytics_client =
        Client::new_with_headers(solarbeam_blocklytics_subgraph.clone(), headers.clone());
    let _moonbeam_blocklytics_client =
        Client::new_with_headers(solarflare_blocklytics_subgraph.clone(), headers.clone());

    // subgraph fetching jobs
    let protocols = vec![
        (
            "sushiswap",
            "moonriver",
            sushi_subgraph_client.clone(),
            sushi_subgraph.clone(),
        ),
        (
            "stellaswap",
            "moonbeam",
            stellaswap_subgraph_client.clone(),
            stellaswap_subgraph.clone(),
        ),
        (
            "solarbeam",
            "moonriver",
            solarbeam_subgraph_client.clone(),
            solarbeam_subgraph.clone(),
        ),
        (
            "beamswap",
            "moonbeam",
            beamswap_subgraph_client.clone(),
            beamswap_subgraph.clone(),
        ),
    ];

    for p in protocols {
        println!("subgraph data for {} on {}", p.0.clone(), p.1.clone());

        let client = Client::new_with_headers(p.3.clone(), headers.clone());

        if p.0.clone() == "sushiswap" {
            let tokens_data = client
                .query_unwrap::<subgraph::SushiTokensData>(sushi_tokens_query.clone())
                .await;

            if tokens_data.is_ok() {
                println!("{} tokens_data {:?}", p.0.clone(), tokens_data.clone());
                for t in tokens_data.clone().unwrap().tokens.clone() {
                    let mut price_usd: f64 = 0.0;
                    if t.day_data.len() >= 1 {
                        price_usd = t.day_data[0].price_usd.parse().unwrap_or_default();
                    }
                    if tokens_data.clone().unwrap().bundles.clone().len() >= 1 {
                        let derived_eth: f64 = t.derived_eth.parse().unwrap_or_default();
                        let eth_price: f64 = tokens_data.clone().unwrap().bundles.clone()[0]
                            .eth_price
                            .parse()
                            .unwrap_or_default();
                        price_usd = derived_eth * eth_price;
                    }

                    let ta = Address::from_str(t.id.as_str()).unwrap();
                    let token_addr = to_checksum(&ta, None);

                    println!("token_addr {:?}", token_addr.clone());

                    let decimals: u32 = t.decimals.parse().unwrap_or_default();

                    let mut logo = "".to_string();
                    if p.0.clone() == "solarbeam" {
                        logo = format!("https://raw.githubusercontent.com/solarbeamio/solarbeam-tokenlist/main/assets/moonriver/{}/logo.png", token_addr.clone());
                    } else if p.0.clone() == "stellaswap" {
                        logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/tokenlist/{}/logo.png", token_addr.clone());
                        // xStella
                        if token_addr.clone()
                            == "0x06A3b410b681c82417A906993aCeFb91bAB6A080".to_string()
                        {
                            logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/xStella.png");
                        }
                        // xcINTR
                        if token_addr.clone()
                            == "0xFffFFFFF4C1cbCd97597339702436d4F18a375Ab".to_string()
                        {
                            logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/INTR.png");
                        }
                    } else if p.0.clone() == "beamswap" {
                        logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token_addr.clone());
                    } else if p.0.clone() == "sushiswap" {
                        logo = format!("https://raw.githubusercontent.com/sushiswap/list/master/logos/token-logos/network/moonriver/{}.jpg",token_addr.clone());
                        // WBTC.eth
                        if token_addr.clone()
                            == "0xE6a991Ffa8CfE62B0bf6BF72959A3d4f11B2E0f5".to_string()
                        {
                            logo = format!("https://raw.githubusercontent.com/sushiswap/icons/master/token/btc.jpg");
                        }
                    }

                    println!("logo {}", logo.clone());

                    let liquidity: f64 = t.liquidity.parse().unwrap_or_default();

                    let f = doc! {
                        "address": token_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

                    println!("token lastUpdatedAtUTC {}", timestamp.clone());

                    let u = doc! {
                        "$set" : {
                            "address": token_addr.clone(),
                            "chain": p.1.clone(),
                            "protocol": p.0.clone(),
                            "name": t.name,
                            "symbol": t.symbol,
                            "decimals": decimals,
                            "logos": [
                                logo.clone(),
                            ],
                            "price": price_usd,
                            "liquidity": liquidity,
                            "totalSupply": 0.0,
                            "isLP": false,
                            "feesAPR": 0.0,
                            "underlyingAssets": [],
                            "underlyingAssetsAlloc": [],
                            "lastUpdatedAtUTC": timestamp.clone(),
                        }
                    };

                    let options = FindOneAndUpdateOptions::builder()
                        .upsert(Some(true))
                        .build();
                    assets_collection
                        .find_one_and_update(f, u, Some(options))
                        .await?;
                }
            } else {
                println!(
                    "couldn't fetch tokens_data for {} {:?}",
                    p.0.clone(),
                    tokens_data.err()
                );
            }
        } else {
            let tokens_data = client
                // p.2.clone()
                .query_unwrap::<subgraph::TokensData>(tokens_query.clone())
                .await;

            if tokens_data.is_ok() {
                println!("{} tokens_data {:?}", p.0.clone(), tokens_data.clone());
                for t in tokens_data.clone().unwrap().tokens.clone() {
                    let mut price_usd: f64 = 0.0;
                    if t.token_day_data.len() >= 1 {
                        price_usd = t.token_day_data[0].price_usd.parse().unwrap_or_default();
                    }
                    if tokens_data.clone().unwrap().bundles.clone().len() >= 1 {
                        let derived_eth: f64 = t.derived_eth.parse().unwrap_or_default();
                        let eth_price: f64 = tokens_data.clone().unwrap().bundles.clone()[0]
                            .eth_price
                            .parse()
                            .unwrap_or_default();
                        price_usd = derived_eth * eth_price;
                    }

                    let ta = Address::from_str(t.id.as_str()).unwrap();
                    let token_addr = to_checksum(&ta, None);

                    println!("token_addr {:?}", token_addr.clone());

                    let decimals: u32 = t.decimals.parse().unwrap_or_default();

                    let mut logo = "".to_string();
                    if p.0.clone() == "solarbeam" {
                        logo = format!("https://raw.githubusercontent.com/solarbeamio/solarbeam-tokenlist/main/assets/moonriver/{}/logo.png", token_addr.clone());
                    } else if p.0.clone() == "stellaswap" {
                        logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/tokenlist/{}/logo.png", token_addr.clone());
                        // xStella
                        if token_addr.clone()
                            == "0x06A3b410b681c82417A906993aCeFb91bAB6A080".to_string()
                        {
                            logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/xStella.png");
                        }
                        // xcINTR
                        if token_addr.clone()
                            == "0xFffFFFFF4C1cbCd97597339702436d4F18a375Ab".to_string()
                        {
                            logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/INTR.png");
                        }
                    } else if p.0.clone() == "beamswap" {
                        logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token_addr.clone());
                    } else if p.0.clone() == "sushiswap" {
                        logo = format!("https://raw.githubusercontent.com/sushiswap/list/master/logos/token-logos/network/moonriver/{}.jpg",token_addr.clone());
                        // WBTC.eth
                        if token_addr.clone()
                            == "0xE6a991Ffa8CfE62B0bf6BF72959A3d4f11B2E0f5".to_string()
                        {
                            logo = format!("https://raw.githubusercontent.com/sushiswap/icons/master/token/btc.jpg");
                        }
                    }

                    println!("logo {}", logo.clone());

                    let liquidity: f64 = t.total_liquidity.parse().unwrap_or_default();

                    // stKSM or wstKSM
                    if p.0.clone() == "solarbeam"
                        && (token_addr.clone() == "0xFfc7780C34B450d917d557E728f033033CB4fA8C"
                            || token_addr.clone() == "0x3bfd113ad0329a7994a681236323fb16E16790e3")
                    {
                        let xcksm = assets_collection.find_one(doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080"}, None).await?;
                        price_usd = xcksm.clone().unwrap().price;
                    }

                    let f = doc! {
                        "address": token_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

                    println!("token lastUpdatedAtUTC {}", timestamp.clone());

                    let u = doc! {
                        "$set" : {
                            "address": token_addr.clone(),
                            "chain": p.1.clone(),
                            "protocol": p.0.clone(),
                            "name": t.name,
                            "symbol": t.symbol,
                            "decimals": decimals,
                            "logos": [
                                logo.clone(),
                            ],
                            "price": price_usd,
                            "liquidity": liquidity,
                            "totalSupply": 0.0,
                            "isLP": false,
                            "feesAPR": 0.0,
                            "underlyingAssets": [],
                            "underlyingAssetsAlloc": [],
                            "lastUpdatedAtUTC": timestamp.clone(),
                        }
                    };

                    let options = FindOneAndUpdateOptions::builder()
                        .upsert(Some(true))
                        .build();
                    assets_collection
                        .find_one_and_update(f, u, Some(options))
                        .await?;
                }
            } else {
                println!(
                    "couldn't fetch tokens_data for {} {:?}",
                    p.0.clone(),
                    tokens_data.err()
                );
            }
        }

        let mut one_day_volume_usd: HashMap<String, f64> = HashMap::new();

        if p.1.clone() == "moonbeam" {
            let block_number = get_one_day_block(
                solarflare_blocklytics_subgraph.to_string(),
                one_day_blocks_query.to_string(),
            )
            .await;
            if block_number != 0 {
                let pairs = get_one_day_pools(
                    p.3.clone().to_string(),
                    one_day_pools_query.to_string(),
                    block_number,
                )
                .await;
                for pair in pairs {
                    let pair_id = Address::from_str(pair.id.as_str()).unwrap();
                    let pair_addr = to_checksum(&pair_id, None);
                    one_day_volume_usd.insert(
                        pair_addr,
                        pair.untracked_volume_usd.parse().unwrap_or_default(),
                    );
                }
            }
        } else if p.1.clone() == "moonriver" {
            let block_number = get_one_day_block(
                solarbeam_blocklytics_subgraph.to_string(),
                one_day_blocks_query.to_string(),
            )
            .await;
            // if p.0.clone() == "sushiswap" {
            //     if block_number != 0 {
            //         let pairs = get_one_day_pools(
            //             p.3.clone().to_string(),
            //             one_day_pools_query.to_string(),
            //             block_number,
            //         )
            //         .await;
            //         for pair in pairs {
            //             let pair_id = Address::from_str(pair.id.as_str()).unwrap();
            //             let pair_addr = to_checksum(&pair_id, None);
            //             one_day_volume_usd.insert(
            //                 pair_addr,
            //                 pair.untracked_volume_usd.parse().unwrap_or_default(),
            //             );
            //         }
            //     }
            // } else {
            if block_number != 0 {
                let pairs = get_one_day_pools(
                    p.3.clone().to_string(),
                    one_day_pools_query.to_string(),
                    block_number,
                )
                .await;
                for pair in pairs {
                    let pair_id = Address::from_str(pair.id.as_str()).unwrap();
                    let pair_addr = to_checksum(&pair_id, None);
                    one_day_volume_usd.insert(
                        pair_addr,
                        pair.untracked_volume_usd.parse().unwrap_or_default(),
                    );
                }
            }
            // }
        }

        if p.0.clone() == "sushiswap" {
            let pairs_data = client
                // p.2.clone()
                .query_unwrap::<subgraph::SushiPairsData>(sushi_pairs_query.clone())
                .await;

            if pairs_data.is_ok() {
                println!("{} pairs_data {:?}", p.0.clone(), pairs_data);

                for pair in pairs_data.clone().unwrap().pairs.clone() {
                    let token0price: f64 = pair.token0price.parse().unwrap_or_default();
                    let token1price: f64 = pair.token1price.parse().unwrap_or_default();

                    let mut token0alloc = 0.0;
                    let mut token1alloc = 0.0;

                    if token0price > 0.0 && token1price > 0.0 {
                        if token0price > token1price {
                            token0alloc = (1.0 / token0price) * 100.0;
                            token1alloc = 100.0 - token0alloc;
                        } else {
                            token1alloc = (1.0 / token1price) * 100.0;
                            token0alloc = 100.0 - token1alloc;
                        }
                    }

                    let pa = Address::from_str(pair.id.as_str()).unwrap();
                    let pair_addr = to_checksum(&pa, None);
                    println!("pair_addr {:?}", pair_addr.clone());

                    let t0a = Address::from_str(pair.token0.id.as_str()).unwrap();
                    let token0_addr = to_checksum(&t0a, None);
                    println!("token0_addr {:?}", token0_addr.clone());

                    let t1a = Address::from_str(pair.token1.id.as_str()).unwrap();
                    let token1_addr = to_checksum(&t1a, None);
                    println!("token1_addr {:?}", token1_addr.clone());

                    let mut token0logo = "".to_string();
                    let mut token1logo = "".to_string();
                    if p.0.clone() == "solarbeam" {
                        token0logo = format!("https://raw.githubusercontent.com/solarbeamio/solarbeam-tokenlist/main/assets/moonriver/{}/logo.png", token0_addr.clone());
                        token1logo = format!("https://raw.githubusercontent.com/solarbeamio/solarbeam-tokenlist/main/assets/moonriver/{}/logo.png", token1_addr.clone());
                    } else if p.0.clone() == "stellaswap" {
                        token0logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/tokenlist/{}/logo.png", token0_addr.clone());
                        token1logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/tokenlist/{}/logo.png", token1_addr.clone());

                        // xStella
                        if token0_addr.clone()
                            == "0x06A3b410b681c82417A906993aCeFb91bAB6A080".to_string()
                        {
                            token0logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/xStella.png");
                        }
                        if token1_addr.clone()
                            == "0x06A3b410b681c82417A906993aCeFb91bAB6A080".to_string()
                        {
                            token1logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/xStella.png");
                        }
                        // xcINTR
                        if token0_addr.clone()
                            == "0xFffFFFFF4C1cbCd97597339702436d4F18a375Ab".to_string()
                        {
                            token0logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/INTR.png");
                        }
                        if token1_addr.clone()
                            == "0xFffFFFFF4C1cbCd97597339702436d4F18a375Ab".to_string()
                        {
                            token1logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/INTR.png");
                        }
                    } else if p.0.clone() == "beamswap" {
                        token0logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token0_addr.clone());
                        token1logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token1_addr.clone());
                    } else if p.0.clone() == "sushiswap" {
                        token0logo=format!("https://raw.githubusercontent.com/sushiswap/list/master/logos/token-logos/network/moonriver/{}.jpg",token0_addr.clone());
                        token1logo=format!("https://raw.githubusercontent.com/sushiswap/list/master/logos/token-logos/network/moonriver/{}.jpg",token1_addr.clone());

                        // WBTC.eth
                        if token0_addr.clone()
                            == "0xE6a991Ffa8CfE62B0bf6BF72959A3d4f11B2E0f5".to_string()
                        {
                            token0logo = format!("https://raw.githubusercontent.com/sushiswap/icons/master/token/btc.jpg");
                        }
                        if token1_addr.clone()
                            == "0xE6a991Ffa8CfE62B0bf6BF72959A3d4f11B2E0f5".to_string()
                        {
                            token1logo = format!("https://raw.githubusercontent.com/sushiswap/icons/master/token/btc.jpg");
                        }
                    }

                    let token0decimals: u32 = pair.token0.decimals.parse().unwrap_or_default();
                    let token1decimals: u32 = pair.token1.decimals.parse().unwrap_or_default();

                    let mut decimals = token0decimals;
                    if token1decimals > token0decimals {
                        decimals = token1decimals;
                    }

                    let liquidity: f64 = pair.reserve_usd.parse().unwrap_or_default();
                    let total_supply: f64 = pair.total_supply.parse().unwrap_or_default();

                    let mut price_usd: f64 = 0.0;

                    if total_supply != 0.0 {
                        price_usd = liquidity / total_supply;
                    }

                    println!("price_usd {}", price_usd);

                    let mut fees_apr = 0.0;
                    let odv = one_day_volume_usd.get(&pair_addr.clone());
                    if odv.is_some() {
                        fees_apr = odv.unwrap() * 0.0025 * 365.0 * 100.0 / liquidity;
                    }

                    let f = doc! {
                        "address": pair_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

                    println!("pair lastUpdatedAtUTC {}", timestamp.clone());

                    let u = doc! {
                        "$set" : {
                            "address": pair_addr.clone(),
                            "chain": p.1.clone(),
                            "protocol": p.0.clone(),
                            "name": format!("{}-{} LP", pair.token0.name, pair.token1.name),
                            "symbol": format!("{}-{} LP", pair.token0.symbol, pair.token1.symbol),
                            "decimals": decimals,
                            "logos": [
                                token0logo.clone(),
                                token1logo.clone(),
                            ],
                            "price": price_usd,
                            "liquidity": liquidity,
                            "totalSupply": total_supply,
                            "isLP": true,
                            "feesAPR": fees_apr,
                            "underlyingAssets": [token0_addr.clone(), token1_addr.clone()],
                            "underlyingAssetsAlloc": [token0alloc, token1alloc],
                            "lastUpdatedAtUTC": timestamp.clone(),
                        }
                    };

                    let options = FindOneAndUpdateOptions::builder()
                        .upsert(Some(true))
                        .build();
                    assets_collection
                        .find_one_and_update(f, u, Some(options))
                        .await?;
                }
            } else {
                println!(
                    "couldn't fetch pairs_data for {} {:?}",
                    p.0.clone(),
                    pairs_data.err()
                );
            }
        } else {
            let pairs_data = client
                // p.2.clone()
                .query_unwrap::<subgraph::PairsData>(pairs_query.clone())
                .await;

            if pairs_data.is_ok() {
                println!("{} pairs_data {:?}", p.0.clone(), pairs_data);

                for pair in pairs_data.clone().unwrap().pairs.clone() {
                    let token0price: f64 = pair.token0price.parse().unwrap_or_default();
                    let token1price: f64 = pair.token1price.parse().unwrap_or_default();

                    let mut token0alloc = 0.0;
                    let mut token1alloc = 0.0;

                    if token0price > 0.0 && token1price > 0.0 {
                        if token0price > token1price {
                            token0alloc = (1.0 / token0price) * 100.0;
                            token1alloc = 100.0 - token0alloc;
                        } else {
                            token1alloc = (1.0 / token1price) * 100.0;
                            token0alloc = 100.0 - token1alloc;
                        }
                    }

                    let pa = Address::from_str(pair.id.as_str()).unwrap();
                    let pair_addr = to_checksum(&pa, None);
                    println!("pair_addr {:?}", pair_addr.clone());

                    let t0a = Address::from_str(pair.token0.id.as_str()).unwrap();
                    let token0_addr = to_checksum(&t0a, None);
                    println!("token0_addr {:?}", token0_addr.clone());

                    let t1a = Address::from_str(pair.token1.id.as_str()).unwrap();
                    let token1_addr = to_checksum(&t1a, None);
                    println!("token1_addr {:?}", token1_addr.clone());

                    let mut token0logo = "".to_string();
                    let mut token1logo = "".to_string();
                    if p.0.clone() == "solarbeam" {
                        token0logo = format!("https://raw.githubusercontent.com/solarbeamio/solarbeam-tokenlist/main/assets/moonriver/{}/logo.png", token0_addr.clone());
                        token1logo = format!("https://raw.githubusercontent.com/solarbeamio/solarbeam-tokenlist/main/assets/moonriver/{}/logo.png", token1_addr.clone());
                    } else if p.0.clone() == "stellaswap" {
                        token0logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/tokenlist/{}/logo.png", token0_addr.clone());
                        token1logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/tokenlist/{}/logo.png", token1_addr.clone());

                        // xStella
                        if token0_addr.clone()
                            == "0x06A3b410b681c82417A906993aCeFb91bAB6A080".to_string()
                        {
                            token0logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/xStella.png");
                        }
                        if token1_addr.clone()
                            == "0x06A3b410b681c82417A906993aCeFb91bAB6A080".to_string()
                        {
                            token1logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/xStella.png");
                        }
                        // xcINTR
                        if token0_addr.clone()
                            == "0xFffFFFFF4C1cbCd97597339702436d4F18a375Ab".to_string()
                        {
                            token0logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/INTR.png");
                        }
                        if token1_addr.clone()
                            == "0xFffFFFFF4C1cbCd97597339702436d4F18a375Ab".to_string()
                        {
                            token1logo = format!("https://raw.githubusercontent.com/stellaswap/assets/main/bridge/INTR.png");
                        }
                    } else if p.0.clone() == "beamswap" {
                        token0logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token0_addr.clone());
                        token1logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token1_addr.clone());
                    } else if p.0.clone() == "sushiswap" {
                        token0logo=format!("https://raw.githubusercontent.com/sushiswap/list/master/logos/token-logos/network/moonriver/{}.jpg",token0_addr.clone());
                        token1logo=format!("https://raw.githubusercontent.com/sushiswap/list/master/logos/token-logos/network/moonriver/{}.jpg",token1_addr.clone());

                        // WBTC.eth
                        if token0_addr.clone()
                            == "0xE6a991Ffa8CfE62B0bf6BF72959A3d4f11B2E0f5".to_string()
                        {
                            token0logo = format!("https://raw.githubusercontent.com/sushiswap/icons/master/token/btc.jpg");
                        }
                        if token1_addr.clone()
                            == "0xE6a991Ffa8CfE62B0bf6BF72959A3d4f11B2E0f5".to_string()
                        {
                            token1logo = format!("https://raw.githubusercontent.com/sushiswap/icons/master/token/btc.jpg");
                        }
                    }

                    let token0decimals: u32 = pair.token0.decimals.parse().unwrap_or_default();
                    let token1decimals: u32 = pair.token1.decimals.parse().unwrap_or_default();

                    let mut decimals = token0decimals;
                    if token1decimals > token0decimals {
                        decimals = token1decimals;
                    }

                    let mut liquidity: f64 = pair.reserve_usd.parse().unwrap_or_default();
                    // wstKSM-xcKSM LP
                    if pair_addr.clone() == "0x5568872bc43Bae3757F697c0e1b241b62Eddcc17" {
                        liquidity *= 2.0;
                    }
                    let total_supply: f64 = pair.total_supply.parse().unwrap_or_default();

                    let mut price_usd: f64 = 0.0;

                    if total_supply != 0.0 {
                        price_usd = liquidity / total_supply;
                    }

                    println!("price_usd {}", price_usd);

                    let mut fees_apr = 0.0;
                    let odv = one_day_volume_usd.get(&pair_addr.clone());
                    if odv.is_some() {
                        fees_apr = odv.unwrap() * 0.0025 * 365.0 * 100.0 / liquidity;
                    }

                    let f = doc! {
                        "address": pair_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

                    println!("pair lastUpdatedAtUTC {}", timestamp.clone());

                    let u = doc! {
                        "$set" : {
                            "address": pair_addr.clone(),
                            "chain": p.1.clone(),
                            "protocol": p.0.clone(),
                            "name": format!("{}-{} LP", pair.token0.name, pair.token1.name),
                            "symbol": format!("{}-{} LP", pair.token0.symbol, pair.token1.symbol),
                            "decimals": decimals,
                            "logos": [
                                token0logo.clone(),
                                token1logo.clone(),
                            ],
                            "price": price_usd,
                            "liquidity": liquidity,
                            "totalSupply": total_supply,
                            "isLP": true,
                            "feesAPR": fees_apr,
                            "underlyingAssets": [token0_addr.clone(), token1_addr.clone()],
                            "underlyingAssetsAlloc": [token0alloc, token1alloc],
                            "lastUpdatedAtUTC": timestamp.clone(),
                        }
                    };

                    let options = FindOneAndUpdateOptions::builder()
                        .upsert(Some(true))
                        .build();
                    assets_collection
                        .find_one_and_update(f, u, Some(options))
                        .await?;
                }
            } else {
                println!(
                    "couldn't fetch pairs_data for {} {:?}",
                    p.0.clone(),
                    pairs_data.err()
                );
            }
        }
    }

    // smart contract fetching jobs

    let pk = dotenv::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = pk.parse().expect("fail parse");

    let moonriver_url = dotenv::var("MOONRIVER_URL").unwrap();
    let moonbeam_url = dotenv::var("MOONBEAM_URL").unwrap();

    let moonriver_provider_service =
        Provider::<Http>::try_from(moonriver_url.clone()).expect("failed");
    let moonriver_provider = SignerMiddleware::new(moonriver_provider_service, wallet.clone());

    let moonbeam_provider_service =
        Provider::<Http>::try_from(moonbeam_url.clone()).expect("failed");
    let moonbeam_provider = SignerMiddleware::new(moonbeam_provider_service, wallet.clone());

    let moonriver_client = SignerMiddleware::new(moonriver_provider.clone(), wallet.clone());
    let moonriver_client = Arc::new(moonriver_client);

    let moonbeam_client = SignerMiddleware::new(moonbeam_provider.clone(), wallet.clone());
    let moonbeam_client = Arc::new(moonbeam_client);

    let solarbeam_chef_address = "0x0329867a8c457e9F75e25b0685011291CD30904F".parse::<Address>()?;
    let solarbeam_chef = IChefV2::new(solarbeam_chef_address, Arc::clone(&moonriver_client));

    let stella_chef_v1_address = "0xEDFB330F5FA216C9D2039B99C8cE9dA85Ea91c1E".parse::<Address>()?;
    let stella_chef_v1 = IChefV2::new(stella_chef_v1_address, Arc::clone(&moonbeam_client));

    let stella_chef_v2_address = "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388".parse::<Address>()?;
    let stella_chef_v2 = IChefV2::new(stella_chef_v2_address, Arc::clone(&moonbeam_client));

    let beam_chef_address = "0xC6ca172FC8BDB803c5e12731109744fb0200587b".parse::<Address>()?;
    let beam_chef = IChefV2::new(beam_chef_address, Arc::clone(&moonbeam_client));

    let sushi_mini_chef_address =
        "0x3dB01570D97631f69bbb0ba39796865456Cf89A5".parse::<Address>()?;
    let sushi_mini_chef = IChefV2::new(sushi_mini_chef_address, Arc::clone(&moonriver_client));

    let _protocols = vec![
        (
            sushi_mini_chef_address,
            sushi_mini_chef,
            "moonriver".to_string(),
            "sushiswap".to_string(),
            "v0".to_string(),
            "0x3dB01570D97631f69bbb0ba39796865456Cf89A5".to_string(),
            sushi_subgraph_client.clone(),
            sushi_subgraph.clone(),
            moonriver_client.clone(),
        ),
        (
            beam_chef_address,
            beam_chef,
            "moonbeam".to_string(),
            "beamswap".to_string(),
            "v2".to_string(),
            "0xC6ca172FC8BDB803c5e12731109744fb0200587b".to_string(),
            beamswap_subgraph_client.clone(),
            beamswap_subgraph.clone(),
            moonbeam_client.clone(),
        ),
        (
            stella_chef_v1_address,
            stella_chef_v1,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v1".to_string(),
            "0xEDFB330F5FA216C9D2039B99C8cE9dA85Ea91c1E".to_string(),
            stellaswap_subgraph_client.clone(),
            stellaswap_subgraph.clone(),
            moonbeam_client.clone(),
        ),
        (
            stella_chef_v2_address,
            stella_chef_v2,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v2".to_string(),
            "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388".to_string(),
            stellaswap_subgraph_client.clone(),
            stellaswap_subgraph.clone(),
            moonbeam_client.clone(),
        ),
        (
            solarbeam_chef_address,
            solarbeam_chef,
            "moonriver".to_string(),
            "solarbeam".to_string(),
            "v2".to_string(),
            "0x0329867a8c457e9F75e25b0685011291CD30904F".to_string(),
            solarbeam_subgraph_client.clone(),
            solarbeam_subgraph.clone(),
            moonriver_client.clone(),
        ),
    ];

    for p in _protocols.clone() {
        let pool_length: U256 = p.1.pool_length().call().await?;
        println!("pool_length {}", pool_length.as_u32());

        for pid in 0..pool_length.as_u32() {
            println!(
                "---------------------\n{} {} pid {}",
                p.3.clone(),
                p.4.clone(),
                pid
            );

            if p.4.clone() == "v0".to_string() {
                let sushi_mini_chef_address =
                    "0x3dB01570D97631f69bbb0ba39796865456Cf89A5".parse::<Address>()?;
                let sushi_mini_chef =
                    IMiniChefV2::new(sushi_mini_chef_address, Arc::clone(&moonriver_client));

                // TODO: fetch this address from minichef contract
                // right now hardcoding to prevent repeated calls (same rewarder is used for all pids)
                let sushi_complex_rewarder_address =
                    "0x1334c8e873E1cae8467156e2A81d1C8b566B2da1".parse::<Address>()?;
                let sushi_complex_rewarder = IComplexRewarderTime::new(
                    sushi_complex_rewarder_address,
                    Arc::clone(&moonriver_client),
                );

                let (acc_native_reward_per_share, last_reward_timestamp, alloc_point): (
                    u128,
                    u64,
                    u64,
                ) = sushi_mini_chef
                    .pool_info(ethers::prelude::U256::from(pid))
                    .call()
                    .await?;

                let ap = alloc_point as u32;

                let farm_type = models::FarmType::StandardAmm;
                let farm_implementation = models::FarmImplementation::Solidity;

                if ap > 0 {
                    let lp_token: Address = sushi_mini_chef
                        .lp_token(ethers::prelude::U256::from(pid))
                        .call()
                        .await?;

                    let asset_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                    println!("asset_addr: {:?}", asset_addr.clone());

                    let asset_filter = doc! { "address": asset_addr.clone() };
                    let asset = assets_collection.find_one(asset_filter, None).await?;

                    let mut asset_price: f64 = 0.0;
                    let mut asset_tvl: f64 = 0.0;

                    let mut rewards = vec![];
                    let mut total_reward_apr = 0.0;

                    if asset.is_some() {
                        println!("asset: {:?}", asset.clone().unwrap().symbol);
                        let sps: U256 = sushi_mini_chef.sushi_per_second().call().await?;
                        let tap: U256 = sushi_mini_chef.total_alloc_point().call().await?;
                        let rps: U256 = sushi_complex_rewarder.reward_per_second().call().await?;

                        let sushi_filter = doc! {"address":"0xf390830DF829cf22c53c8840554B98eafC5dCBc2","protocol":"sushiswap","chain":"moonriver"};
                        let sushi = assets_collection.find_one(sushi_filter, None).await?;

                        let movr_filter = doc! {"address":"0xf50225a84382c74CbdeA10b0c176f71fc3DE0C4d","protocol":"sushiswap","chain":"moonriver"};
                        let movr = assets_collection.find_one(movr_filter, None).await?;

                        if sushi.is_some() || movr.is_some() {
                            if sushi.is_some() {
                                let reward_asset_price = sushi.clone().unwrap().price;
                                println!("reward_asset_price: {:?}", reward_asset_price);

                                asset_price = asset.clone().unwrap().price;
                                println!("asset_price: {:?}", asset_price);

                                let rewards_per_sec: f64 =
                                    sps.as_u128() as f64 * (ap as f64 / tap.as_u128() as f64);

                                let rewards_per_day: f64 = rewards_per_sec * 60.0 * 60.0 * 24.0;
                                asset_tvl = asset.clone().unwrap().liquidity;

                                let ten: i128 = 10;
                                rewards.push(bson!({
                                    "amount": rewards_per_day as f64 / ten.pow(sushi.clone().unwrap().decimals) as f64,
                                    "asset":  sushi.clone().unwrap().symbol,
                                    "valueUSD": (rewards_per_day as f64 / ten.pow(sushi.clone().unwrap().decimals) as f64) * reward_asset_price,
                                    "freq": models::Freq::Daily.to_string(),
                                }));

                                // reward_apr/farm_apr/pool_apr
                                println!(
                                    "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                    rewards_per_sec, rewards_per_day, asset_tvl
                                );

                                let reward_apr = ((rewards_per_day as f64 * reward_asset_price)
                                    / (asset_tvl as f64
                                        * ten.pow(sushi.clone().unwrap().decimals) as f64))
                                    * 365.0
                                    * 100.0;
                                println!("reward_apr: {}", reward_apr);
                                if asset_tvl != 0.0 && asset_price != 0.0 {
                                    total_reward_apr += reward_apr;
                                }
                            }

                            if movr.is_some() {
                                let reward_asset_price = movr.clone().unwrap().price;
                                println!("reward_asset_price: {:?}", reward_asset_price);

                                asset_price = asset.clone().unwrap().price;
                                println!("asset_price: {:?}", asset_price);

                                let (
                                    acc_native_reward_per_share,
                                    last_reward_timestamp,
                                    r_alloc_point,
                                ): (u128, u64, u64) = sushi_mini_chef
                                    .pool_info(ethers::prelude::U256::from(pid))
                                    .call()
                                    .await?;

                                let rap = r_alloc_point as u32;

                                let rewards_per_sec: f64 =
                                    rps.as_u128() as f64 * (rap as f64 / tap.as_u128() as f64);

                                let rewards_per_day: f64 = rewards_per_sec * 60.0 * 60.0 * 24.0;
                                asset_tvl = asset.clone().unwrap().liquidity;

                                let ten: i128 = 10;
                                rewards.push(bson!({
                                    "amount": rewards_per_day as f64 / ten.pow(movr.clone().unwrap().decimals) as f64,
                                    "asset":  movr.clone().unwrap().symbol,
                                    "valueUSD": (rewards_per_day as f64 / ten.pow(movr.clone().unwrap().decimals) as f64) * reward_asset_price,
                                    "freq": models::Freq::Daily.to_string(),
                                }));

                                // reward_apr/farm_apr/pool_apr
                                println!(
                                    "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                    rewards_per_sec, rewards_per_day, asset_tvl
                                );

                                let reward_apr = ((rewards_per_day as f64 * reward_asset_price)
                                    / (asset_tvl as f64
                                        * ten.pow(movr.clone().unwrap().decimals) as f64))
                                    * 365.0
                                    * 100.0;
                                println!("reward_apr: {}", reward_apr);
                                if asset_tvl != 0.0 && asset_price != 0.0 {
                                    total_reward_apr += reward_apr;
                                }
                            }

                            // base_apr/trading_apr
                            let mut base_apr = 0.0;
                            #[derive(Serialize)]
                            pub struct Vars {
                                addr: String,
                            }
                            let vars = Vars {
                                addr: asset.clone().unwrap().address.to_lowercase(),
                            };
                            let pair_day_datas =
                                p.6.query_with_vars_unwrap::<subgraph::SushiPairDayDatas, Vars>(
                                    &sushi_pair_day_datas_query.clone(),
                                    vars,
                                )
                                .await;
                            if pair_day_datas.is_ok() {
                                // TODO: check if formula for sushi base apr is correct
                                println!("ukk {:?}", pair_day_datas.clone().unwrap());
                                let mut daily_volume_lw: f64 = 0.0;
                                for pdd in pair_day_datas.unwrap().pair_day_datas {
                                    let dv: f64 = pdd.volume_usd.parse().unwrap_or_default();
                                    daily_volume_lw += dv;
                                    println!("ukkdv {:?}", dv);
                                }
                                // daily_volume_lw /= 7.0;

                                if asset.clone().unwrap_or_default().total_supply == 0.0
                                    || asset.clone().unwrap_or_default().price == 0.0
                                {
                                    base_apr = 0.0;
                                } else {
                                    base_apr = daily_volume_lw * 0.002 * 365.0 * 100.0
                                        / (asset.clone().unwrap_or_default().total_supply
                                            * asset.clone().unwrap_or_default().price);
                                }
                            }

                            let timestamp = Utc::now().to_string();

                            println!("chef v0 farm lastUpdatedAtUTC {}", timestamp.clone());

                            let ff = doc! {
                                "id": pid as i32,
                                "chef": p.5.clone(),
                                "chain": p.2.clone(),
                                "protocol": p.3.clone(),
                            };
                            let ten: f64 = 10.0;
                            let fu = doc! {
                                "$set" : {
                                    "id": pid,
                                    "chef": p.5.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                    "farmType": farm_type.to_string(),
                                    "farmImpl": farm_implementation.to_string(),
                                    "asset": {
                                        "symbol": asset.clone().unwrap().symbol,
                                        "address": asset_addr.clone(),
                                        "price": asset.clone().unwrap().price,
                                        "logos": asset.clone().unwrap().logos,
                                        // "underlying_assets": farm_assets,
                                    },
                                    // "tvl": asset_tvl as f64 * asset_price / ten.powf(18.0),
                                    "tvl": asset_tvl as f64,
                                    "apr.reward": total_reward_apr,
                                    "apr.base": base_apr,
                                    "rewards": rewards,
                                    "allocPoint": ap,
                                    "lastUpdatedAtUTC": timestamp.clone(),
                                }
                            };
                            let options = FindOneAndUpdateOptions::builder()
                                .upsert(Some(true))
                                .build();
                            farms_collection
                                .find_one_and_update(ff, fu, Some(options))
                                .await?;
                        }
                    }
                } else {
                    println!("allocPoint = 0");
                }
            } else {
                let (
                    lp_token,
                    alloc_point,
                    last_reward_timestamp,
                    acc_native_reward_per_share,
                    deposit_fee_bp,
                    harvest_interval,
                    total_lp,
                ): (Address, U256, _, _, _, _, _) =
                    p.1.pool_info(ethers::prelude::U256::from(pid))
                        .call()
                        .await?;
                println!(
                    "{}, {}, {}, {}, {}, {}, {}",
                    lp_token,
                    alloc_point,
                    last_reward_timestamp,
                    acc_native_reward_per_share,
                    deposit_fee_bp,
                    harvest_interval,
                    total_lp
                );

                let ap = alloc_point.as_u32();

                let mut farm_type = models::FarmType::StandardAmm;
                let farm_implementation = models::FarmImplementation::Solidity;

                if ap > 0 {
                    if p.4.clone() == "v1".to_string() {
                        // chef v1
                        let asset_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                        println!("asset_addr: {:?}", asset_addr.clone());

                        let stella_chef_v1_address =
                            "0xEDFB330F5FA216C9D2039B99C8cE9dA85Ea91c1E".parse::<Address>()?;
                        let stella_chef_v1 = IStellaDistributorV1::new(
                            stella_chef_v1_address,
                            Arc::clone(&moonbeam_client),
                        );

                        let asset_filter = doc! { "address": asset_addr.clone() };
                        let asset = assets_collection.find_one(asset_filter, None).await?;

                        let mut asset_price: f64 = 0.0;
                        let mut asset_tvl: u128 = 0;

                        let mut rewards = vec![];

                        if asset.is_some() {
                            println!("asset: {:?}", asset.clone().unwrap().symbol);
                            let spb: U256 = stella_chef_v1.stella_per_block().call().await?;
                            let tap: U256 = stella_chef_v1.total_alloc_point().call().await?;

                            let average_block_time = 12.4;
                            let stella_filter =
                                doc! {"address":"0x0E358838ce72d5e61E0018a2ffaC4bEC5F4c88d2"};
                            let stella = assets_collection.find_one(stella_filter, None).await?;

                            if stella.is_some() {
                                let reward_asset_price = stella.clone().unwrap().price;
                                println!("reward_asset_price: {:?}", reward_asset_price);

                                asset_price = asset.clone().unwrap().price;
                                println!("asset_price: {:?}", asset_price);

                                let rewards_per_sec: f64 = (spb.as_u128() as f64
                                    * (ap as f64 / tap.as_u128() as f64))
                                    / average_block_time;
                                let rewards_per_day: f64 = rewards_per_sec * 60.0 * 60.0 * 24.0;
                                asset_tvl = total_lp.as_u128();

                                let ten: i128 = 10;
                                rewards.push(bson!({
                                    "amount": rewards_per_day as f64 / ten.pow(stella.clone().unwrap().decimals) as f64,
                                    "asset":  stella.clone().unwrap().symbol,
                                    "valueUSD": (rewards_per_day as f64 / ten.pow(stella.clone().unwrap().decimals) as f64) * reward_asset_price,
                                    "freq": models::Freq::Daily.to_string(),
                                }));

                                // reward_apr/farm_apr/pool_apr
                                println!(
                                    "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                    rewards_per_sec, rewards_per_day, asset_tvl
                                );
                                let mut reward_apr = 0.0;

                                if asset_tvl != 0 && asset_price != 0.0 {
                                    reward_apr = ((rewards_per_day as f64 * reward_asset_price)
                                        / (asset_tvl as f64 * asset_price))
                                        * 365.0
                                        * 100.0;
                                }
                                println!("reward_apr: {}", reward_apr);

                                // base_apr/trading_apr
                                let mut base_apr = 0.0;
                                #[derive(Serialize)]
                                pub struct Vars {
                                    addr: String,
                                }
                                let vars = Vars {
                                    addr: asset.clone().unwrap().address,
                                };
                                let pair_day_datas =
                                    p.6.query_with_vars_unwrap::<subgraph::PairDayDatas, Vars>(
                                        &pair_day_datas_query.clone(),
                                        vars,
                                    )
                                    .await;
                                if pair_day_datas.is_ok() {
                                    let mut daily_volume_lw: f64 = 0.0;
                                    for pdd in pair_day_datas.unwrap().pair_day_datas {
                                        let dv: f64 =
                                            pdd.daily_volume_usd.parse().unwrap_or_default();
                                        daily_volume_lw += dv;
                                    }
                                    daily_volume_lw /= 7.0;

                                    if asset.clone().unwrap_or_default().total_supply == 0.0
                                        || asset.clone().unwrap_or_default().price == 0.0
                                    {
                                        base_apr = 0.0;
                                    } else {
                                        base_apr = daily_volume_lw * 0.002 * 365.0 * 100.0
                                            / (asset.clone().unwrap_or_default().total_supply
                                                * asset.clone().unwrap_or_default().price);
                                    }
                                }

                                let timestamp = Utc::now().to_string();

                                println!("chef v1 farm lastUpdatedAtUTC {}", timestamp.clone());

                                let ff = doc! {
                                    "id": pid as i32,
                                    "chef": p.5.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };
                                let ten: f64 = 10.0;
                                let fu = doc! {
                                    "$set" : {
                                        "id": pid,
                                        "chef": p.5.clone(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "farmType": farm_type.to_string(),
                                        "farmImpl": farm_implementation.to_string(),
                                        "asset": {
                                            "symbol": asset.clone().unwrap().symbol,
                                            "address": asset_addr.clone(),
                                            "price": asset.clone().unwrap().price,
                                            "logos": asset.clone().unwrap().logos,
                                            // "underlying_assets": farm_assets,
                                        },
                                        "tvl": asset_tvl as f64 * asset_price / ten.powf(18.0),
                                        "apr.reward": reward_apr,
                                        "apr.base": base_apr,
                                        "rewards": rewards,
                                        "allocPoint": ap,
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };
                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                farms_collection
                                    .find_one_and_update(ff, fu, Some(options))
                                    .await?;
                            }
                        }
                    } else {
                        let rewarders =
                            p.1.pool_rewarders(ethers::prelude::U256::from(pid))
                                .call()
                                .await?;
                        println!("rewarders: {:?}", rewarders);

                        let (addresses, symbols, decimals, rewards_per_sec) =
                            p.1.pool_rewards_per_sec(ethers::prelude::U256::from(pid))
                                .call()
                                .await?;

                        println!(
                            "pool_rewards_per_sec\naddresses: {:?}, symbols: {:?}, decimals: {:?}, rewards_per_sec: {:?}",
                            addresses, symbols, decimals, rewards_per_sec
                        );

                        // stable amm asset
                        if p.3.clone() == "solarbeam".to_string()
                            && (pid == 8
                                || pid == 9
                                || pid == 13
                                || pid == 16
                                || pid == 17
                                || pid == 25)
                        {
                            farm_type = models::FarmType::StableAmm;

                            let stable_asset =
                                IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let name: String = stable_asset.name().call().await?;
                            let symbol: String = stable_asset.symbol().call().await?;
                            println!("name: {:?}", name);
                            // let split_name = name.split(" ");
                            // let split_name_vec: Vec<&str> = split_name.collect();
                            // if split_name_vec.len() > 1 && (split_name_vec[1] == "Stable") {
                            let owner_addr: Address = stable_asset.owner().call().await?;
                            let owner =
                                IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
                            let stable_lp_underlying_tokens = owner.get_tokens().call().await?;
                            let stable_lp_underlying_balances =
                                owner.get_token_balances().call().await?;
                            println!(
                                "stable_lp_underlying_tokens: {:#?}",
                                stable_lp_underlying_tokens
                            );
                            println!(
                                "stable_lp_underlying_balances: {:#?}",
                                stable_lp_underlying_balances
                            );

                            // busd: "0x5D9ab5522c64E1F6ef5e3627ECCc093f56167818"
                            // usdc: "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D"
                            // usdt: "0xB44a9B6905aF7c801311e8F4E76932ee959c663C"

                            let busd = IAnyswapV5ERC20::new(
                                "0x5D9ab5522c64E1F6ef5e3627ECCc093f56167818".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = IAnyswapV5ERC20::new(
                                "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = IAnyswapV5ERC20::new(
                                "0xB44a9B6905aF7c801311e8F4E76932ee959c663C".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let frax = IAnyswapV5ERC20::new(
                                "0x1A93B23281CC1CDE4C4741353F3064709A16197d".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mai = IAnyswapV5ERC20::new(
                                "0xFb2019DfD635a03cfFF624D210AEe6AF2B00fC2C".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mim = IAnyswapV5ERC20::new(
                                "0x0caE51e1032e8461f4806e26332c030E34De3aDb".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let wbtc = IAnyswapV5ERC20::new(
                                "0x6aB6d61428fde76768D7b45D8BFeec19c6eF91A8".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let xckbtc = IAnyswapV5ERC20::new(
                                "0xFFFfFfFfF6E528AD57184579beeE00c5d5e646F0".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let xcksm = IAnyswapV5ERC20::new(
                                "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let stksm = IAnyswapV5ERC20::new(
                                "0xFfc7780C34B450d917d557E728f033033CB4fA8C".parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x5D9ab5522c64E1F6ef5e3627ECCc093f56167818"};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D"};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xB44a9B6905aF7c801311e8F4E76932ee959c663C"};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;

                            let frax_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x1A93B23281CC1CDE4C4741353F3064709A16197d"};
                            let frax_asset = assets_collection.find_one(frax_filter, None).await?;
                            let mai_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFb2019DfD635a03cfFF624D210AEe6AF2B00fC2C"};
                            let mai_asset = assets_collection.find_one(mai_filter, None).await?;
                            let mim_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x0caE51e1032e8461f4806e26332c030E34De3aDb"};
                            let mim_asset = assets_collection.find_one(mim_filter, None).await?;

                            let wbtc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x6aB6d61428fde76768D7b45D8BFeec19c6eF91A8"};
                            let wbtc_asset = assets_collection.find_one(wbtc_filter, None).await?;
                            let xckbtc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFFFfFfFfF6E528AD57184579beeE00c5d5e646F0"};
                            let xckbtc_asset =
                                assets_collection.find_one(xckbtc_filter, None).await?;

                            let xcksm_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080"};
                            let xcksm_asset =
                                assets_collection.find_one(xcksm_filter, None).await?;
                            let stksm_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFfc7780C34B450d917d557E728f033033CB4fA8C"};
                            let stksm_asset =
                                assets_collection.find_one(stksm_filter, None).await?;

                            let ten: f64 = 10.0;
                            let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                            let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                            let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;

                            let frax_bal: U256 = frax.balance_of(owner_addr).call().await?;
                            let mai_bal: U256 = mai.balance_of(owner_addr).call().await?;
                            let mim_bal: U256 = mim.balance_of(owner_addr).call().await?;

                            let wbtc_bal: U256 = wbtc.balance_of(owner_addr).call().await?;
                            let xckbtc_bal: U256 = xckbtc.balance_of(owner_addr).call().await?;

                            let xcksm_bal: U256 = xcksm.balance_of(owner_addr).call().await?;
                            let stksm_bal: U256 = stksm.balance_of(owner_addr).call().await?;

                            let _3pool = IStableLpToken::new(
                                "0xfb29918d393AaAa7dD118B51A8b7fCf862F5f336".parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            let _3pool_bal: U256 = _3pool.balance_of(owner_addr).call().await?;

                            // TODO: calculate underlyingAssetsAlloc

                            if symbol == "3pool".to_string() {
                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / ten.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / ten.powf(6.0);
                                println!("3pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0xfb29918d393AaAa7dD118B51A8b7fCf862F5f336".clone(),
                                    "chain": p.5.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                println!("token lastUpdatedAtUTC {}", timestamp.clone());

                                let u = doc! {
                                    "$set" : {
                                        "address": "0xfb29918d393AaAa7dD118B51A8b7fCf862F5f336".to_string(),
                                        "chain": p.5.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - USD Pool".to_string(),
                                        "symbol": "3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.33, 0.33, 0.33],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "FRAX-3pool".to_string() {
                                let usd_pool_liq = _3pool_bal.as_u128() as f64 / ten.powf(18.0)
                                    + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x884609A4D86BBA8477112E36e27f7A4ACecB3575".clone(),
                                    "chain": p.5.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                println!("token lastUpdatedAtUTC {}", timestamp.clone());

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x884609A4D86BBA8477112E36e27f7A4ACecB3575".to_string(),
                                        "chain": p.5.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - FRAX Pool".to_string(),
                                        "symbol": "FRAX-3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            frax_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            frax_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "MAI-3pool".to_string() {
                                let usd_pool_liq = _3pool_bal.as_u128() as f64 / ten.powf(18.0)
                                    + mai_bal.as_u128() as f64 * mai_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x8CDB472731B4f815d67e76885a22269ad7f0e398".clone(),
                                    "chain": p.5.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                println!("token lastUpdatedAtUTC {}", timestamp.clone());

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x8CDB472731B4f815d67e76885a22269ad7f0e398".to_string(),
                                        "chain": p.5.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - MAI Pool".to_string(),
                                        "symbol": "MAI-3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            mai_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            mai_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "MIM-3pool".to_string() {
                                let usd_pool_liq = _3pool_bal.as_u128() as f64 / ten.powf(18.0)
                                    + mim_bal.as_u128() as f64 * mim_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x4BaB767c98186bA28eA66f2a69cd0DA351D60b36".clone(),
                                    "chain": p.5.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                println!("token lastUpdatedAtUTC {}", timestamp.clone());

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x4BaB767c98186bA28eA66f2a69cd0DA351D60b36".to_string(),
                                        "chain": p.5.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - MIM Pool".to_string(),
                                        "symbol": "MIM-3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            mim_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            mim_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "kBTC-BTC".to_string() {
                                let wbtc_price = wbtc_asset.clone().unwrap().price;
                                let xckbtc_price = xckbtc_asset.clone().unwrap().price;
                                let pool_liq = wbtc_bal.as_u128() as f64 * wbtc_price
                                    / ten.powf(8.0)
                                    + xckbtc_bal.as_u128() as f64 * xckbtc_price / ten.powf(8.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let pool_price = pool_liq / ts;
                                println!("pool_price {}", pool_price);

                                let f = doc! {
                                    "address": "0x4F707d051b4b49B63e72Cc671e78E152ec66f2fA".clone(),
                                    "chain": p.5.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                println!("token lastUpdatedAtUTC {}", timestamp.clone());

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x4F707d051b4b49B63e72Cc671e78E152ec66f2fA".to_string(),
                                        "chain": p.5.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - kBTC Pool".to_string(),
                                        "symbol": "kBTC-BTC".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            wbtc_asset.clone().unwrap().logos.get(0),
                                            xckbtc_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": pool_price,
                                        "liquidity": pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            wbtc_asset.clone().unwrap().address,
                                            xckbtc_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "stKSM".to_string() {
                                let pool_liq = xcksm_bal.as_u128() as f64
                                    * xcksm_asset.clone().unwrap().price
                                    / ten.powf(12.0)
                                    + stksm_bal.as_u128() as f64
                                        * stksm_asset.clone().unwrap().price
                                        / ten.powf(12.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let pool_price = pool_liq / ts;
                                println!("pool_price {}", pool_price);

                                let f = doc! {
                                    "address": "0x493147C85Fe43F7B056087a6023dF32980Bcb2D1".clone(),
                                    "chain": p.5.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                println!("token lastUpdatedAtUTC {}", timestamp.clone());

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x493147C85Fe43F7B056087a6023dF32980Bcb2D1".to_string(),
                                        "chain": p.5.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - stKSM Pool".to_string(),
                                        "symbol": "stKSM".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            xcksm_asset.clone().unwrap().logos.get(0),
                                            stksm_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": pool_price,
                                        "liquidity": pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            xcksm_asset.clone().unwrap().address,
                                            stksm_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.5, 0.5],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            }

                            // let mut comb: Vec<(H160, U256)> = vec![];
                            // comb = stable_lp_underlying_tokens
                            //     .clone()
                            //     .into_iter()
                            //     .zip(stable_lp_underlying_balances.clone().into_iter())
                            //     .collect();
                        }

                        // 4pool
                        if p.3.clone() == "beamswap".to_string() && (pid == 16) {
                            farm_type = models::FarmType::StableAmm;

                            let stable_asset =
                                IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let name: String = stable_asset.name().call().await?;
                            let symbol: String = stable_asset.symbol().call().await?;
                            println!("name: {:?}", name);
                            // let split_name = name.split(" ");
                            // let split_name_vec: Vec<&str> = split_name.collect();
                            // if split_name_vec.len() > 1 && (split_name_vec[1] == "Stable") {
                            let owner_addr: Address = stable_asset.owner().call().await?;
                            let owner =
                                IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
                            let stable_lp_underlying_tokens = owner.get_tokens().call().await?;
                            let stable_lp_underlying_balances =
                                owner.get_token_balances().call().await?;
                            println!(
                                "stable_lp_underlying_tokens: {:#?}",
                                stable_lp_underlying_tokens
                            );
                            println!(
                                "stable_lp_underlying_balances: {:#?}",
                                stable_lp_underlying_balances
                            );

                            // busd: "0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F"
                            // usdc: "0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b"
                            // usdt: "0xeFAeeE334F0Fd1712f9a8cc375f427D9Cdd40d73"
                            // dai: "0x765277EebeCA2e31912C9946eAe1021199B39C61"

                            let busd = IAnyswapV5ERC20::new(
                                "0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = IAnyswapV5ERC20::new(
                                "0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = IAnyswapV5ERC20::new(
                                "0xeFAeeE334F0Fd1712f9a8cc375f427D9Cdd40d73".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let dai = IAnyswapV5ERC20::new(
                                "0x765277EebeCA2e31912C9946eAe1021199B39C61".parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F"};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b"};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0xeFAeeE334F0Fd1712f9a8cc375f427D9Cdd40d73"};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;
                            let dai_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0x765277EebeCA2e31912C9946eAe1021199B39C61"};
                            let dai_asset = assets_collection.find_one(dai_filter, None).await?;

                            let ten: f64 = 10.0;
                            let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                            let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                            let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;
                            let dai_bal: U256 = dai.balance_of(owner_addr).call().await?;

                            let _4pool = IStableLpToken::new(
                                "0x9BF6910790D70E9b5B07Cb28271C42531B929b4c".parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            // let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                            if symbol == "4pool".to_string() {
                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / ten.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + dai_bal.as_u128() as f64 * dai_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                println!("4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x9BF6910790D70E9b5B07Cb28271C42531B929b4c".clone(),
                                    "chain": p.5.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                println!("token lastUpdatedAtUTC {}", timestamp.clone());

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x9BF6910790D70E9b5B07Cb28271C42531B929b4c".to_string(),
                                        "chain": p.5.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Beamswap Stable DEX - Stable Multichain".to_string(),
                                        "symbol": "4pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                            dai_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                            dai_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            }
                        }

                        if rewards_per_sec.len() > 0 {
                            let mut total_reward_apr = 0.0;

                            let asset_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                            println!("asset_addr: {:?}", asset_addr.clone());

                            let asset_filter = doc! { "address": asset_addr.clone() };
                            let asset = assets_collection.find_one(asset_filter, None).await?;

                            let mut asset_price: f64 = 0.0;
                            let mut asset_tvl: u128 = 0;

                            let mut rewards = vec![];

                            if asset.is_some() {
                                println!("asset: {:?}", asset.clone().unwrap().symbol);

                                for i in 0..symbols.len() {
                                    println!("rwrd[{}]", i);

                                    let s = format!("{:?}", symbols[i].clone());
                                    println!("symbol: {}", s);

                                    let reward_asset_addr =
                                        ethers::utils::to_checksum(&addresses[i].to_owned(), None);
                                    println!("reward_asset_addr: {:?}", reward_asset_addr);

                                    let reward_asset_filter = doc! { "address": reward_asset_addr };
                                    let reward_asset = assets_collection
                                        .find_one(reward_asset_filter, None)
                                        .await?;

                                    if reward_asset.is_some() {
                                        let reward_asset_price =
                                            reward_asset.clone().unwrap().price;
                                        println!("reward_asset_price: {:?}", reward_asset_price);

                                        asset_price = asset.clone().unwrap().price;
                                        println!("asset_price: {:?}", asset_price);

                                        let rewards_per_day: u128 =
                                            rewards_per_sec[i].as_u128() * 60 * 60 * 24;
                                        asset_tvl = total_lp.as_u128();

                                        let ten: i128 = 10;
                                        rewards.push(bson!({
                                            "amount": rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64,
                                            "asset":  reward_asset.clone().unwrap().symbol,
                                            "valueUSD": (rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64) * reward_asset_price,
                                            "freq": models::Freq::Daily.to_string(),
                                        }));

                                        println!("slaayyyyyyyy asset_price {}", asset_price);
                                        // reward_apr/farm_apr/pool_apr
                                        println!(
                                            "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                            rewards_per_sec[i].as_u128(),
                                            rewards_per_day,
                                            asset_tvl
                                        );

                                        let reward_apr = ((rewards_per_day as f64
                                            / ten.pow(decimals[i].as_u128().try_into().unwrap())
                                                as f64
                                            * reward_asset_price)
                                            / (asset_tvl as f64 * asset_price
                                                / ten.pow(18) as f64))
                                            * 365.0
                                            * 100.0;
                                        println!("reward_apr: {}", reward_apr);
                                        if asset_tvl != 0 && asset_price != 0.0 {
                                            total_reward_apr += reward_apr;
                                        }
                                    }
                                }

                                // base_apr/trading_apr
                                let mut base_apr = 0.0;
                                #[derive(Serialize)]
                                pub struct Vars {
                                    addr: String,
                                }
                                let vars = Vars {
                                    addr: asset.clone().unwrap().address,
                                };
                                let pair_day_datas =
                                    p.6.query_with_vars_unwrap::<subgraph::PairDayDatas, Vars>(
                                        &pair_day_datas_query.clone(),
                                        vars,
                                    )
                                    .await;
                                if pair_day_datas.is_ok() {
                                    let mut daily_volume_lw: f64 = 0.0;
                                    for pdd in pair_day_datas.unwrap().pair_day_datas {
                                        let dv: f64 =
                                            pdd.daily_volume_usd.parse().unwrap_or_default();
                                        daily_volume_lw += dv;
                                    }
                                    daily_volume_lw /= 7.0;

                                    if asset.clone().unwrap_or_default().total_supply == 0.0
                                        || asset.clone().unwrap_or_default().price == 0.0
                                    {
                                        base_apr = 0.0;
                                    } else {
                                        base_apr = daily_volume_lw * 0.002 * 365.0 * 100.0
                                            / (asset.clone().unwrap_or_default().total_supply
                                                * asset.clone().unwrap_or_default().price);
                                    }
                                }

                                // let mut farm_assets = vec![];
                                // for ua in asset.clone().unwrap().underlying_assets {
                                //     let underlying_asset_filter = doc! { "address": ua.clone() };
                                //     let underlying_asset = assets_collection
                                //         .find_one(underlying_asset_filter, None)
                                //         .await?;
                                //     if underlying_asset.is_some() {
                                //         farm_assets.push(bson!({
                                //             "symbol": underlying_asset.clone().unwrap().symbol,
                                //             "address": underlying_asset.clone().unwrap().address,
                                //             "underlyingAssets": underlying_asset.clone().unwrap().underlying_assets,
                                //         }));
                                //     }
                                // }

                                let timestamp = Utc::now().to_string();

                                println!("chef v2 farm lastUpdatedAtUTC {}", timestamp.clone());

                                let ff = doc! {
                                    "id": pid as i32,
                                    "chef": p.5.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };
                                let ten: f64 = 10.0;
                                let fu = doc! {
                                    "$set" : {
                                        "id": pid,
                                        "chef": p.5.clone(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "farmType": farm_type.to_string(),
                                        "farmImpl": farm_implementation.to_string(),
                                        "asset": {
                                            "symbol": asset.clone().unwrap().symbol,
                                            "address": asset_addr.clone(),
                                            "price": asset.clone().unwrap().price,
                                            "logos": asset.clone().unwrap().logos,
                                            // "underlying_assets": farm_assets,
                                        },
                                        "tvl": asset_tvl as f64 * asset_price / ten.powf(18.0),
                                        "apr.reward": total_reward_apr,
                                        "apr.base": base_apr,
                                        "rewards": rewards,
                                        "allocPoint": ap,
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };
                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                farms_collection
                                    .find_one_and_update(ff, fu, Some(options))
                                    .await?;
                            } else {
                                println!("pdne");
                            }
                        }
                    }
                } else {
                    println!("allocPoint = 0");
                }
            }
        }
    }

    Ok(())
}

async fn get_one_day_block(subgraph_url: String, query_str: String) -> u64 {
    let date = Utc::now().timestamp() - 86400;
    let start = date / 1000;
    let end = date / 1000 + 600;

    let subgraph_client = Client::new(subgraph_url.clone());
    #[derive(Serialize)]
    pub struct Vars {
        start: i64,
        end: i64,
    }
    let vars = Vars {
        start: start,
        end: end,
    };
    let blocks_data = subgraph_client
        .query_with_vars_unwrap::<subgraph::BlocksData, Vars>(&query_str, vars)
        .await;

    if blocks_data.is_ok() {
        if blocks_data.clone().unwrap().blocks.len() > 0 {
            let block_number = blocks_data.clone().unwrap().blocks[0]
                .number
                .parse()
                .unwrap();
            return block_number;
        }
    }

    0
}

async fn get_one_day_pools(
    subgraph_url: String,
    query_str: String,
    block_number: u64,
) -> Vec<subgraph::Pair> {
    let subgraph_client = Client::new(subgraph_url.clone());
    #[derive(Serialize)]
    pub struct Vars {
        number: u64,
    }
    let vars = Vars {
        number: block_number,
    };
    let pairs_data = subgraph_client
        .query_with_vars_unwrap::<subgraph::PairsData, Vars>(&query_str, vars)
        .await;

    if pairs_data.is_ok() {
        return pairs_data.clone().unwrap().pairs;
    }
    return vec![];
}

async fn fetch_3usd(
    taiga_query_str: String,
    karura_dex_query_str: String,
) -> (f64, Vec<(i32, String, f64, String)>, (f64, f64)) {
    let subql_client =
        Client::new("https://api.subquery.network/sq/nutsfinance/taiga-protocol".to_string());
    #[derive(Serialize)]
    pub struct Vars {
        days: i64,
    }
    let vars = Vars { days: 14 };
    let pool_data = subql_client
        .query_with_vars_unwrap::<subgraph::TapioDD, Vars>(&taiga_query_str, vars)
        .await;

    let mut tvl = 0.0;
    let mut apr = (0.0, 0.0);
    if pool_data.is_ok() {
        tvl = pool_data
            .clone()
            .unwrap()
            .daily_data
            .nodes
            .get(0)
            .unwrap()
            .total_supply;

        apr = fetch_3usd_apr(pool_data.clone().unwrap(), karura_dex_query_str.clone()).await;
    }

    let tai_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "TAI".to_string(), 1).await;
    let tai_ksm_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "sa://0".to_string(), 1).await;
    let lksm_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "LKSM".to_string(), 1).await;
    let kar_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "KAR".to_string(), 1).await;

    let rewards = vec![
        // (amount, asset, value_usd, freq)
        (
            8000,
            "TAI".to_string(),
            tai_price_history[0].0 * 8000.0,
            "Weekly".to_string(),
        ),
        (
            30,
            "taiKSM".to_string(),
            tai_ksm_price_history[0].0 * 30.0,
            "Weekly".to_string(),
        ),
        (
            250,
            "LKSM".to_string(),
            lksm_price_history[0].0 * 250.0,
            "Weekly".to_string(),
        ),
        (
            2000,
            "KAR".to_string(),
            kar_price_history[0].0 * 2000.0,
            "Weekly".to_string(),
        ),
    ];

    (tvl, rewards, apr)
}

async fn fetch_tai_ksm(
    taiga_query_str: String,
    karura_dex_query_str: String,
) -> (f64, Vec<(i32, String, f64, String)>, (f64, f64)) {
    let subql_client =
        Client::new("https://api.subquery.network/sq/nutsfinance/taiga-protocol".to_string());
    #[derive(Serialize)]
    pub struct Vars {
        days: i64,
    }
    let vars = Vars { days: 30 };
    let pool_data = subql_client
        .query_with_vars_unwrap::<subgraph::TapioDD, Vars>(&taiga_query_str, vars)
        .await;

    let mut current_supply = 0.0;
    let mut tvl = 0.0;
    let mut apr = (0.0, 0.0);
    let mut rewards = vec![];

    if pool_data.is_ok() {
        println!("pool_datau {:?}", pool_data.clone().unwrap());

        let tai_price_history =
            get_token_price_history(karura_dex_query_str.clone(), "TAI".to_string(), 1).await;
        let tai_ksm_price_history =
            get_token_price_history(karura_dex_query_str.clone(), "sa://0".to_string(), 1).await;

        let tai_price = tai_price_history[0].0;
        let tai_ksm_price = tai_ksm_price_history[0].0;

        current_supply = pool_data
            .clone()
            .unwrap()
            .daily_data
            .nodes
            .get(0)
            .unwrap()
            .total_supply;

        tvl = current_supply * tai_ksm_price;

        apr = fetch_tai_ksm_apr(pool_data.clone().unwrap(), karura_dex_query_str.clone()).await;

        rewards = vec![
            // (amount, asset, value_usd, freq)
            (
                4000,
                "TAI".to_string(),
                tai_price * 4000.0,
                "Daily".to_string(),
            ),
        ];
    }

    (tvl, rewards, apr)
}

async fn fetch_3usd_apr(pool_data: subgraph::TapioDD, karura_dex_query_str: String) -> (f64, f64) {
    let days = 14;

    let daily_data = pool_data.clone().daily_data.nodes;

    let tai_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "TAI".to_string(), days).await;
    let tai_ksm_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "sa://0".to_string(), days).await;
    let lksm_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "LKSM".to_string(), days).await;
    let kar_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "KAR".to_string(), days).await;

    let mut total = 0.0;
    let daily_total_supply = daily_data.clone().into_iter().map(|node| node.total_supply);
    let dts: Vec<f64> = daily_total_supply.clone().collect();

    for i in 0..daily_total_supply.len() {
        // 8000 TAI per week
        total += (8000.0 * tai_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];

        // 30 taiKSM per week
        total += (30.0 * tai_ksm_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];

        // 250 LKSM per week
        total += (250.0 * lksm_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];

        // 2000 KAR per week
        total += (2000.0 * kar_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];
    }

    let reward_apr = total / daily_total_supply.len() as f64;
    let base_apr = calculate_base_apr(daily_data.clone());

    (base_apr, reward_apr)
}

async fn fetch_tai_ksm_apr(
    pool_data: subgraph::TapioDD,
    karura_dex_query_str: String,
) -> (f64, f64) {
    let days = 30;

    let daily_data = pool_data.clone().daily_data.nodes;

    let tai_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "TAI".to_string(), days).await;
    let tai_ksm_price_history =
        get_token_price_history(karura_dex_query_str.clone(), "sa://0".to_string(), days).await;

    let mut total = 0.0;
    let daily_total_supply = daily_data.clone().into_iter().map(|node| node.total_supply);
    let dts: Vec<f64> = daily_total_supply.clone().collect();

    for i in 0..daily_total_supply.len() {
        // 4000 TAI each day
        total += (4000.0 * tai_price_history.get(i).unwrap().0 * (365.0))
            / (dts[i] * tai_ksm_price_history.get(i).unwrap().0);
    }

    let reward_apr = total / daily_total_supply.len() as f64;
    let base_apr = calculate_base_apr(daily_data.clone());

    (base_apr, reward_apr)
}

fn calculate_base_apr(daily_data: Vec<subgraph::TapioDailyDataNode>) -> f64 {
    let daily_fee_apr = daily_data
        .clone()
        .into_iter()
        .map(|d| d.fee_volume * 365.0 / d.total_supply);
    let daily_yield_apr = daily_data
        .clone()
        .into_iter()
        .map(|d| d.yield_volume * 365.0 / d.total_supply);

    // daily_fee_apr.filter(|apr| apr.to_owned() < 0.5).sum();
    let dfaprf = daily_fee_apr.filter(|apr| apr.to_owned() < 0.5);
    let fee_apr = dfaprf.clone().sum::<f64>() / dfaprf.clone().count() as f64;
    let yield_apr =
        daily_yield_apr.clone().sum::<f64>() as f64 / daily_yield_apr.clone().count() as f64;

    fee_apr + yield_apr
}

async fn get_token_price_history(
    query_str: String,
    asset: String,
    days: i64,
) -> Vec<(f64, String)> {
    let subql_client =
        Client::new("https://api.subquery.network/sq/AcalaNetwork/karura-dex".to_string());
    #[derive(Serialize)]
    pub struct Vars {
        days: i64,
        asset: String,
    }
    let vars = Vars {
        days: days,
        asset: asset,
    };
    let price_history_data = subql_client
        .query_with_vars_unwrap::<subgraph::KaruraTokenPriceHistoryData, Vars>(&query_str, vars)
        .await;

    println!("price_history_data {:?}", price_history_data);

    let ph = price_history_data
        .unwrap_or_default()
        .token
        .daily_data
        .nodes
        .into_iter()
        .map(|x| {
            let ten: f64 = 10.0;
            let p: f64 = x.price.parse().unwrap_or_default();
            return (p / ten.powf(18.0), x.timestamp);
        });
    let ph_vec: Vec<(f64, String)> = ph.collect();

    ph_vec
}

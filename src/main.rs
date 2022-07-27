use std::{collections::HashMap, str::FromStr, sync::Arc, thread, time};

use chrono::prelude::Utc;
use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::{abigen, Address, U256},
    providers::{Http, Provider},
    signers::LocalWallet,
    utils::to_checksum,
};
use ethers_providers::Middleware;
use gql_client::Client;
use mongodb::{
    bson::{bson, doc},
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};

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
    IStandardLpToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
    ]"#,
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let delay = time::Duration::from_secs(60 * 2);
    loop {
        run_jobs().await;
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

    let solarbeam_subgraph = "https://api.thegraph.com/subgraphs/name/solar-ape/solarbeam";
    let stellaswap_subgraph = "https://api.thegraph.com/subgraphs/name/stellaswap/stella-swap";
    let beamswap_subgraph = "https://api.thegraph.com/subgraphs/name/beamswap/beamswap-dex";

    let _solarbeam_blocklytics_subgraph =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/blocklytics";
    let _solarflare_blocklytics_subgraph =
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

    let _one_day_blocks_query = r#"
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

    let _one_day_pools_query = r#"
         query OneDayPools($blocknum: Int!) {
             pairs(orderBy: reserveUSD, orderDirection: desc, first: 1000, block: { number: $blocknum }) {
                 id
                 reserveUSD
                 volumeUSD
                 untrackedVolumeUSD
             }
         }
     "#;

    let solarbeam_subgraph_client =
        Client::new_with_headers(solarbeam_subgraph.clone(), headers.clone());
    let stellaswap_subgraph_client =
        Client::new_with_headers(stellaswap_subgraph.clone(), headers.clone());
    let beamswap_subgraph_client =
        Client::new_with_headers(beamswap_subgraph.clone(), headers.clone());

    // let moonriver_blocklytics_client =
    //     Client::new_with_headers(solarbeam_blocklytics_subgraph.clone(), headers.clone());
    // let moonbeam_blocklytics_client =
    //     Client::new_with_headers(solarflare_blocklytics_subgraph.clone(), headers.clone());

    // subgraph fetching jobs
    let protocols = vec![
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
                } else if p.0.clone() == "beamswap" {
                    logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token_addr.clone());
                }

                println!("logo {}", logo.clone());

                let liquidity: f64 = t.total_liquidity.parse().unwrap_or_default();

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
                } else if p.0.clone() == "beamswap" {
                    token0logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token0_addr.clone());
                    token1logo = format!("https://raw.githubusercontent.com/BeamSwap/beamswap-tokenlist/main/assets/chains/moonbeam/{}/logo.png", token1_addr.clone());
                }

                let token0decimals: u32 = pair.token0.decimals.parse().unwrap_or_default();
                let token1decimals: u32 = pair.token1.decimals.parse().unwrap_or_default();

                let mut decimals = token0decimals;
                if token1decimals > token0decimals {
                    decimals = token1decimals;
                }

                let liquidity: f64 = pair.reserve_usd.parse().unwrap_or_default();
                let total_supply: f64 = pair.total_supply.parse().unwrap_or_default();

                let price_usd: f64 = liquidity / total_supply;

                println!("price_usd {}", price_usd);

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
                        "feesAPR": 0.0,
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

    let _protocols = vec![
        (
            beam_chef_address,
            beam_chef,
            "moonbeam".to_string(),
            "beamswap".to_string(),
            "v2".to_string(),
            "0xC6ca172FC8BDB803c5e12731109744fb0200587b".to_string(),
        ),
        (
            stella_chef_v1_address,
            stella_chef_v1,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v1".to_string(),
            "0xEDFB330F5FA216C9D2039B99C8cE9dA85Ea91c1E".to_string(),
        ),
        (
            stella_chef_v2_address,
            stella_chef_v2,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v2".to_string(),
            "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388".to_string(),
        ),
        (
            solarbeam_chef_address,
            solarbeam_chef,
            "moonriver".to_string(),
            "solarbeam".to_string(),
            "v2".to_string(),
            "0x0329867a8c457e9F75e25b0685011291CD30904F".to_string(),
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

            let farm_type = models::FarmType::StandardAmm;
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
                                "value_usd": (rewards_per_day as f64 / ten.pow(stella.clone().unwrap().decimals) as f64) * reward_asset_price,
                                "freq": models::Freq::Daily.to_string(),
                            }));

                            // reward_apr/farm_apr/pool_apr
                            println!(
                                "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                rewards_per_sec, rewards_per_day, asset_tvl
                            );
                            let reward_apr = ((rewards_per_day as f64 * reward_asset_price)
                                / (asset_tvl as f64 * asset_price))
                                * 365.0
                                * 100.0;
                            println!("reward_apr: {}", reward_apr);

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
                                        // "underlying_assets": farm_assets,
                                    },
                                    "tvl": asset_tvl as f64 * asset_price / ten.powf(18.0),
                                    "apr.reward": reward_apr,
                                    "rewards": rewards,
                                    "allocPoint": ap
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
                                    let reward_asset_price = reward_asset.clone().unwrap().price;
                                    println!("reward_asset_price: {:?}", reward_asset_price);

                                    asset_price = asset.clone().unwrap().price;
                                    println!("asset_price: {:?}", asset_price);

                                    let rewards_per_day: u128 =
                                        rewards_per_sec[i].as_u128() * 60 * 60 * 24;
                                    asset_tvl = total_lp.as_u128();

                                    let ten: i128 = 10;
                                    rewards.push(bson!({
                                        "amount": rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64,
                                        "asset":  asset.clone().unwrap().symbol,
                                        "value_usd": (rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64) * reward_asset_price,
                                        "freq": models::Freq::Daily.to_string(),
                                    }));

                                    // reward_apr/farm_apr/pool_apr
                                    println!(
                                        "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                        rewards_per_sec[i].as_u128(),
                                        rewards_per_day,
                                        asset_tvl
                                    );
                                    let reward_apr = ((rewards_per_day as f64
                                        * reward_asset_price)
                                        / (asset_tvl as f64 * asset_price))
                                        * 365.0
                                        * 100.0;
                                    println!("reward_apr: {}", reward_apr);
                                    total_reward_apr += reward_apr;
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
                                        // "underlying_assets": farm_assets,
                                    },
                                    "tvl": asset_tvl as f64 * asset_price / ten.powf(18.0),
                                    "apr.reward": total_reward_apr,
                                    "rewards": rewards,
                                    "allocPoint": ap
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

    Ok(())
}

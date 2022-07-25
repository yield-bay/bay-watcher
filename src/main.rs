use std::{collections::HashMap, str::FromStr, thread, time};

use chrono::prelude::Utc;
use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::Address,
    providers::{Http, Provider},
    signers::LocalWallet,
    utils::to_checksum,
};
use gql_client::Client;
use mongodb::{
    bson::doc,
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};

mod models;
mod subgraph;

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
    // let farms_collection = db.collection::<models::Farm>("farms");

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

    let solarbeam_client = Client::new_with_headers(solarbeam_subgraph.clone(), headers.clone());
    let stellaswap_client = Client::new_with_headers(stellaswap_subgraph.clone(), headers.clone());
    let beamswap_client = Client::new_with_headers(beamswap_subgraph.clone(), headers.clone());

    // let moonriver_blocklytics_client =
    //     Client::new_with_headers(solarbeam_blocklytics_subgraph.clone(), headers.clone());
    // let moonbeam_blocklytics_client =
    //     Client::new_with_headers(solarflare_blocklytics_subgraph.clone(), headers.clone());

    let protocols = vec![
        (
            "stellaswap",
            "moonbeam",
            stellaswap_client.clone(),
            stellaswap_subgraph.clone(),
        ),
        (
            "solarbeam",
            "moonriver",
            solarbeam_client.clone(),
            solarbeam_subgraph.clone(),
        ),
        (
            "beamswap",
            "moonbeam",
            beamswap_client.clone(),
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

    Ok(())
}

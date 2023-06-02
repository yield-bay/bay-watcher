use std::collections::HashMap;

use chrono::prelude::Utc;
use ethers::{
    middleware::SignerMiddleware,
    prelude::{Address, U256},
    providers::{Http, Provider},
    signers::LocalWallet,
    utils::to_checksum,
};
use gql_client::Client;
use mongodb::{
    bson::{bson, doc, Bson},
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};
use serde::Serialize;
use struct_iterable::Iterable;

use crate::apis;
use crate::constants;
use crate::models;
use crate::subgraph;

pub async fn pulsar_jobs(mongo_uri: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db = client.database(&db_name);

    let farms_collection = db.collection::<models::Farm>("farms");
    let assets_collection = db.collection::<models::Asset>("assets");

    let subgraph_client = Client::new(
        constants::subgraph_urls::STELLASWAP_PULSAR_SUBGRAPH.to_string(),
        60,
    );

    let pool_data = subgraph_client
        .query_unwrap::<subgraph::PulsarData>(constants::chef::LISTED_POOLS_QUERY.clone())
        .await;

    // println!("pool_data {:?}", pool_data);

    let rewards_subgraph_client = Client::new(
        constants::subgraph_urls::STELLASWAP_PULSAR_REWARDS_SUBGRAPH.to_string(),
        60,
    );

    let pool_rewards_data = rewards_subgraph_client
        .query_unwrap::<subgraph::EternalFarmingData>(
            constants::chef::ETERNAL_FARMINGS_QUERY.clone(),
        )
        .await;

    // println!("pool_rewards_data {:?}", pool_rewards_data);

    let reward_apr_resp = reqwest::get("https://apr-api.stellaswap.com/api/v1/eternalAPR")
        .await?
        .json::<apis::pulsar::Root>()
        .await?;

    // println!("reward_apr_resp {:?}", reward_apr_resp);

    let base_apr_resp = reqwest::get("https://apr-api.stellaswap.com/api/v1/poolsAPR")
        .await?
        .json::<apis::pulsar::PoolsAPRRoot>()
        .await?;

    let listed_pools = vec![
        "0x4cc7f3a2d35a2f3eb5312c6fde200fa496c3fa88".to_string(),
        "0x79eb71c1592a678c234ea221ed3fdc10cee89f88".to_string(),
        "0x416bd9798d5214cae6f837c0a53a73beb3ced465".to_string(),
        "0xb13b281503f6ec8a837ae1a21e86a9cae368fcc5".to_string(),
        "0xab8c35164a8e3ef302d18da953923ea31f0fe393".to_string(),
        "0x7e71d586ad01c0bf7953eb82e7b76c1338b0068c".to_string(),
        "0x5daf7f80cc550ee6249a4635c3bb0678e94d3867".to_string(),
        "0x1b11d991f32fb59ec4ee744de68ad65d9e85b2d2".to_string(),
        "0x86c50c9bc4e3f15d6d6eb47393cb07bd701bcf04".to_string(),
        "0x2f7daf9b66a7bee6f9e046973e2e3d01d810207c".to_string(),
        "0xdd228d01e11041050be93fcfaf3005930782810e".to_string(),
        "0xda6381c6c5a20bc03d0dac2bf9d165d5775f41f7".to_string(),
    ];
    // println!("base_apr_resp {:?}", base_apr_resp);
    // base_apr_resp.result
    println!("fetched all data");
    if pool_data.is_ok() {
        if pool_rewards_data.is_ok() {
            let mut h: HashMap<String, subgraph::EternalFarming> = HashMap::new();
            for pr in pool_rewards_data.clone().unwrap().eternal_farmings {
                h.insert(pr.pool.clone(), pr.clone());
            }
            let mut reward_apr_map: HashMap<String, f64> = HashMap::new();
            if reward_apr_resp.is_success {
                for fp in reward_apr_resp.result.farm_pools.clone() {
                    let reward_apr: f64 = fp.1.last_apr.parse().unwrap_or_default();
                    reward_apr_map.insert(fp.0, reward_apr);
                }
                let mut base_apr_map: HashMap<String, f64> = HashMap::new();
                if base_apr_resp.is_success {
                    for (x, v) in base_apr_resp.result.iter() {
                        // println!(
                        //     "xxx {:?} {:?}",
                        //     &x[1..].to_string(),
                        //     v.downcast_ref::<f64>().unwrap()
                        // );
                        let val = *v.downcast_ref::<f64>().unwrap();
                        base_apr_map.insert(x[1..].to_string(), val);
                    }
                    for pool in pool_data.clone().unwrap().pools {
                        let timestamp = Utc::now().to_string();

                        println!("pulsar farm lastUpdatedAtUTC {}", timestamp.clone());

                        let tvl: f64 = pool.total_value_locked_usd.parse().unwrap_or_default();

                        if listed_pools.contains(&pool.id.clone()) {
                            println!("contains");
                            let mut base_apr = 0.0;
                            if base_apr_map.get(&pool.id).is_some() {
                                base_apr = *base_apr_map.get(&pool.id).unwrap();
                            }
                            let mut reward_apr = 0.0;
                            if reward_apr_map.get(&pool.id).is_some() {
                                reward_apr = *reward_apr_map.get(&pool.id).unwrap();
                            }

                            // println!(
                            //     "{:?}: {:?}-{:?} LP, tvl: {:?}, base apr: {:?}, reward apr: {:?}, rewards: {:#?}",
                            //     pool.id,
                            //     pool.token0.symbol,
                            //     pool.token1.symbol,
                            //     tvl,
                            //     // base_apr_map.get(&pool.id).unwrap(),
                            //     // reward_apr_map.get(&pool.id).unwrap(),
                            //     h.get(&pool.id)
                            // );

                            let mut rewards: Vec<Bson> = vec![];

                            if h.get(&pool.id).is_some() {
                                let mut rr = 0.0;
                                if h.get(&pool.id).unwrap().reward_rate != "0".to_string() {
                                    rr = h.get(&pool.id).unwrap().reward_rate.parse().unwrap();
                                }
                                let mut brr = 0.0;
                                if h.get(&pool.id).unwrap().bonus_reward_rate != "0".to_string() {
                                    brr =
                                        h.get(&pool.id).unwrap().bonus_reward_rate.parse().unwrap();
                                }
                                println!(
                                    "pid: {:?}: rr {:?}, brr {:?}",
                                    pool.id.clone(),
                                    rr * 86400.0,
                                    brr * 86400.0
                                );
                                // }

                                // let rr: f64 = h.get(&pool.id).unwrap().reward_rate.parse().unwrap();
                                // let brr: f64 = h.get(&pool.id).unwrap().bonus_reward_rate.parse().unwrap();

                                let rt = &h.get(&pool.id).unwrap().reward_token;

                                let rt_asset_addr = ethers::utils::to_checksum(
                                    &rt.as_str().parse::<Address>()?,
                                    None,
                                );
                                // println!("rt_asset_addr {:?}", asset_addr.clone());
                                let rt_asset_filter = doc! { "address": rt_asset_addr.clone(), "protocol": "stellaswap", "chain": "moonbeam" };
                                let rt_asset =
                                    assets_collection.find_one(rt_asset_filter, None).await?;
                                println!("rt_asset {:?}", rt_asset);
                                let brt = &h.get(&pool.id).unwrap().bonus_reward_token;

                                let brt_asset_addr = ethers::utils::to_checksum(
                                    &brt.as_str().parse::<Address>()?,
                                    None,
                                );
                                // println!("brt_asset_addr {:?}", asset_addr.clone());
                                let brt_asset_filter = doc! { "address": brt_asset_addr.clone(), "protocol": "stellaswap", "chain": "moonbeam" };
                                let brt_asset =
                                    assets_collection.find_one(brt_asset_filter, None).await?;
                                println!("brt_asset {:?}", brt_asset);
                                println!(
                                    "rrrr {:?}, {:?}, {:?}, {:?}",
                                    rr,
                                    constants::utils::TEN_F64
                                        .powf(rt_asset.clone().unwrap().decimals as f64),
                                    rr / constants::utils::TEN_F64
                                        .powf(rt_asset.clone().unwrap().decimals as f64),
                                    rr / constants::utils::TEN_F64
                                        .powf(rt_asset.clone().unwrap().decimals as f64)
                                        * rt_asset.clone().unwrap().price,
                                );
                                if rt_asset.is_some() {
                                    rewards.push(bson!({
                                    "amount": rr * 86400.0 / constants::utils::TEN_F64.powf(rt_asset.clone().unwrap().decimals as f64) as f64,
                                    "asset":  rt_asset.clone().unwrap().symbol,
                                    "valueUSD": (rr * 86400.0 / constants::utils::TEN_F64.powf(rt_asset.clone().unwrap().decimals as f64) as f64) * rt_asset.clone().unwrap().price,
                                    "freq": models::Freq::Daily.to_string(),
                                }))
                                }
                                if brt_asset.is_some() {
                                    rewards.push(bson!({
                                    "amount": brr * 86400.0 / constants::utils::TEN_F64.powf(brt_asset.clone().unwrap().decimals as f64) as f64,
                                    "asset":  brt_asset.clone().unwrap().symbol,
                                    "valueUSD": (brr * 86400.0 / constants::utils::TEN_F64.powf(brt_asset.clone().unwrap().decimals as f64) as f64) * brt_asset.clone().unwrap().price,
                                    "freq": models::Freq::Daily.to_string(),
                                }))
                                }
                            }

                            let token0logo = format!(
                                "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                                pool.token0.symbol
                            );
                            let token1logo = format!(
                                "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                                pool.token1.symbol
                            );

                            let ff = doc! {
                                "id": 0,
                                "chef": "pulsar".to_string(),
                                "chain": "moonbeam".to_string(),
                                "protocol": "Stellaswap Pulsar".to_string(),
                                "asset.address": pool.id.clone(),
                            };
                            let fu = doc! {
                                "$set" : {
                                    "id": 0,
                                    "chef": "pulsar".to_string(),
                                    "chain": "moonbeam".to_string(),
                                    "protocol": "Stellaswap Pulsar".to_string(),
                                    "farmType": models::FarmType::ConcentratedLiquidity.to_string(),
                                    "farmImpl": models::FarmImplementation::Solidity.to_string(),
                                    "asset": {
                                        "symbol": format!("{}-{} LP", pool.token0.symbol, pool.token1.symbol),
                                        "address": pool.id.clone(),
                                        "price": 0 as f64,
                                        "logos": [token0logo, token1logo],
                                    },
                                    "tvl": tvl,
                                    "apr.reward": reward_apr,
                                    "apr.base": base_apr,
                                    "rewards": rewards,
                                    "allocPoint": 1,
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
                }
            }
        }
    }

    Ok(())
}

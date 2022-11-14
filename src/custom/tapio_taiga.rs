use chrono::prelude::Utc;
use gql_client::Client;
use mongodb::{
    bson::{bson, doc, Bson},
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};
use serde::Serialize;

use crate::apis;
use crate::constants;
use crate::models;
use crate::subgraph;

pub async fn tapio_taiga_jobs(mongo_uri: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db = client.database(&db_name);

    let farms_collection = db.collection::<models::Farm>("farms");

    let tapio_rewards_resp = reqwest::get("https://api.taigaprotocol.io/rewards?network=acala")
        .await?
        .json::<apis::tapio::Root>()
        .await?;

    println!("tapio_rewards_resp {:?}", tapio_rewards_resp);

    let taiga_rewards_resp = reqwest::get("https://api.taigaprotocol.io/rewards")
        .await?
        .json::<apis::taiga::Root>()
        .await?;

    println!("taiga_rewards_resp {:?}", taiga_rewards_resp);

    if taiga_rewards_resp.clone().taiksm.is_some() {
        let tai_ksm_base_apr = taiga_rewards_resp.clone().taiksm.unwrap().taiksm_fee.apr * 100.0;
        let tai_ksm_reward_apr = (taiga_rewards_resp.clone().taiksm.unwrap().kar_reward.apr as f64
            + taiga_rewards_resp.clone().taiksm.unwrap().tai_reward.apr
            + taiga_rewards_resp.clone().taiksm.unwrap().taiksm_yield.apr)
            * 100.0;

        let _tai_ksm = fetch_tai_ksm(
            constants::taiga::DAILY_DATA_TAI_KSM_QUERY.to_owned(),
            constants::taiga::TOKEN_PRICE_HISTORY_QUERY.to_owned(),
        )
        .await;

        // if _tai_ksm.0 != 0.0 && _tai_ksm.1.len() > 0 {
        if _tai_ksm.0 != 0.0 {
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
                    "farmType": models::FarmType::StableAmm.to_string(),
                    "farmImpl": models::FarmImplementation::Pallet.to_string(),
                    "asset": {
                        "symbol": "taiKSM".to_string(),
                        "address": "taiKSM".to_string(),
                        "price": 0 as f64,
                        "logos": ["https://raw.githubusercontent.com/yield-bay/assets/main/list/taiKSM.png".to_string()],
                    },
                    "tvl": _tai_ksm.0 as f64,
                    "apr.reward": tai_ksm_reward_apr, // _tai_ksm.2.1 as f64 * 100.0,
                    "apr.base": tai_ksm_base_apr, // _tai_ksm.2.0 as f64 * 100.0,
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
        } else {
            println!("tksmf");
        }
    }

    if taiga_rewards_resp.clone().n3usd.is_some() {
        let _3usd_base_apr = taiga_rewards_resp.clone().n3usd.unwrap().n3usd_fee.apr * 100.0;
        let _3usd_reward_apr = (taiga_rewards_resp.clone().n3usd.unwrap().kar_reward.apr
            + taiga_rewards_resp.clone().n3usd.unwrap().lksm_reward.apr
            + taiga_rewards_resp.clone().n3usd.unwrap().tai_reward.apr
            + taiga_rewards_resp.clone().n3usd.unwrap().taiksm_reward.apr)
            * 100.0;

        let _3usd = fetch_3usd(
            constants::taiga::DAILY_DATA_3_USD_QUERY.to_owned(),
            constants::taiga::TOKEN_PRICE_HISTORY_QUERY.to_owned(),
        )
        .await;

        // if _3usd.0 != 0.0 && _3usd.1.len() > 0 {
        if _3usd.0 != 0.0 {
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
                    "farmType": models::FarmType::StableAmm.to_string(),
                    "farmImpl": models::FarmImplementation::Pallet.to_string(),
                    "asset": {
                        "symbol": "3USD".to_string(),
                        "address": "3USD".to_string(),
                        "price": 0 as f64,
                        "logos": ["https://raw.githubusercontent.com/yield-bay/assets/main/list/3USD.png".to_string()],
                    },
                    "tvl": _3usd.0 as f64,
                    "apr.reward": _3usd_reward_apr, // _3usd.2.1 as f64 * 100.0,
                    "apr.base": _3usd_base_apr, // _3usd.2.0 as f64 * 100.0,
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
        } else {
            println!("3usdf");
        }
    }

    if tapio_rewards_resp.clone().tdot.is_some() {
        let t_dot_base_apr = tapio_rewards_resp.clone().tdot.unwrap().tdot_fee.apr * 100.0;
        let t_dot_reward_apr = tapio_rewards_resp.clone().tdot.unwrap().tdot_yield.apr * 100.0;

        let _t_dot = fetch_t_dot(
            constants::taiga::DAILY_DATA_TAI_KSM_QUERY.to_owned(),
            constants::taiga::TOKEN_PRICE_HISTORY_QUERY.to_owned(),
        )
        .await
        .unwrap();

        let t_dot_rewards: Vec<Bson> = vec![];

        let timestamp = Utc::now().to_string();

        println!("tDOT farm lastUpdatedAtUTC {}", timestamp.clone());

        let t_dot_ff = doc! {
            "id": 0,
            "chef": "tDOT".to_string(),
            "chain": "acala".to_string(),
            "protocol": "tapio".to_string(),
        };
        let t_dot_fu = doc! {
            "$set" : {
                "id": 0,
                "chef": "tDOT".to_string(),
                "chain": "acala".to_string(),
                "protocol": "tapio".to_string(),
                "farmType": models::FarmType::StableAmm.to_string(),
                "farmImpl": models::FarmImplementation::Pallet.to_string(),
                "asset": {
                    "symbol": "tDOT".to_string(),
                    "address": "tDOT".to_string(),
                    "price": 0 as f64,
                    "logos": ["https://raw.githubusercontent.com/yield-bay/assets/main/list/tDOT.png".to_string()],
                },
                "tvl": _t_dot.0 as f64,
                "apr.reward": t_dot_reward_apr, // _3usd.2.1 as f64 * 100.0,
                "apr.base": t_dot_base_apr, // _3usd.2.0 as f64 * 100.0,
                "rewards": t_dot_rewards,
                "allocPoint": 1,
                "lastUpdatedAtUTC": timestamp.clone(),
            }
        };
        let options = FindOneAndUpdateOptions::builder()
            .upsert(Some(true))
            .build();
        farms_collection
            .find_one_and_update(t_dot_ff, t_dot_fu, Some(options))
            .await?;
    }

    Ok(())
}

async fn fetch_3usd(
    taiga_query_str: String,
    karura_dex_query_str: String,
) -> (f64, Vec<(i32, String, f64, String)>, (f64, f64)) {
    let subql_client = Client::new(
        "https://api.subquery.network/sq/nutsfinance/taiga-protocol".to_string(),
        60,
    );
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
        if pool_data.clone().unwrap().daily_data.nodes.len() > 0 {
            tvl = pool_data
                .clone()
                .unwrap()
                .daily_data
                .nodes
                .get(0)
                .unwrap()
                .total_supply;
        }

        apr = fetch_3usd_apr(pool_data.clone().unwrap(), karura_dex_query_str.clone()).await;
    }

    let tai_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "TAI".to_string(),
        1,
    )
    .await;
    let tai_ksm_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "sa://0".to_string(),
        1,
    )
    .await;
    let lksm_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "LKSM".to_string(),
        1,
    )
    .await;
    let kar_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "KAR".to_string(),
        1,
    )
    .await;

    if tai_price_history.len() < 1
        || tai_ksm_price_history.len() < 1
        || lksm_price_history.len() < 1
        || kar_price_history.len() < 1
    {
        return (tvl, vec![], apr);
    }

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

async fn fetch_t_dot(
    tapio_query_str: String,
    acala_dex_query_str: String,
) -> Result<((f64, Vec<(i32, String, f64, String)>, (f64, f64))), Box<dyn std::error::Error>> {
    let subql_client = Client::new(
        "https://api.subquery.network/sq/nutsfinance/tapio-protocol".to_string(),
        60,
    );
    #[derive(Serialize)]
    pub struct Vars {
        days: i64,
    }
    let vars = Vars { days: 30 };
    let pool_data = subql_client
        .query_with_vars_unwrap::<subgraph::TapioDD, Vars>(&tapio_query_str, vars)
        .await;

    let mut current_supply = 0.0;
    let mut tvl = 0.0;
    let mut apr = (0.0, 0.0);
    let mut rewards: Vec<(i32, String, f64, String)> = vec![];

    if pool_data.is_ok() {
        println!(
            "tapio pool_datau {:?}",
            pool_data.clone().unwrap().daily_data.nodes.len()
        );

        // let t_dot_price_history = get_token_price_history(
        //     acala_dex_query_str.clone(),
        //     "tapio".to_string(),
        //     "tDOT".to_string(),
        //     1,
        // )
        // .await;

        // println!("t_dot_price_history {:?}", t_dot_price_history);

        // let mut t_dot_price = 0.0;
        // if t_dot_price_history.len() > 0 {
        //     t_dot_price = t_dot_price_history[0].0;
        // }
        let dot_price = reqwest::get(
            "https://api.coingecko.com/api/v3/simple/price?ids=polkadot&vs_currencies=usd",
        )
        .await?
        .json::<apis::coingecko::Root>()
        .await?;
        println!("DPPP {:?}", dot_price.polkadot.usd);

        let t_dot_price = dot_price.polkadot.usd;

        if pool_data.clone().unwrap().daily_data.nodes.len() > 0 {
            current_supply = pool_data
                .clone()
                .unwrap()
                .daily_data
                .nodes
                .get(0)
                .unwrap()
                .total_supply;
        }
        println!(
            "current_supply {:?}, pdau {:?}, t_dot_price {:?}",
            current_supply,
            pool_data.clone().unwrap(),
            t_dot_price
        );

        tvl = current_supply * t_dot_price;

        // apr = fetch_tai_ksm_apr(pool_data.clone().unwrap(), acala_dex_query_str.clone()).await;
    } else {
        println!("pooldatau notok");
    }

    Ok((tvl, rewards, apr))
}

async fn fetch_tai_ksm(
    taiga_query_str: String,
    karura_dex_query_str: String,
) -> (f64, Vec<(i32, String, f64, String)>, (f64, f64)) {
    let subql_client = Client::new(
        "https://api.subquery.network/sq/nutsfinance/taiga-protocol".to_string(),
        60,
    );
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
        println!(
            "pool_datau {:?}",
            pool_data.clone().unwrap().daily_data.nodes.len()
        );

        let tai_price_history = get_token_price_history(
            karura_dex_query_str.clone(),
            "taiga".to_string(),
            "TAI".to_string(),
            1,
        )
        .await;
        let tai_ksm_price_history = get_token_price_history(
            karura_dex_query_str.clone(),
            "taiga".to_string(),
            "sa://0".to_string(),
            1,
        )
        .await;

        let mut tai_price = 0.0;
        if tai_price_history.len() > 0 && tai_ksm_price_history.len() > 0 {
            tai_price = tai_price_history[0].0;
        }
        let mut tai_ksm_price = 0.0;
        if tai_ksm_price_history.len() > 0 {
            tai_ksm_price = tai_ksm_price_history[0].0;
        }

        if pool_data.clone().unwrap().daily_data.nodes.len() > 0 {
            current_supply = pool_data
                .clone()
                .unwrap()
                .daily_data
                .nodes
                .get(0)
                .unwrap()
                .total_supply;
        }

        tvl = current_supply * tai_ksm_price;

        apr = fetch_tai_ksm_apr(pool_data.clone().unwrap(), karura_dex_query_str.clone()).await;

        if tai_price != 0.0 {
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
    }

    (tvl, rewards, apr)
}

async fn fetch_3usd_apr(pool_data: subgraph::TapioDD, karura_dex_query_str: String) -> (f64, f64) {
    let days = 14;

    let daily_data = pool_data.clone().daily_data.nodes;

    let tai_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "TAI".to_string(),
        days,
    )
    .await;
    let tai_ksm_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "sa://0".to_string(),
        days,
    )
    .await;
    let lksm_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "LKSM".to_string(),
        days,
    )
    .await;
    let kar_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "KAR".to_string(),
        days,
    )
    .await;

    let mut total = 0.0;
    let daily_total_supply = daily_data.clone().into_iter().map(|node| node.total_supply);
    let dts: Vec<f64> = daily_total_supply.clone().collect();

    for i in 0..daily_total_supply.len() {
        if tai_price_history.get(i).is_some() {
            // 8000 TAI per week
            total += (8000.0 * tai_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];
        }
        if tai_ksm_price_history.get(i).is_some() {
            // 30 taiKSM per week
            total += (30.0 * tai_ksm_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];
        }
        if lksm_price_history.get(i).is_some() {
            // 250 LKSM per week
            total += (250.0 * lksm_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];
        }
        if kar_price_history.get(i).is_some() {
            // 2000 KAR per week
            total += (2000.0 * kar_price_history.get(i).unwrap().0 * (365.0 / 7.0)) / dts[i];
        }
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

    let tai_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "TAI".to_string(),
        days,
    )
    .await;
    let tai_ksm_price_history = get_token_price_history(
        karura_dex_query_str.clone(),
        "taiga".to_string(),
        "sa://0".to_string(),
        days,
    )
    .await;

    let mut total = 0.0;
    let daily_total_supply = daily_data.clone().into_iter().map(|node| node.total_supply);
    let dts: Vec<f64> = daily_total_supply.clone().collect();

    for i in 0..daily_total_supply.len() {
        if tai_price_history.get(i).is_some() && tai_ksm_price_history.get(i).is_some() {
            // 4000 TAI each day
            total += (4000.0 * tai_price_history.get(i).unwrap().0 * (365.0))
                / (dts[i] * tai_ksm_price_history.get(i).unwrap().0);
        }
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
    protocol: String,
    asset: String,
    days: i64,
) -> Vec<(f64, String)> {
    let mut subql = "https://dashboard.nuts.finance/api/datasources/proxy/7".to_string();
    if protocol == "tapio".to_string() {
        subql = "https://grafana.acbtc.fi/api/datasources/proxy/11".to_string();
    }
    let subql_client = Client::new(
        // "https://api.subquery.network/sq/AcalaNetwork/karura-dex".to_string(),
        // "https://dashboard.nuts.finance/api/datasources/proxy/7".to_string(),
        subql, 60,
    );
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

    println!(
        "protocol {:?} price_history_data {:?}",
        protocol, price_history_data
    );

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

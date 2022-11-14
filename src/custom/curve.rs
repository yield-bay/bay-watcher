use chrono::prelude::Utc;
use mongodb::{
    bson::{bson, doc},
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};
use std::collections::HashMap;

use crate::apis;
use crate::models;

pub async fn curve_jobs(mongo_uri: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db = client.database(&db_name);

    let farms_collection = db.collection::<models::Farm>("farms");

    let moonbeam_curve_st_dot = "0xc6e37086D09ec2048F151D11CdB9F9BbbdB7d685".to_string();

    let get_factory_apys_resp = reqwest::get("https://api.curve.fi/api/getFactoryAPYs-moonbeam")
        .await?
        .json::<apis::curve::GetFactoryAPYsRoot>()
        .await?;
    println!(
        "get_factory_apys_resp:\n{:#?}",
        get_factory_apys_resp.success
    );

    if get_factory_apys_resp.success {
        for pd in get_factory_apys_resp.clone().data.unwrap().pool_details {
            if pd.pool_address == moonbeam_curve_st_dot.clone() {
                // pd.apy
                // pd.index

                let get_factory_v2_pools_resp =
                    reqwest::get("https://api.curve.fi/api/getFactoryV2Pools-moonbeam")
                        .await?
                        .json::<apis::curve::GetFactoryV2PoolsRoot>()
                        .await?;
                println!(
                    "get_factory_v2_pools_resp:\n{:#?}",
                    get_factory_v2_pools_resp.success
                );

                if get_factory_v2_pools_resp.success {
                    for pda in get_factory_v2_pools_resp.clone().data.unwrap().pool_data {
                        if pda.address == moonbeam_curve_st_dot.clone() {
                            // pda.usd_total

                            let get_facto_gauges_resp =
                                reqwest::get("https://api.curve.fi/api/getFactoGauges/moonbeam")
                                    .await?
                                    .json::<apis::curve::GetFactoGaugesRoot>()
                                    .await?;
                            println!(
                                "get_facto_gauges_resp:\n{:#?}",
                                get_facto_gauges_resp.success
                            );

                            if get_facto_gauges_resp.success {
                                for g in get_facto_gauges_resp.clone().data.unwrap().gauges {
                                    if g.swap_token == moonbeam_curve_st_dot.clone() {
                                        // g.extra_rewards
                                        let ten: f64 = 10.0;

                                        let mut total_apy = 0.0;

                                        let mut rewards = vec![];
                                        // <symbol, (exists, amount, valueUSD, freq)>
                                        let mut reward_asset_map: HashMap<
                                            String,
                                            (bool, f64, f64, String),
                                        > = HashMap::new();
                                        // TODO: check if we need to handle zero case
                                        for er in g.extra_rewards {
                                            let rate = er
                                                .meta_data
                                                .rate
                                                .parse::<f64>()
                                                .unwrap_or_default()
                                                as f64
                                                / ten.powf(
                                                    er.decimals.parse::<f64>().unwrap_or_default(),
                                                )
                                                    as f64;
                                            let amount = rate * 60.0 * 60.0 * 24.0;
                                            rewards.push(bson!({
                                                "amount": amount,
                                                "asset":  er.symbol,
                                                "valueUSD": amount * er.token_price,
                                                "freq": models::Freq::Daily.to_string(),
                                            }));
                                            total_apy += er.apy;
                                        }

                                        let timestamp = Utc::now().to_string();

                                        println!(
                                            "curve v2 farm lastUpdatedAtUTC {}",
                                            timestamp.clone()
                                        );

                                        let ff = doc! {
                                            "id": pd.index as i32,
                                            "chef": "curve v2",
                                            "chain": "moonbeam",
                                            "protocol": "curve",
                                        };
                                        let fu = doc! {
                                            "$set" : {
                                                "id": pd.index,
                                                "chef": "curve v2",
                                                "chain": "moonbeam",
                                                "protocol": "curve",
                                                "farmType": models::FarmType::StableAmm.to_string(),
                                                "farmImpl": models::FarmImplementation::Vyper.to_string(),
                                                "asset": {
                                                    "symbol": "stDOT LP",
                                                    "address": pd.pool_address.clone(),
                                                    "price": 0,
                                                    "logos": [
                                                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/xcDOT.png",
                                                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/stDOT.png",
                                                    ],
                                                },
                                                "tvl": pda.usd_total as f64,
                                                "apr.reward": total_apy,
                                                "apr.base": pd.apy,
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
            }
        }
    }
    Ok(())
}

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

pub async fn demeter_jobs(mongo_uri: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("starting demeter");
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db = client.database(&db_name);

    let farms_collection = db.collection::<models::Farm>("farms");

    let deo_resp = reqwest::get("https://farming-api.cerestoken.io/farming-data")
        .await?
        .json::<apis::demeter::DeoFarms>()
        .await?;

    // println!("deo_resp {:?}", deo_resp);

    for ele in deo_resp {
        let rewards: Vec<Bson> = vec![bson!({
            "amount": ele.reward_token_per_day,
            "asset":  ele.reward_token.clone(),
            "valueUSD": ele.reward_token_per_day * ele.reward_token_price,
            "freq": models::Freq::Daily.to_string(),
        })];

        let mut id_vec = vec![];

        let asset_name = ele.underlying_asset_name.to_string();
        // asset_name.split(" ");
        let f2 = asset_name[0..asset_name.len() - 3].split("-");
        // println!("f2 {:?}", f2.0);
        let mut logos = vec![];
        for a in f2 {
            println!("swsp {:?}", a.to_owned());
            let logo_name = format!(
                "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                a.to_owned()
            );
            println!("logoname:: {logo_name}");

            logos.push(logo_name);

            let char_vec: Vec<char> = a.chars().collect();
            for c in char_vec {
                println!("c: {}, ascii: {}", c, c as u32);
                id_vec.push(c as u32);
            }
        }

        for r in rewards.clone() {
            let s = ele.reward_token.as_str();
            let char_vec: Vec<char> = s.chars().collect();
            for c in char_vec {
                println!("c: {}, ascii: {}", c, c as u32);
                id_vec.push(c as u32);
            }
        }

        // let id = id_vec.iter().fold(0, |acc, elem| acc * 10 + elem);
        let mut id: u128 = 0;
        for elem in id_vec {
            println!("elem {} idnow {}", elem, id);
            id *= 10;
            id += elem as u128;
        }
        println!("id {} asi32 {}", id, id as i32);

        let timestamp = Utc::now().to_string();

        println!("demeter farm lastUpdatedAtUTC {}", timestamp.clone());

        let f = doc! {
            "id": id as i32,
            "chef": "demeterFarmingPlatform".to_string(),
            "chain": "sora".to_string(),
            "protocol": "demeter".to_string(),
        };
        let u = doc! {
            "$set" : {
                "id": id as i32,
                "chef": "demeterFarmingPlatform".to_string(),
                "chain": "sora".to_string(),
                "protocol": "demeter".to_string(),
                "farmType": models::FarmType::StandardAmm.to_string(),
                "farmImpl": models::FarmImplementation::Pallet.to_string(),
                "router": "".to_string(),
                "asset": {
                    "symbol": ele.underlying_asset_name.to_string(),
                    "address": ele.underlying_asset_name.to_string(),
                    "price": 0 as f64,
                    "logos": logos,
                    "underlyingAssets": [],
                },
                "tvl": ele.tvl as f64,
                "apr.reward": ele.apr,
                "apr.base": 0 as f64,
                "rewards": rewards,
                "allocPoint": 1,
                "lastUpdatedAtUTC": timestamp.clone(),
            }
        };
        let options = FindOneAndUpdateOptions::builder()
            .upsert(Some(true))
            .build();
        farms_collection
            .find_one_and_update(f, u, Some(options))
            .await?;
    }

    println!("finished demeter");
    Ok(())
}

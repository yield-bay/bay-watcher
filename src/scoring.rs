use futures::TryStreamExt;
use mongodb::{
    bson::doc,
    options::{ClientOptions, FindOneAndUpdateOptions, FindOptions},
    Client as MongoClient,
};
use serde::{Deserialize, Serialize};

use crate::models;

pub async fn safety_score(mongo_uri: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db = client.database(&db_name);

    let farms_collection = db.collection::<models::Farm>("farms");

    let f = doc! {
        "allocPoint" : {
            "$ne": 0,
        }
    };

    let options = FindOptions::builder().build();

    let mut farms = vec![];

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Farm {
        id: i32,
        asset_addr: String,
        asset: String,
        protocol: String,
        chain: String,
        chef: String,
        farm_type: String,
        tvl: f64,
        base_apr: f64,
        reward_apr: f64,
        rewards_usd: f64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct FarmSafetyScore {
        id: i32,
        asset_addr: String,
        asset: String,
        protocol: String,
        chain: String,
        chef: String,
        farm_type: String,
        tvl: f64,
        base_apr: f64,
        reward_apr: f64,
        rewards_usd: f64,
        total_score: f64,
        tvl_score: f64,
        base_apr_score: f64,
        reward_apr_score: f64,
        rewards_score: f64,
    }

    struct Weightage {
        tvl: f64,
        base_apr: f64,
        reward_apr: f64,
        rewards_usd: f64,
    }

    let mut farms_cursor = farms_collection.find(f, Some(options)).await?;
    while let Some(farm) = farms_cursor.try_next().await? {
        println!(
            "farm: {:?} {:?} {:?}",
            farm.asset.address, farm.protocol, farm.chain
        );
        farms.push(Farm {
            id: farm.id,
            asset_addr: farm.asset.address,
            asset: farm.asset.symbol,
            protocol: farm.protocol,
            chain: farm.chain,
            chef: farm.chef,
            farm_type: farm.farm_type,
            tvl: farm.tvl,
            base_apr: farm.apr.base,
            reward_apr: farm.apr.reward,
            rewards_usd: farm
                .rewards
                .iter()
                .map(|x| {
                    if x.freq == "Weekly" {
                        return x.value_usd / 7.0;
                    } else if x.freq == "Monthly" {
                        return x.value_usd / 30.0;
                    } else if x.freq == "Annually" {
                        return x.value_usd / 365.0;
                    }
                    return x.value_usd;
                })
                .sum(),
        })
    }

    println!("fl {:?}", farms.len());

    fn reward_scores(farms: Vec<Farm>) -> Vec<f64> {
        let mut scores = vec![];
        let rewards: Vec<f64> = farms.iter().map(|x| x.rewards_usd).collect();
        let mut max_reward = 0.0;
        for reward in rewards {
            if reward > max_reward {
                max_reward = reward;
            }
        }

        for farm in farms {
            scores.push(farm.rewards_usd / max_reward)
        }
        scores
    }

    fn reward_apr_scores(farms: Vec<Farm>) -> Vec<f64> {
        let mut scores = vec![];
        let reward_aprs: Vec<f64> = farms.iter().map(|x| x.reward_apr).collect();
        let mut max_apr = 0.0;
        for reward_apr in reward_aprs {
            if reward_apr > max_apr {
                max_apr = reward_apr;
            }
        }

        for farm in farms {
            scores.push(farm.reward_apr / max_apr)
        }
        scores
    }

    fn base_apr_scores(farms: Vec<Farm>) -> Vec<f64> {
        let mut scores = vec![];
        let base_aprs: Vec<f64> = farms.iter().map(|x| x.base_apr).collect();
        let mut max_apr = 0.0;
        for base_apr in base_aprs {
            if base_apr > max_apr {
                max_apr = base_apr;
            }
        }

        for farm in farms {
            if farm.farm_type == "StableAmm" {
                scores.push(0.6)
            } else if farm.farm_type == "SingleStaking" {
                scores.push(0.3)
            }
            // TODO: calculate their base_aprs
            else if farm.asset == "wstKSM-xcKSM LP" || farm.asset == "wstDOT-xcDOT LP" {
                scores.push(0.5)
            } else {
                scores.push(farm.base_apr / max_apr)
            }
        }
        scores
    }

    fn tvl_scores(farms: Vec<Farm>) -> Vec<f64> {
        let mut scores = vec![];
        for farm in farms {
            let mut score = 0.0;
            // > $10M |  1.00
            if farm.tvl >= 10000000.0 {
                score = 1.0
            }
            // $1M - $10M |  0.85
            else if farm.tvl >= 1000000.0 && farm.tvl < 10000000.0 {
                score = 0.85
            }
            // $100K - $1M |  0.75
            else if farm.tvl >= 100000.0 && farm.tvl < 1000000.0 {
                score = 0.75
            }
            // $10K - $100K |  0.6
            else if farm.tvl >= 10000.0 && farm.tvl < 100000.0 {
                score = 0.6
            }
            // $1K - $10K |  0.5
            else if farm.tvl >= 1000.0 && farm.tvl < 10000.0 {
                score = 0.5
            }
            // // < $1K |  0.00
            // else {
            //     score = 0.0
            // }
            scores.push(score)
        }
        scores
    }

    let weightage = Weightage {
        tvl: 0.45,
        base_apr: 0.2,
        reward_apr: 0.15,
        rewards_usd: 0.2,
    };

    let tvl = tvl_scores(farms.clone()); // 45%
    let base_apr = base_apr_scores(farms.clone()); // 20%
    let reward_apr = reward_apr_scores(farms.clone()); // 15%
    let rewards = reward_scores(farms.clone()); // 20%

    let mut safety_scores = vec![];
    let mut min_score = 100.0;
    let mut max_score = -100.0;
    for i in 0..farms.len() {
        let farm = farms[i].clone();
        let total_score = tvl[i] * weightage.tvl
            + base_apr[i] * weightage.base_apr
            + reward_apr[i] * weightage.reward_apr
            + rewards[i] * weightage.rewards_usd;
        safety_scores.push(FarmSafetyScore {
            id: farm.id,
            asset_addr: farm.asset_addr,
            asset: farm.asset,
            protocol: farm.protocol,
            chain: farm.chain,
            chef: farm.chef,
            farm_type: farm.farm_type,
            tvl: farm.tvl,
            base_apr: farm.base_apr,
            reward_apr: farm.reward_apr,
            rewards_usd: farm.rewards_usd,
            total_score: total_score,
            tvl_score: tvl[i],
            base_apr_score: base_apr[i],
            reward_apr_score: reward_apr[i],
            rewards_score: rewards[i],
        });
        if total_score > max_score {
            max_score = total_score;
        }
        if total_score < min_score {
            min_score = total_score;
        }
    }

    println!("safety_scores {:?}", safety_scores.clone());

    for i in 0..safety_scores.len() {
        safety_scores[i].total_score =
            (safety_scores[i].total_score - min_score) / ((max_score - min_score) * 1.01);

        let obj = safety_scores[i].clone();

        let ff = doc! {
            "id": obj.id.clone(),
            "chef": obj.chef.clone(),
            "chain": obj.chain.clone(),
            "protocol": obj.protocol.clone(),
        };
        let fu = doc! {
            "$set" : {
                "id": obj.id.clone(),
                "chef": obj.chef.clone(),
                "chain": obj.chain.clone(),
                "protocol": obj.protocol.clone(),
                "totalScore": obj.total_score.clone(),
                "tvlScore": obj.tvl_score.clone(),
                "baseAPRScore": obj.base_apr_score.clone(),
                "rewardAPRScore": obj.reward_apr_score.clone(),
                "rewardsScore": obj.rewards_score.clone(),
            }
        };
        let options = FindOneAndUpdateOptions::builder()
            .upsert(Some(true))
            .build();
        farms_collection
            .find_one_and_update(ff, fu, Some(options))
            .await?;
    }

    Ok(())
}

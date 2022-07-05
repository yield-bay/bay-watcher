use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::Address,
};
use eyre::Result;
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde::{Deserialize, Serialize};

use std::{convert::TryFrom, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
struct DBPool {
    address: String,
    token0address: String,
    token1address: String,
    chainId: i32,
    feesAPR: f64,
    official: bool,
    community: bool,
    price: f64,
    reserveUSD: f64,
    oneDayVolumeUSD: f64,
    totalSupply: f64,
    token0Logo: String,
    token1Logo: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DBToken {
    name: String,
    symbol: String,
    address: String,
    decimals: i32,
    chainId: i32,
    official: bool,
    community: bool,
    price: f64,
    liquidity: f64,
    logo: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum FarmType {
    StandardAmm,
    StableAmm,
    SingleStaking,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FarmImplementation {
    Solidity,
    Ink,
    Pallet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    name: String,
    address: String,
    symbol: String,
    decimals: i32,
    price: f64,
    logo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    name: String,
    address: String,
    tokens: Vec<Token>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Annually,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reward {
    amount: f64,
    token: Token,
    value_usd: f64,
    freq: Freq,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APR {
    farm: f64,
    trading: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Farm {
    chain: String,
    protocol: String,
    farm_type: FarmType,
    farm_implementation: FarmImplementation,
    asset: Asset,
    id: i32,
    tvl: f64,
    rewards: Vec<Reward>,
    apr: APR,
    url: String,
}

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

#[tokio::main]
async fn main() -> Result<()> {
    println!("\nStart!\n");
    dotenv().ok();

    // Parse a connection string into an options struct.
    let mongo_uri = dotenv::var("DB_CONN_STRING").unwrap();
    println!("mongo_uri: {}", mongo_uri);

    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(mongo_uri).await?;

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None).await? {
        println!("{}", db_name);
    }

    // Get a handle to a database.
    let db = client.database("myFirstDatabase");

    // List the names of the collections in that database.
    for collection_name in db.list_collection_names(None).await? {
        println!("{}", collection_name);
    }

    // Get a handle to a collection in the database.
    let pools_collection = db.collection::<DBPool>("pools");
    let tokens_collection = db.collection::<DBToken>("tokens");

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

    let stella_chef_address = "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388".parse::<Address>()?;
    let stella_chef = IChefV2::new(stella_chef_address, Arc::clone(&moonbeam_client));

    let _chains = vec![
        (moonriver_url, moonriver_client),
        (moonbeam_url, moonbeam_client),
    ];
    let protocols = vec![
        (solarbeam_chef_address, solarbeam_chef),
        (stella_chef_address, stella_chef),
    ];

    // lpToken address, allocPoint uint256, lastRewardTimestamp uint256, accSolarPerShare uint256, depositFeeBP uint16, harvestInterval uint256, totalLp uint256

    for p in protocols {
        let pool_length = p.1.pool_length().call().await?;

        for pid in 0..pool_length.as_u32() {
            println!("pid {}", pid);
            let (
                lp_token,
                alloc_point,
                last_reward_timestamp,
                acc_solar_per_share,
                deposit_fee_bp,
                harvest_interval,
                total_lp,
            ): (Address, _, _, _, _, _, _) =
                p.1.pool_info(ethers::prelude::U256::from(pid))
                    .call()
                    .await?;
            println!(
                "{}, {}, {}, {}, {}, {}, {}",
                lp_token,
                alloc_point,
                last_reward_timestamp,
                acc_solar_per_share,
                deposit_fee_bp,
                harvest_interval,
                total_lp
            );

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

            // TODO: for multi reward farms, calc sum of aprs of all the reward tokens
            if rewards_per_sec.len() > 0 {
                let mut total_farm_apr = 0.0;
                for i in 0..symbols.len() {
                    println!("rwrd[{}]", i);

                    let s = format!("{:?}", symbols[i].clone());
                    println!("s: {}", s);
                    let tcsa = ethers::utils::to_checksum(&addresses[i].to_owned(), None); //1285
                    println!("tcsa: {:?}", tcsa);
                    // let token_filter = doc! { "symbol": s };
                    let token_filter = doc! { "address": tcsa };
                    let token = tokens_collection.find_one(token_filter, None).await?;
                    let token_price = token.unwrap().price;
                    println!("token: {:?}", token_price);

                    // let a = addresses[i].to_string();
                    // println!("a: {} | {}", addresses[i], a);

                    // let a = lp_token.to_owned();
                    // println!(
                    //     "a: {} | {:?} | {:?} | {}",
                    //     a,
                    //     a.to_string(),
                    //     lp_token.to_owned(),
                    //     a.to_string() == "0x069C2065100b4D3D982383f7Ef3EcD1b95C05894"
                    // );
                    let csa = ethers::utils::to_checksum(&lp_token.to_owned(), None); //1285
                    println!("csa: {:?}", csa);

                    let ms = format!("{:?}", lp_token.to_owned());
                    println!(
                        "ms: {} | {}",
                        ms,
                        ms == "0x069c2065100b4d3d982383f7ef3ecd1b95c05894"
                    );
                    let pool_filter = doc! { "address": csa }; // "0xDfEeFA89639125D22ca86E28ce87B164f41AFaE6" };
                    let pool = pools_collection.find_one(pool_filter, None).await?;
                    // println!("pool: {:?}", pool.unwrap());
                    if pool.is_some() {
                        let pool_price = pool.unwrap().price;
                        println!("pool: {:?}", pool_price);

                        // let mut cursor = poolsCollection.find(poolFilter, None).await?;
                        // while let Some(pool) = cursor.try_next().await? {
                        //     println!("price: {:?}", pool.price);
                        // }

                        // TODO: fetch prices from db, fix overflows/typecasting
                        println!("thiss");
                        let solar_price = 1;
                        let spl: U256 = ethers::prelude::U256::from(1);

                        let lp_price = 1;
                        let lpp: U256 = ethers::prelude::U256::from(1);

                        let sepd: u128 = rewards_per_sec[0].as_u128() * 60 * 60 * 24;
                        let ptvl: u128 = total_lp.as_u128();

                        let sepdl: U256 = rewards_per_sec[0];
                        let ptvll: U256 = total_lp;
                        let v = ((sepdl.full_mul(spl)).checked_div(ptvll.full_mul(spl))).unwrap();
                        // .mul(365);

                        println!("v {}", v);

                        // poolAPR
                        println!(
                            "{} {} {} {}",
                            rewards_per_sec[0].as_u128(),
                            total_lp.as_u128(),
                            sepd,
                            ptvl
                        );
                        let farm_apr =
                            ((sepd as f64 * token_price) / (ptvl as f64 * pool_price)) * 365.0;
                        println!("farmAPR: {}", farm_apr);
                        total_farm_apr += farm_apr;

                        // feeAPR
                        // let trading_apr = (lastDayVolume * 0.002 * 365 * 100) / pairLiquidity;
                    } else {
                        // TODO: doesn't work for stable amm pools, veSolar
                        println!("can't find pool");
                    }
                }
                println!("total_farm_apr: {:?}", total_farm_apr);
            }
        }
    }

    Ok(())
}

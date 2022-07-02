use ethers::abi::AbiDecode;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::{types::Address, utils::keccak256};
use ethers_providers::Ws;
use eyre::Result;
use std::convert::TryFrom;
use std::fmt::format;
use std::ops::Mul;
use std::sync::Arc;

use mongodb::{options::ClientOptions, Client};
// This trait is required to use `try_next()` on the cursor
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
};

mod contracts;
use dotenv::dotenv;
#[path = "./utils/addresses.rs"]
mod addresses;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Pool {
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
struct Token {
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

// abigen!(
//     IUniswapV2Pair,
//     r#"[
//         function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
//     ]"#,
// );
abigen!(
    ISolarDistributorV2,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (address, uint256, uint256, uint256, uint16, uint256, uint256)
        function poolTotalLp(uint256) external view returns (uint256)
        function poolRewarders(uint256) external view returns (address [])
        function poolRewardsPerSec(uint256) external view returns (address[], string[], uint256[], uint256[])
    ]"#,
);

#[tokio::main]
// #[cfg(feature = "legacy")]
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
    let poolsCollection = db.collection::<Pool>("pools");
    let tokensCollection = db.collection::<Token>("tokens");

    // let filter = doc! { "address": "0xDfEeFA89639125D22ca86E28ce87B164f41AFaE6" };
    // let filter = doc! { "token0symbol": "WMOVR" };
    // let find_options = FindOptions::builder().sort(doc! { "price": 1.0 }).build();

    // let mut cursor = poolsCollection.find(filter, None).await?;
    // while let Some(pool) = cursor.try_next().await? {
    //     println!("p: {}", pool);
    // }

    // let pool = poolsCollection.find_one(filter, None).await?;
    // println!("pool: {:?}", pool.unwrap());

    let pk = dotenv::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = pk.parse().expect("fail parse");
    // println!("{}", pk);
    // let provider_id = dotenv::var("PROVIDER_ID").unwrap();
    // let url = format!("https://mainnet.infura.io/v3/{}", provider_id);
    // let url = format!("https://rpc.moonriver.moonbeam.network");
    let url = format!("http://127.0.0.1:8545/");

    // connect provider
    let provider_service = Provider::<Http>::try_from(url).expect("failed");

    let provider = SignerMiddleware::new(provider_service, wallet.clone());

    // connect contracts

    let [_bay_vault_factory, solar_distributor] = contracts::get_contracts(&provider);
    println!("contracts connected");

    let vaults = contracts::get_bay_vaults(&provider);

    for v in vaults.clone() {
        let name: String =
            v.0.method::<_, String>("name", ())
                .expect("fail method")
                .call()
                .await
                .expect("fail wait");

        println!(
            "name: {}, vault address: {}, strat address: {}",
            name,
            v.0.address(),
            v.1.address()
        );
    }

    let client = SignerMiddleware::new(provider.clone(), wallet);
    let client = Arc::new(client);

    let address = "0x0329867a8c457e9F75e25b0685011291CD30904F".parse::<Address>()?;
    let chef = ISolarDistributorV2::new(address, Arc::clone(&client));

    // lpToken address, allocPoint uint256, lastRewardTimestamp uint256, accSolarPerShare uint256, depositFeeBP uint16, harvestInterval uint256, totalLp uint256

    let pl = chef.pool_length().call().await?;
    println!("pl: {}", pl.as_u32());

    for pid in 0..pl.as_u32() {
        println!("pid {}", pid);
        let (
            lp_token,
            alloc_point,
            last_reward_timestamp,
            acc_solar_per_share,
            deposit_fee_bp,
            harvest_interval,
            total_lp,
        ): (Address, _, _, _, _, _, _) = chef
            .pool_info(ethers::prelude::U256::from(pid))
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

        // let lp_addr: String = String::from(lp_token);
        // ethers::prelude::H

        let rewarders = chef
            .pool_rewarders(ethers::prelude::U256::from(pid))
            .call()
            .await?;
        println!("rewarders: {:?}", rewarders);

        let (addresses, symbols, decimals, rewards_per_sec) = chef
            .pool_rewards_per_sec(ethers::prelude::U256::from(pid))
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
                let token = tokensCollection.find_one(token_filter, None).await?;
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
                let pool = poolsCollection.find_one(pool_filter, None).await?;
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

    let last_block = client
        .get_block(BlockNumber::Latest)
        .await?
        .unwrap()
        .number
        .unwrap();
    println!("last_block: {}", last_block);

    let strat_harvest_filter =
        Filter::new()
            .from_block(last_block - U64([25; 1]))
            .topic0(ValueOrArray::Value(H256::from(keccak256(
                // "Transfer(address,address,uint256)",
                "StratHarvest(uint256,address,address,uint256,uint256,uint256)",
            ))));

    // let mut stream = client.subscribe_logs(&erc20_transfer_filter).await?;
    // client.get_logs(&erc20_transfer_filter).await?;

    println!("sdaddr: {}", solar_distributor.address());
    // let value = solar_distributor.poolLength().call().await?;
    // println!("v: {}", value);

    let client1: Provider<Ws> =
        Provider::<Ws>::connect("wss://moonriver.api.onfinality.io/public-ws").await?;
    let client1 = Arc::new(client1);

    let mb = client1
        .get_block(BlockNumber::Latest)
        .await?
        .unwrap()
        .number
        .unwrap();
    println!("mb: {}", mb);

    // let s = vaults.clone()[0].2.to_string();
    // let s1 = &*s;
    // let vs =  addresses::vaults() .iter() .map(|v| v.parse::<Address>().unwrap();

    let strat_harvest_filter = Filter::new()
        .from_block(mb - U64([25; 1]))
        // .address(vaults.clone().iter().map(|v| v.2))
        .address(vec!["0xAc4b3DacB91461209Ae9d41EC517c2B9Cb1B7DAF"
            .parse::<Address>()
            .unwrap()])
        .topic0(ValueOrArray::Value(H256::from(keccak256(
            // "Transfer(address,address,uint256)",
            "StratHarvest(uint256,address,address,uint256,uint256,uint256)",
        ))));

    let mut stream = client1.subscribe_logs(&strat_harvest_filter).await?;

    while let Some(log) = stream.next().await {
        println!(
            "block: {:?}, tx: {:?}, token: {:?}, from: {:?}, to: {:?}, amount: {:?}",
            log.block_number,
            log.transaction_hash,
            log.address,
            Address::from(log.topics[1]),
            Address::from(log.topics[2]),
            U256::decode(log.data)
        );
    }

    loop {}

    Ok(())
}

// #[cfg(not(feature = "legacy"))]
// fn main() {}

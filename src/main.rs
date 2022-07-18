use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::Address,
};
use eyre::Result;
use graphql_client::GraphQLQuery;
use mongodb::{
    bson::{bson, doc, Document},
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client,
};
use serde::{Deserialize, Serialize};

use core::time;
use std::{convert::TryFrom, fmt, sync::Arc, thread};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct PDDs;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DBPool {
    address: String,
    token0address: String,
    token1address: String,
    token0symbol: String,
    token1symbol: String,
    token0name: String,
    token1name: String,
    token0decimals: i32,
    token1decimals: i32,
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
    poolType: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum FarmType {
    StandardAmm,
    StableAmm,
    SingleStaking,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum FarmImplementation {
    Solidity,
    Ink,
    Pallet,
}

impl fmt::Display for FarmType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl fmt::Display for FarmImplementation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    name: String,
    address: String,
    symbol: String,
    decimals: i32,
    price: f64,
    logo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    name: String,
    address: String,
    tokens: Vec<Token>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Annually,
}

impl fmt::Display for Freq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reward {
    amount: f64,
    token: Token,
    value_usd: f64,
    freq: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct APR {
    farm: f64,
    trading: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Farm {
    chain: String,
    protocol: String,
    farm_type: String,
    farm_implementation: String,
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
    ]"#
);

#[tokio::main]
async fn main() -> Result<()> {
    println!("\nStart!\n");
    dotenv().ok();

    let delay = time::Duration::from_secs(120);
    loop {
        c().await.unwrap();
        thread::sleep(delay);
    }
}

async fn c() -> Result<()> {
    // Parse a connection string into an options struct.
    let mongo_uri = dotenv::var("DB_CONN_STRING").unwrap();
    println!("mongo_uri: {}", mongo_uri);

    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(mongo_uri).await?;

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // Get a handle to a database.
    let db = client.database("myFirstDatabase");

    let pools_collection = db.collection::<DBPool>("pools");
    let tokens_collection = db.collection::<DBToken>("tokens");

    let farms_collection = db.collection::<Farm>("farms");

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

    // let _chains = vec![
    //     (moonriver_url, moonriver_client),
    //     (moonbeam_url, moonbeam_client),
    // ];
    let protocols = vec![
        (
            stella_chef_address,
            stella_chef,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            moonbeam_url,
            moonbeam_client,
        ),
        (
            solarbeam_chef_address,
            solarbeam_chef,
            "moonriver".to_string(),
            "solarbeam".to_string(),
            moonriver_url,
            moonriver_client,
        ),
    ];

    for p in protocols.clone() {
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

            // TODO: pool_rewarders not returning Vec<H160>
            let rewarders =
                p.1.pool_rewarders(ethers::prelude::U256::from(pid))
                    .call()
                    .await?;
            println!("rewarders: {:#?}", rewarders);

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
                let mut farm_type = FarmType::StandardAmm;
                let farm_implementation = FarmImplementation::Solidity;

                let pool_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                println!("pool_addr: {:?}", pool_addr);
                let pool_addr1 = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                println!("pool_addr1: {:?}", pool_addr1);

                let ms = format!("{:?}", lp_token.to_owned());
                println!("ms: {}", ms,);
                let pool_filter = doc! { "address": pool_addr };
                let pool = pools_collection.find_one(pool_filter, None).await?;

                let mut pool_price: f64 = 0.0;
                let mut pool_tvl: u128 = 0;
                let mut asset: Asset = Asset {
                    name: "".to_string(),
                    address: format!("{:?}", lp_token.to_owned()),
                    tokens: vec![],
                };
                let mut a = doc! {};
                let mut rewards = vec![];

                // If the pool is already stored in our db
                if pool.is_some() {
                    if pool.clone().unwrap().poolType.eq("standardAmm") {
                        println!("pe.standardAmm");
                        let lp = IStandardLpToken::new(lp_token, Arc::clone(&p.5));
                        let name = lp.name().call().await?;
                        println!("name: {:?}", name);

                        for i in 0..symbols.len() {
                            println!("rwrd[{}]", i);

                            let s = format!("{:?}", symbols[i].clone());
                            println!("symbol: {}", s);

                            let token_addr =
                                ethers::utils::to_checksum(&addresses[i].to_owned(), None);
                            println!("token_addr: {:?}", token_addr);

                            let token_filter = doc! { "address": token_addr };
                            let token = tokens_collection.find_one(token_filter, None).await?;
                            let token_price = token.clone().unwrap().price;
                            println!("token_price: {:?}", token_price);

                            pool_price = pool.clone().unwrap().price;
                            println!("pool_price: {:?}", pool_price);

                            let rewards_per_day: u128 = rewards_per_sec[i].as_u128() * 60 * 60 * 24;
                            pool_tvl = total_lp.as_u128();

                            asset = Asset {
                                name: format!(
                                    "{}-{} LP",
                                    pool.clone().unwrap().token0symbol.as_str(),
                                    pool.clone().unwrap().token1symbol.as_str()
                                ),
                                address: format!("{:?}", lp_token.to_owned()),
                                tokens: vec![
                                    Token {
                                        name: pool.clone().unwrap().token0name,
                                        address: pool.clone().unwrap().token0address,
                                        symbol: pool.clone().unwrap().token0symbol,
                                        decimals: pool.clone().unwrap().token0decimals,
                                        price: 0.0,
                                        logo: pool.clone().unwrap().token0Logo,
                                    },
                                    Token {
                                        name: pool.clone().unwrap().token1name,
                                        address: pool.clone().unwrap().token1address,
                                        symbol: pool.clone().unwrap().token1symbol,
                                        decimals: pool.clone().unwrap().token1decimals,
                                        price: 0.0,
                                        logo: pool.clone().unwrap().token1Logo,
                                    },
                                ],
                            };
                            a = doc! {
                                "name": format!(
                                    "{}-{} LP",
                                    pool.clone().unwrap().token0symbol.as_str(),
                                    pool.clone().unwrap().token1symbol.as_str()
                                ),
                                "address": format!("{:?}", lp_token.to_owned()),
                                "tokens": [
                                    {
                                        "name": pool.clone().unwrap().token0name,
                                        "address": pool.clone().unwrap().token0address,
                                        "symbol": pool.clone().unwrap().token0symbol,
                                        "decimals": pool.clone().unwrap().token0decimals,
                                        "price": 0.0,
                                        "logo": pool.clone().unwrap().token0Logo,
                                    }, {
                                        "name": pool.clone().unwrap().token1name,
                                        "address": pool.clone().unwrap().token1address,
                                        "symbol": pool.clone().unwrap().token1symbol,
                                        "decimals": pool.clone().unwrap().token1decimals,
                                        "price": 0.0,
                                        "logo": pool.clone().unwrap().token1Logo,
                                    }
                                ]
                            };

                            let ten: i128 = 10;
                            rewards.push(bson! ({
                                "amount": rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64,
                                "token":  {
                                    "name": token.clone().unwrap().name,
                                    "address": token.clone().unwrap().address,
                                    "symbol": token.clone().unwrap().symbol,
                                    "decimals": token.clone().unwrap().decimals,
                                    "price": token.clone().unwrap().price,
                                    "logo": token.clone().unwrap().logo,
                                },
                                "value_usd": (rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64) * token_price,
                                "freq": Freq::Daily.to_string(),
                            }));

                            // poolAPR
                            println!(
                                "rewards/sec: {} rewards/day: {} pool_tvl: {}",
                                rewards_per_sec[i].as_u128(),
                                rewards_per_day,
                                pool_tvl
                            );
                            let farm_apr = ((rewards_per_day as f64 * token_price)
                                / (pool_tvl as f64 * pool_price))
                                * 365.0
                                * 100.0;
                            println!("farmAPR: {}", farm_apr);
                            total_farm_apr += farm_apr;

                            // feeAPR
                            // let trading_apr = (lastDayVolume * 0.002 * 365 * 100) / pairLiquidity;
                            // let trading_apr = (0.002 * 365.0 * 100.0) / (pool_tvl as f64 * pool_price);
                        }
                    } else if pool.clone().unwrap().poolType.eq("stableAmm") {
                        println!("pe.stableAmm");
                        let lp = IStableLpToken::new(lp_token, Arc::clone(&p.5));
                        let name = lp.name().call().await?;
                        println!("name: {:?}", name);
                        farm_type = FarmType::StableAmm;
                        // TODO: do all the calculations
                    }
                    let ff = doc! {
                        "chain": p.2.clone(),
                        "protocol": p.3.clone(),
                        "id": pid as i32,
                    };
                    // let asset_tokens = asset.tokens.iter().map(|t| {
                    //     "name":t.name.clone()
                    // });
                    // [
                    //                 {
                    //                     "name": asset.tokens[0].name.clone(),
                    //                     "address": asset.tokens[0].address.clone(),
                    //                     "symbol": asset.tokens[0].symbol.clone(),
                    //                     "decimals": asset.tokens[0].decimals,
                    //                     "price": asset.tokens[0].price,
                    //                     "logo": asset.tokens[0].logo.clone(),
                    //                 }, {
                    //                     "name": asset.tokens[1].name.clone(),
                    //                     "address": asset.tokens[1].address.clone(),
                    //                     "symbol": asset.tokens[1].symbol.clone(),
                    //                     "decimals": asset.tokens[1].decimals,
                    //                     "price": asset.tokens[1].price,
                    //                     "logo": asset.tokens[1].logo.clone(),
                    //                 }
                    //             ]
                    let tks = serde_json::to_string(&asset.tokens)?;
                    println!("asset.tokens.exist: {:?}", tks);
                    let ten: f64 = 10.0;
                    let mut uu = doc! {
                        "$set" : {
                            "farm_type": farm_type.to_string(),
                            "farm_implementation": farm_implementation.to_string(),
                            "tvl": pool_tvl as f64 * pool_price / ten.powf(18.0),
                            "asset": {
                                "name": asset.name.clone(),
                                "address": pool_addr1.clone(),
                                "tokens": [
                                    {
                                        "name": asset.tokens[0].name.clone(),
                                        "address": asset.tokens[0].address.clone(),
                                        "symbol": asset.tokens[0].symbol.clone(),
                                        "decimals": asset.tokens[0].decimals,
                                        "price": asset.tokens[0].price,
                                        "logo": asset.tokens[0].logo.clone(),
                                    }, {
                                        "name": asset.tokens[1].name.clone(),
                                        "address": asset.tokens[1].address.clone(),
                                        "symbol": asset.tokens[1].symbol.clone(),
                                        "decimals": asset.tokens[1].decimals,
                                        "price": asset.tokens[1].price,
                                        "logo": asset.tokens[1].logo.clone(),
                                    }
                                ],
                            },
                            "apr": {
                                "farm": total_farm_apr,
                                "trading": 0.0,
                            },
                            "rewards": rewards.clone(),
                            "url": ""
                        }
                    };
                    if asset.tokens.len() == 0 {
                        uu = doc! {
                            "$set" : {
                                "farm_type": farm_type.to_string(),
                                "farm_implementation": farm_implementation.to_string(),
                                "tvl": pool_tvl as f64 * pool_price / ten.powf(18.0),
                                "asset": {
                                    "name": asset.name,
                                    "address": pool_addr1,
                                    "tokens": [],
                                },
                                "apr": {
                                    "farm": total_farm_apr,
                                    "trading": 0.0,
                                },
                                "rewards": rewards,
                                "url": ""
                            }
                        };
                    }
                    println!("uu {:?}", uu);
                    let options = FindOneAndUpdateOptions::builder()
                        .upsert(Some(true))
                        .build();
                    farms_collection
                        .find_one_and_update(ff, uu, Some(options))
                        .await?;
                } else {
                    println!("pne");
                    // either stableAmm or veSolar
                    let lp = IStableLpToken::new(lp_token, Arc::clone(&p.5));
                    let name = lp.name().call().await?;
                    println!("name: {:?}", name);
                    let split_name = name.split(" ");
                    let split_name_vec: Vec<&str> = split_name.collect();
                    if split_name_vec.len() > 1
                        && (split_name_vec[1] == "Stable" || split_name_vec[0] == "StellaSwap")
                    {
                        farm_type = FarmType::StableAmm;
                        let owner_addr = lp.owner().call().await?;
                        let owner = IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.5));
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

                        let mut comb: Vec<(H160, U256)> = vec![];
                        comb = stable_lp_underlying_tokens
                            .clone()
                            .into_iter()
                            .zip(stable_lp_underlying_balances.clone().into_iter())
                            .collect();
                        // .iter()
                        // .flat_map(|&k| {
                        //     stable_lp_underlying_balances
                        //         .clone()
                        //         .iter()
                        //         .map(move |&v| (k, v))
                        //         .into_iter()
                        //         .collect::<Vec<_>>()
                        // })
                        // .collect();
                        let mut comb_na = true;
                        println!("comb0: {:?}, comb: {:?}", comb[0], comb);
                        for c in comb.clone() {
                            println!("c0: {:?}, c1: {:?}", c.0.to_owned(), c.1.to_owned());
                            let ut_addr = ethers::utils::to_checksum(&c.0.to_owned(), None);
                            println!("ut_addr: {:?}", ut_addr);
                            let token_filter = doc! { "address": ut_addr };
                            let token = tokens_collection.find_one(token_filter, None).await?;
                            if token.is_some() {
                                println!("uttoken {:#?}", token.clone().unwrap());
                                println!(
                                    "utprice {:?}, utbal {:?}",
                                    token.clone().unwrap().price,
                                    c.1.to_owned()
                                );
                            } else {
                                println!("uttoken.na");
                                comb_na = false;
                            }
                        }

                        if comb_na == true {
                            for i in 0..symbols.len() {
                                println!("rwrd[{}]", i);

                                let s = format!("{:?}", symbols[i].clone());
                                println!("symbol: {}", s);

                                let token_addr =
                                    ethers::utils::to_checksum(&addresses[i].to_owned(), None);
                                println!("token_addr: {:?}", token_addr);

                                let token_filter = doc! { "address": token_addr };
                                let token = tokens_collection.find_one(token_filter, None).await?;
                                let token_price = token.clone().unwrap().price;
                                println!("token_price: {:?}", token_price);

                                // pool_price = pool.clone().unwrap().price;
                                // println!("pool_price: {:?}", pool_price);

                                let rewards_per_day: u128 =
                                    rewards_per_sec[i].as_u128() * 60 * 60 * 24;
                                pool_tvl = total_lp.as_u128();

                                // a = doc! {
                                //     "name": format!(
                                //         "{}-{} LP",
                                //         pool.clone().unwrap().token0symbol.as_str(),
                                //         pool.clone().unwrap().token1symbol.as_str()
                                //     ),
                                //     "address": format!("{:?}", lp_token.to_owned()),
                                //     "tokens": []
                                // };

                                let ten: i128 = 10;
                                rewards.push(bson! ({
                                "amount": rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64,
                                "token":  {
                                    "name": token.clone().unwrap().name,
                                    "address": token.clone().unwrap().address,
                                    "symbol": token.clone().unwrap().symbol,
                                    "decimals": token.clone().unwrap().decimals,
                                    "price": token.clone().unwrap().price,
                                    "logo": token.clone().unwrap().logo,
                                },
                                "value_usd": (rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64) * token_price,
                                "freq": Freq::Daily.to_string(),
                            }));

                                // poolAPR
                                // println!(
                                //     "rewards/sec: {} rewards/day: {} pool_tvl: {}",
                                //     rewards_per_sec[i].as_u128(),
                                //     rewards_per_day,
                                //     pool_tvl
                                // );
                                // let farm_apr = ((rewards_per_day as f64 * token_price)
                                //     / (pool_tvl as f64 * pool_price))
                                //     * 365.0
                                //     * 100.0;
                                // println!("farmAPR: {}", farm_apr);
                                // total_farm_apr += farm_apr;

                                // feeAPR
                                // let trading_apr = (lastDayVolume * 0.002 * 365 * 100) / pairLiquidity;
                                // let trading_apr = (0.002 * 365.0 * 100.0) / (pool_tvl as f64 * pool_price);
                            }
                        }
                        let stable_lp_symbol = lp.symbol().call().await?;
                        println!("stable_lp_symbol {:?}", stable_lp_symbol.clone());
                        a = doc! {
                            // "name": format!(
                            //     "{}-{} LP",
                            //     pool.clone().unwrap().token0symbol.as_str(),
                            //     pool.clone().unwrap().token1symbol.as_str()
                            // ),
                            "name": stable_lp_symbol.clone(),
                            "address": format!("{:?}", lp_token.to_owned()),
                            "tokens": []
                        };
                        // TODO: find prices

                        let ff = doc! {
                            "chain": p.2.clone(),
                            "protocol": p.3.clone(),
                            "id": pid as i32,
                        };

                        let tks = serde_json::to_string(&asset.tokens)?;
                        println!("asset.tokens.dne: {:?}", tks);
                        let ten: f64 = 10.0;
                        // farm_type = FarmType::StableAmm;
                        let mut uu = doc! {
                            "$set" : {
                                "farm_type": farm_type.to_string(),
                                "farm_implementation": farm_implementation.to_string(),
                                "tvl":0.0, // pool_tvl as f64 * pool_price / ten.powf(18.0),
                                "asset": a,
                                "apr": {
                                    "farm": 0.0,
                                    "trading": 0.0,
                                },
                                "rewards": rewards.clone(),
                                "url": ""
                            }
                        };
                        // if asset.tokens.len() == 0 {
                        //     uu = doc! {
                        //         "$set" : {
                        //             "farm_type": farm_type.to_string(),
                        //             "farm_implementation": farm_implementation.to_string(),
                        //             "tvl": pool_tvl as f64 * pool_price / ten.powf(18.0),
                        //             "asset": {
                        //                 "name": asset.name,
                        //                 "address": pool_addr1,
                        //                 "tokens": [],
                        //             },
                        //             "apr": {
                        //                 "farm": total_farm_apr,
                        //                 "trading": 0.0,
                        //             },
                        //             "rewards": rewards,
                        //             "url": ""
                        //         }
                        //     };
                        // }
                        println!("uune {:?}", uu);
                        let options = FindOneAndUpdateOptions::builder()
                            .upsert(Some(true))
                            .build();
                        farms_collection
                            .find_one_and_update(ff, uu, Some(options))
                            .await?;
                    } else if split_name_vec.len() > 0 && split_name_vec[0] == "Vested" {
                        println!("Vested");
                    }
                    // TODO:combine moonriver and moonbeam below
                    // if p.2.to_owned() == "moonriver" {
                    // } else if p.2.to_owned() == "moonbeam" {
                    //     // let lp_token_address = lp_token.parse::<Address>()?;
                    //     // assume stable lp token
                    //     let lp = IStableLpToken::new(lp_token, Arc::clone(&p.5));
                    //     let name = lp.name().call().await?;
                    //     let split_name = name.split(" ");
                    //     let split_name_vec: Vec<&str> = split_name.collect();
                    //     if split_name_vec[1] == "Stable" {
                    //     } else if split_name_vec[0] == "Vested" {
                    //     }
                    //     // if split_name_vec.get(1) == Some("Stable".to_string()) {}
                    // }
                }

                // if pool.is_some() {
                // for i in 0..symbols.len() {
                //     println!("rwrd[{}]", i);

                //     let s = format!("{:?}", symbols[i].clone());
                //     println!("symbol: {}", s);

                //     let token_addr = ethers::utils::to_checksum(&addresses[i].to_owned(), None);
                //     println!("token_addr: {:?}", token_addr);

                //     let token_filter = doc! { "address": token_addr };
                //     let token = tokens_collection.find_one(token_filter, None).await?;
                //     let token_price = token.clone().unwrap().price;
                //     println!("token_price: {:?}", token_price);

                //     pool_price = pool.clone().unwrap().price;
                //     println!("pool_price: {:?}", pool_price);

                //     let rewards_per_day: u128 = rewards_per_sec[i].as_u128() * 60 * 60 * 24;
                //     pool_tvl = total_lp.as_u128();

                //     asset = Asset {
                //         name: format!(
                //             "{}-{} LP",
                //             pool.clone().unwrap().token0symbol.as_str(),
                //             pool.clone().unwrap().token1symbol.as_str()
                //         ),
                //         address: format!("{:?}", lp_token.to_owned()),
                //         tokens: vec![
                //             Token {
                //                 name: pool.clone().unwrap().token0name,
                //                 address: pool.clone().unwrap().token0address,
                //                 symbol: pool.clone().unwrap().token0symbol,
                //                 decimals: pool.clone().unwrap().token0decimals,
                //                 price: 0.0,
                //                 logo: pool.clone().unwrap().token0Logo,
                //             },
                //             Token {
                //                 name: pool.clone().unwrap().token1name,
                //                 address: pool.clone().unwrap().token1address,
                //                 symbol: pool.clone().unwrap().token1symbol,
                //                 decimals: pool.clone().unwrap().token1decimals,
                //                 price: 0.0,
                //                 logo: pool.clone().unwrap().token1Logo,
                //             },
                //         ],
                //     };

                //     let ten: i128 = 10;
                //     rewards.push(bson! ({
                //         "amount": rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64,
                //         "token":  {
                //             "name": token.clone().unwrap().name,
                //             "address": token.clone().unwrap().address,
                //             "symbol": token.clone().unwrap().symbol,
                //             "decimals": token.clone().unwrap().decimals,
                //             "price": token.clone().unwrap().price,
                //             "logo": token.clone().unwrap().logo,
                //         },
                //         "value_usd": (rewards_per_day as f64 / ten.pow(decimals[i].as_u128().try_into().unwrap()) as f64) * token_price,
                //         "freq": Freq::Daily.to_string(),
                //     }));

                //     // poolAPR
                //     println!(
                //         "rewards/sec: {} rewards/day: {} pool_tvl: {}",
                //         rewards_per_sec[i].as_u128(),
                //         rewards_per_day,
                //         pool_tvl
                //     );
                //     let farm_apr = ((rewards_per_day as f64 * token_price)
                //         / (pool_tvl as f64 * pool_price))
                //         * 365.0
                //         * 100.0;
                //     println!("farmAPR: {}", farm_apr);
                //     total_farm_apr += farm_apr;

                //     // feeAPR
                //     // let trading_apr = (lastDayVolume * 0.002 * 365 * 100) / pairLiquidity;
                //     // let trading_apr = (0.002 * 365.0 * 100.0) / (pool_tvl as f64 * pool_price);
                // }

                // let ff = doc! {
                //     "chain": p.2.clone(),
                //     "protocol": p.3.clone(),
                //     "id": pid as i32,
                // };
                // let ten: f64 = 10.0;
                // let uu = doc! {
                //     "$set" : {
                //         "farm_type": farm_type.to_string(),
                //         "farm_implementation": farm_implementation.to_string(),
                //         "tvl": pool_tvl as f64 * pool_price / ten.powf(18.0),
                //         "asset": {
                //             "name": asset.name,
                //             "address": pool_addr1,
                //             "tokens": [
                //                 {
                //                     "name": asset.tokens[0].name.clone(),
                //                     "address": asset.tokens[0].address.clone(),
                //                     "symbol": asset.tokens[0].symbol.clone(),
                //                     "decimals": asset.tokens[0].decimals,
                //                     "price": asset.tokens[0].price,
                //                     "logo": asset.tokens[0].logo.clone(),
                //                 }, {
                //                     "name": asset.tokens[1].name.clone(),
                //                     "address": asset.tokens[1].address.clone(),
                //                     "symbol": asset.tokens[1].symbol.clone(),
                //                     "decimals": asset.tokens[1].decimals,
                //                     "price": asset.tokens[1].price,
                //                     "logo": asset.tokens[1].logo.clone(),
                //                 }
                //             ],
                //         },
                //         "apr": {
                //             "farm": total_farm_apr,
                //             // "trading": 0.0,
                //         },
                //         "rewards": rewards,
                //         "url": ""
                //     }
                // };
                // let options = FindOneAndUpdateOptions::builder()
                //     .upsert(Some(true))
                //     .build();
                // farms_collection
                //     .find_one_and_update(
                //         ff, // doc! {"$set":{}},
                //         uu, // doc! {upsert:true},
                //         Some(options),
                //     )
                //     .await?;
                // } else {
                //     // TODO: doesn't work for stable amm pools, veSolar
                //     farm_type = FarmType::StableAmm;
                //     println!("can't find pool. farm_type: {:?}", farm_type.to_string());
                // }
                // */
                // for i in 0..symbols.len() {
                //     println!("rwrd[{}]", i);

                //     let s = format!("{:?}", symbols[i].clone());
                //     println!("symbol: {}", s);

                // let token_addr = ethers::utils::to_checksum(&addresses[i].to_owned(), None);
                // println!("token_addr: {:?}", token_addr);

                // let token_filter = doc! { "address": token_addr };
                // let token = tokens_collection.find_one(token_filter, None).await?;
                // let token_price = token.unwrap().price;
                // println!("token_price: {:?}", token_price);

                // let pool_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                // println!("pool_addr: {:?}", pool_addr);

                // let ms = format!("{:?}", lp_token.to_owned());
                // println!("ms: {}", ms,);
                // let pool_filter = doc! { "address": pool_addr };
                // let pool = pools_collection.find_one(pool_filter, None).await?;

                // if pool.is_some() {
                //     let pool_price = pool.unwrap().price;
                //     println!("pool_price: {:?}", pool_price);

                //     let rewards_per_day: u128 = rewards_per_sec[i].as_u128() * 60 * 60 * 24;
                //     let pool_tvl: u128 = total_lp.as_u128();

                //     // poolAPR
                //     println!(
                //         "rewards/sec: {} rewards/day: {} pool_tvl: {}",
                //         rewards_per_sec[i].as_u128(),
                //         rewards_per_day,
                //         pool_tvl
                //     );
                //     let farm_apr = ((rewards_per_day as f64 * token_price)
                //         / (pool_tvl as f64 * pool_price))
                //         * 365.0
                //         * 100.0;
                //     println!("farmAPR: {}", farm_apr);
                //     total_farm_apr += farm_apr;

                //     // feeAPR
                //     // let trading_apr = (lastDayVolume * 0.002 * 365 * 100) / pairLiquidity;
                //     // let trading_apr = (0.002 * 365.0 * 100.0) / (pool_tvl as f64 * pool_price);
                // } else {
                //     // TODO: doesn't work for stable amm pools, veSolar
                //     println!("can't find pool");
                // }
                // }
                println!("total_farm_apr: {:?}", total_farm_apr);
            }
        }
    }
    Ok(())
}

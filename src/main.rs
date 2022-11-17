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
use gql_client::Client;
use mongodb::{
    bson::{bson, doc, Bson},
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};
use serde::Serialize;

mod apis;
mod constants;
mod custom;
mod models;
mod scoring;
mod subgraph;
mod subsquid;

abigen!(
    IChefV2,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (address, uint256, uint256, uint256, uint16, uint256, uint256)
        function poolTotalLp(uint256) external view returns (uint256)
        function poolRewarders(uint256) external view returns (address [])
        function poolRewardsPerSec(uint256) external view returns (address[], string[], uint256[], uint256[])
        function stellaPerSec() external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
    ]"#,
);

abigen!(
    IArthswapChef,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfos(uint256) external view returns (uint128, uint64, uint64)
        function ARSWPerBlock(uint256) external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
        function lpTokens(uint256) external view returns (address)
        function getPeriod(uint256) external view returns (uint256)
    ]"#,
);

abigen!(
    IFarming,
    r#"[
        function poolLength() external view returns (uint256)
        function getPoolInfo(uint256) external view returns (address, uint256, address[], uint256[], uint256[], uint256, uint256, uint256)
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
    IMiniChefV2,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (uint128, uint64, uint64)
        function sushiPerSecond() external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
        function lpToken(uint256) external view returns (address)
    ]"#,
);

abigen!(
    IComplexRewarderTime,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (uint128, uint64, uint64)
        function rewardPerSecond() external view returns (uint256)
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
        function totalSupply() external view returns (uint256)
        function balanceOf(address) external view returns (uint256)
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
    ]"#,
);

abigen!(
    IAnyswapV5ERC20,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
        function balanceOf(address) external view returns (uint256)
    ]"#,
);

abigen!(
    ILpToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
        function balanceOf(address) external view returns (uint256)
        function token0() external view returns (address)
        function token1() external view returns (address)
        function getReserves() external view returns (uint112, uint112, uint32)
        function totalSupply() external view returns (uint256)
    ]"#,
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let delay = time::Duration::from_secs(60 * 3);
    loop {
        run_jobs().await.unwrap();
        thread::sleep(delay);
    }
}

async fn run_jobs() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Parse a connection string into an options struct.
    let mongo_uri = dotenv::var("DB_CONN_STRING").unwrap();
    println!("mongo_uri: {}", mongo_uri.clone());

    let mut headers = HashMap::new();
    headers.insert("content-type", "application/json");

    println!("------------------------------\ncurve_jobs");
    custom::curve::curve_jobs(mongo_uri.clone()).await.unwrap();

    println!("------------------------------\ntapio_taiga_jobs");
    custom::tapio_taiga::tapio_taiga_jobs(mongo_uri.clone())
        .await
        .unwrap();

    let solarbeam_subgraph_client = Client::new_with_headers(
        constants::subgraph_urls::SOLARBEAM_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );
    let solarflare_subgraph_client = Client::new_with_headers(
        constants::subgraph_urls::SOLARFLARE_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );
    let stellaswap_subgraph_client = Client::new_with_headers(
        constants::subgraph_urls::STELLASWAP_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );
    let beamswap_subgraph_client = Client::new_with_headers(
        constants::subgraph_urls::BEAMSWAP_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );
    let sushi_subgraph_client = Client::new_with_headers(
        constants::subgraph_urls::SUSHI_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );
    let zenlink_astar_subsquid_client = Client::new_with_headers(
        constants::subgraph_urls::ZENLINK_ASTAR_SUBSQUID.clone(),
        60,
        headers.clone(),
    );
    let zenlink_moonriver_subsquid_client = Client::new_with_headers(
        constants::subgraph_urls::ZENLINK_MOONRIVER_SUBSQUID.clone(),
        60,
        headers.clone(),
    );
    let zenlink_moonbeam_subsquid_client = Client::new_with_headers(
        constants::subgraph_urls::ZENLINK_MOONBEAM_SUBSQUID.clone(),
        60,
        headers.clone(),
    );

    let _moonriver_blocklytics_client = Client::new_with_headers(
        constants::subgraph_urls::SOLARBEAM_BLOCKLYTICS_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );
    let _moonbeam_blocklytics_client = Client::new_with_headers(
        constants::subgraph_urls::SOLARFLARE_BLOCKLYTICS_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );

    // subgraph fetching jobs

    let protocols = vec![
        (
            "zenlink",
            "moonbeam",
            zenlink_moonbeam_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_MOONBEAM_SUBSQUID.clone(),
        ),
        (
            "solarflare",
            "moonbeam",
            solarflare_subgraph_client.clone(),
            constants::subgraph_urls::SOLARFLARE_SUBGRAPH.clone(),
        ),
        (
            "zenlink",
            "moonriver",
            zenlink_moonriver_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_MOONRIVER_SUBSQUID.clone(),
        ),
        (
            "sushiswap",
            "moonriver",
            sushi_subgraph_client.clone(),
            constants::subgraph_urls::SUSHI_SUBGRAPH.clone(),
        ),
        (
            "stellaswap",
            "moonbeam",
            stellaswap_subgraph_client.clone(),
            constants::subgraph_urls::STELLASWAP_SUBGRAPH.clone(),
        ),
        (
            "solarbeam",
            "moonriver",
            solarbeam_subgraph_client.clone(),
            constants::subgraph_urls::SOLARBEAM_SUBGRAPH.clone(),
        ),
        (
            "beamswap",
            "moonbeam",
            beamswap_subgraph_client.clone(),
            constants::subgraph_urls::BEAMSWAP_SUBGRAPH.clone(),
        ),
        (
            "zenlink",
            "astar",
            zenlink_astar_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_ASTAR_SUBSQUID.clone(),
        ),
    ];

    println!("------------------------------\nsubgraph_jobs");
    subgraph_jobs(mongo_uri.clone(), protocols, headers.clone())
        .await
        .unwrap();

    // smart contract fetching jobs

    println!("------------------------------\nchef_contract_jobs");
    chef_contract_jobs(
        mongo_uri.clone(),
        sushi_subgraph_client.clone(),
        beamswap_subgraph_client.clone(),
        stellaswap_subgraph_client.clone(),
        solarbeam_subgraph_client.clone(),
        zenlink_astar_subsquid_client.clone(),
        zenlink_moonriver_subsquid_client.clone(),
        zenlink_moonbeam_subsquid_client.clone(),
        solarflare_subgraph_client.clone(),
    )
    .await
    .unwrap();

    scoring::safety_score(mongo_uri.clone()).await.unwrap();

    Ok(())
}

async fn chef_contract_jobs(
    mongo_uri: String,
    sushi_subgraph_client: Client,
    beamswap_subgraph_client: Client,
    stellaswap_subgraph_client: Client,
    solarbeam_subgraph_client: Client,
    zenlink_astar_subsquid_client: Client,
    zenlink_moonriver_subsquid_client: Client,
    zenlink_moonbeam_subsquid_client: Client,
    solarflare_subgraph_client: Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db = client.database(&db_name);

    let assets_collection = db.collection::<models::Asset>("assets");
    let farms_collection = db.collection::<models::Farm>("farms");

    let pk = dotenv::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = pk.parse().expect("fail parse");

    let moonriver_url = dotenv::var("MOONRIVER_URL").unwrap();
    let moonbeam_url = dotenv::var("MOONBEAM_URL").unwrap();
    let astar_url = dotenv::var("ASTAR_URL").unwrap();

    let moonriver_provider_service =
        Provider::<Http>::try_from(moonriver_url.clone()).expect("failed");
    let moonriver_provider = SignerMiddleware::new(moonriver_provider_service, wallet.clone());

    let moonbeam_provider_service =
        Provider::<Http>::try_from(moonbeam_url.clone()).expect("failed");
    let moonbeam_provider = SignerMiddleware::new(moonbeam_provider_service, wallet.clone());

    let astar_provider_service = Provider::<Http>::try_from(astar_url.clone()).expect("failed");
    let astar_provider = SignerMiddleware::new(astar_provider_service, wallet.clone());

    let moonriver_client = SignerMiddleware::new(moonriver_provider.clone(), wallet.clone());
    let moonriver_client = Arc::new(moonriver_client);

    let astar_client = SignerMiddleware::new(astar_provider.clone(), wallet.clone());
    let astar_client = Arc::new(astar_client);

    let moonbeam_client = SignerMiddleware::new(moonbeam_provider.clone(), wallet.clone());
    let moonbeam_client = Arc::new(moonbeam_client);

    let solarbeam_chef_address = "0x0329867a8c457e9F75e25b0685011291CD30904F".parse::<Address>()?;
    let solarbeam_chef = IChefV2::new(solarbeam_chef_address, Arc::clone(&moonriver_client));

    let solarflare_chef_address =
        "0x995da7dfB96B4dd1e2bd954bE384A1e66cBB4b8c".parse::<Address>()?;
    let solarflare_chef = IChefV2::new(solarflare_chef_address, Arc::clone(&moonbeam_client));

    let stella_chef_v1_address = "0xEDFB330F5FA216C9D2039B99C8cE9dA85Ea91c1E".parse::<Address>()?;
    let stella_chef_v1 = IChefV2::new(stella_chef_v1_address, Arc::clone(&moonbeam_client));

    let stella_chef_v2_address = "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388".parse::<Address>()?;
    let stella_chef_v2 = IChefV2::new(stella_chef_v2_address, Arc::clone(&moonbeam_client));

    let beam_chef_address = "0xC6ca172FC8BDB803c5e12731109744fb0200587b".parse::<Address>()?;
    let beam_chef = IChefV2::new(beam_chef_address, Arc::clone(&moonbeam_client));

    let sushi_mini_chef_address =
        "0x3dB01570D97631f69bbb0ba39796865456Cf89A5".parse::<Address>()?;
    let sushi_mini_chef = IChefV2::new(sushi_mini_chef_address, Arc::clone(&moonriver_client));

    let zenlink_astar_chef_address =
        "0x460ee9DBc82B2Be84ADE50629dDB09f6A1746545".parse::<Address>()?;
    let zenlink_astar_chef = IChefV2::new(zenlink_astar_chef_address, Arc::clone(&astar_client));

    let zenlink_moonriver_chef_address =
        "0xf4Ec122d32F2117674Ce127b72c40506c52A72F8".parse::<Address>()?;
    let zenlink_moonriver_chef = IChefV2::new(
        zenlink_moonriver_chef_address,
        Arc::clone(&moonriver_client),
    );

    let zenlink_moonbeam_chef_address =
        "0xD6708344553cd975189cf45AAe2AB3cd749661f4".parse::<Address>()?;
    let zenlink_moonbeam_chef =
        IChefV2::new(zenlink_moonbeam_chef_address, Arc::clone(&moonbeam_client));

    let arthswap_astar_chef_address =
        "0xc5b016c5597D298Fe9eD22922CE290A048aA5B75".parse::<Address>()?;
    let arthswap_astar_chef = IChefV2::new(arthswap_astar_chef_address, Arc::clone(&astar_client));

    let wglmr_poop_stellaswap_resp = reqwest::get("https://app.geckoterminal.com/api/p1/glmr/pools/0x4efb208eeeb5a8c85af70e8fbc43d6806b422bec")
        .await?
        .json::<apis::geckoterminal::Root>()
        .await?;

    let stella_poop_price: f64 = wglmr_poop_stellaswap_resp
        .clone()
        .data
        .attributes
        .price_in_usd
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    let wglmr_poop_beamswap_resp = reqwest::get("https://app.geckoterminal.com/api/p1/glmr/pools/0xa049a6260921B5ee3183cFB943133d36d7FdB668")
        .await?
        .json::<apis::geckoterminal::Root>()
        .await?;

    let beam_poop_price: f64 = wglmr_poop_beamswap_resp
        .clone()
        .data
        .attributes
        .price_in_usd
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    let f = doc! {
        "address": "0xFFfffFFecB45aFD30a637967995394Cc88C0c194",
        "chain": "moonbeam",
        "protocol": "stellaswap",
    };

    let timestamp = Utc::now().to_string();

    let poop_logo = format!(
        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
        "POOP"
    );
    let u = doc! {
        "$set" : {
            "address": "0xFFfffFFecB45aFD30a637967995394Cc88C0c194",
            "chain": "moonbeam",
            "protocol": "stellaswap",
            "name": "Raresama POOP",
            "symbol": "POOP",
            "decimals": 18,
            "logos": [
                poop_logo.clone(),
            ],
            "price": stella_poop_price,
            "liquidity": 1.0,
            "totalSupply": 1.0,
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

    let f = doc! {
        "address": "0xFFfffFFecB45aFD30a637967995394Cc88C0c194",
        "chain": "moonbeam",
        "protocol": "beamswap",
    };

    let timestamp = Utc::now().to_string();

    let poop_logo = format!(
        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
        "POOP"
    );
    let u = doc! {
        "$set" : {
            "address": "0xFFfffFFecB45aFD30a637967995394Cc88C0c194",
            "chain": "moonbeam",
            "protocol": "beamswap",
            "name": "Raresama POOP",
            "symbol": "POOP",
            "decimals": 18,
            "logos": [
                poop_logo.clone(),
            ],
            "price": beam_poop_price,
            "liquidity": 1.0,
            "totalSupply": 1.0,
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

    let wglmr_poop_stellaswap_address =
        "0x4EfB208eeEb5A8C85af70e8FBC43D6806b422bec".parse::<Address>()?;
    let wglmr_poop_stellaswap_lp =
        ILpToken::new(wglmr_poop_stellaswap_address, Arc::clone(&moonbeam_client));

    let wglmr_poop_beamswap_address =
        "0xa049a6260921B5ee3183cFB943133d36d7FdB668".parse::<Address>()?;
    let wglmr_poop_beamswap_lp =
        ILpToken::new(wglmr_poop_beamswap_address, Arc::clone(&moonbeam_client));

    let (stellaswap_r0, stellaswap_r1, _): (u128, u128, u32) =
        wglmr_poop_stellaswap_lp.get_reserves().call().await?;
    let (beamswap_r0, beamswap_r1, _): (u128, u128, u32) =
        wglmr_poop_beamswap_lp.get_reserves().call().await?;

    let wglmr_poop_stellaswap_lp_ts: U256 = wglmr_poop_stellaswap_lp.total_supply().call().await?;
    let wglmr_poop_beamswap_lp_ts: U256 = wglmr_poop_beamswap_lp.total_supply().call().await?;

    // let poop_price = 0.059;

    let poop_addr = "0xFFfffFFecB45aFD30a637967995394Cc88C0c194";
    let wglmr_addr = "0xAcc15dC74880C9944775448304B263D191c6077F";

    let wglmr_stellaswap_filter =
        doc! {"chain":"moonbeam", "protocol":"stellaswap", "address": wglmr_addr};
    let wglmr_stellaswap_asset = assets_collection
        .find_one(wglmr_stellaswap_filter, None)
        .await?;
    let poop_stellaswap_filter =
        doc! {"chain":"moonbeam", "protocol":"stellaswap", "address": poop_addr};
    let poop_stellaswap_asset = assets_collection
        .find_one(poop_stellaswap_filter, None)
        .await?;

    let wglmr_beamswap_filter =
        doc! {"chain":"moonbeam", "protocol":"beamswap", "address": wglmr_addr};
    let wglmr_beamswap_asset = assets_collection
        .find_one(wglmr_beamswap_filter, None)
        .await?;
    let poop_beamswap_filter =
        doc! {"chain":"moonbeam", "protocol":"beamswap", "address": poop_addr};
    let poop_beamswap_asset = assets_collection
        .find_one(poop_beamswap_filter, None)
        .await?;

    let ten: f64 = 10.0;

    let stellaswap_wglmr_poop_liq = wglmr_stellaswap_asset.clone().unwrap().price
        * stellaswap_r0 as f64
        + poop_stellaswap_asset.clone().unwrap().price * stellaswap_r1 as f64;
    let beamswap_wglmr_poop_liq = wglmr_beamswap_asset.clone().unwrap().price * beamswap_r0 as f64
        + poop_beamswap_asset.clone().unwrap().price * beamswap_r1 as f64;

    println!(
        "stellaswap_wglmr_poop_liq {:?} wglmr_poop_stellaswap_lp_ts {:?} lpprice {:}",
        stellaswap_wglmr_poop_liq / ten.powf(18.0),
        wglmr_poop_stellaswap_lp_ts.as_u128() as f64 / ten.powf(18.0),
        stellaswap_wglmr_poop_liq / wglmr_poop_stellaswap_lp_ts.as_u128() as f64
    );

    println!(
        "beamswap_wglmr_poop_liq {:?} wglmr_poop_beamswap_lp_ts {:?} lpprice {:}",
        beamswap_wglmr_poop_liq / ten.powf(18.0),
        wglmr_poop_beamswap_lp_ts.as_u128() as f64 / ten.powf(18.0),
        beamswap_wglmr_poop_liq / wglmr_poop_beamswap_lp_ts.as_u128() as f64
    );

    let f = doc! {
        "address": "0x4EfB208eeEb5A8C85af70e8FBC43D6806b422bec",
        "chain": "moonbeam",
        "protocol": "stellaswap",
    };

    let timestamp = Utc::now().to_string();

    let wglmr_logo = format!(
        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
        "WGLMR"
    );
    let poop_logo = format!(
        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
        "POOP"
    );
    let u = doc! {
        "$set" : {
            "address": "0x4EfB208eeEb5A8C85af70e8FBC43D6806b422bec",
            "chain": "moonbeam",
            "protocol": "stellaswap",
            "name": "WGLMR-POOP LP",
            "symbol": "WGLMR-POOP LP",
            "decimals": 18,
            "logos": [
                wglmr_logo.clone(),
                poop_logo.clone(),
            ],
            "price": stellaswap_wglmr_poop_liq / wglmr_poop_stellaswap_lp_ts.as_u128() as f64,
            "liquidity": stellaswap_wglmr_poop_liq / ten.powf(18.0),
            "totalSupply": wglmr_poop_stellaswap_lp_ts.as_u128() as f64 / ten.powf(18.0),
            "isLP": true,
            "feesAPR": 0.0,
            "underlyingAssets": [
                wglmr_stellaswap_asset.clone().unwrap().address,
                poop_stellaswap_asset.clone().unwrap().address,
            ],
            "underlyingAssetsAlloc": [0.5, 0.5],
            "lastUpdatedAtUTC": timestamp.clone(),
        }
    };

    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .build();
    assets_collection
        .find_one_and_update(f, u, Some(options))
        .await?;

    let f = doc! {
        "address": "0xa049a6260921B5ee3183cFB943133d36d7FdB668",
        "chain": "moonbeam",
        "protocol": "beamswap",
    };

    let timestamp = Utc::now().to_string();

    let wglmr_logo = format!(
        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
        "WGLMR"
    );
    let poop_logo = format!(
        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
        "POOP"
    );
    let u = doc! {
        "$set" : {
            "address": "0xa049a6260921B5ee3183cFB943133d36d7FdB668",
            "chain": "moonbeam",
            "protocol": "beamswap",
            "name": "WGLMR-POOP LP",
            "symbol": "WGLMR-POOP LP",
            "decimals": 18,
            "logos": [
                wglmr_logo.clone(),
                poop_logo.clone(),
            ],
            "price": beamswap_wglmr_poop_liq / wglmr_poop_beamswap_lp_ts.as_u128() as f64,
            "liquidity": beamswap_wglmr_poop_liq / ten.powf(18.0),
            "totalSupply": wglmr_poop_beamswap_lp_ts.as_u128() as f64 / ten.powf(18.0),
            "isLP": true,
            "feesAPR": 0.0,
            "underlyingAssets": [
                wglmr_beamswap_asset.clone().unwrap().address,
                poop_beamswap_asset.clone().unwrap().address,
            ],
            "underlyingAssetsAlloc": [0.5, 0.5],
            "lastUpdatedAtUTC": timestamp.clone(),
        }
    };

    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .build();
    assets_collection
        .find_one_and_update(f, u, Some(options))
        .await?;

    let protocols = vec![
        (
            arthswap_astar_chef_address,
            arthswap_astar_chef,
            "astar".to_string(),
            "arthswap".to_string(),
            "v4".to_string(),
            "0xc5b016c5597D298Fe9eD22922CE290A048aA5B75".to_string(),
            zenlink_astar_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_ASTAR_SUBSQUID.clone(),
            astar_client.clone(),
        ),
        (
            zenlink_moonbeam_chef_address,
            zenlink_moonbeam_chef,
            "moonbeam".to_string(),
            "zenlink".to_string(),
            "v3".to_string(),
            "0xD6708344553cd975189cf45AAe2AB3cd749661f4".to_string(),
            zenlink_moonbeam_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_MOONBEAM_SUBSQUID.clone(),
            moonbeam_client.clone(),
        ),
        (
            solarflare_chef_address,
            solarflare_chef,
            "moonbeam".to_string(),
            "solarflare".to_string(),
            "v2".to_string(),
            "0x995da7dfB96B4dd1e2bd954bE384A1e66cBB4b8c".to_string(),
            solarflare_subgraph_client.clone(),
            constants::subgraph_urls::SOLARFLARE_SUBGRAPH.clone(),
            moonbeam_client.clone(),
        ),
        (
            zenlink_moonriver_chef_address,
            zenlink_moonriver_chef,
            "moonriver".to_string(),
            "zenlink".to_string(),
            "v3".to_string(),
            "0xf4Ec122d32F2117674Ce127b72c40506c52A72F8".to_string(),
            zenlink_moonriver_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_MOONRIVER_SUBSQUID.clone(),
            moonriver_client.clone(),
        ),
        (
            sushi_mini_chef_address,
            sushi_mini_chef,
            "moonriver".to_string(),
            "sushiswap".to_string(),
            "v0".to_string(),
            "0x3dB01570D97631f69bbb0ba39796865456Cf89A5".to_string(),
            sushi_subgraph_client.clone(),
            constants::subgraph_urls::SUSHI_SUBGRAPH.clone(),
            moonriver_client.clone(),
        ),
        (
            beam_chef_address,
            beam_chef,
            "moonbeam".to_string(),
            "beamswap".to_string(),
            "v2".to_string(),
            "0xC6ca172FC8BDB803c5e12731109744fb0200587b".to_string(),
            beamswap_subgraph_client.clone(),
            constants::subgraph_urls::BEAMSWAP_SUBGRAPH.clone(),
            moonbeam_client.clone(),
        ),
        (
            stella_chef_v1_address,
            stella_chef_v1,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v1".to_string(),
            "0xEDFB330F5FA216C9D2039B99C8cE9dA85Ea91c1E".to_string(),
            stellaswap_subgraph_client.clone(),
            constants::subgraph_urls::STELLASWAP_SUBGRAPH.clone(),
            moonbeam_client.clone(),
        ),
        (
            stella_chef_v2_address,
            stella_chef_v2,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v2".to_string(),
            "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388".to_string(),
            stellaswap_subgraph_client.clone(),
            constants::subgraph_urls::STELLASWAP_SUBGRAPH.clone(),
            moonbeam_client.clone(),
        ),
        (
            solarbeam_chef_address,
            solarbeam_chef,
            "moonriver".to_string(),
            "solarbeam".to_string(),
            "v2".to_string(),
            "0x0329867a8c457e9F75e25b0685011291CD30904F".to_string(),
            solarbeam_subgraph_client.clone(),
            constants::subgraph_urls::SOLARBEAM_SUBGRAPH.clone(),
            moonriver_client.clone(),
        ),
        (
            zenlink_astar_chef_address,
            zenlink_astar_chef,
            "astar".to_string(),
            "zenlink".to_string(),
            "v3".to_string(),
            "0x460ee9DBc82B2Be84ADE50629dDB09f6A1746545".to_string(),
            zenlink_astar_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_ASTAR_SUBSQUID.clone(),
            astar_client.clone(),
        ),
    ];

    for p in protocols.clone() {
        let pool_length: U256 = p.1.pool_length().call().await?;
        println!("pool_length {}", pool_length.as_u32());

        for pid in 0..pool_length.as_u32() {
            println!(
                "---------------------\n{} {} pid {}",
                p.3.clone(),
                p.4.clone(),
                pid
            );

            if p.3.clone() == "arthswap".to_string() {
                if pid != 31 && pid < 35 {
                    // IArthswapChef
                    let arthswap_chef_address = p.5.parse::<Address>()?;
                    let mut arthswap_chef =
                        IArthswapChef::new(arthswap_chef_address, Arc::clone(&astar_client));

                    let (acc_arsw_per_share, last_reward_block, alloc_point): (u128, u64, u64) =
                        arthswap_chef
                            .pool_infos(ethers::prelude::U256::from(pid))
                            .call()
                            .await?;

                    println!(
                        "acc_arsw_per_share {:?} last_reward_block {:?} alloc_point {:?}",
                        acc_arsw_per_share, last_reward_block, alloc_point
                    );

                    let lp_tokens = arthswap_chef
                        .lp_tokens(ethers::prelude::U256::from(pid))
                        .call()
                        .await?;

                    println!("lp_tokens {:?}", lp_tokens);

                    let asset_addr = ethers::utils::to_checksum(&lp_tokens.to_owned(), None);
                    println!("asset_addr {:?}", asset_addr.clone());
                    let asset_filter = doc! { "address": asset_addr.clone(), "protocol": p.3.clone(), "chain": p.2.clone() };
                    let asset = assets_collection.find_one(asset_filter, None).await?;

                    let ap = alloc_point;

                    println!("asset {:?} alloc_point {:?}", asset, ap);

                    let mut farm_type = models::FarmType::StandardAmm;
                    let farm_implementation = models::FarmImplementation::Solidity;

                    let ten: i128 = 10;

                    let mut rewards = vec![];
                    let mut total_reward_apr = 0.0;

                    let arsw_filter = doc! { "address": "0xDe2578Edec4669BA7F41c5d5D2386300bcEA4678", "protocol": p.3.clone(), "chain": p.2.clone() };
                    let arsw = assets_collection.find_one(arsw_filter, None).await?;
                    let arsw_price = arsw.clone().unwrap().price;
                    let asset_price = asset.clone().unwrap().price;
                    let asset_tvl = asset.clone().unwrap().liquidity;

                    println!("arsw {:?} asset {:?}", arsw.clone(), asset.clone());

                    if ap > 0 {
                        let mut block_time = constants::utils::ASTAR_BLOCK_TIME;

                        let tap: U256 = arthswap_chef.total_alloc_point().call().await?;

                        // TODO: move below 2 calls outside (before) for loop
                        // get current block (astar)
                        let block_number =
                            ethers_providers::Middleware::get_block_number(&p.8.clone()).await?;
                        println!("block_number {:?}", block_number);
                        // get period (call arthswap_chef.get_period)
                        let period: U256 = arthswap_chef
                            .get_period(ethers::prelude::U256::from(block_number.as_u64()))
                            .call()
                            .await?;
                        println!("period {:?}", period);
                        let arsw_per_block: U256 = arthswap_chef
                            .arsw_per_block(ethers::prelude::U256::from(period.as_u64()))
                            .call()
                            .await?;
                        let arsw_per_sec = arsw_per_block.as_u128() as f64 / block_time;

                        let rewards_per_sec: f64 =
                            arsw_per_sec * (ap as f64 / tap.as_u128() as f64);
                        let rewards_per_day: f64 = rewards_per_sec * 60.0 * 60.0 * 24.0;

                        if rewards_per_day != 0.0 {
                            rewards.push(bson!({
                            "amount": rewards_per_day as f64 / ten.pow(arsw.clone().unwrap().decimals) as f64,
                            "asset":  arsw.clone().unwrap().symbol,
                            "valueUSD": (rewards_per_day as f64 / ten.pow(arsw.clone().unwrap().decimals) as f64) * arsw_price,
                            "freq": models::Freq::Daily.to_string(),
                        }));

                            // reward_apr/farm_apr/pool_apr
                            println!(
                                "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                rewards_per_sec, rewards_per_day, asset_tvl
                            );

                            let reward_apr = ((rewards_per_day as f64 * arsw_price)
                                / (asset_tvl as f64
                                    * ten.pow(arsw.clone().unwrap().decimals) as f64))
                                * 365.0
                                * 100.0;
                            println!("reward_apr: {}", reward_apr);
                            if asset_tvl != 0.0 && asset_price != 0.0 {
                                total_reward_apr += reward_apr;
                            }
                        }
                    }

                    let timestamp = Utc::now().to_string();

                    println!("chef v4 farm lastUpdatedAtUTC {}", timestamp.clone());

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
                                "price": asset.clone().unwrap().price,
                                "logos": asset.clone().unwrap().logos,
                            },
                            "tvl": asset_tvl,
                            "apr.reward": total_reward_apr,
                            "apr.base": asset.clone().unwrap().fees_apr,
                            "rewards": rewards,
                            "allocPoint": ap as u32,
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
            } else if p.3.clone() == "zenlink".to_string() {
                let zenlink_chef_address = p.5.parse::<Address>()?;
                let mut zenlink_chef =
                    IFarming::new(zenlink_chef_address, Arc::clone(&astar_client));

                if p.2.clone() == "moonriver".to_string() {
                    zenlink_chef =
                        IFarming::new(zenlink_chef_address, Arc::clone(&moonriver_client));
                } else if p.2.clone() == "moonbeam".to_string() {
                    zenlink_chef =
                        IFarming::new(zenlink_chef_address, Arc::clone(&moonbeam_client));
                }

                let (
                    farming_token,
                    amount,
                    reward_tokens,
                    reward_per_block,
                    acc_reward_per_share,
                    last_reward_block,
                    start_block,
                    claimable_interval,
                ): (
                    Address,
                    U256,
                    Vec<Address>,
                    Vec<U256>,
                    Vec<U256>,
                    U256,
                    U256,
                    U256,
                ) = zenlink_chef
                    .get_pool_info(ethers::prelude::U256::from(pid))
                    .call()
                    .await?;

                let ft_addr = ethers::utils::to_checksum(&farming_token.to_owned(), None);

                let mut farm_type = models::FarmType::StandardAmm;

                if pid == 3 && p.2.clone() == "astar".to_string() {
                    // 4pool on astar

                    // 0xb0Fa056fFFb74c0FB215F86D691c94Ed45b686Aa

                    farm_type = models::FarmType::StableAmm;

                    let stable_asset = IStableLpToken::new(farming_token, Arc::clone(&p.8.clone()));
                    let name: String = stable_asset.name().call().await?;
                    let symbol: String = stable_asset.symbol().call().await?;
                    println!("name: {:?}", name);

                    let owner_addr: Address = stable_asset.owner().call().await?; // or 0xb0Fa056fFFb74c0FB215F86D691c94Ed45b686Aa

                    let owner = IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
                    let stable_lp_underlying_tokens = owner.get_tokens().call().await?;
                    let stable_lp_underlying_balances = owner.get_token_balances().call().await?;
                    println!(
                        "stable_lp_underlying_tokens: {:#?}",
                        stable_lp_underlying_tokens
                    );
                    println!(
                        "stable_lp_underlying_balances: {:#?}",
                        stable_lp_underlying_balances
                    );

                    let bai_addr = "0x733ebcC6DF85f8266349DEFD0980f8Ced9B45f35";
                    let busd_addr = "0x4Bf769b05E832FCdc9053fFFBC78Ca889aCb5E1E";
                    let dai_addr = "0x6De33698e9e9b787e09d3Bd7771ef63557E148bb";
                    let usdc_addr = "0x6a2d262D56735DbA19Dd70682B39F6bE9a931D98";

                    let bai = IAnyswapV5ERC20::new(bai_addr.parse::<Address>()?, p.8.clone());
                    let busd = IAnyswapV5ERC20::new(busd_addr.parse::<Address>()?, p.8.clone());
                    let dai = IAnyswapV5ERC20::new(dai_addr.parse::<Address>()?, p.8.clone());
                    let usdc = IAnyswapV5ERC20::new(usdc_addr.parse::<Address>()?, p.8.clone());

                    let bai_filter =
                        doc! {"chain":p.2.clone(), "protocol":"zenlink", "address": bai_addr};
                    let bai_asset = assets_collection.find_one(bai_filter, None).await?;
                    let busd_filter =
                        doc! {"chain":p.2.clone(), "protocol":"zenlink", "address": busd_addr};
                    let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                    let dai_filter =
                        doc! {"chain":p.2.clone(), "protocol":"zenlink", "address": dai_addr};
                    let dai_asset = assets_collection.find_one(dai_filter, None).await?;
                    let usdc_filter =
                        doc! {"chain":p.2.clone(), "protocol":"zenlink", "address": usdc_addr};
                    let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;

                    let ten: f64 = 10.0;
                    let bai_bal: U256 = bai.balance_of(owner_addr).call().await?;
                    let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                    let dai_bal: U256 = dai.balance_of(owner_addr).call().await?;
                    let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;

                    // let ft_addr = "0x755cbAC2246e8219e720591Dd362a772076ab653";

                    let _4pool =
                        IStableLpToken::new(ft_addr.parse::<Address>()?, Arc::clone(&p.8.clone()));
                    let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                    //
                    let usd_pool_liq = bai_bal.as_u128() as f64 * bai_asset.clone().unwrap().price
                        / ten.powf(18.0)
                        + busd_bal.as_u128() as f64 * busd_asset.clone().unwrap().price
                            / ten.powf(18.0)
                        + dai_bal.as_u128() as f64 * dai_asset.clone().unwrap().price
                            / ten.powf(18.0)
                        + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                            / ten.powf(6.0);
                    println!("4pool usd_pool_liq {}", usd_pool_liq);
                    let total_supply: U256 = stable_asset.total_supply().call().await?;
                    let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                    let usd_pool_price = usd_pool_liq / ts;
                    println!("usd_pool_price {}", usd_pool_price);

                    let f = doc! {
                        "address": ft_addr.to_string(),
                        "chain": p.2.clone(),
                        "protocol": p.3.clone(),
                    };

                    let timestamp = Utc::now().to_string();

                    let u = doc! {
                        "$set" : {
                            "address": ft_addr.to_string(),
                            "chain": p.2.clone(),
                            "protocol": p.3.clone(),
                            "name": "Zenlink Stable AMM 4pool LP".to_string(),
                            "symbol": "4pool".to_string(),
                            "decimals": 18,
                            "logos": [
                                bai_asset.clone().unwrap().logos.get(0),
                                busd_asset.clone().unwrap().logos.get(0),
                                dai_asset.clone().unwrap().logos.get(0),
                                usdc_asset.clone().unwrap().logos.get(0),
                            ],
                            "price": usd_pool_price,
                            "liquidity": usd_pool_liq,
                            "totalSupply": ts,
                            "isLP": true,
                            "feesAPR": 0.0,
                            "underlyingAssets": [
                                bai_asset.clone().unwrap().address,
                                busd_asset.clone().unwrap().address,
                                dai_asset.clone().unwrap().address,
                                usdc_asset.clone().unwrap().address,
                            ],
                            "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                            "lastUpdatedAtUTC": timestamp.clone(),
                        }
                    };

                    let options = FindOneAndUpdateOptions::builder()
                        .upsert(Some(true))
                        .build();
                    assets_collection
                        .find_one_and_update(f, u, Some(options))
                        .await?;
                    //
                } else if pid == 11 && p.2.clone() == "moonriver".to_string() {
                    // 4pool on moonriver

                    farm_type = models::FarmType::StableAmm;

                    let stable_asset = IStableLpToken::new(farming_token, Arc::clone(&p.8.clone()));
                    let name: String = stable_asset.name().call().await?;
                    let symbol: String = stable_asset.symbol().call().await?;
                    println!("name: {:?}", name);

                    let owner_addr: Address = stable_asset.owner().call().await?;

                    let owner = IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
                    let stable_lp_underlying_tokens = owner.get_tokens().call().await?;
                    let stable_lp_underlying_balances = owner.get_token_balances().call().await?;
                    println!(
                        "stable_lp_underlying_tokens: {:#?}",
                        stable_lp_underlying_tokens
                    );
                    println!(
                        "stable_lp_underlying_balances: {:#?}",
                        stable_lp_underlying_balances
                    );

                    let usdt_addr = "0xB44a9B6905aF7c801311e8F4E76932ee959c663C";
                    let frax_addr = "0x1A93B23281CC1CDE4C4741353F3064709A16197d";
                    let usdc_addr = "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D";
                    let xcausd_addr = "0xFfFffFFfa1B026a00FbAA67c86D5d1d5BF8D8228";

                    let usdt = IAnyswapV5ERC20::new(usdt_addr.parse::<Address>()?, p.8.clone());
                    let frax = IAnyswapV5ERC20::new(frax_addr.parse::<Address>()?, p.8.clone());
                    let usdc = IAnyswapV5ERC20::new(usdc_addr.parse::<Address>()?, p.8.clone());
                    let xcausd = IAnyswapV5ERC20::new(xcausd_addr.parse::<Address>()?, p.8.clone());

                    let usdt_filter =
                        doc! {"chain":p.2.clone(), "protocol":"solarbeam", "address": usdt_addr};
                    let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;
                    let frax_filter =
                        doc! {"chain":p.2.clone(), "protocol":"solarbeam", "address": frax_addr};
                    let frax_asset = assets_collection.find_one(frax_filter, None).await?;
                    let usdc_filter =
                        doc! {"chain":p.2.clone(), "protocol":"zenlink", "address": usdc_addr};
                    let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                    let xcausd_filter =
                        doc! {"chain":p.2.clone(), "protocol":"zenlink", "address": xcausd_addr};
                    let xcausd_asset = assets_collection.find_one(xcausd_filter, None).await?;

                    let ten: f64 = 10.0;
                    let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;
                    let frax_bal: U256 = frax.balance_of(owner_addr).call().await?;
                    let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                    let xcausd_bal: U256 = xcausd.balance_of(owner_addr).call().await?;

                    // let ft_addr = "0x755cbAC2246e8219e720591Dd362a772076ab653";

                    let _4pool =
                        IStableLpToken::new(ft_addr.parse::<Address>()?, Arc::clone(&p.8.clone()));
                    let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                    //
                    let usd_pool_liq = usdt_bal.as_u128() as f64
                        * usdt_asset.clone().unwrap().price
                        / ten.powf(6.0)
                        + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                            / ten.powf(18.0)
                        + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                            / ten.powf(6.0)
                        + xcausd_bal.as_u128() as f64 * xcausd_asset.clone().unwrap().price
                            / ten.powf(12.0);
                    println!("4pool usd_pool_liq {}", usd_pool_liq);
                    let total_supply: U256 = stable_asset.total_supply().call().await?;
                    let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                    let usd_pool_price = usd_pool_liq / ts;
                    println!("usd_pool_price {}", usd_pool_price);

                    let f = doc! {
                        "address": ft_addr.to_string(),
                        "chain": p.2.clone(),
                        "protocol": p.3.clone(),
                    };

                    let timestamp = Utc::now().to_string();

                    let u = doc! {
                        "$set" : {
                            "address": ft_addr.to_string(),
                            "chain": p.2.clone(),
                            "protocol": p.3.clone(),
                            "name": "Zenlink Stable AMM 4pool LP".to_string(),
                            "symbol": "4pool".to_string(),
                            "decimals": 18,
                            "logos": [
                                usdt_asset.clone().unwrap().logos.get(0),
                                frax_asset.clone().unwrap().logos.get(0),
                                usdc_asset.clone().unwrap().logos.get(0),
                                xcausd_asset.clone().unwrap().logos.get(0),
                            ],
                            "price": usd_pool_price,
                            "liquidity": usd_pool_liq,
                            "totalSupply": ts,
                            "isLP": true,
                            "feesAPR": 0.0,
                            "underlyingAssets": [
                                usdt_asset.clone().unwrap().address,
                                frax_asset.clone().unwrap().address,
                                usdc_asset.clone().unwrap().address,
                                xcausd_asset.clone().unwrap().address,
                            ],
                            "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                            "lastUpdatedAtUTC": timestamp.clone(),
                        }
                    };

                    let options = FindOneAndUpdateOptions::builder()
                        .upsert(Some(true))
                        .build();
                    assets_collection
                        .find_one_and_update(f, u, Some(options))
                        .await?;
                } else if pid == 1 && p.2.clone() == "moonriver".to_string() {
                    // zlk on moonriver
                    farm_type = models::FarmType::SingleStaking;
                } else if pid == 1 && p.2.clone() == "moonbeam".to_string() {
                    // zlk on moonbeam
                    farm_type = models::FarmType::SingleStaking;
                }

                let mut asset_filter = doc! { "address": ft_addr.clone(), "chain": p.2.clone(), "protocol": "zenlink" };

                let asset = assets_collection.find_one(asset_filter, None).await?;

                let mut asset_price: f64 = 0.0;
                let mut asset_tvl: u128 = 0;

                let mut rewards = vec![];
                // <symbol, (exists, amount, valueUSD, freq)>
                let mut reward_asset_map: HashMap<String, (bool, f64, f64, String)> =
                    HashMap::new();
                let mut total_reward_apr = 0.0;

                if asset.is_some() {
                    println!(
                        "--------------------\nasset: {:?}",
                        asset.clone().unwrap().symbol
                    );

                    for i in 0..reward_tokens.len() {
                        let reward_asset_addr =
                            ethers::utils::to_checksum(&reward_tokens[i].to_owned(), None);
                        println!("reward_asset_addr: {:?}", reward_asset_addr);

                        let reward_asset_filter = doc! { "address": reward_asset_addr, "protocol": "zenlink", "chain": p.2.clone() };
                        let reward_asset = assets_collection
                            .find_one(reward_asset_filter, None)
                            .await?;

                        if reward_asset.is_some() {
                            let reward_asset_price = reward_asset.clone().unwrap().price;
                            println!("reward_asset_price: {:?}", reward_asset_price);

                            asset_price = asset.clone().unwrap().price;

                            println!("asset_price: {:?}", asset_price);

                            let mut block_time = constants::utils::ASTAR_BLOCK_TIME;
                            if p.2.clone() == "moonriver".to_string() {
                                block_time = constants::utils::MOONRIVER_BLOCK_TIME;
                            } else if p.2.clone() == "moonbeam".to_string() {
                                block_time = constants::utils::MOONBEAM_BLOCK_TIME;
                            }

                            let rpb = reward_per_block[i].as_u128();
                            let rewards_per_sec: f64 = rpb as f64 / block_time;
                            let rewards_per_day: u128 = rewards_per_sec as u128 * 60 * 60 * 24;
                            println!(
                                "rpb {:?} rewards_per_sec {:?} rewards_per_day {:?}",
                                rpb, rewards_per_sec, rewards_per_day
                            );

                            asset_tvl = amount.as_u128();

                            let ten: i128 = 10;

                            if rewards_per_day != 0 {
                                if !reward_asset_map
                                    .contains_key(&reward_asset.clone().unwrap().symbol)
                                {
                                    reward_asset_map.insert(
                                        reward_asset.clone().unwrap().symbol,
                                        (
                                            true,
                                            rewards_per_day as f64
                                                / ten.pow(reward_asset.clone().unwrap().decimals)
                                                    as f64,
                                            (rewards_per_day as f64
                                                / ten.pow(reward_asset.clone().unwrap().decimals)
                                                    as f64)
                                                * reward_asset_price,
                                            models::Freq::Daily.to_string(),
                                        ),
                                    );
                                } else {
                                    let er = reward_asset_map
                                        .get(&reward_asset.clone().unwrap().symbol)
                                        .unwrap();
                                    reward_asset_map.insert(
                                        reward_asset.clone().unwrap().symbol,
                                        (
                                            true,
                                            er.1 + rewards_per_day as f64
                                                / ten.pow(reward_asset.clone().unwrap().decimals)
                                                    as f64,
                                            er.2 + (rewards_per_day as f64
                                                / ten.pow(reward_asset.clone().unwrap().decimals)
                                                    as f64)
                                                * reward_asset_price,
                                            models::Freq::Daily.to_string(),
                                        ),
                                    );
                                }

                                // reward_apr/farm_apr/pool_apr
                                println!(
                                    "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                    rewards_per_sec, rewards_per_day, asset_tvl
                                );
                                let mut reward_apr = 0.0;

                                reward_apr = ((rewards_per_day as f64
                                    / ten.pow(reward_asset.clone().unwrap().decimals) as f64
                                    * reward_asset_price)
                                    / (asset_tvl as f64 * asset_price))
                                    * 365.0
                                    * 100.0;
                                if farm_type == models::FarmType::SingleStaking
                                    || farm_type == models::FarmType::StableAmm
                                {
                                    reward_apr *= ten.pow(18) as f64;
                                }

                                println!("reward_apr: {}", reward_apr);
                                if asset_tvl != 0 && asset_price != 0.0 {
                                    total_reward_apr += reward_apr;
                                }
                            }
                        }
                    }

                    for r in reward_asset_map.iter() {
                        rewards.push(bson!({
                            "amount": r.1.1,
                            "asset":  r.0,
                            "valueUSD": r.1.2,
                            "freq": models::Freq::Daily.to_string(),
                        }));
                    }

                    let ten: f64 = 10.0;

                    let mut atvl: f64 = asset_tvl as f64 * asset_price;
                    if farm_type == models::FarmType::SingleStaking
                        || farm_type == models::FarmType::StableAmm
                    {
                        atvl = asset_tvl as f64 * asset_price / ten.powf(18.0);
                    }

                    println!(
                        "rewards {:?} total_reward_apr {:?} tvl {:?}",
                        rewards.clone(),
                        total_reward_apr,
                        atvl
                    );
                    println!("--------------------\n");

                    if rewards.len() > 0 {
                        // base_apr/trading_apr
                        let mut base_apr = 0.0;
                        #[derive(Serialize)]
                        pub struct Vars {
                            addr: String,
                        }
                        let vars = Vars {
                            addr: asset.clone().unwrap().address.to_lowercase(),
                        };
                        let pair_day_datas =
                            p.6.query_with_vars_unwrap::<subsquid::ZenlinkPairDayDatas, Vars>(
                                &constants::subsquid::PAIR_DAY_DATAS_QUERY.clone(),
                                vars,
                            )
                            .await;
                        if pair_day_datas.is_ok() {
                            let mut daily_volume_lw: f64 = 0.0;
                            for pdd in pair_day_datas.clone().unwrap().pair_day_data {
                                let dv: f64 = pdd.daily_volume_usd.parse().unwrap_or_default();
                                daily_volume_lw += dv;
                            }
                            println!("dvsum {:?}", daily_volume_lw);

                            daily_volume_lw /= pair_day_datas.unwrap().pair_day_data.len() as f64;

                            println!("dvlwavg {:?}", daily_volume_lw);

                            if asset.clone().unwrap_or_default().total_supply == 0.0
                                || asset.clone().unwrap_or_default().price == 0.0
                            {
                                base_apr = 0.0;
                                println!("c1");
                            } else {
                                base_apr = daily_volume_lw * 0.0025 * 365.0 * 100.0
                                    / (asset.clone().unwrap_or_default().total_supply
                                        * asset.clone().unwrap_or_default().price);
                                println!(
                                    "c2 {:?}",
                                    (asset.clone().unwrap_or_default().total_supply
                                        * asset.clone().unwrap_or_default().price)
                                );
                            }
                        } else {
                            println!("pddnok");
                        }

                        if base_apr.is_nan() {
                            base_apr = 0.0;
                        }
                        // if stable
                        if (pid == 11 && p.2.clone() == "moonriver".to_string())
                            || (pid == 3 && p.2.clone() == "astar".to_string())
                        {
                            println!("stable zenlink");
                            let zenlink_stable_swaps =
                                p.6.query_unwrap::<subsquid::ZenlinkStableSwaps>(
                                    &constants::subsquid::STABLE_SWAPS_DAY_DATA_QUERY.clone(),
                                )
                                .await;

                            if zenlink_stable_swaps.is_ok() {
                                let mut daily_volume_lw: f64 = 0.0;
                                if zenlink_stable_swaps.clone().unwrap().stable_swaps.len() > 0 {
                                    for ss in zenlink_stable_swaps.clone().unwrap().stable_swaps[0]
                                        .stable_swap_day_data
                                        .clone()
                                    {
                                        let dv: f64 =
                                            ss.daily_volume_usd.parse().unwrap_or_default();
                                        daily_volume_lw += dv;
                                    }
                                    println!("stable zenlinkdvsum {:?}", daily_volume_lw);

                                    daily_volume_lw /=
                                        zenlink_stable_swaps.clone().unwrap().stable_swaps[0]
                                            .stable_swap_day_data
                                            .clone()
                                            .len() as f64;

                                    println!("stable zenlinkdvlwavg {:?}", daily_volume_lw);
                                }

                                if asset.clone().unwrap_or_default().total_supply == 0.0
                                    || asset.clone().unwrap_or_default().price == 0.0
                                {
                                    base_apr = 0.0;
                                    println!("c1");
                                } else {
                                    base_apr = daily_volume_lw * 0.00025 * 365.0 * 100.0
                                        / (asset.clone().unwrap_or_default().total_supply
                                            * asset.clone().unwrap_or_default().price);
                                    println!(
                                        "bapr {:?} c2 {:?}",
                                        base_apr,
                                        (asset.clone().unwrap_or_default().total_supply
                                            * asset.clone().unwrap_or_default().price)
                                    );
                                }
                            } else {
                                println!("ssddnok");
                            }
                        }
                        // else if (pid == 3 && p.2.clone() == "astar".to_string()) {
                        // }

                        let timestamp = Utc::now().to_string();

                        println!(
                            "zenlink chef v3 farm lastUpdatedAtUTC {}",
                            timestamp.clone()
                        );

                        let ff = doc! {
                            "id": pid as i32,
                            "chef": p.5.clone(),
                            "chain": p.2.clone(),
                            "protocol": "zenlink".to_string(),
                        };
                        let fu = doc! {
                            "$set" : {
                                "id": pid,
                                "chef": p.5.clone(),
                                "chain": p.2.clone(),
                                "protocol": "zenlink".to_string(),
                                "farmType": farm_type.to_string(),
                                "farmImpl": models::FarmImplementation::Solidity.to_string(),
                                "asset": {
                                    "symbol": asset.clone().unwrap().symbol,
                                    "address": asset.clone().unwrap().address,
                                    "price": asset.clone().unwrap().price,
                                    "logos": asset.clone().unwrap().logos,
                                },
                                "tvl": atvl,
                                "apr.reward": total_reward_apr,
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
            } else if p.4.clone() == "v0".to_string() {
                let sushi_mini_chef_address =
                    "0x3dB01570D97631f69bbb0ba39796865456Cf89A5".parse::<Address>()?;
                let sushi_mini_chef =
                    IMiniChefV2::new(sushi_mini_chef_address, Arc::clone(&moonriver_client));

                // TODO: fetch this address from minichef contract
                // right now hardcoding to prevent repeated calls (same rewarder is used for all pids)
                let sushi_complex_rewarder_address =
                    "0x1334c8e873E1cae8467156e2A81d1C8b566B2da1".parse::<Address>()?;
                let sushi_complex_rewarder = IComplexRewarderTime::new(
                    sushi_complex_rewarder_address,
                    Arc::clone(&moonriver_client),
                );

                let (_acc_native_reward_per_share, _last_reward_timestamp, alloc_point): (
                    u128,
                    u64,
                    u64,
                ) = sushi_mini_chef
                    .pool_info(ethers::prelude::U256::from(pid))
                    .call()
                    .await?;

                let ap = alloc_point as u32;

                let farm_type = models::FarmType::StandardAmm;
                let farm_implementation = models::FarmImplementation::Solidity;

                if ap > 0 {
                    let lp_token: Address = sushi_mini_chef
                        .lp_token(ethers::prelude::U256::from(pid))
                        .call()
                        .await?;

                    let asset_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);

                    let asset_filter = doc! { "address": asset_addr.clone(), "protocol": p.3.clone(), "chain": p.2.clone() };
                    let asset = assets_collection.find_one(asset_filter, None).await?;

                    let mut asset_price: f64;
                    let mut asset_tvl: f64 = 0.0;

                    let mut rewards = vec![];
                    // <symbol, (exists, amount, valueUSD, freq)>
                    let mut reward_asset_map: HashMap<String, (bool, f64, f64, String)> =
                        HashMap::new();
                    let mut total_reward_apr = 0.0;

                    if asset.is_some() {
                        println!("asset: {:?}", asset.clone().unwrap().symbol);
                        let sps: U256 = sushi_mini_chef.sushi_per_second().call().await?;
                        let tap: U256 = sushi_mini_chef.total_alloc_point().call().await?;
                        let rps: U256 = sushi_complex_rewarder.reward_per_second().call().await?;

                        let sushi_filter = doc! {"address":"0xf390830DF829cf22c53c8840554B98eafC5dCBc2","protocol":"sushiswap","chain":"moonriver"};
                        let sushi = assets_collection.find_one(sushi_filter, None).await?;

                        let movr_filter = doc! {"address":"0xf50225a84382c74CbdeA10b0c176f71fc3DE0C4d","protocol":"sushiswap","chain":"moonriver"};
                        let movr = assets_collection.find_one(movr_filter, None).await?;

                        if sushi.is_some() || movr.is_some() {
                            if sushi.is_some() {
                                let reward_asset_price = sushi.clone().unwrap().price;
                                println!("reward_asset_price: {:?}", reward_asset_price);

                                asset_price = asset.clone().unwrap().price;
                                println!("asset_price: {:?}", asset_price);

                                let rewards_per_sec: f64 =
                                    sps.as_u128() as f64 * (ap as f64 / tap.as_u128() as f64);

                                let rewards_per_day: f64 = rewards_per_sec * 60.0 * 60.0 * 24.0;
                                asset_tvl = asset.clone().unwrap().liquidity;

                                let ten: i128 = 10;

                                if rewards_per_day != 0.0 {
                                    rewards.push(bson!({
                                        "amount": rewards_per_day as f64 / ten.pow(sushi.clone().unwrap().decimals) as f64,
                                        "asset":  sushi.clone().unwrap().symbol,
                                        "valueUSD": (rewards_per_day as f64 / ten.pow(sushi.clone().unwrap().decimals) as f64) * reward_asset_price,
                                        "freq": models::Freq::Daily.to_string(),
                                    }));

                                    // reward_apr/farm_apr/pool_apr
                                    println!(
                                        "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                        rewards_per_sec, rewards_per_day, asset_tvl
                                    );

                                    let reward_apr = ((rewards_per_day as f64
                                        * reward_asset_price)
                                        / (asset_tvl as f64
                                            * ten.pow(sushi.clone().unwrap().decimals) as f64))
                                        * 365.0
                                        * 100.0;
                                    println!("reward_apr: {}", reward_apr);
                                    if asset_tvl != 0.0 && asset_price != 0.0 {
                                        total_reward_apr += reward_apr;
                                    }
                                }
                            }

                            if movr.is_some() {
                                let reward_asset_price = movr.clone().unwrap().price;
                                println!("reward_asset_price: {:?}", reward_asset_price);

                                asset_price = asset.clone().unwrap().price;
                                println!("asset_price: {:?}", asset_price);

                                let (
                                    _acc_native_reward_per_share,
                                    _last_reward_timestamp,
                                    r_alloc_point,
                                ): (u128, u64, u64) = sushi_mini_chef
                                    .pool_info(ethers::prelude::U256::from(pid))
                                    .call()
                                    .await?;

                                let rap = r_alloc_point as u32;

                                let rewards_per_sec: f64 =
                                    rps.as_u128() as f64 * (rap as f64 / tap.as_u128() as f64);

                                let rewards_per_day: f64 = rewards_per_sec * 60.0 * 60.0 * 24.0;
                                asset_tvl = asset.clone().unwrap().liquidity;

                                let ten: i128 = 10;
                                if rewards_per_day != 0.0 {
                                    rewards.push(bson!({
                                        "amount": rewards_per_day as f64 / ten.pow(movr.clone().unwrap().decimals) as f64,
                                        "asset":  movr.clone().unwrap().symbol,
                                        "valueUSD": (rewards_per_day as f64 / ten.pow(movr.clone().unwrap().decimals) as f64) * reward_asset_price,
                                        "freq": models::Freq::Daily.to_string(),
                                    }));

                                    // reward_apr/farm_apr/pool_apr
                                    println!(
                                        "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                        rewards_per_sec, rewards_per_day, asset_tvl
                                    );

                                    let reward_apr = ((rewards_per_day as f64
                                        * reward_asset_price)
                                        / (asset_tvl as f64
                                            * ten.pow(movr.clone().unwrap().decimals) as f64))
                                        * 365.0
                                        * 100.0;
                                    println!("reward_apr: {}", reward_apr);
                                    if asset_tvl != 0.0 && asset_price != 0.0 {
                                        total_reward_apr += reward_apr;
                                    }
                                }
                            }

                            // base_apr/trading_apr
                            let mut base_apr = 0.0;
                            #[derive(Serialize)]
                            pub struct Vars {
                                addr: String,
                            }
                            let vars = Vars {
                                addr: asset.clone().unwrap().address.to_lowercase(),
                            };
                            let pair_day_datas =
                                p.6.query_with_vars_unwrap::<subgraph::SushiPairDayDatas, Vars>(
                                    &constants::chef::SUSHI_PAIR_DAY_DATAS_QUERY.clone(),
                                    vars,
                                )
                                .await;
                            if pair_day_datas.is_ok() {
                                // TODO: check if formula for sushi base apr is correct
                                // println!("ukk {:?}", pair_day_datas.clone().unwrap());
                                let mut daily_volume_lw: f64 = 0.0;
                                for pdd in pair_day_datas.clone().unwrap().pair_day_datas {
                                    let dv: f64 = pdd.volume_usd.parse().unwrap_or_default();
                                    daily_volume_lw += dv;
                                    // println!("ukkdv {:?}", dv);
                                }
                                // daily_volume_lw /= pair_day_datas.unwrap().pair_day_datas.len() as f64;

                                if asset.clone().unwrap_or_default().total_supply == 0.0
                                    || asset.clone().unwrap_or_default().price == 0.0
                                {
                                    base_apr = 0.0;
                                } else {
                                    base_apr = daily_volume_lw * 0.0025 * 365.0 * 100.0
                                        / (asset.clone().unwrap_or_default().total_supply
                                            * asset.clone().unwrap_or_default().price);
                                }
                            }

                            if base_apr.is_nan() {
                                base_apr = 0.0;
                            }

                            let timestamp = Utc::now().to_string();

                            println!("chef v0 farm lastUpdatedAtUTC {}", timestamp.clone());

                            let ff = doc! {
                                "id": pid as i32,
                                "chef": p.5.clone(),
                                "chain": p.2.clone(),
                                "protocol": p.3.clone(),
                            };
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
                                        "price": asset.clone().unwrap().price,
                                        "logos": asset.clone().unwrap().logos,
                                    },
                                    "tvl": asset_tvl as f64,
                                    "apr.reward": total_reward_apr,
                                    "apr.base": base_apr,
                                    "rewards": rewards,
                                    "allocPoint": ap,
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
                } else {
                    println!("allocPoint = 0");

                    let timestamp = Utc::now().to_string();

                    println!("chef v0 farm lastUpdatedAtUTC {}", timestamp.clone());

                    let ff = doc! {
                        "id": pid as i32,
                        "chef": p.5.clone(),
                        "chain": p.2.clone(),
                        "protocol": p.3.clone(),
                    };
                    let fu = doc! {
                        "$set" : {
                            "id": pid,
                            "chef": p.5.clone(),
                            "chain": p.2.clone(),
                            "protocol": p.3.clone(),
                            "farmType": farm_type.to_string(),
                            "farmImpl": farm_implementation.to_string(),
                            "asset": {
                                "symbol": "",
                                "address": "",
                                "price": 0,
                                "logos": [],
                            },
                            "tvl": 0,
                            "apr.reward": 0,
                            "apr.base": 0,
                            "rewards": [],
                            "allocPoint": ap,
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
            } else {
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

                let mut farm_type = models::FarmType::StandardAmm;
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

                        let asset_filter = doc! { "address": asset_addr.clone(), "protocol": p.3.clone(), "chain": p.2.clone() };
                        let asset = assets_collection.find_one(asset_filter, None).await?;

                        let asset_price: f64;
                        let asset_tvl: u128;

                        let mut rewards = vec![];
                        // <symbol, (exists, amount, valueUSD, freq)>
                        let mut reward_asset_map: HashMap<String, (bool, f64, f64, String)> =
                            HashMap::new();

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
                                if rewards_per_day != 0.0 {
                                    rewards.push(bson!({
                                        "amount": rewards_per_day as f64 / ten.pow(stella.clone().unwrap().decimals) as f64,
                                        "asset":  stella.clone().unwrap().symbol,
                                        "valueUSD": (rewards_per_day as f64 / ten.pow(stella.clone().unwrap().decimals) as f64) * reward_asset_price,
                                        "freq": models::Freq::Daily.to_string(),
                                    }));
                                }

                                // reward_apr/farm_apr/pool_apr
                                println!(
                                    "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                    rewards_per_sec, rewards_per_day, asset_tvl
                                );
                                let mut reward_apr = 0.0;

                                if asset_tvl != 0 && asset_price != 0.0 {
                                    reward_apr = ((rewards_per_day as f64 * reward_asset_price)
                                        / (asset_tvl as f64 * asset_price))
                                        * 365.0
                                        * 100.0;
                                }
                                println!("reward_apr: {}", reward_apr);

                                // base_apr/trading_apr
                                let mut base_apr = 0.0;
                                #[derive(Serialize)]
                                pub struct Vars {
                                    addr: String,
                                }
                                let vars = Vars {
                                    addr: asset.clone().unwrap().address.to_lowercase(),
                                };
                                let pair_day_datas =
                                    p.6.query_with_vars_unwrap::<subgraph::PairDayDatas, Vars>(
                                        &constants::chef::PAIR_DAY_DATAS_QUERY.clone(),
                                        vars,
                                    )
                                    .await;
                                if pair_day_datas.is_ok() {
                                    let mut daily_volume_lw: f64 = 0.0;
                                    for pdd in pair_day_datas.clone().unwrap().pair_day_datas {
                                        let dv: f64 =
                                            pdd.daily_volume_usd.parse().unwrap_or_default();
                                        daily_volume_lw += dv;
                                    }
                                    daily_volume_lw /=
                                        pair_day_datas.unwrap().pair_day_datas.len() as f64;

                                    if asset.clone().unwrap_or_default().total_supply == 0.0
                                        || asset.clone().unwrap_or_default().price == 0.0
                                    {
                                        base_apr = 0.0;
                                    } else {
                                        base_apr = daily_volume_lw * 0.002 * 365.0 * 100.0
                                            / (asset.clone().unwrap_or_default().total_supply
                                                * asset.clone().unwrap_or_default().price);
                                    }
                                }

                                if base_apr.is_nan() {
                                    base_apr = 0.0;
                                }

                                let timestamp = Utc::now().to_string();

                                println!("chef v1 farm lastUpdatedAtUTC {}", timestamp.clone());

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
                                            "price": asset.clone().unwrap().price,
                                            "logos": asset.clone().unwrap().logos,
                                        },
                                        "tvl": asset_tvl as f64 * asset_price / ten.powf(18.0),
                                        "apr.reward": reward_apr,
                                        "apr.base": base_apr,
                                        "rewards": rewards,
                                        "allocPoint": ap,
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
                    } else {
                        let (addresses, symbols, decimals, rewards_per_sec) =
                            p.1.pool_rewards_per_sec(ethers::prelude::U256::from(pid))
                                .call()
                                .await?;

                        println!(
                            "pool_rewards_per_sec\naddresses: {:?}, symbols: {:?}, decimals: {:?}, rewards_per_sec: {:?}",
                            addresses, symbols, decimals, rewards_per_sec
                        );

                        // stable amm asset
                        if p.3.clone() == "solarbeam".to_string()
                            && (pid == 8
                                || pid == 9
                                || pid == 13
                                || pid == 16
                                || pid == 17
                                || pid == 25)
                        {
                            farm_type = models::FarmType::StableAmm;

                            let stable_asset =
                                IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let name: String = stable_asset.name().call().await?;
                            let symbol: String = stable_asset.symbol().call().await?;
                            println!("name: {:?}", name);
                            // let split_name = name.split(" ");
                            // let split_name_vec: Vec<&str> = split_name.collect();
                            // if split_name_vec.len() > 1 && (split_name_vec[1] == "Stable") {
                            let owner_addr: Address = stable_asset.owner().call().await?;
                            let owner =
                                IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
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

                            // busd: "0x5D9ab5522c64E1F6ef5e3627ECCc093f56167818"
                            // usdc: "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D"
                            // usdt: "0xB44a9B6905aF7c801311e8F4E76932ee959c663C"

                            let busd = IAnyswapV5ERC20::new(
                                "0x5D9ab5522c64E1F6ef5e3627ECCc093f56167818".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = IAnyswapV5ERC20::new(
                                "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = IAnyswapV5ERC20::new(
                                "0xB44a9B6905aF7c801311e8F4E76932ee959c663C".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let frax = IAnyswapV5ERC20::new(
                                "0x1A93B23281CC1CDE4C4741353F3064709A16197d".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mai = IAnyswapV5ERC20::new(
                                "0xFb2019DfD635a03cfFF624D210AEe6AF2B00fC2C".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mim = IAnyswapV5ERC20::new(
                                "0x0caE51e1032e8461f4806e26332c030E34De3aDb".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let wbtc = IAnyswapV5ERC20::new(
                                "0x6aB6d61428fde76768D7b45D8BFeec19c6eF91A8".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let xckbtc = IAnyswapV5ERC20::new(
                                "0xFFFfFfFfF6E528AD57184579beeE00c5d5e646F0".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let xcksm = IAnyswapV5ERC20::new(
                                "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let stksm = IAnyswapV5ERC20::new(
                                "0xFfc7780C34B450d917d557E728f033033CB4fA8C".parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x5D9ab5522c64E1F6ef5e3627ECCc093f56167818"};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D"};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xB44a9B6905aF7c801311e8F4E76932ee959c663C"};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;

                            let frax_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x1A93B23281CC1CDE4C4741353F3064709A16197d"};
                            let frax_asset = assets_collection.find_one(frax_filter, None).await?;
                            let mai_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFb2019DfD635a03cfFF624D210AEe6AF2B00fC2C"};
                            let mai_asset = assets_collection.find_one(mai_filter, None).await?;
                            let mim_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x0caE51e1032e8461f4806e26332c030E34De3aDb"};
                            let mim_asset = assets_collection.find_one(mim_filter, None).await?;

                            let wbtc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0x6aB6d61428fde76768D7b45D8BFeec19c6eF91A8"};
                            let wbtc_asset = assets_collection.find_one(wbtc_filter, None).await?;
                            let xckbtc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFFFfFfFfF6E528AD57184579beeE00c5d5e646F0"};
                            let xckbtc_asset =
                                assets_collection.find_one(xckbtc_filter, None).await?;

                            let xcksm_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080"};
                            let xcksm_asset =
                                assets_collection.find_one(xcksm_filter, None).await?;
                            let stksm_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFfc7780C34B450d917d557E728f033033CB4fA8C"};
                            let stksm_asset =
                                assets_collection.find_one(stksm_filter, None).await?;

                            let ten: f64 = 10.0;
                            let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                            let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                            let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;

                            let frax_bal: U256 = frax.balance_of(owner_addr).call().await?;
                            let mai_bal: U256 = mai.balance_of(owner_addr).call().await?;
                            let mim_bal: U256 = mim.balance_of(owner_addr).call().await?;

                            let wbtc_bal: U256 = wbtc.balance_of(owner_addr).call().await?;
                            let xckbtc_bal: U256 = xckbtc.balance_of(owner_addr).call().await?;

                            let xcksm_bal: U256 = xcksm.balance_of(owner_addr).call().await?;
                            let stksm_bal: U256 = stksm.balance_of(owner_addr).call().await?;

                            let _3pool = IStableLpToken::new(
                                "0xfb29918d393AaAa7dD118B51A8b7fCf862F5f336".parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            let _3pool_bal: U256 = _3pool.balance_of(owner_addr).call().await?;

                            // TODO: calculate underlyingAssetsAlloc

                            if symbol == "3pool".to_string() {
                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / ten.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / ten.powf(6.0);
                                println!("3pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0xfb29918d393AaAa7dD118B51A8b7fCf862F5f336".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0xfb29918d393AaAa7dD118B51A8b7fCf862F5f336".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - USD Pool".to_string(),
                                        "symbol": "3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.33, 0.33, 0.33],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "FRAX-3pool".to_string() {
                                let usd_pool_liq = _3pool_bal.as_u128() as f64 / ten.powf(18.0)
                                    + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x884609A4D86BBA8477112E36e27f7A4ACecB3575".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x884609A4D86BBA8477112E36e27f7A4ACecB3575".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - FRAX Pool".to_string(),
                                        "symbol": "FRAX-3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            frax_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            frax_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "MAI-3pool".to_string() {
                                let usd_pool_liq = _3pool_bal.as_u128() as f64 / ten.powf(18.0)
                                    + mai_bal.as_u128() as f64 * mai_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x8CDB472731B4f815d67e76885a22269ad7f0e398".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x8CDB472731B4f815d67e76885a22269ad7f0e398".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - MAI Pool".to_string(),
                                        "symbol": "MAI-3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            mai_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            mai_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "MIM-3pool".to_string() {
                                let usd_pool_liq = _3pool_bal.as_u128() as f64 / ten.powf(18.0)
                                    + mim_bal.as_u128() as f64 * mim_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x4BaB767c98186bA28eA66f2a69cd0DA351D60b36".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x4BaB767c98186bA28eA66f2a69cd0DA351D60b36".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - MIM Pool".to_string(),
                                        "symbol": "MIM-3pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            mim_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            mim_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "kBTC-BTC".to_string() {
                                let wbtc_price = wbtc_asset.clone().unwrap().price;
                                let xckbtc_price = xckbtc_asset.clone().unwrap().price;
                                let pool_liq = wbtc_bal.as_u128() as f64 * wbtc_price
                                    / ten.powf(8.0)
                                    + xckbtc_bal.as_u128() as f64 * xckbtc_price / ten.powf(8.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let pool_price = pool_liq / ts;
                                println!("pool_price {}", pool_price);

                                let f = doc! {
                                    "address": "0x4F707d051b4b49B63e72Cc671e78E152ec66f2fA".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x4F707d051b4b49B63e72Cc671e78E152ec66f2fA".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - kBTC Pool".to_string(),
                                        "symbol": "kBTC-BTC".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            wbtc_asset.clone().unwrap().logos.get(0),
                                            xckbtc_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": pool_price,
                                        "liquidity": pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            wbtc_asset.clone().unwrap().address,
                                            xckbtc_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "stKSM".to_string() {
                                let pool_liq = xcksm_bal.as_u128() as f64
                                    * xcksm_asset.clone().unwrap().price
                                    / ten.powf(12.0)
                                    + stksm_bal.as_u128() as f64
                                        * stksm_asset.clone().unwrap().price
                                        / ten.powf(12.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let pool_price = pool_liq / ts;
                                println!("pool_price {}", pool_price);

                                let f = doc! {
                                    "address": "0x493147C85Fe43F7B056087a6023dF32980Bcb2D1".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x493147C85Fe43F7B056087a6023dF32980Bcb2D1".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Solarbeam Stable AMM - stKSM Pool".to_string(),
                                        "symbol": "stKSM".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            xcksm_asset.clone().unwrap().logos.get(0),
                                            stksm_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": pool_price,
                                        "liquidity": pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            xcksm_asset.clone().unwrap().address,
                                            stksm_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.5, 0.5],
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

                            // let mut comb: Vec<(H160, U256)> = vec![];
                            // comb = stable_lp_underlying_tokens
                            //     .clone()
                            //     .into_iter()
                            //     .zip(stable_lp_underlying_balances.clone().into_iter())
                            //     .collect();
                        }

                        // 4pool
                        if p.3.clone() == "beamswap".to_string() && (pid == 16) {
                            farm_type = models::FarmType::StableAmm;

                            let stable_asset =
                                IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let name: String = stable_asset.name().call().await?;
                            let symbol: String = stable_asset.symbol().call().await?;
                            println!("name: {:?}", name);
                            // let split_name = name.split(" ");
                            // let split_name_vec: Vec<&str> = split_name.collect();
                            // if split_name_vec.len() > 1 && (split_name_vec[1] == "Stable") {
                            let owner_addr: Address = stable_asset.owner().call().await?;
                            let owner =
                                IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
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

                            // busd: "0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F"
                            // usdc: "0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b"
                            // usdt: "0xeFAeeE334F0Fd1712f9a8cc375f427D9Cdd40d73"
                            // dai: "0x765277EebeCA2e31912C9946eAe1021199B39C61"

                            let busd = IAnyswapV5ERC20::new(
                                "0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = IAnyswapV5ERC20::new(
                                "0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = IAnyswapV5ERC20::new(
                                "0xeFAeeE334F0Fd1712f9a8cc375f427D9Cdd40d73".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let dai = IAnyswapV5ERC20::new(
                                "0x765277EebeCA2e31912C9946eAe1021199B39C61".parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F"};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b"};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0xeFAeeE334F0Fd1712f9a8cc375f427D9Cdd40d73"};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;
                            let dai_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":"0x765277EebeCA2e31912C9946eAe1021199B39C61"};
                            let dai_asset = assets_collection.find_one(dai_filter, None).await?;

                            let ten: f64 = 10.0;
                            let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                            let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                            let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;
                            let dai_bal: U256 = dai.balance_of(owner_addr).call().await?;

                            let _4pool = IStableLpToken::new(
                                "0x9BF6910790D70E9b5B07Cb28271C42531B929b4c".parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            // let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                            if symbol == "4pool".to_string() {
                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / ten.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + dai_bal.as_u128() as f64 * dai_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                println!("4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0x9BF6910790D70E9b5B07Cb28271C42531B929b4c".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0x9BF6910790D70E9b5B07Cb28271C42531B929b4c".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "Beamswap Stable DEX - Stable Multichain".to_string(),
                                        "symbol": "4pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                            dai_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                            dai_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
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
                        }

                        // stellaswap stable
                        if p.3.clone() == "stellaswap".to_string()
                            && p.4.clone() == "v2"
                            && (pid == 31 || pid == 33)
                        {
                            farm_type = models::FarmType::StableAmm;

                            let stable_asset =
                                IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let name: String = stable_asset.name().call().await?;
                            let symbol: String = stable_asset.symbol().call().await?;
                            println!("name: {:?}", name);

                            let owner_addr: Address = stable_asset.owner().call().await?;
                            let owner =
                                IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
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

                            // frax: "0x322E86852e492a7Ee17f28a78c663da38FB33bfb"
                            // busd: "0x692C57641fc054c2Ad6551Ccc6566EbA599de1BA"
                            // usdc: "0x931715FEE2d06333043d11F658C8CE934aC61D0c"
                            // usdt: "0xFFFFFFfFea09FB06d082fd1275CD48b191cbCD1d"
                            // mai: "0xdFA46478F9e5EA86d57387849598dbFB2e964b02"

                            let frax = IAnyswapV5ERC20::new(
                                "0x322E86852e492a7Ee17f28a78c663da38FB33bfb".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let busd = IAnyswapV5ERC20::new(
                                "0x692C57641fc054c2Ad6551Ccc6566EbA599de1BA".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = IAnyswapV5ERC20::new(
                                "0x931715FEE2d06333043d11F658C8CE934aC61D0c".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = IAnyswapV5ERC20::new(
                                "0xFFFFFFfFea09FB06d082fd1275CD48b191cbCD1d".parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mai = IAnyswapV5ERC20::new(
                                "0xdFA46478F9e5EA86d57387849598dbFB2e964b02".parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":"0x692C57641fc054c2Ad6551Ccc6566EbA599de1BA"};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":"0x931715FEE2d06333043d11F658C8CE934aC61D0c"};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":"0xFFFFFFfFea09FB06d082fd1275CD48b191cbCD1d"};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;
                            let frax_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":"0x322E86852e492a7Ee17f28a78c663da38FB33bfb"};
                            let frax_asset = assets_collection.find_one(frax_filter, None).await?;
                            let mai_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":"0xdFA46478F9e5EA86d57387849598dbFB2e964b02"};
                            let mai_asset = assets_collection.find_one(mai_filter, None).await?;

                            let ten: f64 = 10.0;
                            let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                            let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                            let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;
                            let frax_bal: U256 = frax.balance_of(owner_addr).call().await?;
                            let mai_bal: U256 = mai.balance_of(owner_addr).call().await?;

                            let _4pool = IStableLpToken::new(
                                "0xB326b5189AA42Acaa3C649B120f084Ed8F4dCaA6".parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                            if symbol == "stella4pool".to_string() {
                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / ten.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / ten.powf(6.0)
                                    + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                                        / ten.powf(18.0);
                                println!("stella4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0xB326b5189AA42Acaa3C649B120f084Ed8F4dCaA6".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0xB326b5189AA42Acaa3C649B120f084Ed8F4dCaA6".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "StellaSwap 4pool".to_string(),
                                        "symbol": "4pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            frax_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            frax_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
                                        "lastUpdatedAtUTC": timestamp.clone(),
                                    }
                                };

                                let options = FindOneAndUpdateOptions::builder()
                                    .upsert(Some(true))
                                    .build();
                                assets_collection
                                    .find_one_and_update(f, u, Some(options))
                                    .await?;
                            } else if symbol == "stellaMAI-4pool" {
                                let usd_pool_liq = mai_bal.as_u128() as f64
                                    * mai_asset.clone().unwrap().price
                                    / ten.powf(18.0)
                                    + _4pool_bal.as_u128() as f64 / ten.powf(18.0);

                                println!("stellaMAI-4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64 / ten.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": "0xEceab9F0FcF15Fddbffbd7baE2cEB78CD57b879a".clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": "0xEceab9F0FcF15Fddbffbd7baE2cEB78CD57b879a".to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "StellaSwap MAI-4pool".to_string(),
                                        "symbol": "MAI-4pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            mai_asset.clone().unwrap().logos.get(0),
                                            frax_asset.clone().unwrap().logos.get(0),
                                            busd_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": [
                                            mai_asset.clone().unwrap().address,
                                            frax_asset.clone().unwrap().address,
                                            busd_asset.clone().unwrap().address,
                                            usdc_asset.clone().unwrap().address,
                                            usdt_asset.clone().unwrap().address,
                                        ],
                                        "underlyingAssetsAlloc": [0.25, 0.25, 0.25, 0.25],
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
                        }

                        if rewards_per_sec.len() > 0 {
                            let mut total_reward_apr = 0.0;

                            let asset_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                            println!("asset_addr: {:?}", asset_addr.clone());

                            let asset_filter = doc! { "address": asset_addr.clone(), "protocol": p.3.clone(), "chain": p.2.clone() };
                            let asset = assets_collection.find_one(asset_filter, None).await?;

                            let mut asset_price: f64 = 0.0;
                            let mut asset_tvl: u128 = 0;

                            let mut rewards = vec![];
                            // <symbol, (exists, amount, valueUSD, freq)>
                            let mut reward_asset_map: HashMap<String, (bool, f64, f64, String)> =
                                HashMap::new();

                            if asset.is_some() {
                                for i in 0..symbols.len() {
                                    println!("rwrd[{}]", i);

                                    let s = format!("{:?}", symbols[i].clone());
                                    println!("symbol: {}", s);

                                    let reward_asset_addr =
                                        ethers::utils::to_checksum(&addresses[i].to_owned(), None);
                                    println!("reward_asset_addr: {:?}", reward_asset_addr);

                                    let reward_asset_filter = doc! { "address": reward_asset_addr, "protocol": p.3.clone(), "chain": p.2.clone() };
                                    let reward_asset = assets_collection
                                        .find_one(reward_asset_filter, None)
                                        .await?;

                                    if reward_asset.is_some() {
                                        let reward_asset_price =
                                            reward_asset.clone().unwrap().price;
                                        println!("reward_asset_price: {:?}", reward_asset_price);

                                        if pid == 38 && p.3.clone() == "solarbeam".to_string() {
                                            let solar_filter = doc! { "address": "0x6bD193Ee6D2104F14F94E2cA6efefae561A4334B", "protocol": "solarbeam", "chain": "moonriver" };
                                            let solar = assets_collection
                                                .find_one(solar_filter, None)
                                                .await?;
                                            if solar.is_some() {
                                                asset_price = solar.unwrap().price;
                                            }
                                        } else {
                                            asset_price = asset.clone().unwrap().price;
                                        }

                                        println!("asset_price: {:?}", asset_price);

                                        let mut rewards_per_day: u128 =
                                            rewards_per_sec[i].as_u128() * 60 * 60 * 24;
                                        asset_tvl = total_lp.as_u128();

                                        let ten: i128 = 10;

                                        if p.3.clone() == "stellaswap".to_string()
                                            && p.5.clone()
                                                == "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388"
                                                    .to_string()
                                            && reward_asset.clone().unwrap().symbol
                                                == "STELLA".to_string()
                                        {
                                            let stella_per_sec: U256 =
                                                p.1.stella_per_sec().call().await?;
                                            let total_alloc_point: U256 =
                                                p.1.total_alloc_point().call().await?;

                                            rewards_per_day = ((ap as u128)
                                                * (60 * 60 * 24 * stella_per_sec.as_u128())
                                                / total_alloc_point.as_u128())
                                                as u128;
                                        }

                                        if rewards_per_day != 0 {
                                            if !reward_asset_map
                                                .contains_key(&reward_asset.clone().unwrap().symbol)
                                            {
                                                reward_asset_map.insert(
                                                    reward_asset.clone().unwrap().symbol,
                                                    (
                                                        true,
                                                        rewards_per_day as f64
                                                            / ten.pow(
                                                                reward_asset
                                                                    .clone()
                                                                    .unwrap()
                                                                    .decimals,
                                                            )
                                                                as f64,
                                                        (rewards_per_day as f64
                                                            / ten.pow(
                                                                reward_asset
                                                                    .clone()
                                                                    .unwrap()
                                                                    .decimals,
                                                            )
                                                                as f64)
                                                            * reward_asset_price,
                                                        models::Freq::Daily.to_string(),
                                                    ),
                                                );
                                            } else {
                                                let er = reward_asset_map
                                                    .get(&reward_asset.clone().unwrap().symbol)
                                                    .unwrap();
                                                reward_asset_map.insert(
                                                    reward_asset.clone().unwrap().symbol,
                                                    (
                                                        true,
                                                        er.1 + rewards_per_day as f64
                                                            / ten.pow(
                                                                reward_asset
                                                                    .clone()
                                                                    .unwrap()
                                                                    .decimals,
                                                            )
                                                                as f64,
                                                        er.2 + (rewards_per_day as f64
                                                            / ten.pow(
                                                                reward_asset
                                                                    .clone()
                                                                    .unwrap()
                                                                    .decimals,
                                                            )
                                                                as f64)
                                                            * reward_asset_price,
                                                        models::Freq::Daily.to_string(),
                                                    ),
                                                );
                                            }

                                            // reward_apr/farm_apr/pool_apr
                                            println!(
                                                "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                                rewards_per_sec[i].as_u128(),
                                                rewards_per_day,
                                                asset_tvl
                                            );

                                            let reward_apr = ((rewards_per_day as f64
                                                / ten.pow(decimals[i].as_u128().try_into().unwrap())
                                                    as f64
                                                * reward_asset_price)
                                                / (asset_tvl as f64 * asset_price
                                                    / ten.pow(18) as f64))
                                                * 365.0
                                                * 100.0;
                                            println!("reward_apr: {}", reward_apr);
                                            if asset_tvl != 0 && asset_price != 0.0 {
                                                total_reward_apr += reward_apr;
                                            }
                                        }
                                    }
                                }

                                for r in reward_asset_map.iter() {
                                    rewards.push(bson!({
                                        "amount": r.1.1,
                                        "asset":  r.0,
                                        "valueUSD": r.1.2,
                                        "freq": models::Freq::Daily.to_string(),
                                    }));
                                }

                                // base_apr/trading_apr
                                let mut base_apr = 0.0;
                                #[derive(Serialize)]
                                pub struct Vars {
                                    addr: String,
                                }
                                let vars = Vars {
                                    addr: asset.clone().unwrap().address.to_lowercase(),
                                };
                                println!(
                                    "pddq {:?} addr {:?}",
                                    &constants::chef::PAIR_DAY_DATAS_QUERY.clone(),
                                    asset.clone().unwrap().address.to_lowercase()
                                );
                                let pair_day_datas =
                                    p.6.query_with_vars_unwrap::<subgraph::PairDayDatas, Vars>(
                                        &constants::chef::PAIR_DAY_DATAS_QUERY.clone(),
                                        vars,
                                    )
                                    .await;

                                let usdc_nomad_solarflare_filter = doc! { "address": "0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b", "protocol": "solarflare", "chain": "moonbeam" };
                                let usdc_nomad_solarflare = assets_collection
                                    .find_one(usdc_nomad_solarflare_filter, None)
                                    .await?;

                                println!(
                                    "usdc_nomad_solarflare {:?}",
                                    usdc_nomad_solarflare.clone().unwrap()
                                );

                                if pair_day_datas.is_ok() {
                                    let mut daily_volume_lw: f64 = 0.0;
                                    for pdd in pair_day_datas.clone().unwrap().pair_day_datas {
                                        let dv: f64 =
                                            pdd.daily_volume_usd.parse().unwrap_or_default();
                                        if dv == 0.0 {
                                            println!("dv0000");

                                            for (i, ua) in asset
                                                .clone()
                                                .unwrap_or_default()
                                                .underlying_assets
                                                .iter()
                                                .enumerate()
                                            {
                                                println!("dv {:?} {:?}", i, ua.clone());
                                                let ua_filter = doc! { "address": ua.clone(), "protocol": p.3.clone(), "chain": p.2.clone() };
                                                let ua_obj = assets_collection
                                                    .find_one(ua_filter, None)
                                                    .await?;
                                                let dvt0: f64 = pdd
                                                    .daily_volume_token0
                                                    .parse()
                                                    .unwrap_or_default();
                                                let dvt1: f64 = pdd
                                                    .daily_volume_token1
                                                    .parse()
                                                    .unwrap_or_default();

                                                println!(
                                                    "gm {:?} {:?} {:?}",
                                                    ua_obj.clone().unwrap_or_default().price,
                                                    dvt0,
                                                    dvt1
                                                );
                                                if i == 0 {
                                                    daily_volume_lw += dvt0
                                                        * ua_obj.clone().unwrap_or_default().price;
                                                } else if i == 1 {
                                                    daily_volume_lw += dvt1
                                                        * ua_obj.clone().unwrap_or_default().price;
                                                } else {
                                                    println!("nadaaa");
                                                }
                                            }
                                        } else {
                                            daily_volume_lw += dv;
                                        }
                                    }
                                    daily_volume_lw /=
                                        pair_day_datas.unwrap().pair_day_datas.len() as f64;

                                    if asset.clone().unwrap_or_default().total_supply == 0.0
                                        || asset.clone().unwrap_or_default().price == 0.0
                                    {
                                        base_apr = 0.0;
                                    } else {
                                        base_apr = daily_volume_lw * 0.002 * 365.0 * 100.0
                                            / (asset.clone().unwrap_or_default().total_supply
                                                * asset.clone().unwrap_or_default().price);
                                        if p.3.clone() == "solarflare" {
                                            println!(
                                                "thisisdway {:?}",
                                                usdc_nomad_solarflare.clone().unwrap().price
                                            );
                                            base_apr /= usdc_nomad_solarflare.unwrap().price;
                                        }
                                    }
                                } else {
                                    println!("pddnotok");
                                }
                                if base_apr.is_nan() {
                                    base_apr = 0.0;
                                }

                                let timestamp = Utc::now().to_string();

                                println!("chef v2 farm lastUpdatedAtUTC {}", timestamp.clone());

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
                                            "price": asset.clone().unwrap().price,
                                            "logos": asset.clone().unwrap().logos,
                                        },
                                        "tvl": asset_tvl as f64 * asset_price / ten.powf(18.0),
                                        "apr.reward": total_reward_apr,
                                        "apr.base": base_apr,
                                        "rewards": rewards,
                                        "allocPoint": ap,
                                        "lastUpdatedAtUTC": timestamp.clone(),
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

                    let timestamp = Utc::now().to_string();

                    println!("chef v1/v2 farm lastUpdatedAtUTC {}", timestamp.clone());

                    let ff = doc! {
                        "id": pid as i32,
                        "chef": p.5.clone(),
                        "chain": p.2.clone(),
                        "protocol": p.3.clone(),
                    };
                    let fu = doc! {
                        "$set" : {
                            "id": pid,
                            "chef": p.5.clone(),
                            "chain": p.2.clone(),
                            "protocol": p.3.clone(),
                            "farmType": farm_type.to_string(),
                            "farmImpl": farm_implementation.to_string(),
                            "asset": {
                                "symbol": "",
                                "address": "",
                                "price": 0,
                                "logos": [],
                            },
                            "tvl": 0,
                            "apr.reward": 0,
                            "apr.base": 0,
                            "rewards": [],
                            "allocPoint": ap,
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

    Ok(())
}

async fn subgraph_jobs(
    mongo_uri: String,
    protocols: Vec<(&str, &str, gql_client::Client, &str)>,
    headers: HashMap<&str, &str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = dotenv::var("DB_NAME").unwrap();
    let db = client.database(&db_name);

    let assets_collection = db.collection::<models::Asset>("assets");

    let arsw_price = reqwest::get(
        "https://api.coingecko.com/api/v3/simple/price?ids=arthswap&vs_currencies=usd",
    )
    .await?
    .json::<apis::coingecko::ASRoot>()
    .await?;

    let arsw_p = arsw_price.arthswap.usd;
    println!("arsw_price {:?}", arsw_p);

    let f = doc! {
        "address": "0xDe2578Edec4669BA7F41c5d5D2386300bcEA4678",
        "chain": "astar",
        "protocol": "arthswap",
    };

    let timestamp = Utc::now().to_string();

    let u = doc! {
        "$set" : {
            "address": "0xDe2578Edec4669BA7F41c5d5D2386300bcEA4678",
            "chain": "astar",
            "protocol": "arthswap",
            "name": "ArthSwap Token",
            "symbol": "ARSW",
            "decimals": 18,
            "logos": [
                "https://raw.githubusercontent.com/yield-bay/assets/main/list/ARSW.png",
            ],
            "price": arsw_p,
            "liquidity": 1.0,
            "totalSupply": 1.0,
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

    let dexscreener_pairs_arthswap_url = "https://api.dexscreener.com/latest/dex/pairs/astar/0xD72A602C714ae36D990dc835eA5F96Ef87657D5e,0xeee106Aa8a0DE519E8Eb21C66A5c2275b46b3F4d,0xBB1290c1829007F440C771b37718FAbf309cd527,0x50497E7181eB9e8CcD70a9c44FB997742149482a,0x806f746a7c4293092ac7aa604347BE123322dF1e,0x996D73aC8F97cf15BD476b77CB92ce47cA0E71Fe,0x87988EbDE7E661F44eB3a586C5E0cEAB533a2d9C,0xF4119c3d9e65602bb34f2455644e45c98d29bB4b,0x73EEa1180c2D1772eA2118FdA888A81943bAc3C8,0xde2EDAa0cD4aFd59d9618c31A060EAb93Ce45e01,0x61a49ba86e168cd25ca795b07b0a93236bb25127,0x92127ec0ebef8b30378d757bbe8dce18210b848b,0xca59df939290421047876c917789afdb68d5d6f1,0xccefddff4808f3e1e0340e19e43f1e9fd088b3f2,0xF041a8e6e27341F5f865a22f01Fa37e065c32156,0xac4b7043da7152726d54b0fb1628a2fff73f874e,0xef8b14e08c292cc552494ec428a75c8a3cd417b6,0x3d78a6cca5c717c0e8702896892f3522d0b07010,0x7644Bf8086d40eD430D5096305830aA97Be77268,0xcf83a3d83c1265780d9374e8a7c838fe22bd3dc6,0x78d5c2adeb11be00033cc4edb2c2889cf945415e,0xaa1fa6a811d82fa4383b522b4af4de3a5041063e,0xb60a1827db219729f837f2d0982b4cdb5a9ba4b1,0x40E938688a121370092A06745704c112C5ee5791,0xbd13fd873d36f7d2a349b35e6854e3183ede18ab,0x7843ecd6f3234d72d0b7034dd9894b77c416c6ef,0x8897d79334c2d517b83e7846da4b922e68fda61b,0x49d1db92a8a1511a6eeb867221d801bc974a3073,0x9c728cb130ed60eebaf84e6b260d369fa6415f5e,0x3f61a095cc21f99e0bf82966579595f2fc0d4d59";
    let dexscreener_pairs_arthswap_url_2="https://api.dexscreener.com/latest/dex/pairs/astar/0x2Cd341F19387D15E8FcD6C9D10Ac08353AB2e2F3,0x3FFCb129Cf2392685d49f7C7B336359528C0958a,0x4d0c348742d5f60baacfebffd2d80a3adfa3f0fe,0x900e71a3745cb660aae9e351ff665c081f1a1ea4,0xDdeA1b3343c438c2E2d636D070cfb4F63d26636e,0x848162f2FaE144D1baF057406940eE88071Bb7d2";

    let mut arthswap_pairs = reqwest::get(dexscreener_pairs_arthswap_url)
        .await?
        .json::<apis::dexscreener::Root>()
        .await?;

    let mut arthswap_pairs_2 = reqwest::get(dexscreener_pairs_arthswap_url_2)
        .await?
        .json::<apis::dexscreener::Root>()
        .await?;

    println!("{:?}", arthswap_pairs.pairs.len());

    arthswap_pairs.pairs.append(&mut arthswap_pairs_2.pairs);

    println!("apl {:?}\n{:?}", arthswap_pairs.pairs.len(), arthswap_pairs);

    if arthswap_pairs.pairs.len() > 0 {
        for pair in arthswap_pairs.clone().pairs.clone() {
            let pa = Address::from_str(pair.pair_address.as_str()).unwrap();
            let pair_addr = to_checksum(&pa, None);

            let t0a = Address::from_str(pair.base_token.address.as_str()).unwrap();
            let token0_addr = to_checksum(&t0a, None);

            let t1a = Address::from_str(pair.quote_token.address.as_str()).unwrap();
            let token1_addr = to_checksum(&t1a, None);

            let token0logo = format!(
                "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                pair.base_token.symbol
            );
            let token1logo = format!(
                "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                pair.quote_token.symbol
            );

            println!(
                "token0logo {:?} token1logo {:?}",
                token0logo.clone(),
                token1logo.clone()
            );

            let token0decimals = 18; //: u32 = pair.base_token.decimals.parse().unwrap_or_default();
            let token1decimals = 18; //: u32 = pair.quote_token.decimals.parse().unwrap_or_default();

            let mut decimals = token0decimals;
            if token1decimals > token0decimals {
                decimals = token1decimals;
            }

            let liquidity: f64 = pair.liquidity.usd as f64;

            let price_usd: f64 = pair.price_usd.parse().unwrap_or_default();

            let total_supply: f64 = liquidity / price_usd;

            let odv = pair.volume.h24;
            let fees_apr = odv * 0.0025 * 365.0 * 100.0 / liquidity;

            let f = doc! {
                "address": pair_addr.clone(),
                "chain": "astar",
                "protocol": "arthswap",
            };

            let timestamp = Utc::now().to_string();

            let u = doc! {
                "$set" : {
                    "address": pair_addr.clone(),
                    "chain": "astar",
                    "protocol": "arthswap",
                    "name": format!("{}-{} LP", pair.base_token.name, pair.quote_token .name),
                    "symbol": format!("{}-{} LP", pair.base_token.symbol, pair.quote_token.symbol),
                    "decimals": decimals,
                    "logos": [
                        token0logo.clone(),
                        token1logo.clone(),
                    ],
                    "price": price_usd,
                    "liquidity": liquidity,
                    "totalSupply": total_supply,
                    "isLP": true,
                    "feesAPR": fees_apr,
                    "underlyingAssets": [token0_addr.clone(), token1_addr.clone()],
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
    }

    for p in protocols {
        println!("subgraph data for {} on {}", p.0.clone(), p.1.clone());

        let client = Client::new_with_headers(p.3.clone(), 60, headers.clone());

        let mut nomad_usdc_price = 1.0;

        if p.0.clone() == "sushiswap" {
            let tokens_data = client
                .query_unwrap::<subgraph::SushiTokensData>(
                    constants::chef::SUSHI_TOKENS_QUERY.clone(),
                )
                .await;

            if tokens_data.is_ok() {
                for t in tokens_data.clone().unwrap().tokens.clone() {
                    let mut price_usd: f64 = 0.0;
                    if t.day_data.len() >= 1 {
                        price_usd = t.day_data[0].price_usd.parse().unwrap_or_default();
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

                    let decimals: u32 = t.decimals.parse().unwrap_or_default();

                    let logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        t.symbol
                    );

                    let liquidity: f64 = t.liquidity.parse().unwrap_or_default();

                    let f = doc! {
                        "address": token_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

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
        } else if p.0.clone() == "zenlink" {
            let tokens_data = client
                .query_unwrap::<subsquid::TokensData>(constants::subsquid::TOKENS_QUERY.clone())
                .await;

            if tokens_data.is_ok() {
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

                    if token_addr.clone() == "0x733ebcC6DF85f8266349DEFD0980f8Ced9B45f35" {
                        // BAI
                        price_usd = 1.0;
                    }
                    if token_addr.clone() == "0x6De33698e9e9b787e09d3Bd7771ef63557E148bb" {
                        // DAI
                        price_usd = 1.0;
                    }
                    if token_addr.clone() == "0xFfFffFFfa1B026a00FbAA67c86D5d1d5BF8D8228" {
                        // xcAUSD
                        price_usd = 1.0;
                    }

                    let decimals: u32 = t.decimals as u32;

                    let logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        t.symbol
                    );

                    let liquidity: f64 = t.total_liquidity.parse().unwrap_or_default();

                    // stKSM or wstKSM
                    if p.0.clone() == "solarbeam"
                        && (token_addr.clone() == "0xFfc7780C34B450d917d557E728f033033CB4fA8C"
                            || token_addr.clone() == "0x3bfd113ad0329a7994a681236323fb16E16790e3")
                    {
                        let xcksm = assets_collection.find_one(doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080"}, None).await?;
                        price_usd = xcksm.clone().unwrap().price;
                    }

                    let f = doc! {
                        "address": token_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

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
        } else {
            let tokens_data = client
                .query_unwrap::<subgraph::TokensData>(constants::chef::TOKENS_QUERY.clone())
                .await;

            if tokens_data.is_ok() {
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

                    let decimals: u32 = t.decimals.parse().unwrap_or_default();

                    let logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        t.symbol
                    );

                    let liquidity: f64 = t.total_liquidity.parse().unwrap_or_default();

                    // stKSM or wstKSM
                    if p.0.clone() == "solarbeam"
                        && (token_addr.clone() == "0xFfc7780C34B450d917d557E728f033033CB4fA8C"
                            || token_addr.clone() == "0x3bfd113ad0329a7994a681236323fb16E16790e3")
                    {
                        let xcksm = assets_collection.find_one(doc! {"chain":"moonriver", "protocol":"solarbeam", "address":"0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080"}, None).await?;
                        price_usd = xcksm.clone().unwrap().price;
                    }

                    if p.0.clone() == "solarflare" {
                        // let mut nomad_usdc_price = 1.0;
                        for ft in tokens_data.clone().unwrap().tokens.clone() {
                            if ft.id == "0x818ec0a7fe18ff94269904fced6ae3dae6d6dc0b" {
                                nomad_usdc_price =
                                    ft.token_day_data[0].price_usd.parse().unwrap_or_default();
                                println!("found moonbeam nomadusdc {:?}", nomad_usdc_price);
                            }
                        }
                        if t.id != "0x818ec0a7fe18ff94269904fced6ae3dae6d6dc0b" {
                            price_usd = price_usd / nomad_usdc_price;
                        }
                    }

                    let f = doc! {
                        "address": token_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

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
        }

        let mut one_day_volume_usd: HashMap<String, f64> = HashMap::new();

        if p.1.clone() == "moonbeam" {
            let block_number = get_one_day_block(
                constants::subgraph_urls::SOLARFLARE_BLOCKLYTICS_SUBGRAPH.to_string(),
                constants::chef::ONE_DAY_BLOCKS_QUERY.to_string(),
            )
            .await;
            if block_number != 0 {
                let pairs = get_one_day_pools(
                    p.3.clone().to_string(),
                    constants::chef::ONE_DAY_POOLS_QUERY.to_string(),
                    block_number,
                )
                .await;
                for pair in pairs {
                    let pair_id = Address::from_str(pair.id.as_str()).unwrap();
                    let pair_addr = to_checksum(&pair_id, None);
                    one_day_volume_usd.insert(
                        pair_addr,
                        pair.untracked_volume_usd.parse().unwrap_or_default(),
                    );
                }
            }
        } else if p.1.clone() == "moonriver" {
            let block_number = get_one_day_block(
                constants::subgraph_urls::SOLARBEAM_BLOCKLYTICS_SUBGRAPH.to_string(),
                constants::chef::ONE_DAY_BLOCKS_QUERY.to_string(),
            )
            .await;
            // if p.0.clone() == "sushiswap" {
            //     if block_number != 0 {
            //         let pairs = get_one_day_pools(
            //             p.3.clone().to_string(),
            //             one_day_pools_query.to_string(),
            //             block_number,
            //         )
            //         .await;
            //         for pair in pairs {
            //             let pair_id = Address::from_str(pair.id.as_str()).unwrap();
            //             let pair_addr = to_checksum(&pair_id, None);
            //             one_day_volume_usd.insert(
            //                 pair_addr,
            //                 pair.untracked_volume_usd.parse().unwrap_or_default(),
            //             );
            //         }
            //     }
            // } else {
            if block_number != 0 {
                let pairs = get_one_day_pools(
                    p.3.clone().to_string(),
                    constants::chef::ONE_DAY_POOLS_QUERY.to_string(),
                    block_number,
                )
                .await;
                for pair in pairs {
                    let pair_id = Address::from_str(pair.id.as_str()).unwrap();
                    let pair_addr = to_checksum(&pair_id, None);
                    one_day_volume_usd.insert(
                        pair_addr,
                        pair.untracked_volume_usd.parse().unwrap_or_default(),
                    );
                }
            }
            // }
        }

        if p.0.clone() == "sushiswap" {
            let pairs_data = client
                .query_unwrap::<subgraph::SushiPairsData>(
                    constants::chef::SUSHI_PAIRS_QUERY.clone(),
                )
                .await;

            if pairs_data.is_ok() {
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

                    let t0a = Address::from_str(pair.token0.id.as_str()).unwrap();
                    let token0_addr = to_checksum(&t0a, None);

                    let t1a = Address::from_str(pair.token1.id.as_str()).unwrap();
                    let token1_addr = to_checksum(&t1a, None);

                    let token0logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        pair.token0.symbol
                    );
                    let token1logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        pair.token1.symbol
                    );

                    println!(
                        "token0logo {:?} token1logo {:?}",
                        token0logo.clone(),
                        token1logo.clone()
                    );

                    let token0decimals: u32 = pair.token0.decimals.parse().unwrap_or_default();
                    let token1decimals: u32 = pair.token1.decimals.parse().unwrap_or_default();

                    let mut decimals = token0decimals;
                    if token1decimals > token0decimals {
                        decimals = token1decimals;
                    }

                    let liquidity: f64 = pair.reserve_usd.parse().unwrap_or_default();
                    let total_supply: f64 = pair.total_supply.parse().unwrap_or_default();

                    let mut price_usd: f64 = 0.0;

                    if total_supply != 0.0 {
                        price_usd = liquidity / total_supply;
                    }

                    let mut fees_apr = 0.0;
                    let odv = one_day_volume_usd.get(&pair_addr.clone());
                    if odv.is_some() {
                        fees_apr = odv.unwrap() * 0.0025 * 365.0 * 100.0 / liquidity;
                    }

                    let f = doc! {
                        "address": pair_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

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
                            "feesAPR": fees_apr,
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
        } else if p.0.clone() == "zenlink" {
            let pairs_data = client
                .query_unwrap::<subsquid::PairsData>(constants::subsquid::PAIRS_QUERY.clone())
                .await;

            if pairs_data.is_ok() {
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

                    let t0a = Address::from_str(pair.token0.id.as_str()).unwrap();
                    let token0_addr = to_checksum(&t0a, None);

                    let t1a = Address::from_str(pair.token1.id.as_str()).unwrap();
                    let token1_addr = to_checksum(&t1a, None);

                    let token0logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        pair.token0.symbol
                    );
                    let token1logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        pair.token1.symbol
                    );

                    println!(
                        "token0logo {:?} token1logo {:?}",
                        token0logo.clone(),
                        token1logo.clone()
                    );

                    let token0decimals: u32 = pair.token0.decimals as u32;
                    let token1decimals: u32 = pair.token1.decimals as u32;

                    let mut decimals = token0decimals;
                    if token1decimals > token0decimals {
                        decimals = token1decimals;
                    }

                    let mut liquidity: f64 = pair.reserve_usd.parse().unwrap_or_default();
                    // wstKSM-xcKSM LP
                    if pair_addr.clone() == "0x5568872bc43Bae3757F697c0e1b241b62Eddcc17" {
                        liquidity *= 2.0;
                    }
                    let total_supply: f64 = pair.total_supply.parse().unwrap_or_default();

                    let mut price_usd: f64 = 0.0;

                    if total_supply != 0.0 {
                        price_usd = liquidity / total_supply;
                    }

                    let mut fees_apr = 0.0;
                    let odv = one_day_volume_usd.get(&pair_addr.clone());
                    if odv.is_some() {
                        fees_apr = odv.unwrap() * 0.0025 * 365.0 * 100.0 / liquidity;
                    }

                    let f = doc! {
                        "address": pair_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

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
                            "feesAPR": fees_apr,
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
        } else {
            let pairs_data = client
                .query_unwrap::<subgraph::PairsData>(constants::chef::PAIRS_QUERY.clone())
                .await;

            if pairs_data.is_ok() {
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

                    let t0a = Address::from_str(pair.token0.id.as_str()).unwrap();
                    let token0_addr = to_checksum(&t0a, None);

                    let t1a = Address::from_str(pair.token1.id.as_str()).unwrap();
                    let token1_addr = to_checksum(&t1a, None);

                    let token0logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        pair.token0.symbol
                    );
                    let token1logo = format!(
                        "https://raw.githubusercontent.com/yield-bay/assets/main/list/{}.png",
                        pair.token1.symbol
                    );

                    println!(
                        "token0logo {:?} token1logo {:?}",
                        token0logo.clone(),
                        token1logo.clone()
                    );

                    let token0decimals: u32 = pair.token0.decimals.parse().unwrap_or_default();
                    let token1decimals: u32 = pair.token1.decimals.parse().unwrap_or_default();

                    let mut decimals = token0decimals;
                    if token1decimals > token0decimals {
                        decimals = token1decimals;
                    }

                    let mut liquidity: f64 = pair.reserve_usd.parse().unwrap_or_default();
                    // wstKSM-xcKSM LP
                    if pair_addr.clone() == "0x5568872bc43Bae3757F697c0e1b241b62Eddcc17" {
                        liquidity *= 2.0;
                    }

                    if p.0.clone() == "solarflare" {
                        liquidity = liquidity / nomad_usdc_price;
                    }

                    let total_supply: f64 = pair.total_supply.parse().unwrap_or_default();

                    let mut price_usd: f64 = 0.0;

                    if total_supply != 0.0 {
                        price_usd = liquidity / total_supply;
                    }

                    let mut fees_apr = 0.0;
                    let odv = one_day_volume_usd.get(&pair_addr.clone());
                    if odv.is_some() {
                        fees_apr = odv.unwrap() * 0.0025 * 365.0 * 100.0 / liquidity;
                        if p.0.clone() == "solarflare" {
                            fees_apr = (odv.unwrap() / nomad_usdc_price) * 0.0025 * 365.0 * 100.0
                                / liquidity;
                        }
                    }

                    let f = doc! {
                        "address": pair_addr.clone(),
                        "chain": p.1.clone(),
                        "protocol": p.0.clone(),
                    };

                    let timestamp = Utc::now().to_string();

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
                            "feesAPR": fees_apr,
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
    }

    Ok(())
}

async fn get_one_day_block(subgraph_url: String, query_str: String) -> u64 {
    let date = Utc::now().timestamp() - 86400;
    let start = date / 1000;
    let end = date / 1000 + 600;

    let subgraph_client = Client::new(subgraph_url.clone(), 60);
    #[derive(Serialize)]
    pub struct Vars {
        start: i64,
        end: i64,
    }
    let vars = Vars {
        start: start,
        end: end,
    };
    let blocks_data = subgraph_client
        .query_with_vars_unwrap::<subgraph::BlocksData, Vars>(&query_str, vars)
        .await;

    if blocks_data.is_ok() {
        if blocks_data.clone().unwrap().blocks.len() > 0 {
            let block_number = blocks_data.clone().unwrap().blocks[0]
                .number
                .parse()
                .unwrap();
            return block_number;
        }
    }

    0
}

async fn get_one_day_pools(
    subgraph_url: String,
    query_str: String,
    block_number: u64,
) -> Vec<subgraph::Pair> {
    let subgraph_client = Client::new(subgraph_url.clone(), 60);
    #[derive(Serialize)]
    pub struct Vars {
        number: u64,
    }
    let vars = Vars {
        number: block_number,
    };
    let pairs_data = subgraph_client
        .query_with_vars_unwrap::<subgraph::PairsData, Vars>(&query_str, vars)
        .await;

    if pairs_data.is_ok() {
        return pairs_data.clone().unwrap().pairs;
    }
    return vec![];
}

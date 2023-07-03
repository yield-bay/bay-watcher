use std::{collections::HashMap, str::FromStr, sync::Arc, thread, time};

use chrono::prelude::Utc;
use dotenv::dotenv;
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

mod apis;
mod constants;
mod contracts;
mod custom;
mod models;
mod scoring;
mod subgraph;
mod subsquid;

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

    println!("------------------------------\npulsar_jobs");
    custom::pulsar::pulsar_jobs(mongo_uri.clone())
        .await
        .unwrap();

    // println!("------------------------------\ndemeter_jobs");
    // custom::demeter::demeter_jobs(mongo_uri.clone())
    //     .await
    //     .unwrap();

    println!("------------------------------\ncurve_jobs");
    custom::curve::curve_jobs(mongo_uri.clone()).await.unwrap();

    // println!("------------------------------\ntapio_taiga_jobs");
    // custom::tapio_taiga::tapio_taiga_jobs(mongo_uri.clone())
    //     .await
    //     .unwrap();

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

    let solarbeam_stable_subgraph_client = Client::new_with_headers(
        constants::subgraph_urls::SOLARBEAM_STABLE_SUBGRAPH.clone(),
        60,
        headers.clone(),
    );
    let stellaswap_stable_subgraph_client = Client::new_with_headers(
        constants::subgraph_urls::STELLASWAP_STABLE_SUBGRAPH.clone(),
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
        solarbeam_stable_subgraph_client.clone(),
        stellaswap_stable_subgraph_client.clone(),
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
    solarbeam_stable_subgraph_client: Client,
    stellaswap_stable_subgraph_client: Client,
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

    let solarbeam_chef_address =
        constants::addresses::solarbeam_on_moonriver::SOLARBEAM_CHEF.parse::<Address>()?;
    let solarbeam_chef =
        contracts::IChefV2::new(solarbeam_chef_address, Arc::clone(&moonriver_client));

    let solarflare_chef_address =
        constants::addresses::solarflare_on_moonbeam::SOLARFLARE_CHEF.parse::<Address>()?;
    let solarflare_chef =
        contracts::IChefV2::new(solarflare_chef_address, Arc::clone(&moonbeam_client));

    let stella_chef_v1_address =
        constants::addresses::stellaswap_on_moonbeam::STELLA_CHEF_V1.parse::<Address>()?;
    let stella_chef_v1 =
        contracts::IChefV2::new(stella_chef_v1_address, Arc::clone(&moonbeam_client));

    let stella_chef_v2_address =
        constants::addresses::stellaswap_on_moonbeam::STELLA_CHEF_V2.parse::<Address>()?;
    let stella_chef_v2 =
        contracts::IChefV2::new(stella_chef_v2_address, Arc::clone(&moonbeam_client));

    let beam_chef_address =
        constants::addresses::beamswap_on_moonbeam::BEAM_CHEF.parse::<Address>()?;
    let beam_chef = contracts::IChefV2::new(beam_chef_address, Arc::clone(&moonbeam_client));

    let sushi_mini_chef_address =
        constants::addresses::sushi_on_moonriver::SUSHI_MINI_CHEF.parse::<Address>()?;
    let sushi_mini_chef =
        contracts::IChefV2::new(sushi_mini_chef_address, Arc::clone(&moonriver_client));

    let zenlink_astar_chef_address =
        constants::addresses::zenlink_on_astar::ZENLINK_CHEF.parse::<Address>()?;
    let zenlink_astar_chef =
        contracts::IChefV2::new(zenlink_astar_chef_address, Arc::clone(&astar_client));

    let zenlink_moonriver_chef_address =
        constants::addresses::zenlink_on_moonriver::ZENLINK_CHEF.parse::<Address>()?;
    let zenlink_moonriver_chef = contracts::IChefV2::new(
        zenlink_moonriver_chef_address,
        Arc::clone(&moonriver_client),
    );

    let zenlink_moonbeam_chef_address =
        constants::addresses::zenlink_on_moonbeam::ZENLINK_CHEF.parse::<Address>()?;
    let zenlink_moonbeam_chef =
        contracts::IChefV2::new(zenlink_moonbeam_chef_address, Arc::clone(&moonbeam_client));

    let arthswap_astar_chef_address =
        constants::addresses::arthswap_on_astar::ARTHSWAP_CHEF.parse::<Address>()?;
    let arthswap_astar_chef =
        contracts::IChefV2::new(arthswap_astar_chef_address, Arc::clone(&astar_client));

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
        "address": constants::addresses::stellaswap_on_moonbeam::POOP,
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
            "address": constants::addresses::stellaswap_on_moonbeam::POOP,
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
        "address": constants::addresses::stellaswap_on_moonbeam::POOP,
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
            "address": constants::addresses::stellaswap_on_moonbeam::POOP,
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
        constants::addresses::stellaswap_on_moonbeam::WGLMR_POOP_LP.parse::<Address>()?;
    let wglmr_poop_stellaswap_lp =
        contracts::ILpToken::new(wglmr_poop_stellaswap_address, Arc::clone(&moonbeam_client));

    let wglmr_poop_beamswap_address =
        constants::addresses::beamswap_on_moonbeam::WGLMR_POOP_LP.parse::<Address>()?;
    let wglmr_poop_beamswap_lp =
        contracts::ILpToken::new(wglmr_poop_beamswap_address, Arc::clone(&moonbeam_client));

    let (stellaswap_r0, stellaswap_r1, _): (u128, u128, u32) =
        wglmr_poop_stellaswap_lp.get_reserves().call().await?;
    let (beamswap_r0, beamswap_r1, _): (u128, u128, u32) =
        wglmr_poop_beamswap_lp.get_reserves().call().await?;

    let wglmr_poop_stellaswap_lp_ts: U256 = wglmr_poop_stellaswap_lp.total_supply().call().await?;
    let wglmr_poop_beamswap_lp_ts: U256 = wglmr_poop_beamswap_lp.total_supply().call().await?;

    let wglmr_stellaswap_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address": constants::addresses::stellaswap_on_moonbeam::WGLMR};
    let wglmr_stellaswap_asset = assets_collection
        .find_one(wglmr_stellaswap_filter, None)
        .await?;
    let poop_stellaswap_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address": constants::addresses::stellaswap_on_moonbeam::POOP};
    let poop_stellaswap_asset = assets_collection
        .find_one(poop_stellaswap_filter, None)
        .await?;

    let wglmr_beamswap_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address": constants::addresses::beamswap_on_moonbeam::WGLMR};
    let wglmr_beamswap_asset = assets_collection
        .find_one(wglmr_beamswap_filter, None)
        .await?;
    let poop_beamswap_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address": constants::addresses::beamswap_on_moonbeam::POOP};
    let poop_beamswap_asset = assets_collection
        .find_one(poop_beamswap_filter, None)
        .await?;

    let stellaswap_wglmr_poop_liq = wglmr_stellaswap_asset.clone().unwrap().price
        * stellaswap_r0 as f64
        + poop_stellaswap_asset.clone().unwrap().price * stellaswap_r1 as f64;
    let beamswap_wglmr_poop_liq = wglmr_beamswap_asset.clone().unwrap().price * beamswap_r0 as f64
        + poop_beamswap_asset.clone().unwrap().price * beamswap_r1 as f64;

    println!(
        "stellaswap_wglmr_poop_liq {:?} wglmr_poop_stellaswap_lp_ts {:?} lpprice {:}",
        stellaswap_wglmr_poop_liq / constants::utils::TEN_F64.powf(18.0),
        wglmr_poop_stellaswap_lp_ts.as_u128() as f64 / constants::utils::TEN_F64.powf(18.0),
        stellaswap_wglmr_poop_liq / wglmr_poop_stellaswap_lp_ts.as_u128() as f64
    );

    println!(
        "beamswap_wglmr_poop_liq {:?} wglmr_poop_beamswap_lp_ts {:?} lpprice {:}",
        beamswap_wglmr_poop_liq / constants::utils::TEN_F64.powf(18.0),
        wglmr_poop_beamswap_lp_ts.as_u128() as f64 / constants::utils::TEN_F64.powf(18.0),
        beamswap_wglmr_poop_liq / wglmr_poop_beamswap_lp_ts.as_u128() as f64
    );

    let f = doc! {
        "address": constants::addresses::stellaswap_on_moonbeam::WGLMR_POOP_LP,
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
            "address": constants::addresses::stellaswap_on_moonbeam::WGLMR_POOP_LP,
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
            "liquidity": stellaswap_wglmr_poop_liq / constants::utils::TEN_F64.powf(18.0),
            "totalSupply": wglmr_poop_stellaswap_lp_ts.as_u128() as f64 / constants::utils::TEN_F64.powf(18.0),
            "isLP": true,
            "feesAPR": 0.0,
            "underlyingAssets": [
                bson!({
                    "symbol": wglmr_stellaswap_asset.clone().unwrap().symbol,
                    "address":  wglmr_stellaswap_asset.clone().unwrap().address,
                    "decimals": wglmr_stellaswap_asset.clone().unwrap().decimals,
                }),
                bson!({
                    "symbol": poop_stellaswap_asset.clone().unwrap().symbol,
                    "address":  poop_stellaswap_asset.clone().unwrap().address,
                    "decimals": poop_stellaswap_asset.clone().unwrap().decimals,
                }),
            ],
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
        "address": constants::addresses::beamswap_on_moonbeam::WGLMR_POOP_LP,
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
            "address": constants::addresses::beamswap_on_moonbeam::WGLMR_POOP_LP,
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
            "liquidity": beamswap_wglmr_poop_liq / constants::utils::TEN_F64.powf(18.0),
            "totalSupply": wglmr_poop_beamswap_lp_ts.as_u128() as f64 / constants::utils::TEN_F64.powf(18.0),
            "isLP": true,
            "feesAPR": 0.0,
            "underlyingAssets": [
                bson!({
                    "symbol": wglmr_beamswap_asset.clone().unwrap().symbol,
                    "address":  wglmr_beamswap_asset.clone().unwrap().address,
                    "decimals": wglmr_beamswap_asset.clone().unwrap().decimals,
                }),
                bson!({
                    "symbol": poop_beamswap_asset.clone().unwrap().symbol,
                    "address":  poop_beamswap_asset.clone().unwrap().address,
                    "decimals": poop_beamswap_asset.clone().unwrap().decimals,
                }),
            ],
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

    let protocols = vec![
        (
            arthswap_astar_chef_address,
            arthswap_astar_chef,
            "astar".to_string(),
            "arthswap".to_string(),
            "v4".to_string(),
            constants::addresses::arthswap_on_astar::ARTHSWAP_CHEF.to_string(),
            zenlink_astar_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_ASTAR_SUBSQUID.clone(),
            astar_client.clone(),
            constants::addresses::arthswap_on_astar::ARTHSWAP_ROUTER.to_string(),
        ),
        (
            zenlink_moonbeam_chef_address,
            zenlink_moonbeam_chef,
            "moonbeam".to_string(),
            "zenlink".to_string(),
            "v3".to_string(),
            constants::addresses::zenlink_on_moonbeam::ZENLINK_CHEF.to_string(),
            zenlink_moonbeam_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_MOONBEAM_SUBSQUID.clone(),
            moonbeam_client.clone(),
            constants::addresses::zenlink_on_moonbeam::ZENLINK_ROUTER.to_string(),
        ),
        (
            solarflare_chef_address,
            solarflare_chef,
            "moonbeam".to_string(),
            "solarflare".to_string(),
            "v2".to_string(),
            constants::addresses::solarflare_on_moonbeam::SOLARFLARE_CHEF.to_string(),
            solarflare_subgraph_client.clone(),
            constants::subgraph_urls::SOLARFLARE_SUBGRAPH.clone(),
            moonbeam_client.clone(),
            constants::addresses::solarflare_on_moonbeam::SOLARFLARE_ROUTER.to_string(),
        ),
        (
            zenlink_moonriver_chef_address,
            zenlink_moonriver_chef,
            "moonriver".to_string(),
            "zenlink".to_string(),
            "v3".to_string(),
            constants::addresses::zenlink_on_moonriver::ZENLINK_CHEF.to_string(),
            zenlink_moonriver_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_MOONRIVER_SUBSQUID.clone(),
            moonriver_client.clone(),
            constants::addresses::zenlink_on_moonriver::ZENLINK_ROUTER.to_string(),
        ),
        (
            sushi_mini_chef_address,
            sushi_mini_chef,
            "moonriver".to_string(),
            "sushiswap".to_string(),
            "v0".to_string(),
            constants::addresses::sushi_on_moonriver::SUSHI_MINI_CHEF.to_string(),
            sushi_subgraph_client.clone(),
            constants::subgraph_urls::SUSHI_SUBGRAPH.clone(),
            moonriver_client.clone(),
            constants::addresses::sushi_on_moonriver::SUSHI_ROUTER.to_string(),
        ),
        (
            beam_chef_address,
            beam_chef,
            "moonbeam".to_string(),
            "beamswap".to_string(),
            "v2".to_string(),
            constants::addresses::beamswap_on_moonbeam::BEAM_CHEF.to_string(),
            beamswap_subgraph_client.clone(),
            constants::subgraph_urls::BEAMSWAP_SUBGRAPH.clone(),
            moonbeam_client.clone(),
            constants::addresses::beamswap_on_moonbeam::BEAM_ROUTER.to_string(),
        ),
        (
            stella_chef_v1_address,
            stella_chef_v1,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v1".to_string(),
            constants::addresses::stellaswap_on_moonbeam::STELLA_CHEF_V1.to_string(),
            stellaswap_subgraph_client.clone(),
            constants::subgraph_urls::STELLASWAP_SUBGRAPH.clone(),
            moonbeam_client.clone(),
            constants::addresses::stellaswap_on_moonbeam::STELLA_ROUTER.to_string(),
        ),
        (
            stella_chef_v2_address,
            stella_chef_v2,
            "moonbeam".to_string(),
            "stellaswap".to_string(),
            "v2".to_string(),
            constants::addresses::stellaswap_on_moonbeam::STELLA_CHEF_V2.to_string(),
            stellaswap_subgraph_client.clone(),
            constants::subgraph_urls::STELLASWAP_SUBGRAPH.clone(),
            moonbeam_client.clone(),
            constants::addresses::stellaswap_on_moonbeam::STELLA_ROUTER.to_string(),
        ),
        (
            solarbeam_chef_address,
            solarbeam_chef,
            "moonriver".to_string(),
            "solarbeam".to_string(),
            "v2".to_string(),
            constants::addresses::solarbeam_on_moonriver::SOLARBEAM_CHEF.to_string(),
            solarbeam_subgraph_client.clone(),
            constants::subgraph_urls::SOLARBEAM_SUBGRAPH.clone(),
            moonriver_client.clone(),
            constants::addresses::solarbeam_on_moonriver::SOLARBEAM_ROUTER.to_string(),
        ),
        (
            zenlink_astar_chef_address,
            zenlink_astar_chef,
            "astar".to_string(),
            "zenlink".to_string(),
            "v3".to_string(),
            constants::addresses::zenlink_on_astar::ZENLINK_CHEF.to_string(),
            zenlink_astar_subsquid_client.clone(),
            constants::subgraph_urls::ZENLINK_ASTAR_SUBSQUID.clone(),
            astar_client.clone(),
            constants::addresses::zenlink_on_astar::ZENLINK_ROUTER.to_string(),
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

            let mut router = p.9.clone();

            if p.3.clone() == "arthswap".to_string() {
                if pid != 31 && pid < 35 {
                    let arthswap_chef_address = p.5.parse::<Address>()?;
                    let arthswap_chef = contracts::IArthswapChef::new(
                        arthswap_chef_address,
                        Arc::clone(&astar_client),
                    );

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

                    let farm_type = models::FarmType::StandardAmm;
                    let farm_implementation = models::FarmImplementation::Solidity;

                    let mut underlying_assets = vec![];
                    for ua in asset.clone().unwrap().underlying_assets {
                        underlying_assets.push(bson!({
                            "symbol": ua.symbol,
                            "address": ua.address,
                            "decimals": ua.decimals,
                        }))
                    }
                    let mut rewards: Vec<Bson> = vec![];
                    let mut total_reward_apr = 0.0;

                    let arsw_filter = doc! { "address": constants::addresses::arthswap_on_astar::ARSW, "protocol": p.3.clone(), "chain": p.2.clone() };
                    let arsw = assets_collection.find_one(arsw_filter, None).await?;
                    let arsw_price = arsw.clone().unwrap().price;
                    let asset_price = asset.clone().unwrap().price;
                    let asset_tvl = asset.clone().unwrap().liquidity;

                    println!("arsw {:?} asset {:?}", arsw.clone(), asset.clone());

                    if ap > 0 {
                        let block_time = constants::utils::ASTAR_BLOCK_TIME;

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
                                "amount": rewards_per_day as f64 / constants::utils::TEN_I128.pow(arsw.clone().unwrap().decimals) as f64,
                                "asset":  arsw.clone().unwrap().symbol,
                                "valueUSD": (rewards_per_day as f64 / constants::utils::TEN_I128.pow(arsw.clone().unwrap().decimals) as f64) * arsw_price,
                                "freq": models::Freq::Daily.to_string(),
                            }));

                            // reward_apr/farm_apr/pool_apr
                            println!(
                                "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                rewards_per_sec, rewards_per_day, asset_tvl
                            );

                            let reward_apr = ((rewards_per_day as f64 * arsw_price)
                                / (asset_tvl as f64
                                    * constants::utils::TEN_I128.pow(arsw.clone().unwrap().decimals)
                                        as f64))
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
                    let fu = doc! {
                        "$set" : {
                            "id": pid,
                            "chef": p.5.clone(),
                            "chain": p.2.clone(),
                            "protocol": p.3.clone(),
                            "farmType": farm_type.to_string(),
                            "farmImpl": farm_implementation.to_string(),
                            "router": router,
                            "asset": {
                                "symbol": asset.clone().unwrap().symbol,
                                "address": asset_addr.clone(),
                                "price": asset.clone().unwrap().price,
                                "logos": asset.clone().unwrap().logos,
                                "underlyingAssets": underlying_assets,
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
                    contracts::IFarming::new(zenlink_chef_address, Arc::clone(&astar_client));

                if p.2.clone() == "moonriver".to_string() {
                    zenlink_chef = contracts::IFarming::new(
                        zenlink_chef_address,
                        Arc::clone(&moonriver_client),
                    );
                } else if p.2.clone() == "moonbeam".to_string() {
                    zenlink_chef = contracts::IFarming::new(
                        zenlink_chef_address,
                        Arc::clone(&moonbeam_client),
                    );
                }

                let (
                    farming_token,
                    amount,
                    reward_tokens,
                    reward_per_block,
                    _acc_reward_per_share,
                    _last_reward_block,
                    _start_block,
                    _claimable_interval,
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

                let mut underlying_assets: Vec<Bson> = vec![];
                let mut farm_type = models::FarmType::StandardAmm;

                // let mut router = "".to_string();
                // if p.2.clone() == "moonriver".to_string() {
                //     router = constants::addresses::zenlink_on_moonriver::ZENLINK_ROUTER.to_string();
                // } else if p.2.clone() == "moonbeam".to_string() {
                //     router = constants::addresses::zenlink_on_moonbeam::ZENLINK_ROUTER.to_string();
                // } else if p.2.clone() == "astar".to_string() {
                //     router = constants::addresses::zenlink_on_astar::ZENLINK_ROUTER.to_string();
                // }

                let asset_filter = doc! { "address": ft_addr.clone(), "chain": p.2.clone(), "protocol": p.3.clone() };

                let asset = assets_collection.find_one(asset_filter, None).await?;

                if asset.is_some() {
                    for ua in asset.clone().unwrap().underlying_assets {
                        underlying_assets.push(bson!({
                            "symbol": ua.symbol,
                            "address": ua.address,
                            "decimals": ua.decimals,
                        }))
                    }
                }

                if pid == 3 && p.2.clone() == "astar".to_string() {
                    farm_type = models::FarmType::StableAmm;

                    let stable_asset =
                        contracts::IStableLpToken::new(farming_token, Arc::clone(&p.8.clone()));

                    let owner_addr: Address = stable_asset.owner().call().await?;
                    let stable_owner_addr =
                        ethers::utils::to_checksum(&owner_addr.to_owned(), None);
                    router = stable_owner_addr.clone();

                    let owner =
                        contracts::IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
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

                    let bai = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::zenlink_on_astar::BAI.parse::<Address>()?,
                        p.8.clone(),
                    );
                    let busd = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::zenlink_on_astar::BUSD.parse::<Address>()?,
                        p.8.clone(),
                    );
                    let dai = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::zenlink_on_astar::DAI.parse::<Address>()?,
                        p.8.clone(),
                    );
                    let usdc = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::zenlink_on_astar::USDC.parse::<Address>()?,
                        p.8.clone(),
                    );

                    let bai_filter = doc! {"chain": p.2.clone(), "protocol": p.3.clone(), "address": constants::addresses::zenlink_on_astar::BAI};
                    let bai_asset = assets_collection.find_one(bai_filter, None).await?;
                    let busd_filter = doc! {"chain": p.2.clone(), "protocol": p.3.clone(), "address": constants::addresses::zenlink_on_astar::BUSD};
                    let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                    let dai_filter = doc! {"chain": p.2.clone(), "protocol": p.3.clone(), "address": constants::addresses::zenlink_on_astar::DAI};
                    let dai_asset = assets_collection.find_one(dai_filter, None).await?;
                    let usdc_filter = doc! {"chain": p.2.clone(), "protocol": p.3.clone(), "address": constants::addresses::zenlink_on_astar::USDC};
                    let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;

                    let bai_bal: U256 = bai.balance_of(owner_addr).call().await?;
                    let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                    let dai_bal: U256 = dai.balance_of(owner_addr).call().await?;
                    let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;

                    let _4pool = contracts::IStableLpToken::new(
                        ft_addr.parse::<Address>()?,
                        Arc::clone(&p.8.clone()),
                    );
                    let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                    underlying_assets = vec![
                        bson!({
                            "symbol": bai_asset.clone().unwrap().symbol,
                            "address":  bai_asset.clone().unwrap().address,
                            "decimals": bai_asset.clone().unwrap().decimals,
                        }),
                        bson!({
                            "symbol": busd_asset.clone().unwrap().symbol,
                            "address":  busd_asset.clone().unwrap().address,
                            "decimals": busd_asset.clone().unwrap().decimals,
                        }),
                        bson!({
                            "symbol": dai_asset.clone().unwrap().symbol,
                            "address":  dai_asset.clone().unwrap().address,
                            "decimals": dai_asset.clone().unwrap().decimals,
                        }),
                        bson!({
                            "symbol": usdc_asset.clone().unwrap().symbol,
                            "address":  usdc_asset.clone().unwrap().address,
                            "decimals": usdc_asset.clone().unwrap().decimals,
                        }),
                    ];

                    let usd_pool_liq = bai_bal.as_u128() as f64 * bai_asset.clone().unwrap().price
                        / constants::utils::TEN_F64.powf(18.0)
                        + busd_bal.as_u128() as f64 * busd_asset.clone().unwrap().price
                            / constants::utils::TEN_F64.powf(18.0)
                        + dai_bal.as_u128() as f64 * dai_asset.clone().unwrap().price
                            / constants::utils::TEN_F64.powf(18.0)
                        + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                            / constants::utils::TEN_F64.powf(6.0);
                    println!("4pool usd_pool_liq {}", usd_pool_liq);
                    let total_supply: U256 = stable_asset.total_supply().call().await?;
                    let ts = total_supply.as_u128() as f64 / constants::utils::TEN_F64.powf(18.0);

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
                            "underlyingAssets": underlying_assets.clone(),
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
                } else if pid == 11 && p.2.clone() == "moonriver".to_string() {
                    farm_type = models::FarmType::StableAmm;

                    let stable_asset =
                        contracts::IStableLpToken::new(farming_token, Arc::clone(&p.8.clone()));

                    let owner_addr: Address = stable_asset.owner().call().await?;
                    let stable_owner_addr =
                        ethers::utils::to_checksum(&owner_addr.to_owned(), None);
                    router = stable_owner_addr.clone();

                    let owner =
                        contracts::IStableLpTokenOwner::new(owner_addr, Arc::clone(&p.8.clone()));
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

                    let usdt = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::solarbeam_on_moonriver::USDT.parse::<Address>()?,
                        p.8.clone(),
                    );
                    let frax = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::solarbeam_on_moonriver::FRAX.parse::<Address>()?,
                        p.8.clone(),
                    );
                    let usdc = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::zenlink_on_moonriver::USDC.parse::<Address>()?,
                        p.8.clone(),
                    );
                    let xcausd = contracts::IAnyswapV5ERC20::new(
                        constants::addresses::zenlink_on_moonriver::XCAUSD.parse::<Address>()?,
                        p.8.clone(),
                    );

                    let usdt_filter = doc! {"chain": p.2.clone(), "protocol": "solarbeam", "address": constants::addresses::solarbeam_on_moonriver::USDT};
                    let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;
                    let frax_filter = doc! {"chain": p.2.clone(), "protocol": "solarbeam", "address": constants::addresses::solarbeam_on_moonriver::FRAX};
                    let frax_asset = assets_collection.find_one(frax_filter, None).await?;
                    let usdc_filter = doc! {"chain": p.2.clone(), "protocol": p.3.clone(), "address": constants::addresses::zenlink_on_moonriver::USDC};
                    let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                    let xcausd_filter = doc! {"chain": p.2.clone(), "protocol": p.3.clone(), "address": constants::addresses::zenlink_on_moonriver::XCAUSD};
                    let xcausd_asset = assets_collection.find_one(xcausd_filter, None).await?;

                    let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;
                    let frax_bal: U256 = frax.balance_of(owner_addr).call().await?;
                    let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                    let xcausd_bal: U256 = xcausd.balance_of(owner_addr).call().await?;

                    let _4pool = contracts::IStableLpToken::new(
                        ft_addr.parse::<Address>()?,
                        Arc::clone(&p.8.clone()),
                    );
                    let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                    underlying_assets = vec![
                        bson!({
                            "symbol": usdt_asset.clone().unwrap().symbol,
                            "address":  usdt_asset.clone().unwrap().address,
                            "decimals": usdt_asset.clone().unwrap().decimals,
                        }),
                        bson!({
                            "symbol": frax_asset.clone().unwrap().symbol,
                            "address":  frax_asset.clone().unwrap().address,
                            "decimals": frax_asset.clone().unwrap().decimals,
                        }),
                        bson!({
                            "symbol": usdc_asset.clone().unwrap().symbol,
                            "address":  usdc_asset.clone().unwrap().address,
                            "decimals": usdc_asset.clone().unwrap().decimals,
                        }),
                        bson!({
                            "symbol": xcausd_asset.clone().unwrap().symbol,
                            "address":  xcausd_asset.clone().unwrap().address,
                            "decimals": xcausd_asset.clone().unwrap().decimals,
                        }),
                    ];

                    let usd_pool_liq = usdt_bal.as_u128() as f64
                        * usdt_asset.clone().unwrap().price
                        / constants::utils::TEN_F64.powf(6.0)
                        + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                            / constants::utils::TEN_F64.powf(18.0)
                        + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                            / constants::utils::TEN_F64.powf(6.0)
                        + xcausd_bal.as_u128() as f64 * xcausd_asset.clone().unwrap().price
                            / constants::utils::TEN_F64.powf(12.0);
                    println!("4pool usd_pool_liq {}", usd_pool_liq);
                    let total_supply: U256 = stable_asset.total_supply().call().await?;
                    let ts = total_supply.as_u128() as f64 / constants::utils::TEN_F64.powf(18.0);

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
                            "underlyingAssets": underlying_assets.clone(),
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
                } else if pid == 1 && p.2.clone() == "moonriver".to_string() {
                    // zlk on moonriver
                    farm_type = models::FarmType::SingleStaking;
                    underlying_assets = vec![bson!({
                        "symbol": "ZLK",
                        "address":  constants::addresses::zenlink_on_moonriver::ZLK,
                        "decimals": 18,
                    })];
                } else if pid == 1 && p.2.clone() == "moonbeam".to_string() {
                    // zlk on moonbeam
                    farm_type = models::FarmType::SingleStaking;
                    underlying_assets = vec![bson!({
                        "symbol": "ZLK",
                        "address":  constants::addresses::zenlink_on_moonbeam::ZLK,
                        "decimals": 18,
                    })];
                }

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

                        let reward_asset_filter = doc! { "address": reward_asset_addr, "chain": p.2.clone(), "protocol": p.3.clone() };
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

                            if rewards_per_day != 0 {
                                if !reward_asset_map
                                    .contains_key(&reward_asset.clone().unwrap().symbol)
                                {
                                    reward_asset_map.insert(
                                        reward_asset.clone().unwrap().symbol,
                                        (
                                            true,
                                            rewards_per_day as f64
                                                / constants::utils::TEN_I128
                                                    .pow(reward_asset.clone().unwrap().decimals)
                                                    as f64,
                                            (rewards_per_day as f64
                                                / constants::utils::TEN_I128
                                                    .pow(reward_asset.clone().unwrap().decimals)
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
                                                / constants::utils::TEN_I128
                                                    .pow(reward_asset.clone().unwrap().decimals)
                                                    as f64,
                                            er.2 + (rewards_per_day as f64
                                                / constants::utils::TEN_I128
                                                    .pow(reward_asset.clone().unwrap().decimals)
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

                                let mut reward_apr = ((rewards_per_day as f64
                                    / constants::utils::TEN_I128
                                        .pow(reward_asset.clone().unwrap().decimals)
                                        as f64
                                    * reward_asset_price)
                                    / (asset_tvl as f64 * asset_price))
                                    * 365.0
                                    * 100.0;
                                if farm_type == models::FarmType::SingleStaking
                                    || farm_type == models::FarmType::StableAmm
                                {
                                    reward_apr *= constants::utils::TEN_I128.pow(18) as f64;
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

                    let mut atvl: f64 = asset_tvl as f64 * asset_price;
                    if farm_type == models::FarmType::SingleStaking
                        || farm_type == models::FarmType::StableAmm
                    {
                        atvl =
                            asset_tvl as f64 * asset_price / constants::utils::TEN_F64.powf(18.0);
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

                        let timestamp = Utc::now().to_string();

                        if pid != 12 && p.2.clone() == "moonriver".to_string() {
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
                                    "router": router,
                                    "asset": {
                                        "symbol": asset.clone().unwrap().symbol,
                                        "address": asset.clone().unwrap().address,
                                        "price": asset.clone().unwrap().price,
                                        "logos": asset.clone().unwrap().logos,
                                        "underlyingAssets": underlying_assets.clone(),
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
                }
            } else if p.4.clone() == "v0".to_string() {
                let sushi_mini_chef_address =
                    constants::addresses::sushi_on_moonriver::SUSHI_MINI_CHEF.parse::<Address>()?;
                let sushi_mini_chef = contracts::IMiniChefV2::new(
                    sushi_mini_chef_address,
                    Arc::clone(&moonriver_client),
                );

                // TODO: fetch this address from minichef contract
                // right now hardcoding to prevent repeated calls (same rewarder is used for all pids)
                let sushi_complex_rewarder_address =
                    constants::addresses::sushi_on_moonriver::SUSHI_COMPLEX_REWARDER
                        .parse::<Address>()?;
                let sushi_complex_rewarder = contracts::IComplexRewarderTime::new(
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

                let mut underlying_assets: Vec<Bson> = vec![];
                let farm_type = models::FarmType::StandardAmm;
                let farm_implementation = models::FarmImplementation::Solidity;

                // if ap > 0 {
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
                let reward_asset_map: HashMap<String, (bool, f64, f64, String)> = HashMap::new();
                let mut total_reward_apr = 0.0;

                if asset.is_some() {
                    println!("asset: {:?}", asset.clone().unwrap().symbol);
                    // asset.clone().unwrap().under
                    let lp = contracts::ILpToken::new(lp_token, Arc::clone(&moonriver_client));
                    lp.token_0().call().await;
                    let sps: U256 = sushi_mini_chef.sushi_per_second().call().await?;
                    let tap: U256 = sushi_mini_chef.total_alloc_point().call().await?;
                    let rps: U256 = sushi_complex_rewarder.reward_per_second().call().await?;

                    let sushi_filter = doc! {"address": constants::addresses::sushi_on_moonriver::SUSHI,"protocol":"sushiswap","chain":"moonriver"};
                    let sushi = assets_collection.find_one(sushi_filter, None).await?;

                    let movr_filter = doc! {"address": constants::addresses::sushi_on_moonriver::MOVR,"protocol":"sushiswap","chain":"moonriver"};
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

                            if rewards_per_day != 0.0 {
                                rewards.push(bson!({
                                        "amount": rewards_per_day as f64 / constants::utils::TEN_I128.pow(sushi.clone().unwrap().decimals) as f64,
                                        "asset":  sushi.clone().unwrap().symbol,
                                        "valueUSD": (rewards_per_day as f64 / constants::utils::TEN_I128.pow(sushi.clone().unwrap().decimals) as f64) * reward_asset_price,
                                        "freq": models::Freq::Daily.to_string(),
                                    }));

                                // reward_apr/farm_apr/pool_apr
                                println!(
                                    "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                    rewards_per_sec, rewards_per_day, asset_tvl
                                );

                                let reward_apr = ((rewards_per_day as f64 * reward_asset_price)
                                    / (asset_tvl as f64
                                        * constants::utils::TEN_I128
                                            .pow(sushi.clone().unwrap().decimals)
                                            as f64))
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

                            if rewards_per_day != 0.0 {
                                rewards.push(bson!({
                                        "amount": rewards_per_day as f64 / constants::utils::TEN_I128.pow(movr.clone().unwrap().decimals) as f64,
                                        "asset":  movr.clone().unwrap().symbol,
                                        "valueUSD": (rewards_per_day as f64 / constants::utils::TEN_I128.pow(movr.clone().unwrap().decimals) as f64) * reward_asset_price,
                                        "freq": models::Freq::Daily.to_string(),
                                    }));

                                // reward_apr/farm_apr/pool_apr
                                println!(
                                    "rewards/sec: {} rewards/day: {} asset_tvl: {}",
                                    rewards_per_sec, rewards_per_day, asset_tvl
                                );

                                let reward_apr = ((rewards_per_day as f64 * reward_asset_price)
                                    / (asset_tvl as f64
                                        * constants::utils::TEN_I128
                                            .pow(movr.clone().unwrap().decimals)
                                            as f64))
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

                        let mut uas = vec![];
                        for ua in asset.clone().unwrap().underlying_assets {
                            uas.push(bson!({
                                "symbol": ua.symbol,
                                "address": ua.address,
                                "decimals": ua.decimals,
                            }))
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
                                "router": router,
                                "asset": {
                                    "symbol": asset.clone().unwrap().symbol,
                                    "address": asset_addr.clone(),
                                    "price": asset.clone().unwrap().price,
                                    "logos": asset.clone().unwrap().logos,
                                    "underlyingAssets": uas,
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
                // } else {
                //     println!("allocPoint = 0");

                //     let timestamp = Utc::now().to_string();

                //     println!("chef v0 farm lastUpdatedAtUTC {}", timestamp.clone());

                //     let ff = doc! {
                //         "id": pid as i32,
                //         "chef": p.5.clone(),
                //         "chain": p.2.clone(),
                //         "protocol": p.3.clone(),
                //     };
                //     let fu = doc! {
                //         "$set" : {
                //             "id": pid,
                //             "chef": p.5.clone(),
                //             "chain": p.2.clone(),
                //             "protocol": p.3.clone(),
                //             "farmType": farm_type.to_string(),
                //             "farmImpl": farm_implementation.to_string(),
                //             "asset": {
                //                 "symbol": "",
                //                 "address": "",
                //                 "price": 0,
                //                 "logos": [],
                //             },
                //             "tvl": 0,
                //             "apr.reward": 0,
                //             "apr.base": 0,
                //             "rewards": [],
                //             "allocPoint": ap,
                //             "lastUpdatedAtUTC": timestamp.clone(),
                //         }
                //     };
                //     let options = FindOneAndUpdateOptions::builder()
                //         .upsert(Some(true))
                //         .build();
                //     farms_collection
                //         .find_one_and_update(ff, fu, Some(options))
                //         .await?;
                // }
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

                let mut underlying_assets: Vec<Bson> = vec![];
                let mut farm_type = models::FarmType::StandardAmm;
                let farm_implementation = models::FarmImplementation::Solidity;

                if ap > 0 {
                    if p.4.clone() == "v1".to_string() {
                        // chef v1
                        let asset_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                        println!("asset_addr: {:?}", asset_addr.clone());

                        let stella_chef_v1_address =
                            constants::addresses::stellaswap_on_moonbeam::STELLA_CHEF_V1
                                .parse::<Address>()?;
                        let stella_chef_v1 = contracts::IStellaDistributorV1::new(
                            stella_chef_v1_address,
                            Arc::clone(&moonbeam_client),
                        );

                        let asset_filter = doc! { "address": asset_addr.clone(), "protocol": p.3.clone(), "chain": p.2.clone() };
                        let asset = assets_collection.find_one(asset_filter, None).await?;

                        let asset_price: f64;
                        let asset_tvl: u128;

                        let mut rewards = vec![];
                        // <symbol, (exists, amount, valueUSD, freq)>
                        let reward_asset_map: HashMap<String, (bool, f64, f64, String)> =
                            HashMap::new();

                        if asset.is_some() {
                            println!("asset: {:?}", asset.clone().unwrap().symbol);
                            let spb: U256 = stella_chef_v1.stella_per_block().call().await?;
                            let tap: U256 = stella_chef_v1.total_alloc_point().call().await?;

                            let average_block_time = 12.4;
                            let stella_filter = doc! {"address":constants::addresses::stellaswap_on_moonbeam::STELLA, "protocol":p.3.clone(), "chain":p.2.clone()};
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

                                if rewards_per_day != 0.0 {
                                    rewards.push(bson!({
                                        "amount": rewards_per_day as f64 / constants::utils::TEN_I128.pow(stella.clone().unwrap().decimals) as f64,
                                        "asset":  stella.clone().unwrap().symbol,
                                        "valueUSD": (rewards_per_day as f64 / constants::utils::TEN_I128.pow(stella.clone().unwrap().decimals) as f64) * reward_asset_price,
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

                                let mut uas = vec![];
                                for ua in asset.clone().unwrap().underlying_assets {
                                    uas.push(bson!({
                                        "symbol": ua.symbol,
                                        "address": ua.address,
                                        "decimals": ua.decimals,
                                    }))
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
                                let fu = doc! {
                                    "$set" : {
                                        "id": pid,
                                        "chef": p.5.clone(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "farmType": farm_type.to_string(),
                                        "farmImpl": farm_implementation.to_string(),
                                        "router": router,
                                        "asset": {
                                            "symbol": asset.clone().unwrap().symbol,
                                            "address": asset_addr.clone(),
                                            "price": asset.clone().unwrap().price,
                                            "logos": asset.clone().unwrap().logos,
                                            "underlyingAssets": uas,
                                        },
                                        "tvl": asset_tvl as f64 * asset_price / constants::utils::TEN_F64.powf(18.0),
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

                        let mut stable_owner_addr = "".to_string();

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
                                contracts::IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let symbol: String = stable_asset.symbol().call().await?;

                            let owner_addr: Address = stable_asset.owner().call().await?;
                            stable_owner_addr =
                                ethers::utils::to_checksum(&owner_addr.to_owned(), None);
                            router = stable_owner_addr.clone();

                            let owner = contracts::IStableLpTokenOwner::new(
                                owner_addr,
                                Arc::clone(&p.8.clone()),
                            );
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

                            let busd = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::BUSD
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::USDC
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::USDT
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let frax = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::FRAX
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mai = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::MAI
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mim = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::MIM
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let wbtc = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::WBTC
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let xckbtc = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::XCKBTC
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let xcksm = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::XCKSM
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let stksm = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::solarbeam_on_moonriver::STKSM
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::BUSD};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::USDC};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::USDT};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;

                            let frax_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::FRAX};
                            let frax_asset = assets_collection.find_one(frax_filter, None).await?;
                            let mai_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::MAI};
                            let mai_asset = assets_collection.find_one(mai_filter, None).await?;
                            let mim_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::MIM};
                            let mim_asset = assets_collection.find_one(mim_filter, None).await?;

                            let wbtc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::WBTC};
                            let wbtc_asset = assets_collection.find_one(wbtc_filter, None).await?;
                            let xckbtc_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::XCKBTC};
                            let xckbtc_asset =
                                assets_collection.find_one(xckbtc_filter, None).await?;

                            let xcksm_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::XCKSM};
                            let xcksm_asset =
                                assets_collection.find_one(xcksm_filter, None).await?;
                            let stksm_filter = doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::STKSM};
                            let stksm_asset =
                                assets_collection.find_one(stksm_filter, None).await?;

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

                            let _3pool = contracts::IStableLpToken::new(
                                constants::addresses::solarbeam_on_moonriver::_3POOL
                                    .parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            let _3pool_bal: U256 = _3pool.balance_of(owner_addr).call().await?;

                            // TODO: calculate underlyingAssetsAlloc

                            if symbol == "3pool".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0);
                                println!("3pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::solarbeam_on_moonriver::_3POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::solarbeam_on_moonriver::_3POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "FRAX-3pool".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": frax_asset.clone().unwrap().symbol,
                                        "address":  frax_asset.clone().unwrap().address,
                                        "decimals": frax_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = _3pool_bal.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::solarbeam_on_moonriver::FRAX_3POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::solarbeam_on_moonriver::FRAX_3POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "MAI-3pool".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": mai_asset.clone().unwrap().symbol,
                                        "address":  mai_asset.clone().unwrap().address,
                                        "decimals": mai_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = _3pool_bal.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + mai_bal.as_u128() as f64 * mai_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::solarbeam_on_moonriver::MAI_3POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::solarbeam_on_moonriver::MAI_3POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "MIM-3pool".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": mim_asset.clone().unwrap().symbol,
                                        "address":  mim_asset.clone().unwrap().address,
                                        "decimals": mim_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = _3pool_bal.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + mim_bal.as_u128() as f64 * mim_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(18.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::solarbeam_on_moonriver::MIM_3POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::solarbeam_on_moonriver::MIM_3POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "kBTC-BTC".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": wbtc_asset.clone().unwrap().symbol,
                                        "address":  wbtc_asset.clone().unwrap().address,
                                        "decimals": wbtc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": xckbtc_asset.clone().unwrap().symbol,
                                        "address":  xckbtc_asset.clone().unwrap().address,
                                        "decimals": xckbtc_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let wbtc_price = wbtc_asset.clone().unwrap().price;
                                let xckbtc_price = xckbtc_asset.clone().unwrap().price;
                                let pool_liq = wbtc_bal.as_u128() as f64 * wbtc_price
                                    / constants::utils::TEN_F64.powf(8.0)
                                    + xckbtc_bal.as_u128() as f64 * xckbtc_price
                                        / constants::utils::TEN_F64.powf(8.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let pool_price = pool_liq / ts;
                                println!("pool_price {}", pool_price);

                                let f = doc! {
                                    "address": constants::addresses::solarbeam_on_moonriver::KBTC_BTC.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::solarbeam_on_moonriver::KBTC_BTC.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "stKSM".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": xcksm_asset.clone().unwrap().symbol,
                                        "address":  xcksm_asset.clone().unwrap().address,
                                        "decimals": xcksm_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": stksm_asset.clone().unwrap().symbol,
                                        "address":  stksm_asset.clone().unwrap().address,
                                        "decimals": stksm_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let pool_liq = xcksm_bal.as_u128() as f64
                                    * xcksm_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(12.0)
                                    + stksm_bal.as_u128() as f64
                                        * stksm_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(12.0);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let pool_price = pool_liq / ts;
                                println!("pool_price {}", pool_price);

                                let f = doc! {
                                    "address": constants::addresses::solarbeam_on_moonriver::STKSM_POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::solarbeam_on_moonriver::STKSM_POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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

                        // 4pool
                        if p.3.clone() == "beamswap".to_string() && (pid == 16) {
                            farm_type = models::FarmType::StableAmm;

                            let stable_asset =
                                contracts::IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let symbol: String = stable_asset.symbol().call().await?;

                            let owner_addr: Address = stable_asset.owner().call().await?;
                            stable_owner_addr =
                                ethers::utils::to_checksum(&owner_addr.to_owned(), None);
                            router = stable_owner_addr.clone();

                            let owner = contracts::IStableLpTokenOwner::new(
                                owner_addr,
                                Arc::clone(&p.8.clone()),
                            );
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

                            let busd = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::beamswap_on_moonbeam::BUSD
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::beamswap_on_moonbeam::USDC
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::beamswap_on_moonbeam::USDT
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let dai = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::beamswap_on_moonbeam::DAI
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":constants::addresses::beamswap_on_moonbeam::BUSD};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":constants::addresses::beamswap_on_moonbeam::USDC};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":constants::addresses::beamswap_on_moonbeam::USDT};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;
                            let dai_filter = doc! {"chain":"moonbeam", "protocol":"beamswap", "address":constants::addresses::beamswap_on_moonbeam::DAI};
                            let dai_asset = assets_collection.find_one(dai_filter, None).await?;

                            let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                            let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                            let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;
                            let dai_bal: U256 = dai.balance_of(owner_addr).call().await?;

                            let _4pool = contracts::IStableLpToken::new(
                                constants::addresses::beamswap_on_moonbeam::_4POOL
                                    .parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );

                            if symbol == "4pool".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": dai_asset.clone().unwrap().symbol,
                                        "address":  dai_asset.clone().unwrap().address,
                                        "decimals": dai_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0)
                                    + dai_bal.as_u128() as f64 * dai_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(18.0);
                                println!("4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::beamswap_on_moonbeam::_4POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::beamswap_on_moonbeam::_4POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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

                        // stellaswap stable
                        if p.3.clone() == "stellaswap".to_string()
                            && p.4.clone() == "v2"
                            && (pid == 31
                                || pid == 33
                                || pid == 34
                                || pid == 35
                                || pid == 37
                                || pid == 38
                                || pid == 39)
                        {
                            farm_type = models::FarmType::StableAmm;

                            let stable_asset =
                                contracts::IStableLpToken::new(lp_token, Arc::clone(&p.8.clone()));
                            let symbol: String = stable_asset.symbol().call().await?;

                            let owner_addr: Address = stable_asset.owner().call().await?;
                            stable_owner_addr =
                                ethers::utils::to_checksum(&owner_addr.to_owned(), None);
                            router = stable_owner_addr.clone();

                            let owner = contracts::IStableLpTokenOwner::new(
                                owner_addr,
                                Arc::clone(&p.8.clone()),
                            );
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

                            let frax = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::stellaswap_on_moonbeam::FRAX
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let busd = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::stellaswap_on_moonbeam::BUSD
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdc = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::stellaswap_on_moonbeam::USDC
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let usdt = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::stellaswap_on_moonbeam::USDT
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let mai = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::stellaswap_on_moonbeam::MAI
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let athusd = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::stellaswap_on_moonbeam::ATH_USD
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );
                            let axlusdc = contracts::IAnyswapV5ERC20::new(
                                constants::addresses::stellaswap_on_moonbeam::AXL_USDC
                                    .parse::<Address>()?,
                                p.8.clone(),
                            );

                            let busd_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":constants::addresses::stellaswap_on_moonbeam::BUSD};
                            let busd_asset = assets_collection.find_one(busd_filter, None).await?;
                            let usdc_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":constants::addresses::stellaswap_on_moonbeam::USDC};
                            let usdc_asset = assets_collection.find_one(usdc_filter, None).await?;
                            let usdt_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":constants::addresses::stellaswap_on_moonbeam::USDT};
                            let usdt_asset = assets_collection.find_one(usdt_filter, None).await?;
                            let frax_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":constants::addresses::stellaswap_on_moonbeam::FRAX};
                            let frax_asset = assets_collection.find_one(frax_filter, None).await?;
                            let mai_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":constants::addresses::stellaswap_on_moonbeam::MAI};
                            let mai_asset = assets_collection.find_one(mai_filter, None).await?;
                            let athusd_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":constants::addresses::stellaswap_on_moonbeam::ATH_USD};
                            let athusd_asset =
                                assets_collection.find_one(athusd_filter, None).await?;
                            let axlusdc_filter = doc! {"chain":"moonbeam", "protocol":"stellaswap", "address":constants::addresses::stellaswap_on_moonbeam::AXL_USDC};
                            let axlusdc_asset =
                                assets_collection.find_one(axlusdc_filter, None).await?;

                            let busd_bal: U256 = busd.balance_of(owner_addr).call().await?;
                            let usdc_bal: U256 = usdc.balance_of(owner_addr).call().await?;
                            let usdt_bal: U256 = usdt.balance_of(owner_addr).call().await?;
                            let frax_bal: U256 = frax.balance_of(owner_addr).call().await?;
                            let mai_bal: U256 = mai.balance_of(owner_addr).call().await?;
                            let athusd_bal: U256 = athusd.balance_of(owner_addr).call().await?;
                            let axlusdc_bal: U256 = axlusdc.balance_of(owner_addr).call().await?;

                            let _4pool = contracts::IStableLpToken::new(
                                constants::addresses::stellaswap_on_moonbeam::_4POOL
                                    .parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            let _4pool_bal: U256 = _4pool.balance_of(owner_addr).call().await?;

                            let tripool = contracts::IStableLpToken::new(
                                constants::addresses::stellaswap_on_moonbeam::TRI_POOL
                                    .parse::<Address>()?,
                                Arc::clone(&p.8.clone()),
                            );
                            let tripool_bal: U256 = tripool.balance_of(owner_addr).call().await?;

                            if symbol == "stella4pool".to_string() {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": frax_asset.clone().unwrap().symbol,
                                        "address":  frax_asset.clone().unwrap().address,
                                        "decimals": frax_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = busd_bal.as_u128() as f64
                                    * busd_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0)
                                    + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(18.0);
                                println!("stella4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::stellaswap_on_moonbeam::_4POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::stellaswap_on_moonbeam::_4POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "stellaMAI-4pool" {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": mai_asset.clone().unwrap().symbol,
                                        "address":mai_asset.clone().unwrap().address,
                                        "decimals": mai_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": frax_asset.clone().unwrap().symbol,
                                        "address":  frax_asset.clone().unwrap().address,
                                        "decimals": frax_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = mai_bal.as_u128() as f64
                                    * mai_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + _4pool_bal.as_u128() as f64
                                        / constants::utils::TEN_F64.powf(18.0);

                                println!("stellaMAI-4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::stellaswap_on_moonbeam::MAI_4POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::stellaswap_on_moonbeam::MAI_4POOL.to_string(),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "stella-athUSD-4pool" {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": athusd_asset.clone().unwrap().symbol,
                                        "address":athusd_asset.clone().unwrap().address,
                                        "decimals": athusd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": frax_asset.clone().unwrap().symbol,
                                        "address":  frax_asset.clone().unwrap().address,
                                        "decimals": frax_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = athusd_bal.as_u128() as f64
                                    * athusd_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + _4pool_bal.as_u128() as f64
                                        / constants::utils::TEN_F64.powf(18.0);

                                println!("stella-athUSD-4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::stellaswap_on_moonbeam::ATH_USD_4POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::stellaswap_on_moonbeam::ATH_USD_4POOL.to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "StellaSwap athUSD-4pool".to_string(),
                                        "symbol": "athUSD-4pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            athusd_asset.clone().unwrap().logos.get(0),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "stella-axlUSDC-4pool" {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": axlusdc_asset.clone().unwrap().symbol,
                                        "address":axlusdc_asset.clone().unwrap().address,
                                        "decimals": axlusdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": busd_asset.clone().unwrap().symbol,
                                        "address":  busd_asset.clone().unwrap().address,
                                        "decimals": busd_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": frax_asset.clone().unwrap().symbol,
                                        "address":  frax_asset.clone().unwrap().address,
                                        "decimals": frax_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = axlusdc_bal.as_u128() as f64
                                    * axlusdc_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(6.0)
                                    + _4pool_bal.as_u128() as f64
                                        / constants::utils::TEN_F64.powf(18.0);

                                println!("stella-axlUSDC-4pool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::stellaswap_on_moonbeam::AXL_USDC_4POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::stellaswap_on_moonbeam::AXL_USDC_4POOL.to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "StellaSwap axlUSDC-4pool".to_string(),
                                        "symbol": "axlUSDC-4pool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            axlusdc_asset.clone().unwrap().logos.get(0),
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
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "stella-tripool" {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": frax_asset.clone().unwrap().symbol,
                                        "address":  frax_asset.clone().unwrap().address,
                                        "decimals": frax_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = usdc_bal.as_u128() as f64
                                    * usdc_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(6.0)
                                    + frax_bal.as_u128() as f64 * frax_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(18.0)
                                    + usdt_bal.as_u128() as f64 * usdt_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0);
                                println!("stella-tripool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::stellaswap_on_moonbeam::TRI_POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::stellaswap_on_moonbeam::TRI_POOL.to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "StellaSwap Tripool".to_string(),
                                        "symbol": "tripool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                            frax_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "stella-axlDualPool" {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": axlusdc_asset.clone().unwrap().symbol,
                                        "address":  axlusdc_asset.clone().unwrap().address,
                                        "decimals": axlusdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = axlusdc_bal.as_u128() as f64
                                    * axlusdc_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(6.0)
                                    + usdc_bal.as_u128() as f64 * usdc_asset.clone().unwrap().price
                                        / constants::utils::TEN_F64.powf(6.0);
                                println!("stella-axlDualPool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::stellaswap_on_moonbeam::AXL_DUAL_POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::stellaswap_on_moonbeam::AXL_DUAL_POOL.to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "StellaSwap Axelar Dual Pool".to_string(),
                                        "symbol": "axlDualPool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            axlusdc_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": underlying_assets.clone(),
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
                            } else if symbol == "stellaMAI-tripool" {
                                underlying_assets = vec![
                                    bson!({
                                        "symbol": mai_asset.clone().unwrap().symbol,
                                        "address":  mai_asset.clone().unwrap().address,
                                        "decimals": mai_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdc_asset.clone().unwrap().symbol,
                                        "address":  usdc_asset.clone().unwrap().address,
                                        "decimals": usdc_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": frax_asset.clone().unwrap().symbol,
                                        "address":  frax_asset.clone().unwrap().address,
                                        "decimals": frax_asset.clone().unwrap().decimals,
                                    }),
                                    bson!({
                                        "symbol": usdt_asset.clone().unwrap().symbol,
                                        "address":  usdt_asset.clone().unwrap().address,
                                        "decimals": usdt_asset.clone().unwrap().decimals,
                                    }),
                                ];

                                let usd_pool_liq = mai_bal.as_u128() as f64
                                    * mai_asset.clone().unwrap().price
                                    / constants::utils::TEN_F64.powf(18.0)
                                    + tripool_bal.as_u128() as f64
                                        / constants::utils::TEN_F64.powf(18.0);
                                println!("stellaMAI-tripool usd_pool_liq {}", usd_pool_liq);
                                let total_supply: U256 = stable_asset.total_supply().call().await?;
                                let ts = total_supply.as_u128() as f64
                                    / constants::utils::TEN_F64.powf(18.0);

                                let usd_pool_price = usd_pool_liq / ts;
                                println!("usd_pool_price {}", usd_pool_price);

                                let f = doc! {
                                    "address": constants::addresses::stellaswap_on_moonbeam::MAI_TRI_POOL.clone(),
                                    "chain": p.2.clone(),
                                    "protocol": p.3.clone(),
                                };

                                let timestamp = Utc::now().to_string();

                                let u = doc! {
                                    "$set" : {
                                        "address": constants::addresses::stellaswap_on_moonbeam::MAI_TRI_POOL.to_string(),
                                        "chain": p.2.clone(),
                                        "protocol": p.3.clone(),
                                        "name": "StellaSwap MAI Tripool".to_string(),
                                        "symbol": "MAI-tripool".to_string(),
                                        "decimals": 18,
                                        "logos": [
                                            mai_asset.clone().unwrap().logos.get(0),
                                            usdc_asset.clone().unwrap().logos.get(0),
                                            usdt_asset.clone().unwrap().logos.get(0),
                                            frax_asset.clone().unwrap().logos.get(0),
                                        ],
                                        "price": usd_pool_price,
                                        "liquidity": usd_pool_liq,
                                        "totalSupply": ts,
                                        "isLP": true,
                                        "feesAPR": 0.0,
                                        "underlyingAssets": underlying_assets.clone(),
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

                        if p.3.clone() == "beamswap".to_string() && p.4.clone() == "v2" && pid == 5
                        {
                            farm_type = models::FarmType::SingleStaking;
                        }

                        if p.3.clone() == "solarflare".to_string()
                            && p.4.clone() == "v2"
                            && pid == 3
                        {
                            farm_type = models::FarmType::SingleStaking;
                            underlying_assets = vec![bson!({
                                "symbol": "WGLMR",
                                "address":  constants::addresses::solarflare_on_moonbeam::WGLMR,
                                "decimals": 18,
                            })];
                        }

                        if rewards_per_sec.len() > 0 {
                            let mut total_reward_apr = 0.0;

                            let asset_addr = ethers::utils::to_checksum(&lp_token.to_owned(), None);
                            println!("asset_addr: {:?}", asset_addr.clone());

                            let asset_filter = doc! { "address": asset_addr.clone(), "protocol": p.3.clone(), "chain": p.2.clone() };
                            let asset = assets_collection.find_one(asset_filter, None).await?;

                            let mut uas = vec![];
                            for ua in asset.clone().unwrap().underlying_assets {
                                uas.push(bson!({
                                    "symbol": ua.symbol,
                                    "address": ua.address,
                                    "decimals": ua.decimals,
                                }))
                            }
                            if farm_type.to_string() == "StableAmm" {
                                println!("uassss {:?}", uas);
                            }

                            if underlying_assets.len() == 0 {
                                underlying_assets = uas;
                            }

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
                                            let solar_filter = doc! { "address": constants::addresses::solarbeam_on_moonriver::SOLAR, "protocol": "solarbeam", "chain": "moonriver" };
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

                                        if p.3.clone() == "beamswap".to_string() && pid == 24 {
                                            let dexscreener_pairs_rum_url="https://api.dexscreener.com/latest/dex/pairs/moonbeam/0x8A2982bA47Aa7a3A072E62930BEe8649B53a3dfe";

                                            let glmb_d2o_pairs =
                                                reqwest::get(dexscreener_pairs_rum_url)
                                                    .await?
                                                    .json::<apis::dx2::Root>()
                                                    .await?;

                                            asset_price = glmb_d2o_pairs
                                                .pair
                                                .price_usd
                                                .clone()
                                                .parse()
                                                .unwrap_or_default();
                                            let liq: u128 = (glmb_d2o_pairs.pair.liquidity.usd
                                                / asset_price)
                                                as u128;
                                            asset_tvl =
                                                liq * constants::utils::TEN_F64.powf(18.0) as u128;
                                            println!(
                                                "meow asset_price {:?} asset_tvl {:?}",
                                                asset_price, asset_tvl
                                            );
                                        }

                                        if p.3.clone() == "stellaswap".to_string()
                                            && p.5.clone()
                                                == constants::addresses::stellaswap_on_moonbeam::STELLA_CHEF_V2
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
                                                            / constants::utils::TEN_I128.pow(
                                                                reward_asset
                                                                    .clone()
                                                                    .unwrap()
                                                                    .decimals,
                                                            )
                                                                as f64,
                                                        (rewards_per_day as f64
                                                            / constants::utils::TEN_I128.pow(
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
                                                            / constants::utils::TEN_I128.pow(
                                                                reward_asset
                                                                    .clone()
                                                                    .unwrap()
                                                                    .decimals,
                                                            )
                                                                as f64,
                                                        er.2 + (rewards_per_day as f64
                                                            / constants::utils::TEN_I128.pow(
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
                                                / constants::utils::TEN_I128
                                                    .pow(decimals[i].as_u128().try_into().unwrap())
                                                    as f64
                                                * reward_asset_price)
                                                / (asset_tvl as f64 * asset_price
                                                    / constants::utils::TEN_I128.pow(18) as f64))
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
                                    "pddq {:?} addr {:?} stable_owner_addr {:?}",
                                    &constants::chef::PAIR_DAY_DATAS_QUERY.clone(),
                                    asset.clone().unwrap().address.to_lowercase(),
                                    stable_owner_addr.clone().to_lowercase()
                                );
                                if p.3.clone() == "solarbeam".to_string()
                                    && (pid == 8
                                        || pid == 9
                                        || pid == 13
                                        || pid == 16
                                        || pid == 17
                                        || pid == 25)
                                {
                                    println!("stablesolarbeam");
                                    let vars = Vars {
                                        addr: stable_owner_addr.clone().to_lowercase(),
                                    };
                                    let swap_data =  solarbeam_stable_subgraph_client.query_with_vars_unwrap::<subgraph::SolarbeamStableData, Vars>(
                                            &constants::chef::SOLARBEAM_STABLE_SWAPS_DAY_DATA_QUERY.clone(),
                                            vars,
                                        )
                                        .await;

                                    if swap_data.is_ok() {
                                        println!(
                                            "solarbeam swap_data {:?}",
                                            swap_data.clone().unwrap()
                                        );
                                        let mut daily_volume_lw: f64 = 0.0;
                                        for pdd in swap_data.clone().unwrap().swap.daily_data {
                                            let dv: f64 = pdd.volume.parse().unwrap_or_default();

                                            daily_volume_lw += dv * asset.clone().unwrap().price;
                                        }
                                        println!("daily_volume_lw {:?}", daily_volume_lw);
                                        daily_volume_lw /=
                                            swap_data.clone().unwrap().swap.daily_data.len() as f64;
                                        println!("daily_volume_lwad {:?}", daily_volume_lw);

                                        if asset.clone().unwrap_or_default().total_supply == 0.0
                                            || asset.clone().unwrap_or_default().price == 0.0
                                        {
                                            println!("ts0 or p0");
                                            base_apr = 0.0;
                                        } else {
                                            base_apr = daily_volume_lw * 0.002 * 365.0 * 100.0
                                                / (asset.clone().unwrap_or_default().total_supply
                                                    * asset.clone().unwrap_or_default().price);
                                        }
                                    }
                                } else if p.3.clone() == "stellaswap".to_string()
                                    && p.4.clone() == "v2"
                                    && (pid == 31 || pid == 33 || pid == 34 || pid == 35)
                                {
                                    println!("stablestellaswap");
                                    let vars = Vars {
                                        addr: stable_owner_addr.clone().to_lowercase(),
                                    };
                                    let swap_data = stellaswap_stable_subgraph_client.query_with_vars_unwrap::<subgraph::StellaStableData, Vars>(
                                            &constants::chef::STELLASWAP_STABLE_SWAPS_DAY_DATA_QUERY.clone(),
                                            vars,
                                        )
                                        .await;

                                    if swap_data.is_ok() {
                                        println!(
                                            "stellaswap swap_data {:?}",
                                            swap_data.clone().unwrap()
                                        );
                                        let mut daily_volume_lw: f64 = 0.0;
                                        for pdd in swap_data.clone().unwrap().swap.daily_volumes {
                                            let dv: f64 = pdd.volume.parse().unwrap_or_default();

                                            daily_volume_lw += dv * asset.clone().unwrap().price;
                                        }
                                        println!("daily_volume_lw {:?}", daily_volume_lw);
                                        daily_volume_lw /=
                                            swap_data.clone().unwrap().swap.daily_volumes.len()
                                                as f64;
                                        println!("daily_volume_lwad {:?}", daily_volume_lw);

                                        if asset.clone().unwrap_or_default().total_supply == 0.0
                                            || asset.clone().unwrap_or_default().price == 0.0
                                        {
                                            println!("ts0 or p0");
                                            base_apr = 0.0;
                                        } else {
                                            base_apr = daily_volume_lw * 0.002 * 365.0 * 100.0
                                                / (asset.clone().unwrap_or_default().total_supply
                                                    * asset.clone().unwrap_or_default().price);
                                        }
                                    } else {
                                        println!("swap_dataerr {:?}", swap_data);
                                    }
                                } else {
                                    println!("notstable");
                                    let pair_day_datas =
                                        p.6.query_with_vars_unwrap::<subgraph::PairDayDatas, Vars>(
                                            &constants::chef::PAIR_DAY_DATAS_QUERY.clone(),
                                            vars,
                                        )
                                        .await;

                                    let usdc_nomad_solarflare_filter = doc! { "address": constants::addresses::beamswap_on_moonbeam::USDC, "protocol": "solarflare", "chain": "moonbeam" };
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
                                                    println!("dv {:?} {:?}", i, ua.clone().address);
                                                    let ua_filter = doc! { "address": ua.clone().address, "protocol": p.3.clone(), "chain": p.2.clone() };
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
                                                            * ua_obj
                                                                .clone()
                                                                .unwrap_or_default()
                                                                .price;
                                                    } else if i == 1 {
                                                        daily_volume_lw += dvt1
                                                            * ua_obj
                                                                .clone()
                                                                .unwrap_or_default()
                                                                .price;
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
                                            if p.3.clone() == "beamswap".to_string() && pid == 24 {
                                                println!("meowbaseapr");
                                                let dexscreener_pairs_rum_url="https://api.dexscreener.com/latest/dex/pairs/moonbeam/0x8A2982bA47Aa7a3A072E62930BEe8649B53a3dfe";

                                                let glmb_d2o_pairs =
                                                    reqwest::get(dexscreener_pairs_rum_url)
                                                        .await?
                                                        .json::<apis::dx2::Root>()
                                                        .await?;

                                                base_apr = glmb_d2o_pairs.pair.volume.h24
                                                    * 0.002
                                                    * 365.0
                                                    * 100.0
                                                    / (asset
                                                        .clone()
                                                        .unwrap_or_default()
                                                        .total_supply
                                                        * asset_price);
                                                println!("meowbase_apr {:?}", base_apr);
                                            }
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
                                }
                                if base_apr.is_nan() {
                                    base_apr = 0.0;
                                }

                                let timestamp = Utc::now().to_string();

                                if !(p.3.clone() == "solarbeam".to_string()
                                    && (pid == 39 || pid == 40 || pid == 43))
                                {
                                    println!("chef v2 farm lastUpdatedAtUTC {}", timestamp.clone());

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
                                            "router": router,
                                            "asset": {
                                                "symbol": asset.clone().unwrap().symbol,
                                                "address": asset_addr.clone(),
                                                "price": asset.clone().unwrap().price,
                                                "logos": asset.clone().unwrap().logos,
                                                "underlyingAssets": underlying_assets,
                                            },
                                            "tvl": asset_tvl as f64 * asset_price / constants::utils::TEN_F64.powf(18.0),
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
                            "router": router,
                            "asset": {
                                "symbol": "",
                                "address": "",
                                "price": 0,
                                "logos": [],
                                "underlyingAssets": [],
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

    let ldo_price = reqwest::get(
        "https://api.coingecko.com/api/v3/simple/price?ids=lido-dao&vs_currencies=usd",
    )
    .await?
    .json::<apis::coingecko::LDORoot>()
    .await?;

    println!("ldo_price {:?}", ldo_price);
    let ldo_p = ldo_price.lido_dao.usd;
    println!("ldo_price {:?}", ldo_p);

    let f = doc! {
        "address": constants::addresses::beamswap_on_moonbeam::LDO,
        "chain": "moonbeam",
        "protocol": "beamswap",
    };

    let timestamp = Utc::now().to_string();

    let u = doc! {
        "$set" : {
            "address": constants::addresses::beamswap_on_moonbeam::LDO,
            "chain": "moonbeam",
            "protocol": "beamswap",
            "name": "Lido DAO Token",
            "symbol": "LDO",
            "decimals": 18,
            "logos": [
                "https://raw.githubusercontent.com/yield-bay/assets/main/list/LDO.png",
            ],
            "price": ldo_p,
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

    let arsw_price_resp = reqwest::get(
        "https://api.coingecko.com/api/v3/simple/price?ids=arthswap&vs_currencies=usd",
    )
    .await?;
    match arsw_price_resp.status() {
        reqwest::StatusCode::OK => {
            // on success, parse our JSON to an APIResponse
            match arsw_price_resp.json::<apis::coingecko::ASRoot>().await {
                Ok(arsw_price) => {
                    println!("Success! {:?}", arsw_price);
                    let arsw_p = arsw_price.arthswap.usd;
                    println!("arsw_price {:?}", arsw_p);

                    let f = doc! {
                        "address": constants::addresses::arthswap_on_astar::ARSW,
                        "chain": "astar",
                        "protocol": "arthswap",
                    };

                    let timestamp = Utc::now().to_string();

                    let u = doc! {
                        "$set" : {
                            "address": constants::addresses::arthswap_on_astar::ARSW,
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
                }
                Err(_) => println!("Hm, the response didn't match the shape we expected."),
            };
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            println!("Need to grab a new token");
        }
        other => {
            println!("Uh oh! Something unexpected happened: {:?}", other);
        }
    }

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

    println!("apl {:?}", arthswap_pairs.pairs.len());

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

            let token0decimals: u32 = 18; //: u32 = pair.base_token.decimals.parse().unwrap_or_default();
            let token1decimals: u32 = 18; //: u32 = pair.quote_token.decimals.parse().unwrap_or_default();

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

            println!("beforeset {:?}", pair_addr.clone());

            let u = doc! {
                "$set" : {
                    "address": pair_addr.clone(),
                    "chain": "astar",
                    "protocol": "arthswap",
                    "name": format!("{}-{} LP", pair.base_token.name, pair.quote_token.name),
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
                    "underlyingAssets": [
                        bson!({
                            "symbol": pair.base_token.symbol,
                            "address": token0_addr.clone(),
                            "decimals": token0decimals,
                        }),
                        bson!({
                            "symbol": pair.quote_token.symbol,
                            "address": token1_addr.clone(),
                            "decimals": token1decimals,
                        })
                    ],
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

    println!("arthswapdone");

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

                    if token_addr.clone() == constants::addresses::zenlink_on_astar::BAI {
                        // BAI
                        price_usd = 1.0;
                    }
                    if token_addr.clone() == constants::addresses::zenlink_on_astar::DAI {
                        // DAI
                        price_usd = 1.0;
                    }
                    if token_addr.clone() == constants::addresses::zenlink_on_moonriver::XCAUSD {
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
                        && (token_addr.clone()
                            == constants::addresses::solarbeam_on_moonriver::STKSM
                            || token_addr.clone()
                                == constants::addresses::solarbeam_on_moonriver::WSTKSM)
                    {
                        let xcksm = assets_collection.find_one(doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::XCKSM}, None).await?;
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
                        && (token_addr.clone()
                            == constants::addresses::solarbeam_on_moonriver::STKSM
                            || token_addr.clone()
                                == constants::addresses::solarbeam_on_moonriver::WSTKSM)
                    {
                        let xcksm = assets_collection.find_one(doc! {"chain":"moonriver", "protocol":"solarbeam", "address":constants::addresses::solarbeam_on_moonriver::XCKSM}, None).await?;
                        price_usd = xcksm.clone().unwrap().price;
                    }

                    // wstDOT
                    if p.0.clone() == "beamswap"
                        && (token_addr.clone()
                            == constants::addresses::beamswap_on_moonbeam::WSTDOT)
                    {
                        let xcdot = assets_collection.find_one(doc! {"chain":"moonbeam", "protocol":"beamswap", "address":constants::addresses::beamswap_on_moonbeam::XCDOT}, None).await?;
                        price_usd = xcdot.clone().unwrap().price;
                    }

                    // axlUSDC
                    if p.0.clone() == "stellaswap"
                        && token_addr.clone()
                            == constants::addresses::stellaswap_on_moonbeam::AXL_USDC
                    {
                        price_usd = 1.0;
                    }

                    // athUSDC
                    if p.0.clone() == "stellaswap"
                        && token_addr.clone()
                            == constants::addresses::stellaswap_on_moonbeam::ATH_USD
                    {
                        price_usd = 1.0;
                    }

                    if t.symbol == "RUM" {
                        let dexscreener_pairs_rum_url="https://api.dexscreener.com/latest/dex/pairs/moonriver/0xbbcef4055ba5c9aa9c1c1b77915887011435a5ab";

                        let rum_pairs = reqwest::get(dexscreener_pairs_rum_url)
                            .await?
                            .json::<apis::dx2::Root>()
                            .await?;

                        if rum_pairs.pairs.len() > 0 {
                            let pair = rum_pairs.pairs[0].clone();

                            price_usd = pair.price_usd.parse().unwrap_or_default();
                            println!("RUM price {:?}", price_usd);
                        }
                    }

                    if p.0.clone() == "solarflare" {
                        for ft in tokens_data.clone().unwrap().tokens.clone() {
                            if ft.id == constants::addresses::beamswap_on_moonbeam::USDC {
                                nomad_usdc_price =
                                    ft.token_day_data[0].price_usd.parse().unwrap_or_default();
                                println!("found moonbeam nomadusdc {:?}", nomad_usdc_price);
                            }
                        }
                        if t.id != constants::addresses::beamswap_on_moonbeam::USDC {
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
        }

        if p.0.clone() == "sushiswap" {
            let pairs_data = client
                .query_unwrap::<subgraph::SushiPairsData>(
                    constants::chef::SUSHI_PAIRS_QUERY.clone(),
                )
                .await;

            if pairs_data.is_ok() {
                for pair in pairs_data.clone().unwrap().pairs.clone() {
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
                            "underlyingAssets": [
                                bson!({
                                    "symbol": pair.token0.symbol,
                                    "address": token0_addr.clone(),
                                    "decimals": token0decimals,
                                }),
                                bson!({
                                    "symbol": pair.token1.symbol,
                                    "address": token1_addr.clone(),
                                    "decimals": token1decimals,
                                })
                            ],
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
                        "zlkpairtoken0logo {:?} token1logo {:?} paa {:?}",
                        token0logo.clone(),
                        token1logo.clone(),
                        pair_addr.clone(),
                    );

                    let token0decimals: u32 = pair.token0.decimals as u32;
                    let token1decimals: u32 = pair.token1.decimals as u32;

                    let mut decimals = token0decimals;
                    if token1decimals > token0decimals {
                        decimals = token1decimals;
                    }

                    let mut liquidity: f64 = pair.reserve_usd.parse().unwrap_or_default();
                    // wstKSM-xcKSM LP
                    if pair_addr.clone()
                        == constants::addresses::solarbeam_on_moonriver::XCKSM_WSTKSM_LP
                    {
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
                            "underlyingAssets": [
                                bson!({
                                    "symbol": pair.token0.symbol,
                                    "address": token0_addr.clone(),
                                    "decimals": token0decimals,
                                }),
                                bson!({
                                    "symbol": pair.token1.symbol,
                                    "address": token1_addr.clone(),
                                    "decimals": token1decimals,
                                })
                            ],
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
                    if pair_addr.clone()
                        == constants::addresses::solarbeam_on_moonriver::XCKSM_WSTKSM_LP
                    {
                        liquidity *= 2.0;
                    }

                    // wstDOT-xcDOT LP
                    if pair_addr.clone()
                        == constants::addresses::beamswap_on_moonbeam::XCDOT_WSTDOT_LP
                    {
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
                            "underlyingAssets": [
                                bson!({
                                    "symbol": pair.token0.symbol,
                                    "address": token0_addr.clone(),
                                    "decimals": token0decimals,
                                }),
                                bson!({
                                    "symbol": pair.token1.symbol,
                                    "address": token1_addr.clone(),
                                    "decimals": token1decimals,
                                })
                            ],
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

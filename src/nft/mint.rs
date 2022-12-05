use crate::constants;
use crate::models;

use std::sync::Arc;

use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::{abigen, Address},
    providers::{Http, Provider},
    signers::LocalWallet,
};
use mongodb::{
    bson::doc,
    options::{ClientOptions, FindOneAndUpdateOptions},
    Client as MongoClient,
};

abigen!(
    IBayNFT,
    r#"[
        function owns(address) external view returns (bool)
        function batchGift(address[], string)
    ]"#,
);

pub async fn check_status_and_mint(mongo_uri: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("Bay Watcher".to_string());
    let client = MongoClient::with_options(client_options)?;
    let db_name = "leaderboard".to_string();
    let db = client.database(&db_name);

    let users_collection = db.collection::<models::User>("User");

    let pk = dotenv::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = pk.parse().expect("fail parse");

    let moonriver_url = dotenv::var("MOONRIVER_URL").unwrap();

    let moonriver_provider_service =
        Provider::<Http>::try_from(moonriver_url.clone()).expect("failed");
    let moonriver_provider = SignerMiddleware::new(moonriver_provider_service, wallet.clone());

    let moonriver_client = SignerMiddleware::new(moonriver_provider.clone(), wallet.clone());
    let moonriver_client = Arc::new(moonriver_client);

    let bay_nft_address = constants::addresses::bay::BAY_NFT.parse::<Address>()?;
    let bay_nft = IBayNFT::new(bay_nft_address, Arc::clone(&moonriver_client));

    let mut users_cursor = users_collection
        .find(
            doc! {
                "users_brought": {"$gt": 5},
                "owns_nft": false,
            },
            None,
        )
        .await?;
    let mut users = vec![];
    let mut user_objs = vec![];

    while users_cursor.advance().await? {
        let u = users_cursor.deserialize_current()?;
        let user_address = u.address.as_str().parse::<Address>()?;
        println!("user_address {:?}", user_address);
        let owns = bay_nft.owns(user_address).call().await?;
        println!("owns {:?}", owns);
        if !owns {
            users.push(user_address);
            user_objs.push(u);
        }
    }

    bay_nft
        .batch_gift(
            users,
            "Qmax53G4ajDtU3uTWmswUE2C9FVTEPHhdpfeTkQxGr9WK8".to_string(),
        )
        .call()
        .await?;

    for uo in user_objs {
        let ff = doc! {
            "address": uo.address.clone(),
        };
        let fu = doc! {
            "$set" : {
                "id": uo.address.clone(),
                "owns_nft": true,
            }
        };
        let options = FindOneAndUpdateOptions::builder()
            .upsert(Some(true))
            .build();
        users_collection
            .find_one_and_update(ff, fu, Some(options))
            .await?;
    }

    Ok(())
}

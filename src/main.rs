use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use std::convert::TryFrom;
mod contracts;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    println!("\nStart!\n");
    dotenv().ok();

    let pk = dotenv::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = pk.parse().expect("fail parse");
    // println!("{}", pk);
    // let provider_id = dotenv::var("PROVIDER_ID").unwrap();
    // let url = format!("https://mainnet.infura.io/v3/{}", provider_id);
    // let url = format!("https://rpc.moonriver.moonbeam.network");
    let url = format!("http://127.0.0.1:8545/");

    // connect provider
    let provider_service = Provider::<Http>::try_from(url).expect("failed");

    let provider = SignerMiddleware::new(provider_service, wallet);

    // connect contracts

    let [_bay_vault_factory, _solar_distributor] = contracts::get_contracts(&provider);
    println!("contracts connected");

    let vaults = contracts::get_bay_vaults(&provider);

    for v in vaults {
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
}

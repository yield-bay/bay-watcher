use ethers::abi::AbiDecode;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::{types::Address, utils::keccak256};
use ethers_providers::Ws;
use eyre::Result;
use std::convert::TryFrom;
use std::ops::Mul;
use std::sync::Arc;
mod contracts;
use dotenv::dotenv;
#[path = "./utils/addresses.rs"]
mod addresses;

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
        ) = chef
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

        if rewards_per_sec.len() > 0 && symbols[0] == "SOLAR" {
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
            let farm_apr = ((sepd * 1 / 10) / (ptvl * 134493)) * 365;
            println!("farmAPR: {}", farm_apr);

            // feeAPR
            // let trading_apr = (lastDayVolume * 0.002 * 365 * 100) / pairLiquidity;
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

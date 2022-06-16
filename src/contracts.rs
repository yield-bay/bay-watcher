use ethers::middleware::SignerMiddleware;
use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::Address,
};
#[path = "./utils/abis/abis.rs"]
mod abis;
#[path = "./utils/addresses.rs"]
mod addresses;

pub fn get_bay_vaults(
    provider: &SignerMiddleware<Provider<Http>, LocalWallet>,
) -> Vec<(
    Contract<&SignerMiddleware<Provider<Http>, LocalWallet>>,
    Contract<&SignerMiddleware<Provider<Http>, LocalWallet>>,
)> {
    let abi_original: String = abis::bay_vault();
    let abi: Abi = serde_json::from_str(&abi_original).expect("failed");

    let mut contracts: Vec<(
        Contract<&SignerMiddleware<Provider<Http>, LocalWallet>>,
        Contract<&SignerMiddleware<Provider<Http>, LocalWallet>>,
    )> = vec![];

    for v in (addresses::contracts()).vaults.iter() {
        let va: Address = v.1.clone().i_bay_vault;
        let sa: Address = v.1.clone().i_strategy;

        let vac = Contract::new(va, abi.clone(), provider);
        let sac = Contract::new(sa, abi.clone(), provider);

        contracts.push((vac, sac));
    }

    return contracts;
}

fn get_bay_vault_factory(
    provider: &SignerMiddleware<Provider<Http>, LocalWallet>,
) -> Contract<&SignerMiddleware<Provider<Http>, LocalWallet>> {
    let abi_original: String = abis::bay_vault_factory();
    let abi: Abi = serde_json::from_str(&abi_original).expect("failed");
    let address: Address = (addresses::contracts()).i_bay_vault_factory;
    let contract = Contract::new(address, abi, provider);
    return contract;
}

pub fn _get_bay_vault(
    provider: &SignerMiddleware<Provider<Http>, LocalWallet>,
    pid: u8,
) -> Contract<&SignerMiddleware<Provider<Http>, LocalWallet>> {
    let abi_original: String = abis::bay_vault();
    let abi: Abi = serde_json::from_str(&abi_original).expect("failed");
    let address: Address = (addresses::contracts())
        .vaults
        .get(&pid)
        .unwrap()
        .i_bay_vault;
    let contract = Contract::new(address, abi, provider);
    return contract;
}

fn _get_strategy(
    provider: &SignerMiddleware<Provider<Http>, LocalWallet>,
    pid: u8,
) -> Contract<&SignerMiddleware<Provider<Http>, LocalWallet>> {
    let abi_original: String = abis::bay_vault();
    let abi: Abi = serde_json::from_str(&abi_original).expect("failed");
    let address: Address = (addresses::contracts())
        .vaults
        .get(&pid)
        .unwrap()
        .i_strategy;
    let contract = Contract::new(address, abi, provider);
    return contract;
}

fn get_solar_distributor(
    provider: &SignerMiddleware<Provider<Http>, LocalWallet>,
) -> Contract<&SignerMiddleware<Provider<Http>, LocalWallet>> {
    let abi_original: String = abis::solar_distributor();
    let abi: Abi = serde_json::from_str(&abi_original).expect("failed");
    let address: Address = (addresses::contracts()).i_solar_distributor_v2;
    let contract = Contract::new(address, abi, provider);
    return contract;
}

pub fn get_contracts(
    provider: &SignerMiddleware<Provider<Http>, LocalWallet>,
) -> [Contract<&SignerMiddleware<Provider<Http>, LocalWallet>>; 2] {
    let static_provider = &provider;

    let bay_vault_factory = get_bay_vault_factory(static_provider);
    let solar_distributor = get_solar_distributor(static_provider);

    return [bay_vault_factory, solar_distributor];
}

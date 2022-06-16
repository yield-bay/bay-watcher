#[path = "./IBayVaultFactoryABI.rs"]
mod bay_vault_factory;

#[path = "./IBayVaultABI.rs"]
mod bay_vault;

#[path = "./IMultiRewardStrat.rs"]
mod multi_reward_strat;

#[path = "./ISolarDistributorV2ABI.rs"]
mod solar_distributor;

pub fn bay_vault_factory() -> String {
    return bay_vault_factory::i_bay_vault_factory_abi();
}

pub fn bay_vault() -> String {
    return bay_vault::i_bay_vault_abi();
}

pub fn _multi_reward_strat() -> String {
    return multi_reward_strat::_i_multi_reward_strat_abi();
}

pub fn solar_distributor() -> String {
    return solar_distributor::i_solar_distributor_v2_abi();
}

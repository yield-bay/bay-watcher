use ethers::types::Address;

pub struct Contracts {
    // pub i_sorted_trove: Address,
    // pub i_trove_manager: Address,
    // pub i_price_feed_v3: Address,
    // pub vault0: Address,
    pub i_bay_vault: Address,
    pub i_solar_distributor_v2: Address,
}

pub fn contracts() -> Contracts {
    Contracts {
        // i_sorted_trove: "0x8FdD3fbFEb32b28fb73555518f8b361bCeA741A6"
        //     .parse::<Address>()
        //     .expect("fail"),
        // i_trove_manager: "0xA39739EF8b0231DbFA0DcdA07d7e29faAbCf4bb2"
        //     .parse::<Address>()
        //     .expect("fail"),
        // i_price_feed_v3: "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419"
        //     .parse::<Address>()
        //     .expect("fail"),
        i_bay_vault: "0xfFE8Ea8C8Ab569c6104e42C787370f1290fa629E"
            .parse::<Address>()
            .expect("fail"),
        i_solar_distributor_v2: "0x0329867a8c457e9F75e25b0685011291CD30904F"
            .parse::<Address>()
            .expect("fail"),
    }
}

use ethers::types::Address;

use std::collections::HashMap;

pub struct Vault {
    pub pid: u8,
    pub i_bay_vault: Address,
    pub i_strategy: Address,
}

pub struct Contracts {
    pub i_utils: Address,
    pub i_warp_in: Address,
    pub i_warp_out: Address,
    pub i_bay_vault_factory: Address,
    pub i_bay_router: Address,
    pub vaults: HashMap<u8, Vault>,
    pub i_solar_distributor_v2: Address,
    // pub i_bay_vault: Address,
}

pub fn vaults() -> Vec<&'static str> {
    return vec!["0xfFE8Ea8C8Ab569c6104e42C787370f1290fa629E"];
}

pub fn contracts() -> Contracts {
    let mut vaults: HashMap<u8, Vault> = HashMap::new();
    vaults.insert(
        11,
        Vault {
            pid: 11,
            i_bay_vault: "0xfFE8Ea8C8Ab569c6104e42C787370f1290fa629E"
                .parse::<Address>()
                .expect("fail"),
            i_strategy: "0x49fd2BE640DB2910c2fAb69bB8531Ab6E76127ff"
                .parse::<Address>()
                .expect("fail"),
        },
    );

    Contracts {
        i_utils: "0xC9a43158891282A2B1475592D5719c001986Aaec"
            .parse::<Address>()
            .expect("fail"),
        i_warp_in: "0x1c85638e118b37167e9298c2268758e058DdfDA0"
            .parse::<Address>()
            .expect("fail"),
        i_warp_out: "0x367761085BF3C12e5DA2Df99AC6E1a824612b8fb"
            .parse::<Address>()
            .expect("fail"),
        i_bay_vault_factory: "0x4C2F7092C2aE51D986bEFEe378e50BD4dB99C901"
            .parse::<Address>()
            .expect("fail"),
        i_bay_router: "0x86A2EE8FAf9A840F7a2c64CA3d51209F9A02081D"
            .parse::<Address>()
            .expect("fail"),
        vaults: vaults,
        i_solar_distributor_v2: "0x0329867a8c457e9F75e25b0685011291CD30904F"
            .parse::<Address>()
            .expect("fail"),
        // i_bay_vault: "0xfFE8Ea8C8Ab569c6104e42C787370f1290fa629E"
        //     .parse::<Address>()
        //     .expect("fail"),
    }
}

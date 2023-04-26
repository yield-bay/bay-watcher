pub mod taiga {
    pub const DAILY_DATA_TAI_KSM_QUERY: &str = r#"
        query {
            dailyData(first:30, orderBy: TIMESTAMP_DESC, filter: {poolId: {equalTo: 0}}) {
                nodes {
                    yieldVolume
                    feeVolume
                    totalSupply
                }
            }
        }
    "#;

    pub const DAILY_DATA_3_USD_QUERY: &str = r#"
        query DD3USD($days: Int) {
            dailyData(first: $days, orderBy: TIMESTAMP_DESC, filter: {poolId: {equalTo: 1}}) {
                nodes {
                    yieldVolume
                    feeVolume
                    totalSupply
                }
            }
        }
    "#;

    pub const TOKEN_PRICE_HISTORY_QUERY: &str = r#"
        query TPHQ($asset: String!, $days: Int!) {
            token(id: $asset) {
                dailyData(first: $days, orderBy: TIMESTAMP_DESC) {
                    nodes {
                        price
                        timestamp
                    }
                }
            }
        }
    "#;
}

pub mod chef {
    pub const TOKENS_QUERY: &str = r#"
        query {
            tokens(orderBy: tradeVolumeUSD, orderDirection: desc, first: 1000) {
                id
                symbol
                name
                decimals
                totalLiquidity
                derivedETH
                tokenDayData(first: 1, orderBy: date, orderDirection: desc) {
                    priceUSD
                }
            }
            bundles(first: 1) {
                ethPrice
            }
        }
    "#;

    pub const SUSHI_TOKENS_QUERY: &str = r#"
        query {
            tokens(orderBy: volumeUSD, orderDirection: desc, first: 1000) {
                id
                symbol
                name
                decimals
                liquidity
                derivedETH
                dayData(first: 1, orderBy: date, orderDirection: desc) {
                    priceUSD
                }
            }
            bundles(first: 1) {
                ethPrice
            }
        }
    "#;

    pub const PAIRS_QUERY: &str = r#"
        query {
            pairs(orderBy: reserveUSD, orderDirection: desc, first: 1000) {
                id
                reserveUSD
                volumeUSD
                untrackedVolumeUSD
                totalSupply
                reserve0
                reserve1
                token0Price
                token1Price
                token0 {
                    id
                    symbol
                    name
                    decimals
                    totalLiquidity
                    tokenDayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
                token1 {
                    id
                    symbol
                    name
                    decimals
                    totalLiquidity
                    tokenDayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
            }
        }
    "#;

    pub const SUSHI_PAIRS_QUERY: &str = r#"
        query {
            pairs(orderBy: reserveUSD, orderDirection: desc, first: 1000) {
                id
                reserveUSD
                volumeUSD
                untrackedVolumeUSD
                totalSupply
                reserve0
                reserve1
                token0Price
                token1Price
                token0 {
                    id
                    symbol
                    name
                    decimals
                    liquidity
                    dayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
                token1 {
                    id
                    symbol
                    name
                    decimals
                    liquidity
                    dayData(first: 1, orderBy: date, orderDirection: desc) {
                        priceUSD
                    }
                }
            }
        }
    "#;

    pub const ONE_DAY_BLOCKS_QUERY: &str = r#"
        query OneDayBlocks($start: Int!, $end: Int!) {
            blocks(
                first: 1
                orderBy: timestamp
                orderDirection: asc
                where: { timestamp_gt: $start, timestamp_lt: $end }
            ) {
                id
                number
                timestamp
            }
        }
    "#;

    pub const ONE_DAY_POOLS_QUERY: &str = r#"
        query OneDayPools($blocknum: Int!) {
            pairs(orderBy: reserveUSD, orderDirection: desc, first: 1000, block: { number: $blocknum }) {
                id
                reserveUSD
                volumeUSD
                untrackedVolumeUSD
            }
        }
    "#;

    pub const PAIR_DAY_DATAS_QUERY: &str = r#"
        query PairDayDatas($addr: String) {
            pairDayDatas(
                orderDirection: desc
                orderBy: date
                first: 7
                where: {pairAddress: $addr}
            ) {
                date
                dailyVolumeUSD
                dailyVolumeToken0
                dailyVolumeToken1
                pairAddress
                id
                token0 {
                    symbol
                }
                token1 {
                    symbol
                }
            }
        }
    "#;

    pub const SUSHI_PAIR_DAY_DATAS_QUERY: &str = r#"
        query PairDayDatas($addr: String) {
            pairDayDatas(
                orderDirection: desc
                orderBy: date
                first: 1
                skip: 1
                where: {pair: $addr}
            ) {
                date
                volumeUSD
                id
                pair {
                    id
                }
                token0 {
                    symbol
                }
                token1 {
                    symbol
                }
            }
        }
    "#;

    pub const SOLARBEAM_STABLE_SWAPS_DAY_DATA_QUERY: &str = r#"
        query Swap($addr: String) {
            swap(id: $addr) {
                id
                address
                tokens {
                    symbol
                }
                dailyData(orderBy: timestamp, orderDirection: desc, first: 7) {
                    id
                    timestamp
                    volume
                }
            }
        }
    "#;

    pub const STELLASWAP_STABLE_SWAPS_DAY_DATA_QUERY: &str = r#"
        query Swap($addr: String) {
            swap(id: $addr) {
                id
                address
                tokens {
                    symbol
                }
                dailyVolumes(orderBy: timestamp, orderDirection: desc, first: 7) {
                    id
                    timestamp
                    volume
                }
            }
        }
    "#;
}

pub mod subsquid {
    pub const TOKENS_QUERY: &str = r#"
        query {
            tokens(orderBy: tradeVolumeUSD_DESC, limit: 1000) {
                id
                symbol
                name
                decimals
                totalLiquidity
                derivedETH
                tokenDayData(limit: 1, orderBy: date_DESC) {
                    priceUSD
                }
            }
            bundles(limit: 1) {
                ethPrice
            }
        }
    "#;

    pub const PAIRS_QUERY: &str = r#"
        query {
            pairs(orderBy: reserveUSD_DESC, limit: 1000) {
                id
                reserveUSD
                volumeUSD
                untrackedVolumeUSD
                totalSupply
                reserve0
                reserve1
                token0Price
                token1Price
                token0 {
                    id
                    symbol
                    name
                    decimals
                    totalLiquidity
                    tokenDayData(limit: 1, orderBy: date_DESC) {
                        priceUSD
                    }
                }
                token1 {
                    id
                    symbol
                    name
                    decimals
                    totalLiquidity
                    tokenDayData(limit: 1, orderBy: date_DESC) {
                        priceUSD
                    }
                }
            }
        }
    "#;

    pub const PAIR_DAY_DATAS_QUERY: &str = r#"
        query PairDayDatas($addr: String) {
            pairDayData(
                orderBy: date_DESC
                limit: 7
                where: {pairAddress_eq: $addr}
            ) {
                date
                dailyVolumeUSD
                pairAddress
                id
            }
        }
    "#;

    pub const STABLE_SWAPS_DAY_DATA_QUERY: &str = r#"
        query {
            stableSwaps {
                    id
                    lpToken
                    stableSwapDayData(orderBy: date_DESC, limit: 7) {
                        id
                        date
                        dailyVolumeUSD
                    }
            }
        }
    "#;
}

pub mod subgraph_urls {
    pub const SOLARBEAM_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solar-ape/solarbeam";
    pub const SOLARFLARE_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/solarflare-subgraph";
    pub const STELLASWAP_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/stellaswap/stella-swap";
    pub const BEAMSWAP_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/beamswap/beamswap-dex";
    pub const SUSHI_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/sushiswap/exchange-moonriver";
    pub const ZENLINK_ASTAR_SUBSQUID: &str =
        "https://squid.subsquid.io/zenlink-astar-squid-yb/v/1/graphql";
    pub const ZENLINK_MOONRIVER_SUBSQUID: &str =
        "https://squid.subsquid.io/zenlink-moonriver-squid-yb/v/1/graphql";
    pub const ZENLINK_MOONBEAM_SUBSQUID: &str =
        "https://squid.subsquid.io/zenlink-moonbeam-squid-yb/v/1/graphql";

    pub const SOLARBEAM_STABLE_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/capjacksparrow42/solarbeam-stable-amm";
    pub const STELLASWAP_STABLE_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/stellaswap/stable-amm";

    pub const SOLARBEAM_BLOCKLYTICS_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/blocklytics";
    pub const SOLARFLARE_BLOCKLYTICS_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/solarflare-blocklytics";
}

pub mod utils {
    pub const MOONRIVER_BLOCK_TIME: f64 = 14.6;
    pub const MOONBEAM_BLOCK_TIME: f64 = 12.4;
    pub const ASTAR_BLOCK_TIME: f64 = 12.8;

    pub const TEN_F64: f64 = 10.0;
    pub const TEN_I128: i128 = 10;
}

pub mod addresses {
    pub mod arthswap_on_astar {
        pub const ARSW: &str = "0xDe2578Edec4669BA7F41c5d5D2386300bcEA4678";
        pub const ARTHSWAP_CHEF: &str = "0xc5b016c5597D298Fe9eD22922CE290A048aA5B75";
    }
    pub mod solarbeam_on_moonriver {
        pub const SOLARBEAM_CHEF: &str = "0x0329867a8c457e9F75e25b0685011291CD30904F";

        pub const SOLAR: &str = "0x6bD193Ee6D2104F14F94E2cA6efefae561A4334B";
        pub const BUSD: &str = "0x5D9ab5522c64E1F6ef5e3627ECCc093f56167818";
        pub const USDC: &str = "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D";
        pub const USDT: &str = "0xB44a9B6905aF7c801311e8F4E76932ee959c663C";
        pub const FRAX: &str = "0x1A93B23281CC1CDE4C4741353F3064709A16197d";
        pub const MAI: &str = "0xFb2019DfD635a03cfFF624D210AEe6AF2B00fC2C";
        pub const MIM: &str = "0x0caE51e1032e8461f4806e26332c030E34De3aDb";
        pub const WBTC: &str = "0x6aB6d61428fde76768D7b45D8BFeec19c6eF91A8";
        pub const XCKBTC: &str = "0xFFFfFfFfF6E528AD57184579beeE00c5d5e646F0";
        pub const XCKSM: &str = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
        pub const STKSM: &str = "0xFfc7780C34B450d917d557E728f033033CB4fA8C";
        pub const WSTKSM: &str = "0x3bfd113ad0329a7994a681236323fb16E16790e3";

        pub const _3POOL: &str = "0xfb29918d393AaAa7dD118B51A8b7fCf862F5f336";
        pub const FRAX_3POOL: &str = "0x884609A4D86BBA8477112E36e27f7A4ACecB3575";
        pub const MAI_3POOL: &str = "0x8CDB472731B4f815d67e76885a22269ad7f0e398";
        pub const MIM_3POOL: &str = "0x4BaB767c98186bA28eA66f2a69cd0DA351D60b36";
        pub const KBTC_BTC: &str = "0x4F707d051b4b49B63e72Cc671e78E152ec66f2fA";
        pub const STKSM_POOL: &str = "0x493147C85Fe43F7B056087a6023dF32980Bcb2D1";

        pub const XCKSM_WSTKSM_LP: &str = "0x5568872bc43Bae3757F697c0e1b241b62Eddcc17";
    }
    pub mod solarflare_on_moonbeam {
        pub const SOLARFLARE_CHEF: &str = "0x995da7dfB96B4dd1e2bd954bE384A1e66cBB4b8c";
    }
    pub mod zenlink_on_moonriver {
        pub const ZENLINK_CHEF: &str = "0xf4Ec122d32F2117674Ce127b72c40506c52A72F8";

        pub const USDC: &str = "0xE3F5a90F9cb311505cd691a46596599aA1A0AD7D";
        pub const XCAUSD: &str = "0xFfFffFFfa1B026a00FbAA67c86D5d1d5BF8D8228";
    }
    pub mod zenlink_on_moonbeam {
        pub const ZENLINK_CHEF: &str = "0xD6708344553cd975189cf45AAe2AB3cd749661f4";
    }
    pub mod zenlink_on_astar {
        pub const ZENLINK_CHEF: &str = "0x460ee9DBc82B2Be84ADE50629dDB09f6A1746545";

        pub const BAI: &str = "0x733ebcC6DF85f8266349DEFD0980f8Ced9B45f35";
        pub const BUSD: &str = "0x4Bf769b05E832FCdc9053fFFBC78Ca889aCb5E1E";
        pub const DAI: &str = "0x6De33698e9e9b787e09d3Bd7771ef63557E148bb";
        pub const USDC: &str = "0x6a2d262D56735DbA19Dd70682B39F6bE9a931D98";
    }
    pub mod stellaswap_on_moonbeam {
        pub const STELLA_CHEF_V1: &str = "0xEDFB330F5FA216C9D2039B99C8cE9dA85Ea91c1E";
        pub const STELLA_CHEF_V2: &str = "0xF3a5454496E26ac57da879bf3285Fa85DEBF0388";

        pub const WGLMR: &str = "0xAcc15dC74880C9944775448304B263D191c6077F";
        pub const POOP: &str = "0xFFfffFFecB45aFD30a637967995394Cc88C0c194";
        pub const STELLA: &str = "0x0E358838ce72d5e61E0018a2ffaC4bEC5F4c88d2";
        pub const FRAX: &str = "0x322E86852e492a7Ee17f28a78c663da38FB33bfb";
        pub const BUSD: &str = "0x692C57641fc054c2Ad6551Ccc6566EbA599de1BA";
        pub const USDC: &str = "0x931715FEE2d06333043d11F658C8CE934aC61D0c";
        pub const USDT: &str = "0xFFFFFFfFea09FB06d082fd1275CD48b191cbCD1d";
        pub const MAI: &str = "0xdFA46478F9e5EA86d57387849598dbFB2e964b02";
        pub const ATH_USD: &str = "0x9D5d41D8C03e38194A577347206F8829B9cF7C9a";
        pub const AXL_USDC: &str = "0xCa01a1D0993565291051daFF390892518ACfAD3A";

        pub const _4POOL: &str = "0xB326b5189AA42Acaa3C649B120f084Ed8F4dCaA6";
        pub const MAI_4POOL: &str = "0xEceab9F0FcF15Fddbffbd7baE2cEB78CD57b879a";
        pub const ATH_USD_4POOL: &str = "0xe196001e2a4798E437E80493216c2aD1b9f5c190";
        pub const AXL_USDC_4POOL: &str = "0xacb7dA459719EA26054D0481c5B3AE5903bd9906";
        pub const TRI_POOL: &str = "0x4FB1b0452341ebB0DF325a8286763447dd6F15fF";
        pub const AXL_DUAL_POOL: &str = "0x6Cd1c3807DbB49785b86cF006Fe2C90287c183B2";
        pub const MAI_TRI_POOL: &str = "0x8BED562B515805CFFBFC3B4105343895B6e01A1A";

        pub const WGLMR_POOP_LP: &str = "0x4EfB208eeEb5A8C85af70e8FBC43D6806b422bec";
    }
    pub mod beamswap_on_moonbeam {
        pub const BEAM_CHEF: &str = "0xC6ca172FC8BDB803c5e12731109744fb0200587b";

        pub const WGLMR: &str = "0xAcc15dC74880C9944775448304B263D191c6077F";
        pub const POOP: &str = "0xFFfffFFecB45aFD30a637967995394Cc88C0c194";
        pub const BUSD: &str = "0xA649325Aa7C5093d12D6F98EB4378deAe68CE23F";
        pub const USDC: &str = "0x818ec0A7Fe18Ff94269904fCED6AE3DaE6d6dC0b";
        pub const USDT: &str = "0xeFAeeE334F0Fd1712f9a8cc375f427D9Cdd40d73";
        pub const DAI: &str = "0x765277EebeCA2e31912C9946eAe1021199B39C61";
        pub const LDO: &str = "0x9Fda7cEeC4c18008096C2fE2B85F05dc300F94d0";
        pub const XCDOT: &str = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
        // pub const STDOT: &str = "";
        pub const WSTDOT: &str = "0x191cf2602Ca2e534c5Ccae7BCBF4C46a704bb949";

        pub const _4POOL: &str = "0x9BF6910790D70E9b5B07Cb28271C42531B929b4c";

        pub const WGLMR_POOP_LP: &str = "0xa049a6260921B5ee3183cFB943133d36d7FdB668";

        pub const XCDOT_WSTDOT_LP: &str = "0x79f05B32e29139C35Cd219aEDB5D99cedb1915aC";
    }
    pub mod sushi_on_moonriver {
        pub const SUSHI_MINI_CHEF: &str = "0x3dB01570D97631f69bbb0ba39796865456Cf89A5";

        pub const SUSHI_COMPLEX_REWARDER: &str = "0x1334c8e873E1cae8467156e2A81d1C8b566B2da1";

        pub const SUSHI: &str = "0xf390830DF829cf22c53c8840554B98eafC5dCBc2";
        pub const MOVR: &str = "0xf50225a84382c74CbdeA10b0c176f71fc3DE0C4d";
    }
}

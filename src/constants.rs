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

    pub const SOLARBEAM_BLOCKLYTICS_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/blocklytics";
    pub const SOLARFLARE_BLOCKLYTICS_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/solarflare-blocklytics";
}

pub mod utils {
    pub const MOONRIVER_BLOCK_TIME: f64 = 14.6;
    pub const MOONBEAM_BLOCK_TIME: f64 = 12.4;
    pub const ASTAR_BLOCK_TIME: f64 = 12.8;
}

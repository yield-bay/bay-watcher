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

pub mod subgraph_urls {
    pub const SOLARBEAM_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solar-ape/solarbeam";
    pub const STELLASWAP_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/stellaswap/stella-swap";
    pub const BEAMSWAP_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/beamswap/beamswap-dex";
    pub const SUSHI_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/sushiswap/exchange-moonriver";

    pub const SOLARBEAM_BLOCKLYTICS_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/blocklytics";
    pub const SOLARFLARE_BLOCKLYTICS_SUBGRAPH: &str =
        "https://api.thegraph.com/subgraphs/name/solarbeamio/solarflare-blocklytics";
}

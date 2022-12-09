# bay-watcher

The primary ETL service for Yield Bay.

## Getting Started

- Create a mongodb database
- Copy sample env file `cp .env.sample .env` and set env variables.
- `cargo build`
- `cargo run`

## How to integrate a new protocol

1.  Fetch dex related data from their subgraph/subsquid/subql

    - Create subgraph client as shown in [`run_jobs` function](src/main.rs#L56).
    - Add details to [protocols array](src/main.rs#L121).

2.  Fetch farm related data from the smart contract (NOTE: for evm chef-style farms)

    - Add farm details to `chef_contract_jobs` function.
    - We find the number of pools in that particular farm and iterate over them.
    - Edge cases like stable swap farms are handled by checking the pids.
    - Farm chef contracts may vary for different protocols, leading to slightly different function calls.

3.  Custom integrations

    - Ones that aren't chef-style or don't use the above pattern - those protocols might offer a different API to fetch yield farming data.
    - Done inside the [src/custom](src/custom) directory.

## Safety Score

- The Yield farm safety score system is described [here](https://hackmd.io/@rz4NXhzNS0qhxd_EzzPY_Q/BJFENaxuo).
- Implementation present in [src/scoring.rs](src/scoring.rs).

## Misc Details

- The constants (which include the graphql query strings, api urls, smart contract addresses, and other utilities) are present in [src/constants.rs](src/constants.rs).
- The human-readable abis are present in [src/contracts.rs](src/contracts.rs).
- We run all the tasks in an infinite loop with a delay of 3 mins in between.

### Farm model (non-obvious fields)

| Field             |                                                           Description                                                           |
| :---------------- | :-----------------------------------------------------------------------------------------------------------------------------: |
| **id**            |                                          Farm id, pid in case of chef-style contracts                                           |
| **chef**          |                              Chef contract address. Some other unique identifier if not chef-style                              |
| **asset.address** |                                                    Underlying asset address                                                     |
| **apr.base**      |                                          The trading APR of the protocol (usually DEX)                                          |
| **apr.reward**    |                                         The APR from the incentive provided by the farm                                         |
| **allocPoint**    | Represents the share of reward in the whole farm in chef-style farms. Its utility for us is that `0` indicates an inactive farm |

The combination (**id**, **chef**, **chain**, **protocol**, **asset.address**) can be considered the primary key (although we are using mongodb, which uses object ids).

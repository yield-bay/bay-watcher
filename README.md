# bay-watcher

The primary ETL service for Yield Bay.

## Getting Started

- Create a mongodb database
- Copy sample env file `cp .env.sample .env` and set env variables.
- `cargo build`
- `cargo run`

## How to integrate a new protocol

1.  Fetch dex related data from their subgraph/subsquid/subql

    - Create subgraph client as shown in [`run_jobs` function](src/main.rs#L56)
    - Add details to [protocols array](src/main.rs#L121)

2.  Fetch farm related data from the smart contract (NOTE: for evm chef-style farms)

    - Add farm details to `chef_contract_jobs` function.
    - We find the number of pools in that particular farm and iterate over them.
    - Edge cases like stable swap farms are handled by checking the pids.
    - Farm chef contracts may vary for different protocols, leading to slightly different function calls.

3.  Custom integrations

    - Ones that aren't chef-style or don't use the above pattern - those protocols might offer a different API to fetch yield farming data.
    - Done inside the [src/custom](src/custom) directory.

## Safety Score

- The Yield farm safety score system is implemented in [src/scoring.rs](src/scoring.rs).

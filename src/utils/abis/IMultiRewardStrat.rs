pub fn _i_multi_reward_strat_abi() -> String {
    r#"
    [
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "_poolId",
            "type": "uint256"
            },
            {
            "internalType": "contract BayVault",
            "name": "_vault",
            "type": "address"
            },
            {
            "internalType": "contract ISolarRouter02",
            "name": "_router",
            "type": "address"
            },
            {
            "internalType": "contract ISolarDistributorV2",
            "name": "_chef",
            "type": "address"
            },
            {
            "internalType": "contract IYieldBayWarpIn",
            "name": "_warpIn",
            "type": "address"
            },
            {
            "internalType": "address[]",
            "name": "_outputToNativeRoute",
            "type": "address[]"
            },
            {
            "internalType": "address[]",
            "name": "_outputToLp0Route",
            "type": "address[]"
            },
            {
            "internalType": "address[]",
            "name": "_outputToLp1Route",
            "type": "address[]"
            },
            {
            "internalType": "contract IComplexRewarder[]",
            "name": "_rewarders",
            "type": "address[]"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "constructor"
        },
        {
        "inputs": [
            {
            "internalType": "address",
            "name": "addr",
            "type": "address"
            },
            {
            "internalType": "string",
            "name": "message",
            "type": "string"
            }
        ],
        "name": "AddressNotUpdated",
        "type": "error"
        },
        {
        "inputs": [],
        "name": "InsufficientAllowance",
        "type": "error"
        },
        {
        "inputs": [],
        "name": "InsufficientBalance",
        "type": "error"
        },
        {
        "inputs": [
            {
            "internalType": "enum FeeType",
            "name": "feeType",
            "type": "uint8"
            },
            {
            "internalType": "uint256",
            "name": "fee",
            "type": "uint256"
            }
        ],
        "name": "InvalidFee",
        "type": "error"
        },
        {
        "inputs": [
            {
            "internalType": "string",
            "name": "message",
            "type": "string"
            }
        ],
        "name": "InvalidRoute",
        "type": "error"
        },
        {
        "inputs": [],
        "name": "OnlyStrategist",
        "type": "error"
        },
        {
        "inputs": [],
        "name": "OnlyVault",
        "type": "error"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "value",
            "type": "uint256"
            }
        ],
        "name": "SlippageOutOfBounds",
        "type": "error"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": true,
            "internalType": "address",
            "name": "previousOwner",
            "type": "address"
            },
            {
            "indexed": true,
            "internalType": "address",
            "name": "newOwner",
            "type": "address"
            }
        ],
        "name": "OwnershipTransferred",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "address",
            "name": "account",
            "type": "address"
            }
        ],
        "name": "Paused",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": true,
            "internalType": "uint256",
            "name": "poolId",
            "type": "uint256"
            },
            {
            "indexed": true,
            "internalType": "uint256",
            "name": "amount",
            "type": "uint256"
            }
        ],
        "name": "StratDeposit",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "poolId",
            "type": "uint256"
            },
            {
            "indexed": false,
            "internalType": "address",
            "name": "harvester",
            "type": "address"
            },
            {
            "indexed": true,
            "internalType": "address",
            "name": "harvestRewardRecipient",
            "type": "address"
            },
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "liquidityAdded",
            "type": "uint256"
            },
            {
            "indexed": true,
            "internalType": "uint256",
            "name": "depositTokenHarvested",
            "type": "uint256"
            },
            {
            "indexed": true,
            "internalType": "uint256",
            "name": "tvl",
            "type": "uint256"
            }
        ],
        "name": "StratHarvest",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": true,
            "internalType": "uint256",
            "name": "poolId",
            "type": "uint256"
            },
            {
            "indexed": true,
            "internalType": "uint256",
            "name": "amount",
            "type": "uint256"
            }
        ],
        "name": "StratWithdraw",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "address",
            "name": "account",
            "type": "address"
            }
        ],
        "name": "Unpaused",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "oldValue",
            "type": "uint256"
            },
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "UpdateHarvestReward",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "oldValue",
            "type": "uint256"
            },
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "UpdateMaxTokensToDepositWithoutHarvest",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "oldValue",
            "type": "uint256"
            },
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "UpdateMinTokensToHarvest",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "address",
            "name": "oldValue",
            "type": "address"
            },
            {
            "indexed": false,
            "internalType": "address",
            "name": "newValue",
            "type": "address"
            }
        ],
        "name": "UpdateStrategist",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "oldValue",
            "type": "uint256"
            },
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "UpdateStrategistFee",
        "type": "event"
        },
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "oldValue",
            "type": "uint256"
            },
            {
            "indexed": false,
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "UpdateVaultFee",
        "type": "event"
        },
        {
        "inputs": [],
        "name": "BPS_DIVISOR",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "balance",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "balanceOfDeposit",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "balanceOfPool",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "chef",
        "outputs": [
            {
            "internalType": "contract ISolarDistributorV2",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "amount",
            "type": "uint256"
            }
        ],
        "name": "deposit",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "depositAll",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "depositToken",
        "outputs": [
            {
            "internalType": "contract ERC20",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "address",
            "name": "harvestRewardRecipient",
            "type": "address"
            }
        ],
        "name": "harvest",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "harvestOnDeposit",
        "outputs": [
            {
            "internalType": "bool",
            "name": "",
            "type": "bool"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "harvestRewardBps",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "lastHarvest",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "lp",
        "outputs": [
            {
            "internalType": "address",
            "name": "token0",
            "type": "address"
            },
            {
            "internalType": "address",
            "name": "token1",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "maxTokensToDepositWithoutHarvest",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "minTokensToHarvest",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "native",
        "outputs": [
            {
            "internalType": "contract IERC20",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "output",
        "outputs": [
            {
            "internalType": "contract IERC20",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "name": "outputToLp0Route",
        "outputs": [
            {
            "internalType": "address",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "name": "outputToLp1Route",
        "outputs": [
            {
            "internalType": "address",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "name": "outputToNativeRoute",
        "outputs": [
            {
            "internalType": "address",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "owner",
        "outputs": [
            {
            "internalType": "address",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "panic",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "pause",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "paused",
        "outputs": [
            {
            "internalType": "bool",
            "name": "",
            "type": "bool"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "poolId",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "renounceOwnership",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "retireStrat",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "name": "rewarders",
        "outputs": [
            {
            "internalType": "contract IComplexRewarder",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "rewardsAvailable",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "router",
        "outputs": [
            {
            "internalType": "contract ISolarRouter02",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "slippage",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "strategist",
        "outputs": [
            {
            "internalType": "address",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "strategistFeeBps",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "address",
            "name": "newOwner",
            "type": "address"
            }
        ],
        "name": "transferOwnership",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "unpause",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "updateHarvestReward",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "updateMaxTokensToDepositWithoutHarvest",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "updateMinTokensToHarvest",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "address[]",
            "name": "_outputToLp0Route",
            "type": "address[]"
            }
        ],
        "name": "updateOutputToLp0Route",
        "outputs": [
            {
            "internalType": "address[]",
            "name": "",
            "type": "address[]"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "address[]",
            "name": "_outputToLp1Route",
            "type": "address[]"
            }
        ],
        "name": "updateOutputToLp1Route",
        "outputs": [
            {
            "internalType": "address[]",
            "name": "",
            "type": "address[]"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "address[]",
            "name": "_outputToNativeRoute",
            "type": "address[]"
            }
        ],
        "name": "updateOutputToNativeRoute",
        "outputs": [
            {
            "internalType": "address[]",
            "name": "",
            "type": "address[]"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "index",
            "type": "uint256"
            },
            {
            "internalType": "address[]",
            "name": "_rewardToNativeRoute",
            "type": "address[]"
            },
            {
            "internalType": "address[]",
            "name": "_rewardToLp0Route",
            "type": "address[]"
            },
            {
            "internalType": "address[]",
            "name": "_rewardToLp1Route",
            "type": "address[]"
            }
        ],
        "name": "updateRewardRoutesForRewarder",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "_slippage",
            "type": "uint256"
            }
        ],
        "name": "updateSlippage",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "address",
            "name": "newValue",
            "type": "address"
            }
        ],
        "name": "updateStrategist",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "updateStrategistFee",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "newValue",
            "type": "uint256"
            }
        ],
        "name": "updateVaultFee",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "vault",
        "outputs": [
            {
            "internalType": "contract BayVault",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "vaultFeeBps",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "warpIn",
        "outputs": [
            {
            "internalType": "contract IYieldBayWarpIn",
            "name": "",
            "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "amount",
            "type": "uint256"
            }
        ],
        "name": "withdraw",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        },
        {
        "inputs": [],
        "name": "withdrawAll",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        }
    ]
    "#
    .to_string()
}

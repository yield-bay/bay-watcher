pub fn i_utils_abi() -> String {
    r#"
    [
        {
        "inputs": [
            {
            "internalType": "contract ISolarPair",
            "name": "solarPair",
            "type": "ISolarPair"
            },
            {
            "internalType": "uint256",
            "name": "token0Amount",
            "type": "uint256"
            },
            {
            "internalType": "uint256",
            "name": "token1Amount",
            "type": "uint256"
            },
            {
            "internalType": "uint256",
            "name": "slippage",
            "type": "uint256"
            }
        ],
        "name": "calculateMinimumLP",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "minLP",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "contract ISolarRouter02",
            "name": "solarRouter",
            "type": "ISolarRouter02"
            },
            {
            "internalType": "uint256",
            "name": "amount",
            "type": "uint256"
            },
            {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
            }
        ],
        "name": "getAmountsOut",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        }
    ]
    "#
    .to_string()
}

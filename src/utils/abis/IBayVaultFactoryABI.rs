pub fn i_bay_vault_factory_abi() -> String {
    r#"
    [
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
              "indexed": true,
              "internalType": "contract BayVault",
              "name": "vault",
              "type": "address"
            },
            {
              "indexed": false,
              "internalType": "contract ERC20",
              "name": "underlying",
              "type": "address"
            }
          ],
          "name": "VaultDeployed",
          "type": "event"
        },
        {
          "inputs": [
            {
              "internalType": "contract ERC20",
              "name": "underlying",
              "type": "address"
            },
            {
              "internalType": "string",
              "name": "underlyingName",
              "type": "string"
            },
            {
              "internalType": "string",
              "name": "underlyingSymbol",
              "type": "string"
            },
            {
              "internalType": "address",
              "name": "bayTreasury",
              "type": "address"
            }
          ],
          "name": "deployVault",
          "outputs": [
            {
              "internalType": "contract BayVault",
              "name": "vault",
              "type": "address"
            }
          ],
          "stateMutability": "nonpayable",
          "type": "function"
        },
        {
          "inputs": [
            {
              "internalType": "contract ERC20",
              "name": "underlying",
              "type": "address"
            },
            {
              "internalType": "string",
              "name": "underlyingName",
              "type": "string"
            },
            {
              "internalType": "string",
              "name": "underlyingSymbol",
              "type": "string"
            },
            {
              "internalType": "address",
              "name": "bayTreasury",
              "type": "address"
            }
          ],
          "name": "getVaultFromUnderlying",
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
          "inputs": [
            {
              "internalType": "contract BayVault",
              "name": "vault",
              "type": "address"
            }
          ],
          "name": "isVaultDeployed",
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
          "name": "renounceOwnership",
          "outputs": [],
          "stateMutability": "nonpayable",
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
          "name": "underlyingTokensList",
          "outputs": [
            {
              "internalType": "address[]",
              "name": "",
              "type": "address[]"
            }
          ],
          "stateMutability": "view",
          "type": "function"
        },
        {
          "inputs": [
            {
              "internalType": "uint256",
              "name": "index",
              "type": "uint256"
            }
          ],
          "name": "vaultAt",
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
              "internalType": "contract ERC20",
              "name": "",
              "type": "address"
            }
          ],
          "name": "vaultForUnderlyingToken",
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
          "name": "vaultsCount",
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
          "name": "vaultsList",
          "outputs": [
            {
              "internalType": "address[]",
              "name": "",
              "type": "address[]"
            }
          ],
          "stateMutability": "view",
          "type": "function"
        }
    ]
    "#
    .to_string()
}

use ethers::prelude::abigen;

abigen!(
    IChefV2,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (address, uint256, uint256, uint256, uint16, uint256, uint256)
        function poolTotalLp(uint256) external view returns (uint256)
        function poolRewarders(uint256) external view returns (address [])
        function poolRewardsPerSec(uint256) external view returns (address[], string[], uint256[], uint256[])
        function stellaPerSec() external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
    ]"#,
);

abigen!(
    IArthswapChef,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfos(uint256) external view returns (uint128, uint64, uint64)
        function ARSWPerBlock(uint256) external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
        function lpTokens(uint256) external view returns (address)
        function getPeriod(uint256) external view returns (uint256)
    ]"#,
);

abigen!(
    IFarming,
    r#"[
        function poolLength() external view returns (uint256)
        function getPoolInfo(uint256) external view returns (address, uint256, address[], uint256[], uint256[], uint256, uint256, uint256)
    ]"#,
);

abigen!(
    IStellaDistributorV1,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (address, uint256, uint256, uint256, uint16, uint256, uint256)
        function stellaPerBlock() external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
    ]"#,
);

abigen!(
    IMiniChefV2,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (uint128, uint64, uint64)
        function sushiPerSecond() external view returns (uint256)
        function totalAllocPoint() external view returns (uint256)
        function lpToken(uint256) external view returns (address)
    ]"#,
);

abigen!(
    IComplexRewarderTime,
    r#"[
        function poolLength() external view returns (uint256)
        function poolInfo(uint256) external view returns (uint128, uint64, uint64)
        function rewardPerSecond() external view returns (uint256)
    ]"#,
);

abigen!(
    IStandardLpToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
    ]"#,
);

abigen!(
    IStableLpToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function owner() external view returns (address)
        function totalSupply() external view returns (uint256)
        function balanceOf(address) external view returns (uint256)
    ]"#,
);

abigen!(
    IVestedToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function owner() external view returns (address)
    ]"#,
);

abigen!(
    IStableLpTokenOwner,
    r#"[
        function getNumberOfTokens() external view returns (uint256)
        function getToken(uint8) external view returns (address)
        function getTokenBalance(uint8) external view returns (uint256)
        function getTokenBalances() external view returns (uint256[])
        function getTokenIndex(address) external view returns (uint256)
        function getTokenPrecisionMultipliers() external view returns (uint256[])
        function getTokens() external view returns (address[])
        function getVirtualPrice() external view returns (uint256)
    ]"#,
);

abigen!(
    IAnyswapV5ERC20,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
        function balanceOf(address) external view returns (uint256)
    ]"#,
);

abigen!(
    ILpToken,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
        function balanceOf(address) external view returns (uint256)
        function token0() external view returns (address)
        function token1() external view returns (address)
        function getReserves() external view returns (uint112, uint112, uint32)
        function totalSupply() external view returns (uint256)
    ]"#,
);

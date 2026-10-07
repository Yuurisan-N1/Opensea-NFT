pub mod api;
pub mod seadrop;
pub mod stage;

use ethers::types::Address;

use crate::core::error::{MintError, MintResult};

pub const SEADROP_ADDRESS: &str = "0x00005EA00Ac477B1030CE78506496e8C2dE24bf5";

pub struct ChainSpec {
    pub key: &'static str,
    pub name: &'static str,
    pub chain_id: u64,
    pub native: &'static str,
    pub rpc: &'static [&'static str],
    pub ws: &'static str,
}

pub const CHAINS: &[ChainSpec] = &[
    ChainSpec { key: "ethereum", name: "Ethereum", chain_id: 1, native: "ETH", rpc: &["https://ethereum-rpc.publicnode.com", "https://eth.drpc.org", "https://ethereum.publicnode.com"], ws: "wss://ethereum-rpc.publicnode.com" },
    ChainSpec { key: "base", name: "Base", chain_id: 8453, native: "ETH", rpc: &["https://base-rpc.publicnode.com", "https://base.publicnode.com", "https://mainnet.base.org"], ws: "wss://base-rpc.publicnode.com" },
    ChainSpec { key: "arbitrum", name: "Arbitrum", chain_id: 42161, native: "ETH", rpc: &["https://arbitrum-one-rpc.publicnode.com", "https://arbitrum-one.publicnode.com", "https://arb1.arbitrum.io/rpc"], ws: "wss://arbitrum-one-rpc.publicnode.com" },
    ChainSpec { key: "optimism", name: "Optimism", chain_id: 10, native: "ETH", rpc: &["https://optimism-rpc.publicnode.com", "https://optimism.drpc.org", "https://mainnet.optimism.io"], ws: "wss://optimism-rpc.publicnode.com" },
    ChainSpec { key: "polygon", name: "Polygon", chain_id: 137, native: "POL", rpc: &["https://polygon-bor-rpc.publicnode.com", "https://polygon.drpc.org", "https://polygon-rpc.com"], ws: "wss://polygon-bor-rpc.publicnode.com" },
    ChainSpec { key: "apechain", name: "ApeChain", chain_id: 33139, native: "APE", rpc: &["https://rpc.apechain.com/http", "https://apechain.calderachain.xyz/http"], ws: "" },
    ChainSpec { key: "unichain", name: "Unichain", chain_id: 130, native: "ETH", rpc: &["https://unichain-rpc.publicnode.com", "https://unichain.publicnode.com", "https://mainnet.unichain.org"], ws: "wss://unichain-rpc.publicnode.com" },
    ChainSpec { key: "zora", name: "Zora", chain_id: 7777777, native: "ETH", rpc: &["https://rpc.zora.energy", "https://zora.drpc.org"], ws: "" },
    ChainSpec { key: "soneium", name: "Soneium", chain_id: 1868, native: "ETH", rpc: &["https://rpc.soneium.org"], ws: "" },
    ChainSpec { key: "shape", name: "Shape", chain_id: 360, native: "ETH", rpc: &["https://mainnet.shape.network"], ws: "" },
    ChainSpec { key: "robinhood", name: "Robinhood", chain_id: 4663, native: "ETH", rpc: &["https://rpc.mainnet.chain.robinhood.com", "https://robinhood-rpc.publicnode.com", "https://robinhood.drpc.org"], ws: "wss://robinhood-rpc.publicnode.com" },
    ChainSpec { key: "arc", name: "Arc", chain_id: 5042, native: "USDC", rpc: &["https://rpc.mainnet.arc.io", "https://arc.drpc.org"], ws: "wss://arc.drpc.org" },
    ChainSpec { key: "monad", name: "Monad", chain_id: 143, native: "MON", rpc: &["https://rpc.monad.xyz", "https://rpc1.monad.xyz"], ws: "" },
    ChainSpec { key: "abstract", name: "Abstract", chain_id: 2741, native: "ETH", rpc: &["https://api.mainnet.abs.xyz"], ws: "" },
    ChainSpec { key: "berachain", name: "Berachain", chain_id: 80094, native: "BERA", rpc: &["https://rpc.berachain.com", "https://berachain.drpc.org"], ws: "" },
    ChainSpec { key: "sonic", name: "Sonic", chain_id: 146, native: "S", rpc: &["https://rpc.soniclabs.com"], ws: "" },
    ChainSpec { key: "ink", name: "Ink", chain_id: 57073, native: "ETH", rpc: &["https://rpc-gel.inkonchain.com", "https://rpc.inkonchain.com", "https://ink.drpc.org"], ws: "" },
    ChainSpec { key: "blast", name: "Blast", chain_id: 81457, native: "ETH", rpc: &["https://rpc.blast.io", "https://blast.drpc.org"], ws: "" },
    ChainSpec { key: "bnb", name: "BNB Chain", chain_id: 56, native: "BNB", rpc: &["https://bsc-rpc.publicnode.com", "https://bsc-dataseed.binance.org"], ws: "wss://bsc-rpc.publicnode.com" },
    ChainSpec { key: "avalanche", name: "Avalanche", chain_id: 43114, native: "AVAX", rpc: &["https://avalanche-c-chain-rpc.publicnode.com", "https://avalanche.drpc.org", "https://api.avax.network/ext/bc/C/rpc"], ws: "wss://avalanche-c-chain-rpc.publicnode.com" },
];

pub fn chain_by_key(key: &str) -> MintResult<&'static ChainSpec> {
    CHAINS
        .iter()
        .find(|spec| spec.key.eq_ignore_ascii_case(key))
        .ok_or(MintError::UnsupportedChain)
}

pub fn chain_keys() -> Vec<String> {
    CHAINS.iter().map(|spec| spec.key.to_string()).collect()
}

pub fn seadrop_address() -> Address {
    SEADROP_ADDRESS.parse().unwrap()
}

use std::fmt;

use ethers::types::{Address, H256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainId(pub u64);

impl ChainId {
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WalletAddress(pub Address);

impl WalletAddress {
    pub fn to_hex(&self) -> String {
        format!("{:#x}", self.0)
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TxHash(pub H256);

impl TxHash {
    pub fn to_hex(&self) -> String {
        format!("{:#x}", self.0)
    }

    pub fn as_h256(&self) -> H256 {
        self.0
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexBytes(pub Vec<u8>);

impl HexBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        HexBytes(bytes)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.0))
    }

    pub fn selector(&self) -> String {
        if self.0.len() < 4 {
            return String::new();
        }
        format!("0x{}", hex::encode(&self.0[..4]))
    }
}

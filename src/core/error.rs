use thiserror::Error;

#[derive(Debug, Error)]
pub enum MintError {
    #[error("RPC request did not succeed")]
    Rpc,
    #[error("Drop is not minting at this moment")]
    DropNotMinting,
    #[error("Price is above the configured limit")]
    PriceAboveLimit,
    #[error("Wallet balance is not enough for this mint")]
    InsufficientBalance,
    #[error("Target contract address is not valid")]
    BadContract,
    #[error("Allow list proof is not available for this wallet")]
    MissingProof,
    #[error("Receipt wait timed out")]
    ReceiptTimeout,
    #[error("Mint transaction was rejected by the node")]
    Rejected,
    #[error("Private key is not usable")]
    BadKey,
    #[error("Chain is not part of the supported list")]
    UnsupportedChain,
    #[error("Configuration file could not be read")]
    Config,
    #[error("Input was not accepted")]
    Input,
}

pub type MintResult<T> = Result<T, MintError>;

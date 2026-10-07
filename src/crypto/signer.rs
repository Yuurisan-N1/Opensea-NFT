use anyhow::{Context, Result};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Address, Bytes, Eip1559TransactionRequest, U256};

use crate::network::rpc::FeeParams;

pub struct PreSigned {
    pub raw: String,
    pub nonce: u64,
    pub gas: u64,
}

pub struct SignRequest {
    pub chain_id: u64,
    pub to: Address,
    pub data: Bytes,
    pub value: U256,
    pub gas: u64,
    pub nonce: u64,
}

pub async fn presign(
    wallet: &LocalWallet,
    fee: &FeeParams,
    request: &SignRequest,
) -> Result<PreSigned> {
    let from = wallet.address();

    let tx: TypedTransaction = Eip1559TransactionRequest::new()
        .from(from)
        .to(request.to)
        .nonce(U256::from(request.nonce))
        .data(request.data.clone())
        .value(request.value)
        .gas(U256::from(request.gas))
        .max_fee_per_gas(fee.max_fee)
        .max_priority_fee_per_gas(fee.priority_fee)
        .chain_id(request.chain_id)
        .into();

    let signature = wallet
        .sign_transaction(&tx)
        .await
        .context("Transaction could not be signed offline")?;

    let signed = tx.rlp_signed(&signature);
    let raw = format!("0x{}", hex::encode(signed.as_ref()));

    Ok(PreSigned {
        raw,
        nonce: request.nonce,
        gas: request.gas,
    })
}

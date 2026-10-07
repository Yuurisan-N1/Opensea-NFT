use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{
    transaction::eip2718::TypedTransaction, Address, BlockId, BlockNumber, Bytes,
    TransactionReceipt, TransactionRequest, H256, U256,
};

use crate::core::error::MintError;
use crate::ui::logger::{lg, ly};

pub type Client = Arc<SignerMiddleware<Provider<Http>, LocalWallet>>;

pub fn load_proxy() -> Option<String> {
    let path = proxy_path();
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        return Some(normalize_proxy(trimmed));
    }
    None
}

fn normalize_proxy(line: &str) -> String {
    if line.starts_with("http://") || line.starts_with("https://") || line.starts_with("socks5://") {
        return line.to_string();
    }
    let parts: Vec<&str> = line.split(':').collect();
    match parts.len() {
        2 => format!("http://{}:{}", parts[0].trim(), parts[1].trim()),
        4 => format!(
            "http://{}:{}@{}:{}",
            parts[2].trim(),
            parts[3].trim(),
            parts[0].trim(),
            parts[1].trim()
        ),
        _ => line.to_string(),
    }
}

fn proxy_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
    path.pop();
    path.push("proxy.txt");
    if !path.exists() {
        let local = std::path::PathBuf::from("proxy.txt");
        if local.exists() {
            return local;
        }
    }
    path
}

pub fn http_client(proxy: Option<&str>, timeout_secs: u64) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .pool_max_idle_per_host(8)
        .tcp_keepalive(Duration::from_secs(30));

    if let Some(url) = proxy {
        builder = builder.proxy(reqwest::Proxy::all(url).context("Proxy URL is not valid")?);
    }

    builder.build().context("HTTP client could not be built")
}

pub fn build_provider(rpc_url: &str, http: &reqwest::Client) -> Result<Provider<Http>> {
    let url = url::Url::parse(rpc_url).context("RPC URL is not valid")?;
    Ok(Provider::new(Http::new_with_client(url, http.clone())))
}

pub async fn connect(
    rpc_url: &str,
    chain_id: u64,
    wallet: LocalWallet,
    http: &reqwest::Client,
) -> Result<Client> {
    let provider = build_provider(rpc_url, http)?;
    let wallet = wallet.with_chain_id(chain_id);
    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    let _ = client.get_block_number().await.context("RPC is not reachable")?;
    Ok(client)
}

pub async fn pending_nonce(client: &Client) -> Result<u64> {
    let address = client.address();
    let nonce = client
        .get_transaction_count(address, Some(BlockId::Number(BlockNumber::Pending)))
        .await
        .context("Nonce lookup failed")?;
    Ok(nonce.as_u64())
}

pub async fn balance_of(client: &Client) -> Result<U256> {
    client
        .get_balance(client.address(), None)
        .await
        .context("Balance lookup failed")
}

pub async fn gas_price(client: &Client) -> Result<U256> {
    client.get_gas_price().await.context("Gas price lookup failed")
}

pub struct FeeParams {
    pub max_fee: U256,
    pub priority_fee: U256,
    pub base_fee: U256,
}

pub async fn fee_params(client: &Client, priority_wei: u128, max_fee_wei: u128) -> FeeParams {
    let base_fee = match client.get_block(BlockNumber::Latest).await {
        Ok(Some(block)) => block.base_fee_per_gas.unwrap_or_else(U256::zero),
        _ => U256::zero(),
    };

    let priority_fee = U256::from(priority_wei);
    let ceiling = U256::from(max_fee_wei);
    let suggested = base_fee.saturating_mul(U256::from(2)) + priority_fee;
    let max_fee = if suggested > ceiling { ceiling } else { suggested };

    FeeParams {
        max_fee,
        priority_fee,
        base_fee,
    }
}

pub async fn estimate_gas(
    client: &Client,
    to: Address,
    data: Bytes,
    value: U256,
    fallback: u64,
) -> u64 {
    let from = client.address();
    let tx = TransactionRequest::new()
        .from(from)
        .to(to)
        .data(data)
        .value(value);
    let typed: TypedTransaction = tx.into();
    match client.estimate_gas(&typed, None).await {
        Ok(gas) => (gas * U256::from(13) / U256::from(10) + U256::from(6_000)).as_u64(),
        Err(_) => fallback,
    }
}

pub async fn eth_call(client: &Client, to: Address, data: Bytes) -> Result<Bytes> {
    let tx = TransactionRequest::new()
        .from(client.address())
        .to(to)
        .data(data);
    let typed: TypedTransaction = tx.into();
    client
        .call(&typed, None)
        .await
        .context("Contract call did not return data")
}

pub async fn send_raw(rpc_url: &str, http: &reqwest::Client, raw_hex: &str) -> Result<H256> {
    let payload = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_sendRawTransaction\",\"params\":[\"{}\"]}}",
        raw_hex
    );

    let response = http
        .post(rpc_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(payload)
        .send()
        .await
        .context("Broadcast request did not complete")?;

    let mut body = response
        .bytes()
        .await
        .context("Broadcast response could not be read")?
        .to_vec();

    if body.is_empty() {
        anyhow::bail!(MintError::Rpc);
    }

    let parsed: serde_json::Value =
        simd_json::from_slice(&mut body).context("Broadcast response is not valid JSON")?;

    if let Some(error) = parsed.get("error") {
        let message = error
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown node error");
        let lowered = message.to_lowercase();
        if lowered.contains("not minting")
            || lowered.contains("dropnotminting")
            || lowered.contains("not active")
            || lowered.contains("not started")
        {
            anyhow::bail!(MintError::DropNotMinting);
        }
        if lowered.contains("already known") || lowered.contains("nonce too low") {
            anyhow::bail!(MintError::Rejected);
        }
        anyhow::bail!("{}", crate::ui::logger::short(message, 60));
    }

    let result = parsed
        .get("result")
        .and_then(|value| value.as_str())
        .context("Broadcast response carried no transaction hash")?;

    let clean = result.trim_start_matches("0x");
    let bytes = hex::decode(clean).context("Broadcast hash is not valid hex")?;
    if bytes.len() != 32 {
        anyhow::bail!("Broadcast hash has an unexpected length");
    }

    Ok(H256::from_slice(&bytes))
}

pub async fn wait_receipt(
    client: &Client,
    tx_hash: H256,
    timeout_secs: u64,
    poll_ms: u64,
) -> Result<TransactionReceipt> {
    let started = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let poll = Duration::from_millis(poll_ms.max(250));

    loop {
        if let Ok(Some(receipt)) = client.get_transaction_receipt(tx_hash).await {
            return Ok(receipt);
        }
        if started.elapsed() > timeout {
            anyhow::bail!(MintError::ReceiptTimeout);
        }
        tokio::time::sleep(poll).await;
    }
}

pub fn receipt_ok(receipt: &TransactionReceipt) -> bool {
    receipt.status.map(|s| s.as_u64()).unwrap_or(0) == 1
}

pub fn report_endpoint(label: &str, rpc_url: &str) {
    lg(&format!("RPC endpoint {} is ready on {}", label, crate::ui::logger::short(rpc_url, 44)));
}

pub fn report_retry(label: &str, attempt: usize) {
    ly(&format!(
        "RPC endpoint {} did not answer and attempt {} follows",
        label, attempt
    ));
}

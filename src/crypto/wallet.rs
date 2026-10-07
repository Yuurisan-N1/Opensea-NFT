use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use ethers::providers::Middleware;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, U256};
use futures_util::future::join_all;
use regex::Regex;

use crate::network::rpc::{self, Client};
use crate::protocols::opensea;
use crate::ui::logger::{lg, lr, ly};

pub struct WalletEntry {
    pub wallet: LocalWallet,
    pub address: Address,
}

pub fn load_wallet() -> Result<WalletEntry> {
    let path = env_path();
    if path.exists() {
        dotenvy::from_path(&path).ok();
    } else {
        dotenvy::dotenv().ok();
    }

    let raw = std::env::var("PRIVATEKEY").context("Private key is missing in the env file")?;
    let normalized = normalize_key(&raw)?;

    let wallet: LocalWallet = normalized
        .parse()
        .context("Private key could not be parsed")?;
    let address = wallet.address();

    Ok(WalletEntry { wallet, address })
}

fn normalize_key(value: &str) -> Result<String> {
    let trimmed = value.trim().trim_matches('"').trim_matches('\'');
    let body = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let pattern = Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();
    if !pattern.is_match(body) {
        anyhow::bail!("Private key format is not valid");
    }
    Ok(format!("0x{}", body.to_lowercase()))
}

fn env_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push(".env");
    if !path.exists() {
        let local = PathBuf::from(".env");
        if local.exists() {
            return local;
        }
    }
    path
}

pub async fn announce(http: &reqwest::Client, address: Address) {
    lg(&format!("Wallet {:#x}", address));

    let probes = opensea::CHAINS.iter().map(|spec| {
        let endpoint = spec.rpc.first().copied().unwrap_or_default();
        let http = http.clone();
        let name = spec.name;
        let native = spec.native;
        async move {
            if endpoint.is_empty() {
                return (name, native, None);
            }
            let provider = match rpc::build_provider(endpoint, &http) {
                Ok(ready) => ready,
                Err(_) => return (name, native, None),
            };
            let read = tokio::time::timeout(
                Duration::from_secs(8),
                provider.get_balance(address, None),
            )
            .await;
            let balance = read.ok().and_then(|result| result.ok());
            (name, native, balance)
        }
    });

    let mut shown = 0usize;
    for (name, native, balance) in join_all(probes).await {
        let value = match balance {
            Some(amount) => amount,
            None => continue,
        };
        if displays_zero(value) {
            continue;
        }
        lg(&format!("{} balance {}", name, format_amount(value, native)));
        shown += 1;
    }

    if shown == 0 {
        ly("Wallet carries no native balance on any supported chain");
    }
}

pub async fn report_balance(client: &Client, limit: U256, symbol: &str) -> U256 {
    let balance = match crate::network::rpc::balance_of(client).await {
        Ok(value) => value,
        Err(_) => {
            lr("Wallet balance could not be read from the endpoint");
            return U256::zero();
        }
    };

    if displays_zero(balance) {
        return balance;
    }

    let human = format_amount(balance, symbol);

    if balance < limit {
        ly(&format!(
            "Wallet balance {} is below the configured limit {}",
            human,
            format_amount(limit, symbol)
        ));
    } else {
        lg(&format!("Wallet balance {} is ready for this mint", human));
    }

    balance
}

pub fn format_amount(value: U256, symbol: &str) -> String {
    let scale = U256::from(10u64).pow(U256::from(18));
    let whole = value / scale;
    let fraction = value % scale;
    let fraction_str = format!("{:018}", fraction);
    let trimmed = fraction_str.trim_end_matches('0');
    if trimmed.is_empty() {
        format!("{} {}", whole, symbol)
    } else {
        format!("{}.{} {}", whole, &trimmed[..trimmed.len().min(6)], symbol)
    }
}

fn displays_zero(value: U256) -> bool {
    value < U256::from(10u64).pow(U256::from(12))
}

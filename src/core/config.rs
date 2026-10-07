use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::core::error::{MintError, MintResult};

#[derive(Debug, Clone, Deserialize)]
pub struct Gas {
    pub priority_gwei: f64,
    pub max_gwei: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_quantity")]
    pub quantity: u64,
    pub max_price_native: f64,
    #[serde(default)]
    pub max_price_per_chain: HashMap<String, f64>,
    pub gas: Gas,
    #[serde(default = "default_rpc_fanout")]
    pub rpc_fanout: usize,
    #[serde(default = "default_fire_window_ms")]
    pub fire_window_ms: u64,
    #[serde(default = "default_fire_interval_ms")]
    pub fire_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub settings: Settings,
}

impl AppConfig {
    pub fn load() -> MintResult<Self> {
        let path = config_path();
        let raw = std::fs::read_to_string(&path).map_err(|_| MintError::Config)?;
        let parsed: AppConfig = serde_json::from_str(&raw).map_err(|_| MintError::Config)?;
        Ok(parsed)
    }

    pub fn price_limit_for(&self, chain_key: &str, native: &str) -> f64 {
        let qualified = format!("{}_{}", chain_key, native.to_ascii_lowercase());
        self.settings
            .max_price_per_chain
            .get(chain_key)
            .or_else(|| self.settings.max_price_per_chain.get(&qualified))
            .copied()
            .unwrap_or(self.settings.max_price_native)
    }

    pub fn chain_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        for raw in self.settings.max_price_per_chain.keys() {
            let base = raw.split('_').next().unwrap_or(raw).to_string();
            if !keys.iter().any(|seen| seen.eq_ignore_ascii_case(&base)) {
                keys.push(base);
            }
        }
        keys.sort();
        keys
    }

    pub fn priority_wei(&self) -> u128 {
        let gwei = self.settings.gas.priority_gwei.max(0.0);
        (gwei * 1_000_000_000.0) as u128
    }

    pub fn max_fee_wei(&self) -> u128 {
        let gwei = self.settings.gas.max_gwei.max(0.0);
        (gwei * 1_000_000_000.0) as u128
    }
}

fn default_quantity() -> u64 {
    1
}

fn default_rpc_fanout() -> usize {
    4
}

fn default_fire_window_ms() -> u64 {
    3000
}

fn default_fire_interval_ms() -> u64 {
    15
}

fn config_path() -> PathBuf {
    let mut p = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    p.pop();
    p.push("config.json");
    if !p.exists() {
        let local = PathBuf::from("config.json");
        if local.exists() {
            return local;
        }
    }
    p
}

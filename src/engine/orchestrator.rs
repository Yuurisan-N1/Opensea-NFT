use std::time::Duration;

use anyhow::{Context, Result};
use ethers::signers::LocalWallet;
use ethers::types::{Address, U256};

use crate::core::config::AppConfig;
use crate::crypto::{signer, wallet};
use crate::engine::sniper;
use crate::network::clock::{self, ClockSync};
use crate::network::rpc;
use crate::network::ws;
use crate::protocols::opensea::{self, api, seadrop, stage};
use crate::ui::logger::{self, countdown, lg, lr, ly, short};
use crate::ui::{banner, prompt};

const PRESIGN_LEAD_MS: i128 = 2000;

struct ChainJob {
    key: String,
    name: &'static str,
    native: &'static str,
    collection: Option<String>,
    client: rpc::Client,
    endpoints: Vec<String>,
    ws: &'static str,
    windows: Vec<stage::StageWindow>,
    limit: U256,
}

struct MenuRow {
    label: String,
    native: &'static str,
    price: U256,
    start: u64,
    end: u64,
    limit: u64,
}

pub async fn run() -> Result<()> {
    banner::print_banner();

    let config = AppConfig::load().map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let entry = wallet::load_wallet()?;

    let proxy = rpc::load_proxy();
    if let Some(url) = proxy.as_deref() {
        ly(&format!("Using proxy {}", mask_proxy(url)));
    }
    let http = rpc::http_client(proxy.as_deref(), 30)?;

    wallet::announce(&http, entry.address).await;

    let contract_text = prompt::ask_contract_address().map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let contract: Address = contract_text
        .parse()
        .context("Target contract address could not be parsed")?;

    let contract_rendered = format!("{:#x}", contract);

    let synced = clock::synchronize().await;
    let now = clock::now_ms(&synced);

    let mut jobs: Vec<ChainJob> = Vec::new();
    for key in config.chain_keys() {
        match prepare_chain(&key, &config, &entry.wallet, &http, contract, now).await {
            Ok(job) => {
                if job.windows.is_empty() {
                    continue;
                }
                jobs.push(job);
            }
            Err(error) => {
                ly(&format!(
                    "{} was skipped because {}",
                    key,
                    short(&error.to_string(), 34)
                ));
            }
        }
    }

    if jobs.is_empty() {
        lr("No chain carried a usable drop stage for this contract");
        return Ok(());
    }

    match jobs.iter().find_map(|job| job.collection.clone()) {
        Some(collection) => lg(&format!(
            "Target contract {} named {}",
            contract_rendered, collection
        )),
        None => lg(&format!(
            "Target contract {}",
            contract_rendered
        )),
    }

    let mut rows: Vec<MenuRow> = Vec::new();
    for job in &jobs {
        for window in stage::menu(&job.windows) {
            if rows.iter().any(|row| row.label == window.label) {
                continue;
            }
            rows.push(MenuRow {
                label: window.label.clone(),
                native: job.native,
                price: window.price,
                start: window.start,
                end: window.end,
                limit: window.limit_per_wallet,
            });
        }
    }
    rows.sort_by_key(|row| row.start);

    if rows.is_empty() {
        lr("This contract has no stage that can still be armed");
        return Ok(());
    }

    for index in 0..rows.len() {
        let row = &rows[index];
        let window_text = if row.end == 0 {
            format!("starts {}", logger::when(row.start))
        } else {
            format!(
                "starts {} and ends {}",
                logger::when(row.start),
                logger::when(row.end)
            )
        };
        lg(&format!(
            "{} {} {} with price {} and limit {}",
            index + 1,
            row.label,
            window_text,
            wallet::format_amount(row.price, row.native),
            row.limit
        ));
    }

    let picked =
        prompt::ask_stage_numbers(rows.len()).map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let mut chosen: Vec<String> = Vec::new();
    for value in &picked {
        chosen.push(rows[value - 1].label.clone());
    }

    jobs.retain_mut(|job| {
        job.windows
            .retain(|window| chosen.iter().any(|label| label == &window.label));
        !job.windows.is_empty()
    });

    if jobs.is_empty() {
        lr("The stages you picked are not open on any supported chain for this contract");
        return Ok(());
    }

    schedule(&config, &entry.wallet, entry.address, &http, &synced, contract, &mut jobs).await
}

async fn prepare_chain(
    key: &str,
    config: &AppConfig,
    wallet_ref: &LocalWallet,
    http: &reqwest::Client,
    contract: Address,
    now: i128,
) -> Result<ChainJob> {
    let spec = opensea::chain_by_key(key).map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let mut client: Option<rpc::Client> = None;
    let mut last_error = String::new();
    for endpoint in spec.rpc {
        match rpc::connect(endpoint, spec.chain_id, wallet_ref.clone(), http).await {
            Ok(ready) => {
                rpc::report_endpoint(spec.name, endpoint);
                client = Some(ready);
                break;
            }
            Err(error) => last_error = error.to_string(),
        }
    }

    let client = client.ok_or_else(|| anyhow::anyhow!("{}", short(&last_error, 40)))?;

    let contract_hex = format!("{:#x}", contract);
    let (slug, collection) = match api::fetch_collection(http, &contract_hex, spec.key).await {
        Some((slug, name)) => (Some(slug), Some(name)),
        None => {
            let name = match rpc::eth_call(&client, contract, seadrop::encode_name()).await {
                Ok(bytes) => seadrop::decode_string(&bytes.0),
                Err(_) => None,
            };
            (None, name)
        }
    };

    let limit = U256::from((config.price_limit_for(spec.key, spec.native) * 1_000_000_000_000_000_000.0) as u128);

    let mut windows: Vec<stage::StageWindow> = Vec::new();

    if let Some(slug) = slug.as_deref() {
        for remote in api::fetch_stages(http, slug).await {
            windows.push(stage::build_window(
                &remote.label,
                remote.start_time,
                remote.end_time,
                remote.price_wei,
                remote.limit_per_wallet,
                remote.signed,
                now,
            ));
        }
    }

    if !windows.iter().any(|window| !window.signed) {
        let call = rpc::eth_call(&client, opensea::seadrop_address(), seadrop::encode_get_public_drop(contract)).await;
        if let Ok(drop_data) = call {
            if let Some(public) = seadrop::decode_public_drop(&drop_data.0) {
                if public.start_time > 0 {
                    windows.push(stage::build_window(
                        "Public stage",
                        public.start_time,
                        public.end_time,
                        public.mint_price,
                        public.max_total_mintable_by_wallet,
                        false,
                        now,
                    ));
                }
            }
        }
    }

    Ok(ChainJob {
        key: spec.key.to_string(),
        name: spec.name,
        native: spec.native,
        collection,
        client,
        endpoints: spec.rpc.iter().map(|value| value.to_string()).collect(),
        ws: spec.ws,
        windows,
        limit,
    })
}

async fn schedule(
    config: &AppConfig,
    wallet_ref: &LocalWallet,
    address: Address,
    http: &reqwest::Client,
    synced: &ClockSync,
    contract: Address,
    jobs: &mut [ChainJob],
) -> Result<()> {
    let quantity = U256::from(config.settings.quantity);

    loop {
        let now_ms = clock::now_ms(synced);

        let mut best: Option<(usize, usize, i128)> = None;
        for (job_index, job) in jobs.iter().enumerate() {
            for (window_index, window) in job.windows.iter().enumerate() {
                if window.state == stage::StageState::Disabled || window.state == stage::StageState::Ended {
                    continue;
                }
                if window.end != 0 && stage::ms_until(window.end, now_ms) <= 0 {
                    continue;
                }
                let start_ms = (window.start as i128) * 1000;
                if best.map(|(_, _, current)| start_ms < current).unwrap_or(true) {
                    best = Some((job_index, window_index, start_ms));
                }
            }
        }

        let picked = match best {
            Some(value) => value,
            None => {
                lr("Every stage you armed has already ended for this contract");
                return Ok(());
            }
        };
        let (job_index, window_index, start_ms) = picked;

        let chain_name = jobs[job_index].name;
        let native = jobs[job_index].native;
        let chain_id = opensea::chain_by_key(&jobs[job_index].key)
            .map(|spec| spec.chain_id)
            .unwrap_or(0);
        let limit = jobs[job_index].limit;
        let client = jobs[job_index].client.clone();
        let endpoints = jobs[job_index].endpoints.clone();
        let ws_url = jobs[job_index].ws;
        let window = jobs[job_index].windows[window_index].clone();

        if window.price > limit {
            lr(&format!(
                "{} {} price {} is above the configured limit",
                chain_name,
                window.label,
                wallet::format_amount(window.price, native)
            ));
            jobs[job_index].windows[window_index].state = stage::StageState::Ended;
            continue;
        }

        let required = window.price * quantity;
        let balance = wallet::report_balance(&client, required, native).await;
        if balance < required {
            lr(&format!(
                "{} {} was skipped because the wallet cannot pay it",
                chain_name, window.label
            ));
            jobs[job_index].windows[window_index].state = stage::StageState::Ended;
            continue;
        }

        let remaining_ms = start_ms - clock::now_ms(synced);
        if remaining_ms > PRESIGN_LEAD_MS + 1000 {
            let seconds = ((remaining_ms - PRESIGN_LEAD_MS) / 1000) as u64;
            lg(&format!(
                "{} {} is ready and the sniper waits for the opening",
                chain_name, window.label
            ));
            countdown(seconds, "Next mint window in").await;
        }

        wait_until(start_ms - PRESIGN_LEAD_MS, synced).await;

        let fee_recipient = resolve_fee_recipient(&client, contract, address).await;
        let value = window.price * quantity;

        let data = if !window.signed {
            seadrop::encode_mint_public(contract, fee_recipient, address, quantity)
        } else {
            lr(&format!(
                "{} {} needs an OpenSea signature and the public API does not serve one",
                chain_name, window.label
            ));
            jobs[job_index].windows[window_index].state = stage::StageState::Ended;
            continue;
        };

        let gas = rpc::estimate_gas(&client, contract, data.clone(), value, 260_000).await;
        let fee = rpc::fee_params(&client, config.priority_wei(), config.max_fee_wei()).await;
        let nonce = rpc::pending_nonce(&client).await?;

        let signed = signer::presign(
            wallet_ref,
            &fee,
            &signer::SignRequest {
                chain_id,
                to: contract,
                data,
                value,
                gas,
                nonce,
            },
        )
        .await?;

        lg(&format!(
            "{} {} transaction is signed with nonce {} and gas {}",
            chain_name, window.label, signed.nonce, signed.gas
        ));

        wait_until(start_ms, synced).await;

        let outcome = sniper::fire_parallel(
            &signed.raw,
            &endpoints,
            http,
            config.settings.rpc_fanout,
            config.settings.fire_window_ms,
            config.settings.fire_interval_ms,
        )
        .await;

        match outcome {
            Ok(fired) => {
                lg(&format!("Transaction hash {}", fired.tx_hash));
                if !ws_url.is_empty() {
                    let _ = ws::await_inclusion(ws_url, &client, fired.tx_hash, 45).await;
                }
                match rpc::wait_receipt(&client, fired.tx_hash, 180, 2000).await {
                    Ok(receipt) => {
                        if rpc::receipt_ok(&receipt) {
                            lg(&format!(
                                "Mint confirmed on {} in block {}",
                                chain_name,
                                receipt.block_number.map(|value| value.as_u64()).unwrap_or(0)
                            ));
                        } else {
                            lr(&format!("Mint on {} reverted inside the block", chain_name));
                        }
                    }
                    Err(_) => {
                        ly(&format!("Receipt for {} was not seen before the timeout", chain_name));
                    }
                }
            }
            Err(error) => {
                lr(&format!(
                    "{} broadcast did not succeed with {}",
                    chain_name,
                    short(&error, 34)
                ));
            }
        }

        for window in jobs[job_index].windows.iter_mut() {
            window.state = stage::StageState::Ended;
        }
    }
}

async fn resolve_fee_recipient(client: &rpc::Client, contract: Address, fallback: Address) -> Address {
    let data = seadrop::encode_get_creator_payout_address(contract);
    match rpc::eth_call(client, opensea::seadrop_address(), data).await {
        Ok(bytes) => seadrop::decode_address(&bytes.0).unwrap_or(fallback),
        Err(_) => fallback,
    }
}

async fn wait_until(target_ms: i128, synced: &ClockSync) {
    loop {
        let remaining = target_ms - clock::now_ms(synced);
        if remaining <= 0 {
            return;
        }
        let step = if remaining > 300 {
            60
        } else if remaining > 40 {
            8
        } else {
            1
        };
        tokio::time::sleep(Duration::from_millis(step)).await;
    }
}

fn mask_proxy(raw: &str) -> String {
    if let Ok(parsed) = url::Url::parse(raw) {
        let host = parsed.host_str().unwrap_or("");
        let port = parsed.port().map(|value| format!(":{}", value)).unwrap_or_default();
        let masked = if host.len() > 4 {
            format!("{}***", &host[..4])
        } else {
            format!("{}***", host)
        };
        return format!("{}://{}{}", parsed.scheme(), masked, port);
    }
    "proxy***".to_string()
}

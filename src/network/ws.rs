use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ethers::providers::Middleware;
use ethers::types::H256;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::network::rpc::{self, Client};
use crate::ui::logger::{lg, ly};

pub async fn subscribe_new_heads(ws_url: &str) -> Result<()> {
    let mut request = ws_url
        .into_client_request()
        .context("WebSocket handshake request could not be built")?;
    request
        .headers_mut()
        .insert("User-Agent", "opensea-mint-bot".parse().unwrap());

    let (mut socket, _) = tokio_tungstenite::connect_async(request)
        .await
        .context("WebSocket connection did not open")?;

    let payload = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"]}";
    socket
        .send(Message::Text(payload.to_string()))
        .await
        .context("Subscription frame could not be sent")?;

    match socket.next().await {
        Some(Ok(Message::Text(text))) => {
            if text.contains("result") {
                lg("Block subscription is live on the websocket channel");
                Ok(())
            } else {
                anyhow::bail!("Block subscription was refused by the node")
            }
        }
        _ => anyhow::bail!("Block subscription produced no confirmation"),
    }
}

pub async fn await_inclusion(
    ws_url: &str,
    client: &Client,
    tx_hash: H256,
    timeout_secs: u64,
) -> Result<H256> {
    let mut request = ws_url
        .into_client_request()
        .context("WebSocket handshake request could not be built")?;
    request
        .headers_mut()
        .insert("User-Agent", "opensea-mint-bot".parse().unwrap());

    let (mut socket, _) = tokio_tungstenite::connect_async(request)
        .await
        .context("WebSocket connection did not open")?;

    let payload = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"]}";
    socket
        .send(Message::Text(payload.to_string()))
        .await
        .context("Subscription frame could not be sent")?;

    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut heads = 0u64;

    while Instant::now() < deadline {
        let next = tokio::time::timeout(Duration::from_secs(12), socket.next()).await;
        match next {
            Ok(Some(Ok(Message::Text(text)))) => {
                if !text.contains("blockNumber") {
                    continue;
                }
                heads += 1;
                if let Ok(Some(_)) = client.get_transaction_receipt(tx_hash).await {
                    lg(&format!(
                        "Transaction appeared in a block after {} head updates",
                        heads
                    ));
                    return Ok(tx_hash);
                }
            }
            Ok(Some(Ok(_))) => {}
            Ok(Some(Err(error))) => {
                ly(&format!(
                    "Websocket stream stopped with {}",
                    crate::ui::logger::short(&error.to_string(), 40)
                ));
                break;
            }
            Ok(None) => break,
            Err(_) => {
                heads += 1;
                if let Ok(Some(_)) = client.get_transaction_receipt(tx_hash).await {
                    return Ok(tx_hash);
                }
            }
        }
    }

    let receipt = rpc::wait_receipt(client, tx_hash, 30, 500).await?;
    let _ = receipt;
    Ok(tx_hash)
}

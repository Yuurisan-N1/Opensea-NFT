use ethers::types::U256;
use serde_json::json;

const GRAPHQL_ENDPOINT: &str = "https://opensea.io/__api/graphql/";

#[derive(Debug, Clone)]
pub struct DropStage {
    pub stage_index: i64,
    pub start_time: u64,
    pub end_time: u64,
    pub price_wei: U256,
    pub label: String,
    pub signed: bool,
    pub limit_per_wallet: u64,
}

const COLLECTION_QUERY: &str = "query CollectionByContract($address: Address!, $chain: ChainIdentifier!) { collectionByContract(contract: {address: $address, chain: $chain}) { slug name } }";

const STAGES_QUERY: &str = "query DropStages($slug: String!) { dropBySlug(slug: $slug) { stages { stageIndex startTime endTime stageType maxTotalMintableByWallet label price { native { unit symbol } } } } }";

async fn graphql(http: &reqwest::Client, body: serde_json::Value) -> Option<serde_json::Value> {
    let response = http
        .post(GRAPHQL_ENDPOINT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ORIGIN, "https://opensea.io")
        .header(reqwest::header::REFERER, "https://opensea.io/")
        .json(&body)
        .send()
        .await
        .ok()?;
    response.json().await.ok()
}

pub async fn fetch_collection(
    http: &reqwest::Client,
    contract: &str,
    chain: &str,
) -> Option<(String, String)> {
    let body = json!({
        "query": COLLECTION_QUERY,
        "variables": { "address": contract, "chain": chain },
    });

    let parsed = graphql(http, body).await?;
    let node = parsed.get("data")?.get("collectionByContract")?;
    if node.is_null() {
        return None;
    }
    let slug = node.get("slug").and_then(|value| value.as_str())?.to_string();
    if slug.is_empty() {
        return None;
    }
    let name = node
        .get("name")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(&slug)
        .to_string();
    Some((slug, name))
}

pub async fn fetch_stages(http: &reqwest::Client, slug: &str) -> Vec<DropStage> {
    let body = json!({
        "query": STAGES_QUERY,
        "variables": { "slug": slug },
    });

    let parsed = match graphql(http, body).await {
        Some(value) => value,
        None => return Vec::new(),
    };

    let stages = match parsed
        .get("data")
        .and_then(|data| data.get("dropBySlug"))
        .and_then(|drop| drop.get("stages"))
        .and_then(|stages| stages.as_array())
    {
        Some(list) => list,
        None => return Vec::new(),
    };

    stages
        .iter()
        .filter_map(|entry| {
            let stage_index = entry.get("stageIndex").and_then(|v| v.as_i64())?;
            let start_time = parse_time(entry.get("startTime")?)?;
            let end_time = entry.get("endTime").and_then(parse_time).unwrap_or(0);
            let stage_type = entry
                .get("stageType")
                .and_then(|v| v.as_str())
                .unwrap_or("PUBLIC_SALE");
            let label = entry
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let limit_per_wallet = entry
                .get("maxTotalMintableByWallet")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let price_wei = entry
                .get("price")
                .and_then(|price| price.get("native"))
                .and_then(|native| native.get("unit"))
                .and_then(|unit| unit.as_f64())
                .map(units_to_wei)
                .unwrap_or_else(U256::zero);

            Some(DropStage {
                stage_index,
                start_time,
                end_time,
                price_wei,
                label: stage_label(stage_type, &label),
                signed: stage_type.eq_ignore_ascii_case("SIGNED_PRESALE"),
                limit_per_wallet,
            })
        })
        .collect()
}

fn parse_time(value: &serde_json::Value) -> Option<u64> {
    if let Some(number) = value.as_u64() {
        return Some(number);
    }
    let text = value.as_str()?;
    let parsed = chrono::DateTime::parse_from_rfc3339(text).ok()?;
    let seconds = parsed.timestamp();
    if seconds <= 0 {
        return None;
    }
    Some(seconds as u64)
}

fn units_to_wei(units: f64) -> U256 {
    if units <= 0.0 {
        return U256::zero();
    }
    U256::from((units * 1_000_000_000_000_000_000.0).round() as u128)
}

fn stage_label(stage_type: &str, label: &str) -> String {
    let trimmed = label.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    let fallback = stage_type.trim();
    if fallback.is_empty() {
        return "stage".to_string();
    }
    fallback.replace('_', " ").to_ascii_lowercase()
}

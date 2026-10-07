use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use ethers::abi::{encode, Token};
use ethers::types::{Address, Bytes, H256, U256};
use ethers::utils::keccak256;

fn selector(signature: &str) -> [u8; 4] {
    let digest = keccak256(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

fn cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn selector_hex(signature: &str) -> String {
    let mut guard = cache().lock().unwrap();
    if let Some(found) = guard.get(signature) {
        return found.clone();
    }
    let bytes = selector(signature);
    let hexed = format!("0x{}", hex::encode(bytes));
    guard.insert(signature.to_string(), hexed.clone());
    hexed
}

pub fn mint_public_selector() -> String {
    selector_hex("mintPublic(address,address,address,uint256)")
}

pub fn mint_allow_list_selector() -> String {
    selector_hex("mintAllowList(address,address,address,uint256,(uint256,uint256,uint256,uint256,uint256,uint256,uint256,bool),bytes32[])")
}

pub fn get_public_drop_selector() -> String {
    selector_hex("getPublicDrop(address)")
}

pub fn get_allow_list_merkle_root_selector() -> String {
    selector_hex("getAllowListMerkleRoot(address)")
}

#[derive(Debug, Clone)]
pub struct MintParams {
    pub mint_price: U256,
    pub max_total_mintable_by_wallet: U256,
    pub start_time: U256,
    pub end_time: U256,
    pub drop_stage_index: U256,
    pub max_token_supply_for_stage: U256,
    pub fee_bps: U256,
    pub restrict_fee_recipients: bool,
}

impl MintParams {
    pub fn as_tokens(&self) -> Vec<Token> {
        vec![
            Token::Uint(self.mint_price),
            Token::Uint(self.max_total_mintable_by_wallet),
            Token::Uint(self.start_time),
            Token::Uint(self.end_time),
            Token::Uint(self.drop_stage_index),
            Token::Uint(self.max_token_supply_for_stage),
            Token::Uint(self.fee_bps),
            Token::Bool(self.restrict_fee_recipients),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct PublicDrop {
    pub mint_price: U256,
    pub start_time: u64,
    pub end_time: u64,
    pub max_total_mintable_by_wallet: u64,
    pub fee_bps: u64,
    pub restrict_fee_recipients: bool,
}

fn with_selector(signature: &str, tokens: Vec<Token>) -> Bytes {
    let mut data = selector(signature).to_vec();
    data.extend_from_slice(&encode(&tokens));
    Bytes::from(data)
}

pub fn encode_mint_public(
    nft_contract: Address,
    fee_recipient: Address,
    minter_if_not_payer: Address,
    quantity: U256,
) -> Bytes {
    with_selector(
        "mintPublic(address,address,address,uint256)",
        vec![
            Token::Address(nft_contract),
            Token::Address(fee_recipient),
            Token::Address(minter_if_not_payer),
            Token::Uint(quantity),
        ],
    )
}

pub fn encode_mint_allow_list(
    nft_contract: Address,
    fee_recipient: Address,
    minter_if_not_payer: Address,
    quantity: U256,
    params: &MintParams,
    proof: &[H256],
) -> Bytes {
    with_selector(
        "mintAllowList(address,address,address,uint256,(uint256,uint256,uint256,uint256,uint256,uint256,uint256,bool),bytes32[])",
        vec![
            Token::Address(nft_contract),
            Token::Address(fee_recipient),
            Token::Address(minter_if_not_payer),
            Token::Uint(quantity),
            Token::Tuple(params.as_tokens()),
            Token::Array(proof.iter().map(|node| Token::FixedBytes(node.as_bytes().to_vec())).collect()),
        ],
    )
}

pub fn encode_get_public_drop(nft_contract: Address) -> Bytes {
    with_selector(
        "getPublicDrop(address)",
        vec![Token::Address(nft_contract)],
    )
}

pub fn encode_get_allow_list_merkle_root(nft_contract: Address) -> Bytes {
    with_selector(
        "getAllowListMerkleRoot(address)",
        vec![Token::Address(nft_contract)],
    )
}

pub fn encode_get_creator_payout_address(nft_contract: Address) -> Bytes {
    with_selector(
        "getCreatorPayoutAddress(address)",
        vec![Token::Address(nft_contract)],
    )
}

pub fn encode_name() -> Bytes {
    with_selector("name()", Vec::new())
}

pub fn decode_string(data: &[u8]) -> Option<String> {
    if data.len() < 64 {
        return None;
    }
    let offset = U256::from_big_endian(word(data, 0)?).as_usize();
    if offset + 32 > data.len() {
        return None;
    }
    let length = U256::from_big_endian(&data[offset..offset + 32]).as_usize();
    if length == 0 || offset + 32 + length > data.len() {
        return None;
    }
    let text = String::from_utf8_lossy(&data[offset + 32..offset + 32 + length])
        .trim()
        .to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn decode_address(data: &[u8]) -> Option<Address> {
    if data.len() < 32 {
        return None;
    }
    Some(Address::from_slice(&data[12..32]))
}

fn word(data: &[u8], index: usize) -> Option<&[u8]> {
    let start = index * 32;
    let end = start + 32;
    if end > data.len() {
        return None;
    }
    Some(&data[start..end])
}

pub fn decode_public_drop(data: &[u8]) -> Option<PublicDrop> {
    if data.len() < 32 * 6 {
        return None;
    }

    let mint_price = U256::from_big_endian(word(data, 0)?);
    let start_time = U256::from_big_endian(word(data, 1)?).as_u64();
    let end_time = U256::from_big_endian(word(data, 2)?).as_u64();
    let max_total = U256::from_big_endian(word(data, 3)?).as_u64();
    let fee_bps = U256::from_big_endian(word(data, 4)?).as_u64();
    let restrict = U256::from_big_endian(word(data, 5)?) != U256::zero();

    Some(PublicDrop {
        mint_price,
        start_time,
        end_time,
        max_total_mintable_by_wallet: max_total,
        fee_bps,
        restrict_fee_recipients: restrict,
    })
}

pub fn decode_bytes32(data: &[u8]) -> Option<H256> {
    if data.len() < 32 {
        return None;
    }
    Some(H256::from_slice(&data[..32]))
}

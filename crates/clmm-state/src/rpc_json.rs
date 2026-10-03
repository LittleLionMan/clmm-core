use anchor_lang::prelude::Pubkey;
use base64::Engine;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum RpcJsonError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("RPC returned an error: {0}")]
    Rpc(String),
    #[error("account data encoding must be base64, got {0}")]
    Encoding(String),
    #[error("invalid base64 data: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("invalid owner address: {0}")]
    Owner(String),
}

#[derive(Deserialize)]
struct RpcEnvelope {
    result: Option<RpcResult>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct RpcResult {
    value: RpcValue,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RpcValue {
    Multiple(Vec<Option<RpcAccount>>),
    Single(Option<RpcAccount>),
}

#[derive(Deserialize)]
struct RpcAccount {
    data: (String, String),
    owner: String,
}

#[derive(Debug)]
pub struct RawAccount {
    pub owner: Pubkey,
    pub data: Vec<u8>,
}

pub fn parse_accounts(json: &str) -> Result<Vec<Option<RawAccount>>, RpcJsonError> {
    let envelope: RpcEnvelope = serde_json::from_str(json)?;
    if let Some(error) = envelope.error {
        return Err(RpcJsonError::Rpc(error.to_string()));
    }
    let Some(result) = envelope.result else {
        return Err(RpcJsonError::Rpc("response has neither result nor error".to_owned()));
    };
    let accounts = match result.value {
        RpcValue::Multiple(accounts) => accounts,
        RpcValue::Single(account) => vec![account],
    };
    accounts
        .into_iter()
        .map(|account| account.map(convert).transpose())
        .collect()
}

fn convert(account: RpcAccount) -> Result<RawAccount, RpcJsonError> {
    let (encoded, encoding) = account.data;
    if encoding != "base64" {
        return Err(RpcJsonError::Encoding(encoding));
    }
    let data = base64::engine::general_purpose::STANDARD.decode(encoded)?;
    let owner = Pubkey::from_str(&account.owner).map_err(|_| RpcJsonError::Owner(account.owner.clone()))?;
    Ok(RawAccount { owner, data })
}

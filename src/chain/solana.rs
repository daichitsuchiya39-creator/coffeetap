use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;

#[derive(Debug, Clone)]
pub struct SolTransfer {
    pub source: Option<String>,
    pub destination: String,
    pub lamports: u64,
}

#[derive(Debug, Clone)]
pub struct VerifiedTransaction {
    pub slot: u64,
    pub transfers: Vec<SolTransfer>,
}

pub fn verify_signature_on_chain(rpc_url: &str, signature: &str) -> Result<VerifiedTransaction> {
    let client = RpcClient::new(rpc_url.to_string());
    let signature = Signature::from_str(signature)
        .with_context(|| format!("署名の形式が不正です: {signature}"))?;

    let tx = client
        .get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .with_context(|| format!("RPCからTxを取得できませんでした: signature={signature}"))?;

    let value = serde_json::to_value(tx).context("Tx JSON変換に失敗しました")?;
    let slot = value
        .get("slot")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("Txのslotを取得できませんでした"))?;

    let transfers = extract_sol_transfers(&value);
    if transfers.is_empty() {
        return Err(anyhow!(
            "SOL transfer命令が見つかりませんでした。対象Txがsystem transferか確認してください"
        ));
    }

    Ok(VerifiedTransaction { slot, transfers })
}

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL as f64
}

pub fn sol_to_lamports(amount_sol: f64) -> Result<u64> {
    if amount_sol.is_sign_negative() {
        return Err(anyhow!("金額は0以上で指定してください"));
    }

    let lamports = (amount_sol * LAMPORTS_PER_SOL as f64).ceil() as u64;
    Ok(lamports)
}

fn extract_sol_transfers(value: &Value) -> Vec<SolTransfer> {
    let mut transfers = Vec::new();
    walk_value(value, &mut transfers);
    transfers
}

fn walk_value(value: &Value, transfers: &mut Vec<SolTransfer>) {
    match value {
        Value::Object(map) => {
            let is_system = map.get("program").and_then(Value::as_str) == Some("system");
            if is_system {
                maybe_push_system_transfer(map, transfers);
            }

            for child in map.values() {
                walk_value(child, transfers);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk_value(item, transfers);
            }
        }
        _ => {}
    }
}

fn maybe_push_system_transfer(
    map: &serde_json::Map<String, Value>,
    transfers: &mut Vec<SolTransfer>,
) {
    let Some(parsed) = map.get("parsed") else {
        return;
    };

    let transfer_type = parsed.get("type").and_then(Value::as_str);
    if transfer_type != Some("transfer") {
        return;
    }

    let info = parsed.get("info");
    let destination = info
        .and_then(|v| v.get("destination"))
        .and_then(Value::as_str);
    let source = info
        .and_then(|v| v.get("source"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let lamports = info.and_then(|v| v.get("lamports")).and_then(Value::as_u64);

    if let (Some(destination), Some(lamports)) = (destination, lamports) {
        transfers.push(SolTransfer {
            source,
            destination: destination.to_string(),
            lamports,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_transfer_from_json() {
        let sample = serde_json::json!({
            "slot": 100,
            "transaction": {
                "transaction": {
                    "message": {
                        "instructions": [
                            {
                                "program": "system",
                                "parsed": {
                                    "type": "transfer",
                                    "info": {
                                        "source": "Donor111",
                                        "destination": "Creator111",
                                        "lamports": 1500000000u64
                                    }
                                }
                            }
                        ]
                    }
                }
            }
        });

        let transfers = extract_sol_transfers(&sample);
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].destination, "Creator111");
        assert_eq!(transfers[0].lamports, 1_500_000_000);
    }
}

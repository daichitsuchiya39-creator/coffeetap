use anyhow::{Result, anyhow};

use crate::chain::solana::{lamports_to_sol, sol_to_lamports, verify_signature_on_chain};
use crate::db::Db;
use crate::models::{Creator, NewTap};

pub fn run(
    db: &Db,
    rpc_url: &str,
    signature: &str,
    creator_name: Option<&str>,
    min_amount_sol: f64,
) -> Result<()> {
    if db.signature_exists(signature)? {
        return Err(anyhow!("このsignatureは既に検証済みです: {signature}"));
    }

    let verified = verify_signature_on_chain(rpc_url, signature)?;

    let creator = resolve_creator(db, creator_name, &verified.transfers)?;
    let required_lamports = sol_to_lamports(min_amount_sol)?;

    let matched = verified
        .transfers
        .iter()
        .filter(|t| t.destination == creator.pubkey)
        .max_by_key(|t| t.lamports)
        .ok_or_else(|| {
            anyhow!(
                "受取先不一致: creator={} pubkey={} 宛の送金がTxに見つかりません",
                creator.name,
                creator.pubkey
            )
        })?;

    if matched.lamports < required_lamports {
        return Err(anyhow!(
            "金額不足: required={} SOL, actual={} SOL",
            min_amount_sol,
            lamports_to_sol(matched.lamports)
        ));
    }

    let tap = NewTap {
        creator_id: creator.id,
        currency: "sol".to_string(),
        amount: lamports_to_sol(matched.lamports),
        signature: signature.to_string(),
        donor_pubkey: matched.source.clone(),
        slot: verified.slot as i64,
    };
    db.insert_tap(&tap)?;

    println!(
        "検証OK: creator={} amount={} SOL signature={}",
        creator.name, tap.amount, tap.signature
    );
    println!("次: history --creator {}", creator.name);
    Ok(())
}

fn resolve_creator(
    db: &Db,
    creator_name: Option<&str>,
    transfers: &[crate::chain::solana::SolTransfer],
) -> Result<Creator> {
    if let Some(name) = creator_name {
        return db
            .get_creator_by_name(name)?
            .ok_or_else(|| anyhow!("creatorが見つかりません: {name}"));
    }

    let creators = db.list_creators()?;
    for creator in creators {
        if transfers.iter().any(|t| t.destination == creator.pubkey) {
            return Ok(creator);
        }
    }

    Err(anyhow!(
        "Txの受取先に一致するcreatorがDBにありません。--creator を指定してください"
    ))
}

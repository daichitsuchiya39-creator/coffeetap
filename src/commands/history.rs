use anyhow::Result;

use crate::db::Db;

pub fn run(db: &Db, creator_name: &str) -> Result<()> {
    let rows = db.history_by_creator_name(creator_name)?;

    if rows.is_empty() {
        println!("履歴はまだありません: creator={creator_name}");
        return Ok(());
    }

    for row in rows {
        let donor = row.donor_pubkey.unwrap_or_else(|| "unknown".to_string());
        println!(
            "id={} creator={} amount={} {} sig={} donor={} slot={} at={}",
            row.id,
            row.creator_name,
            row.amount,
            row.currency,
            row.signature,
            donor,
            row.slot,
            row.created_at
        );
    }

    Ok(())
}

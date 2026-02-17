use anyhow::Result;

use crate::db::Db;

pub fn run(db: &Db, name: &str, pubkey: &str) -> Result<()> {
    let creator = db.add_creator(name, pubkey)?;
    println!(
        "クリエイター登録完了: {} ({})",
        creator.name, creator.pubkey
    );
    println!(
        "次: create-link --creator {} --amount 1 --currency sol",
        creator.name
    );
    Ok(())
}

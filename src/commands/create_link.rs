use anyhow::{Result, anyhow};

use crate::db::Db;

pub fn run(db: &Db, creator_name: &str, amount: f64, currency: &str) -> Result<()> {
    if !currency.eq_ignore_ascii_case("sol") {
        return Err(anyhow!(
            "MVP時点の対応通貨はsolのみです: currency={currency}"
        ));
    }

    if amount <= 0.0 {
        return Err(anyhow!("amountは0より大きく指定してください"));
    }

    let creator = db
        .get_creator_by_name(creator_name)?
        .ok_or_else(|| anyhow!("指定されたcreatorが存在しません: {creator_name}"))?;

    let message = format!("Support {}", creator.name).replace(' ', "%20");
    let uri = format!(
        "solana:{}?amount={}&label=CoffeeTap&message={message}",
        creator.pubkey, amount
    );

    println!("支援リンクを生成しました");
    println!("{uri}");
    println!(
        "次: 送金後に verify --signature <SIG> --creator {} --min-amount {}",
        creator.name, amount
    );
    Ok(())
}

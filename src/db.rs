use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, Error as SqlError, ErrorCode, OptionalExtension, params};

use crate::models::{Creator, NewTap, TapWithCreator};

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn =
            Connection::open(path).with_context(|| format!("DBを開けませんでした: path={path}"))?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS creators (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                pubkey TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS taps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                creator_id INTEGER NOT NULL,
                currency TEXT NOT NULL,
                amount REAL NOT NULL,
                signature TEXT NOT NULL UNIQUE,
                donor_pubkey TEXT,
                slot INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (creator_id) REFERENCES creators(id)
            );
            ",
        )?;
        Ok(())
    }

    pub fn add_creator(&self, name: &str, pubkey: &str) -> Result<Creator> {
        self.conn
            .execute(
                "INSERT INTO creators(name, pubkey) VALUES(?1, ?2)",
                params![name, pubkey],
            )
            .map_err(map_unique_err)?;

        let creator = self
            .get_creator_by_name(name)?
            .ok_or_else(|| anyhow!("クリエイター作成後の読込に失敗しました: {name}"))?;
        Ok(creator)
    }

    pub fn get_creator_by_name(&self, name: &str) -> Result<Option<Creator>> {
        self.conn
            .query_row(
                "SELECT id, name, pubkey FROM creators WHERE name = ?1",
                params![name],
                |row| {
                    Ok(Creator {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        pubkey: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_creators(&self) -> Result<Vec<Creator>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, pubkey FROM creators ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Creator {
                id: row.get(0)?,
                name: row.get(1)?,
                pubkey: row.get(2)?,
            })
        })?;

        let mut creators = Vec::new();
        for row in rows {
            creators.push(row?);
        }
        Ok(creators)
    }

    pub fn signature_exists(&self, signature: &str) -> Result<bool> {
        let exists = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM taps WHERE signature = ?1)",
            params![signature],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(exists == 1)
    }

    pub fn insert_tap(&self, tap: &NewTap) -> Result<()> {
        self.conn
            .execute(
                "
                INSERT INTO taps(creator_id, currency, amount, signature, donor_pubkey, slot)
                VALUES(?1, ?2, ?3, ?4, ?5, ?6)
                ",
                params![
                    tap.creator_id,
                    tap.currency,
                    tap.amount,
                    tap.signature,
                    tap.donor_pubkey,
                    tap.slot,
                ],
            )
            .map_err(map_unique_err)?;
        Ok(())
    }

    pub fn history_by_creator_name(&self, creator_name: &str) -> Result<Vec<TapWithCreator>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT
                taps.id,
                creators.name,
                taps.currency,
                taps.amount,
                taps.signature,
                taps.donor_pubkey,
                taps.slot,
                taps.created_at
            FROM taps
            INNER JOIN creators ON creators.id = taps.creator_id
            WHERE creators.name = ?1
            ORDER BY taps.id DESC
            ",
        )?;

        let rows = stmt.query_map(params![creator_name], |row| {
            Ok(TapWithCreator {
                id: row.get(0)?,
                creator_name: row.get(1)?,
                currency: row.get(2)?,
                amount: row.get(3)?,
                signature: row.get(4)?,
                donor_pubkey: row.get(5)?,
                slot: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut taps = Vec::new();
        for row in rows {
            taps.push(row?);
        }
        Ok(taps)
    }
}

fn map_unique_err(err: SqlError) -> anyhow::Error {
    match err {
        SqlError::SqliteFailure(code, _) if code.code == ErrorCode::ConstraintViolation => {
            anyhow!("一意制約違反: 既に同じ値が登録されています")
        }
        other => anyhow!(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn duplicate_signature_is_detected() {
        let file = NamedTempFile::new().expect("tmp file");
        let db = Db::open(file.path().to_str().expect("path string")).expect("db open");

        let creator = db.add_creator("alice", "AlicePubkey").expect("add creator");

        let tap = NewTap {
            creator_id: creator.id,
            currency: "sol".to_string(),
            amount: 1.0,
            signature: "sig-1".to_string(),
            donor_pubkey: Some("donor".to_string()),
            slot: 123,
        };

        db.insert_tap(&tap).expect("first insert");
        let duplicated = db.insert_tap(&tap);
        assert!(duplicated.is_err());
    }
}

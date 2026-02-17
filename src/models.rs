#[derive(Debug, Clone)]
pub struct Creator {
    pub id: i64,
    pub name: String,
    pub pubkey: String,
}

#[derive(Debug, Clone)]
pub struct NewTap {
    pub creator_id: i64,
    pub currency: String,
    pub amount: f64,
    pub signature: String,
    pub donor_pubkey: Option<String>,
    pub slot: i64,
}

#[derive(Debug, Clone)]
pub struct TapWithCreator {
    pub id: i64,
    pub creator_name: String,
    pub currency: String,
    pub amount: f64,
    pub signature: String,
    pub donor_pubkey: Option<String>,
    pub slot: i64,
    pub created_at: String,
}

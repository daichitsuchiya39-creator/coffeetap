mod chain;
mod commands;
mod db;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use crate::db::Db;

#[derive(Parser, Debug)]
#[command(name = "coffeetap")]
#[command(about = "CoffeeTap MVP CLI", long_about = None)]
struct Cli {
    #[arg(long, default_value = "coffeetap.db")]
    db_path: String,

    #[arg(
        long,
        env = "SOLANA_RPC_URL",
        default_value = "https://api.devnet.solana.com"
    )]
    rpc_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    AddCreator {
        #[arg(long)]
        name: String,
        #[arg(long)]
        pubkey: String,
    },
    CreateLink {
        #[arg(long)]
        creator: String,
        #[arg(long)]
        amount: f64,
        #[arg(long, default_value = "sol")]
        currency: String,
    },
    Verify {
        #[arg(long)]
        signature: String,
        #[arg(long)]
        creator: Option<String>,
        #[arg(long, default_value_t = 0.0)]
        min_amount: f64,
    },
    History {
        #[arg(long)]
        creator: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let db = Db::open(&cli.db_path)?;

    match cli.command {
        Commands::AddCreator { name, pubkey } => commands::add_creator::run(&db, &name, &pubkey),
        Commands::CreateLink {
            creator,
            amount,
            currency,
        } => commands::create_link::run(&db, &creator, amount, &currency),
        Commands::Verify {
            signature,
            creator,
            min_amount,
        } => commands::verify::run(
            &db,
            &cli.rpc_url,
            &signature,
            creator.as_deref(),
            min_amount,
        ),
        Commands::History { creator } => commands::history::run(&db, &creator),
    }
}

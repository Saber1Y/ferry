//! Drive a migration through ferry-core, the same calls the desktop app makes.
//!
//! Usage (local regtest chain from `regtest/setup-demo.sh`):
//!   migrate <dir> plan
//!   echo "<phrase>" | migrate <dir> commit
//!   migrate <dir> session        one scheduler session, prints what it did as JSON
//!   migrate <dir> progress

use std::io::Read;

use ferry_core::{FerryNetwork, Server, Wallet};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("usage: <dir> <plan|commit|session|progress>");
    let command = args.next().expect("usage: <dir> <plan|commit|session|progress>");
    // Matches regtest/heights.env and app/src-tauri.
    let wallet = Wallet::open(&dir, FerryNetwork::regtest(1, 300))?;
    let sync = Server::new("127.0.0.1", 9067);
    let broadcast = Server::new("127.0.0.1", 9077);

    match command.as_str() {
        "plan" => println!("{}", serde_json::to_string(&wallet.plan_migration()?)?),
        "commit" => {
            let mut phrase = String::new();
            std::io::stdin().read_to_string(&mut phrase)?;
            let preview = wallet.commit_migration(SecretString::new(phrase))?;
            println!("{}", serde_json::to_string(&preview)?);
        }
        "session" => println!(
            "{}",
            serde_json::to_string(&wallet.run_session(&sync, &broadcast).await?)?
        ),
        "progress" => println!("{}", serde_json::to_string(&wallet.migration_progress()?)?),
        other => anyhow::bail!("unknown command {other}"),
    }
    Ok(())
}

//! Restore a wallet from a recovery phrase (read from stdin), sync it, and print its balances.
//!
//! Usage (local regtest chain from `regtest/setup-demo.sh`):
//!   echo "<phrase>" | cargo run -p ferry-core --example restore_and_sync -- <dir> <birthday> [server]

use std::io::Read;

use ferry_core::{FerryNetwork, Server, Wallet};
use secrecy::SecretString;

const ZAT_PER_ZEC: f64 = 100_000_000.0;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("usage: <dir> <birthday> [server]");
    let birthday: u32 = args.next().expect("usage: <dir> <birthday> [server]").parse()?;
    let server = Server::parse(&args.next().unwrap_or_else(|| "127.0.0.1:9067".into()))?;
    // Matches regtest/heights.env.
    let network = FerryNetwork::regtest(1, 300);

    let wallet = match Wallet::open(&dir, network) {
        Ok(wallet) => wallet,
        Err(_) => {
            let mut phrase = String::new();
            std::io::stdin().read_to_string(&mut phrase)?;
            let wallet =
                Wallet::restore(&dir, network, SecretString::new(phrase), birthday, &server).await?;
            println!("restored into {dir}");
            wallet
        }
    };

    let started = std::time::Instant::now();
    wallet.sync(&server).await?;
    println!("synced from {server} in {:.1?}", started.elapsed());

    let b = wallet.balances()?.expect("a synced wallet has a summary");
    println!("chain tip {}, scanned to {}", b.chain_tip, b.scanned_to);
    println!(
        "Orchard  spendable {:.8} (total {:.8})",
        b.orchard_spendable as f64 / ZAT_PER_ZEC,
        b.orchard_total as f64 / ZAT_PER_ZEC
    );
    println!(
        "Ironwood spendable {:.8} (total {:.8})",
        b.ironwood_spendable as f64 / ZAT_PER_ZEC,
        b.ironwood_total as f64 / ZAT_PER_ZEC
    );
    Ok(())
}

//! Print a wallet's Orchard-only unified address, for receiving Orchard funds to migrate.
//!
//! Usage:
//!   cargo run -p ferry-core --example orchard_address -- <wallet dir> main

use ferry_core::{FerryNetwork, Wallet};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("usage: <dir> <main|test>");
    let network = match args.next().as_deref() {
        Some("main") => FerryNetwork::Main,
        Some("test") => FerryNetwork::Test,
        _ => anyhow::bail!("usage: <dir> <main|test>"),
    };
    println!("{}", Wallet::open(&dir, network)?.orchard_address()?);
    Ok(())
}

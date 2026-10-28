//! Ask lightwalletd servers which chain they serve and whether its consensus rules match the
//! ones FERRY was built for (the consensus branch ID at the server's tip).
//!
//! Usage:
//!   cargo run -p ferry-core --example server_info -- main zec.rocks:443 mainnet.lightwalletd.com:9067

use ferry_core::{FerryNetwork, Server};
use zcash_client_backend::proto::service::Empty;
use zcash_protocol::consensus::{BlockHeight, BranchId};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let network = match args.next().as_deref() {
        Some("main") => FerryNetwork::Main,
        Some("test") => FerryNetwork::Test,
        _ => anyhow::bail!("usage: <main|test> <host:port>..."),
    };
    for server in args {
        let server = Server::parse(&server)?;
        let info = match server.connect().await {
            Ok(mut client) => client.get_lightd_info(Empty {}).await,
            Err(e) => {
                println!("{server}: could not connect: {e}");
                continue;
            }
        };
        let info = match info {
            Ok(info) => info.into_inner(),
            Err(e) => {
                println!("{server}: {e}");
                continue;
            }
        };
        let tip = BlockHeight::from_u32(u32::try_from(info.block_height)?);
        let expected = format!("{:x}", u32::from(BranchId::for_height(&network, tip)));
        let verdict = if info.consensus_branch_id == expected { "matches" } else { "DIFFERS" };
        println!(
            "{server}: {} tip {} branch {} ({verdict} FERRY's {expected}), {} {}",
            info.chain_name, info.block_height, info.consensus_branch_id, info.vendor, info.version
        );
    }
    Ok(())
}

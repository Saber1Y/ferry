//! FERRY's engine.
//!
//! FERRY moves a wallet's funds from the Orchard pool to the Ironwood pool the private way
//! (ZIP 318): canonical amounts, pre-signed once, then proved and broadcast one at a time at
//! random intervals. This crate holds everything that is not user interface: networks,
//! lightwalletd connections, the wallet, the migration scheduler and its local history.

pub mod cache;
pub mod error;
pub mod lightwalletd;
pub mod migration;
pub mod network;
pub mod wallet;

pub use error::Error;
pub use lightwalletd::Server;
pub use migration::{HistoryEntry, PlanPreview, Progress, SessionOutcome};
pub use network::FerryNetwork;
pub use wallet::{Balances, Wallet};

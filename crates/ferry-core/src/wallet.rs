//! A FERRY wallet: restored from a recovery phrase, synced from a lightwalletd server, and read
//! for its Orchard and Ironwood balances.

use std::path::{Path, PathBuf};

use bip0039::{English, Mnemonic};
use rand::{rand_core::UnwrapErr, rngs::SysRng};
use secrecy::{ExposeSecret, SecretString, SecretVec};
use zcash_client_backend::data_api::{
    AccountBirthday, WalletRead, WalletWrite, chain::ChainState, wallet::ConfirmationsPolicy,
};
use zcash_client_backend::proto::service;
use zcash_client_sqlite::{WalletDb, util::SystemClock, wallet::init::init_wallet_db};
use zcash_protocol::consensus::{BlockHeight, NetworkUpgrade, Parameters};
use zeroize::Zeroize;

use crate::cache::MemoryBlockCache;
use crate::error::Error;
use crate::lightwalletd::Server;
use crate::network::FerryNetwork;

/// The wallet database file inside a wallet directory.
pub const WALLET_DB: &str = "wallet.sqlite";

/// How many blocks the sync loop downloads and scans per batch.
const SYNC_BATCH_SIZE: u32 = 1000;

pub type Db = WalletDb<rusqlite::Connection, FerryNetwork, SystemClock, UnwrapErr<SysRng>>;

/// A wallet on disk.
pub struct Wallet {
    dir: PathBuf,
    network: FerryNetwork,
}

/// The wallet's balances in zatoshis, as of its last sync.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Balances {
    /// The chain tip the wallet last saw.
    pub chain_tip: u32,
    /// The height the wallet has fully scanned to.
    pub scanned_to: u32,
    /// Orchard funds that can be spent now, which is what a migration can move.
    pub orchard_spendable: u64,
    /// All Orchard funds, including unconfirmed change.
    pub orchard_total: u64,
    pub ironwood_spendable: u64,
    pub ironwood_total: u64,
}

impl Wallet {
    /// Restores a wallet from a recovery phrase into `dir`.
    ///
    /// `birthday` is the height of the first block that could hold the wallet's funds. Funds
    /// received below it are never scanned, so they would silently be left out of a migration:
    /// when unsure, pass an earlier height (at worst the network's Sapling activation height).
    ///
    /// The phrase and seed live only in zeroizing memory for the length of this call. Nothing
    /// that can spend is written to disk.
    pub async fn restore(
        dir: impl AsRef<Path>,
        network: FerryNetwork,
        phrase: SecretString,
        birthday: u32,
        server: &Server,
    ) -> Result<Wallet, Error> {
        let dir = dir.as_ref().to_path_buf();
        let db_path = dir.join(WALLET_DB);
        if db_path.exists() {
            return Err(Error::WalletExists(dir.display().to_string()));
        }

        let mnemonic = <Mnemonic<English>>::from_phrase(phrase.expose_secret().trim())
            .map_err(|e| Error::InvalidPhrase(e.to_string()))?;
        drop(phrase);
        let seed = {
            let mut seed = mnemonic.to_seed("");
            let secret = SecretVec::new(seed.to_vec());
            seed.zeroize();
            secret
        };

        let birthday = birthday.max(network.lowest_birthday());

        let mut client = server.connect().await?;
        let tip = client
            .get_latest_block(service::ChainSpec::default())
            .await?
            .into_inner()
            .height;
        let tip = BlockHeight::from_u32(u32::try_from(tip).expect("block heights fit in u32"));
        let birthday = account_birthday(&mut client, &network, BlockHeight::from_u32(birthday), tip)
            .await?;

        std::fs::create_dir_all(&dir)?;
        let mut db = open_db(&db_path, network)?;
        init_wallet_db(&mut db, None).map_err(|e| Error::Wallet(e.to_string()))?;
        db.create_account("FERRY", &seed, &birthday, None)
            .map_err(|e| Error::Wallet(e.to_string()))?;

        Ok(Wallet { dir, network })
    }

    /// Opens an existing wallet in `dir`.
    pub fn open(dir: impl AsRef<Path>, network: FerryNetwork) -> Result<Wallet, Error> {
        let dir = dir.as_ref().to_path_buf();
        if !dir.join(WALLET_DB).exists() {
            return Err(Error::NoWallet(dir.display().to_string()));
        }
        Ok(Wallet { dir, network })
    }

    pub fn network(&self) -> FerryNetwork {
        self.network
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Opens a connection to the wallet database.
    pub fn db(&self) -> Result<Db, Error> {
        open_db(&self.dir.join(WALLET_DB), self.network)
    }

    /// Scans the chain from `server` until the wallet is up to date.
    pub async fn sync(&self, server: &Server) -> Result<(), Error> {
        let mut client = server.connect().await?;
        let cache = MemoryBlockCache::new();
        let mut db = self.db()?;
        zcash_client_backend::sync::run(&mut client, &self.network, &cache, &mut db, SYNC_BATCH_SIZE)
            .await
            .map_err(|e| Error::Sync(e.to_string()))
    }

    /// The wallet's balances as of its last sync, summed over its accounts.
    pub fn balances(&self) -> Result<Option<Balances>, Error> {
        let db = self.db()?;
        let Some(summary) = db
            .get_wallet_summary(ConfirmationsPolicy::default())
            .map_err(|e| Error::Wallet(e.to_string()))?
        else {
            return Ok(None);
        };
        let mut b = Balances {
            chain_tip: u32::from(summary.chain_tip_height()),
            scanned_to: u32::from(summary.fully_scanned_height()),
            orchard_spendable: 0,
            orchard_total: 0,
            ironwood_spendable: 0,
            ironwood_total: 0,
        };
        for account in summary.account_balances().values() {
            b.orchard_spendable += account.orchard_balance().spendable_value().into_u64();
            b.orchard_total += account.orchard_balance().total().into_u64();
            b.ironwood_spendable += account.ironwood_balance().spendable_value().into_u64();
            b.ironwood_total += account.ironwood_balance().total().into_u64();
        }
        Ok(Some(b))
    }
}

fn open_db(path: &Path, network: FerryNetwork) -> Result<Db, Error> {
    WalletDb::for_path(path, network, SystemClock, UnwrapErr(SysRng))
        .map_err(|e| Error::Wallet(e.to_string()))
}

/// The account birthday for `birthday_height`: the commitment tree state as of the block before
/// it, so scanning can start there instead of at the beginning of the chain.
async fn account_birthday(
    client: &mut service::compact_tx_streamer_client::CompactTxStreamerClient<
        tonic::transport::Channel,
    >,
    network: &FerryNetwork,
    birthday_height: BlockHeight,
    tip: BlockHeight,
) -> Result<AccountBirthday, Error> {
    let sapling = network
        .activation_height(NetworkUpgrade::Sapling)
        .expect("every FERRY network activates Sapling");
    let birthday_height = birthday_height.max(sapling);

    if birthday_height == sapling {
        // There is no tree state before the first shielded block; start from an empty tree.
        // (A lightwalletd also reads a tree-state request for height 0 as "no height given".)
        let block = client
            .get_block(service::BlockId {
                height: u64::from(birthday_height),
                ..Default::default()
            })
            .await?
            .into_inner();
        Ok(AccountBirthday::from_parts(
            ChainState::empty(birthday_height.saturating_sub(1), block.prev_hash()),
            Some(tip),
        ))
    } else {
        let treestate = client
            .get_tree_state(service::BlockId {
                height: u64::from(birthday_height) - 1,
                ..Default::default()
            })
            .await?
            .into_inner();
        AccountBirthday::from_treestate(treestate, Some(tip))
            .map_err(|_| Error::Wallet("the server returned an unreadable tree state".into()))
    }
}

//! The Orchard to Ironwood migration: plan, sign once, then prove and broadcast one piece at a
//! time (ZIP 318).
//!
//! The planning, building, signing and proving are librustzcash's `zcash_pool_migration`
//! engine. FERRY adds what the engine leaves to "the consuming application": broadcasting, the
//! session rules that keep broadcasts unlinkable, and follow-up runs for balance the engine
//! defers. Every rule here was first exercised on a local regtest node (see `evidence/`).

use std::path::Path;

use rand::{rand_core::Rng, rand_core::UnwrapErr, rngs::SysRng};
use secrecy::{ExposeSecret, SecretString};
use shardtree::{error::ShardTreeError, store::ShardStore};
use zcash_client_backend::data_api::{
    Account as _, InputSource, WalletCommitmentTrees, WalletRead,
    wallet::{TargetHeight, input_selection::LockFilter},
};
use zcash_client_backend::proto::service;
use zcash_client_sqlite::{
    AccountUuid, WalletDb, pool_migration::orchard_ironwood::PoolMigrations, util::SystemClock,
};
use zcash_keys::keys::{UnifiedFullViewingKey, UnifiedSpendingKey};
use zcash_pool_migration::engine::{
    CommitError, MigrationBackend, MigrationError, MigrationState, MigrationTransaction,
    MigrationTransferId, MigrationTxKind, MigrationTxState, PoolMigrationRead, PoolMigrationWrite,
    ProveOutcome, ProvedTransaction, commit_preparation, plan_migration, prove_preparation,
    prove_transfer,
};
use zcash_pool_migration::satisfiability::{
    AdvanceConfig, DuenessTargets, ReorgSettleDepth, ReplanThreshold, StepSatisfiability,
    advance_migration,
};
use zcash_pool_migration::scheduling::SchedulingParams;
use zcash_pool_migration::state::AdvanceStep;
use zcash_pool_migration::wallet::{WalletMigration, WalletMigrationProver};
use zcash_protocol::consensus::BlockHeight;
use zcash_protocol::value::Zatoshis;
use zcash_protocol::{ShieldedPool, TxId};

use crate::error::Error;
use crate::lightwalletd::Server;
use crate::network::FerryNetwork;
use crate::wallet::{WALLET_DB, Wallet};

/// How far the chain must move past a divergence before it is treated as permanent. The same
/// settle margin zcash-devtool uses at the 75-second block target.
const REORG_SETTLE_DEPTH: ReorgSettleDepth = ReorgSettleDepth::new(10);

/// Minimum confirmations for a preparation transaction's anchor. ZIP 318 anchors preparations at
/// the wallet's ordinary near-tip depth, and the anchor must be a height the wallet holds an
/// Orchard tree checkpoint for (it does not checkpoint every block). See regtest run 1.
const PREPARATION_ANCHOR_CONFIRMATIONS: u32 = 3;

/// ZIP 318's provisional delay distribution between pool-crossing broadcasts, in blocks.
pub const MEAN_DELAY_BLOCKS: f64 = 66.0;
pub const MAX_DELAY_BLOCKS: u32 = 576;

/// The file in the wallet directory holding the height before which no crossing may be sent.
const GATE_FILE: &str = "ferry-next-crossing-height";

/// ZIP 318: Orchard value below 0.01 ZEC is dust that stays behind.
pub const DUST_ZATOSHIS: u64 = 1_000_000;

/// What a migration run would do, for the consent screen. Amounts in zatoshis.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct PlanPreview {
    /// The amount each crossing moves into Ironwood, in the engine's shuffled order.
    pub crossings: Vec<u64>,
    /// The sum of `crossings`.
    pub value_migrated: u64,
    /// Orchard value this run leaves behind: dust, plus anything deferred to a follow-up run.
    pub left_in_orchard: u64,
    /// Network fees for this run.
    pub fees: u64,
    pub preparation_transactions: usize,
    pub preparation_layers: usize,
    /// The chain height the plan was made at.
    pub planned_at: u32,
    /// The scheduled broadcast height of the earliest and latest crossing.
    pub first_crossing_height: u32,
    pub last_crossing_height: u32,
}

impl PlanPreview {
    /// Whether this run leaves more than dust behind, so a follow-up run will be needed.
    pub fn needs_follow_up(&self) -> bool {
        self.left_in_orchard >= DUST_ZATOSHIS
    }
}

/// One migration transaction's progress, for the progress screen.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct PieceProgress {
    pub id: u32,
    /// `"preparation"` or `"crossing"`.
    pub kind: &'static str,
    /// For a crossing, the amount it moves into Ironwood.
    pub amount: Option<u64>,
    /// The engine's state for it, for example `signed`, `proved`, `broadcast` or `mined`.
    pub state: String,
    pub scheduled_height: u32,
}

/// The migration's progress.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Progress {
    pub status: String,
    /// Whether the migration still has work to do (not complete, failed or superseded).
    pub active: bool,
    pub pieces: Vec<PieceProgress>,
    /// No crossing is sent before this height (FERRY's random spacing).
    pub next_crossing_not_before: u32,
}

/// What one scheduler session did.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SessionOutcome {
    /// Synced from the sync server and proved whatever was ready.
    Synced { proved: Vec<u32> },
    /// Sent exactly one migration transaction through the broadcast server.
    Broadcast {
        id: u32,
        crossing: bool,
        txid: String,
        next_crossing_not_before: u32,
    },
    /// This run is finished but more than dust is left: the user signs a follow-up run.
    FollowUpNeeded { preview: PlanPreview },
    /// Nothing is left to migrate.
    Complete,
}

impl Wallet {
    fn migration_connection(&self) -> Result<rusqlite::Connection, Error> {
        open_connection(&self.dir().join(WALLET_DB))
    }

    /// Previews the next migration run without changing anything. `None` if there is nothing
    /// above dust to migrate.
    pub fn plan_migration(&self) -> Result<Option<PlanPreview>, Error> {
        let network = self.network();
        let mut conn = self.migration_connection()?;
        let db = WalletDb::from_connection(&mut conn, network, SystemClock, UnwrapErr(SysRng));
        let (account, _, _) = account_of(&db)?;
        let backend = PlanBackend { wallet: &db, account };
        preview(&network, &backend, &db)
    }

    /// Signs a whole migration run with the account's spending key, derived from `phrase` for
    /// the length of this call only, then persists every pre-signed transaction.
    ///
    /// Fails with [`Error::WrongPhrase`] if the phrase does not belong to this wallet.
    pub fn commit_migration(&self, phrase: SecretString) -> Result<PlanPreview, Error> {
        let network = self.network();
        let mut conn = self.migration_connection()?;

        let (account, ufvk, index) = {
            let db = WalletDb::from_connection(&mut conn, network, SystemClock, UnwrapErr(SysRng));
            account_of(&db)?
        };
        let index =
            index.ok_or_else(|| Error::Wallet("the account was not derived from a seed".into()))?;

        let usk = {
            let mnemonic = <bip0039::Mnemonic<bip0039::English>>::from_phrase(
                phrase.expose_secret().trim(),
            )
            .map_err(|e| Error::InvalidPhrase(e.to_string()))?;
            drop(phrase);
            let mut seed = mnemonic.to_seed("");
            let usk = UnifiedSpendingKey::from_seed(&network, &seed, index);
            zeroize::Zeroize::zeroize(&mut seed);
            usk.map_err(|e| Error::Wallet(format!("could not derive the spending key: {e:?}")))?
        };
        if usk.to_unified_full_viewing_key().encode(&network) != ufvk.encode(&network) {
            return Err(Error::WrongPhrase);
        }

        let loaded = PoolMigrations::for_account(network, SystemClock, &conn, account)
            .map_err(|e| Error::Wallet(e.to_string()))?
            .get_migration()
            .map_err(|e| Error::Wallet(e.to_string()))?;

        let mut rng = UnwrapErr(SysRng);
        let (state, preview) = {
            let db = WalletDb::from_connection(&mut conn, network, SystemClock, UnwrapErr(SysRng));
            let target = chain_height(&db)? + 1;
            let mut migration =
                WalletMigration::new(&db, account, ufvk, InMemoryStore::from(loaded));
            let plan = plan_migration(&network, &migration, &mut rng).map_err(plan_error)?;
            let preview = preview_of(&plan, spendable_orchard(&db, account)?, target - 1);
            let state = commit_preparation(
                &network,
                target,
                &mut migration,
                usk.orchard(),
                &plan,
                &mut rng,
                ReplanThreshold::DEFAULT,
            )
            .map_err(|e| match e {
                CommitError::MigrationInProgress => Error::MigrationInProgress,
                other => Error::Wallet(other.to_string()),
            })?;
            (state, preview)
        };
        drop(usk);

        PoolMigrations::for_account(network, SystemClock, &mut conn, account)
            .map_err(|e| Error::Wallet(e.to_string()))?
            .replace_migration(&state)
            .map_err(|e| Error::Wallet(e.to_string()))?;
        Ok(preview)
    }

    /// The current migration's progress, or `None` if no migration is in progress.
    pub fn migration_progress(&self) -> Result<Option<Progress>, Error> {
        let network = self.network();
        let mut conn = self.migration_connection()?;
        let (account, _, _) = {
            let db = WalletDb::from_connection(&mut conn, network, SystemClock, UnwrapErr(SysRng));
            account_of(&db)?
        };
        let Some(state) = PoolMigrations::for_account(network, SystemClock, &conn, account)
            .map_err(|e| Error::Wallet(e.to_string()))?
            .get_migration()
            .map_err(|e| Error::Wallet(e.to_string()))?
        else {
            return Ok(None);
        };
        Ok(Some(progress_of(&state, self.next_crossing_not_before())))
    }

    /// The height before which no crossing may be broadcast.
    pub fn next_crossing_not_before(&self) -> u32 {
        std::fs::read_to_string(self.dir().join(GATE_FILE))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    fn set_next_crossing_not_before(&self, height: u32) -> Result<(), Error> {
        std::fs::write(self.dir().join(GATE_FILE), height.to_string())?;
        Ok(())
    }

    /// Runs one scheduler session (ZIP 318 "Background scheduling"):
    ///
    /// * a session either broadcasts or syncs, never both, so a sync (which tells a server about
    ///   the wallet) cannot be correlated with a broadcast;
    /// * a session broadcasts at most one migration transaction, because several pool-crossing
    ///   transactions sent together leak the balance;
    /// * after a crossing, the next one waits a fresh random delay, so a backlog of overdue
    ///   crossings is spread out instead of sent in a burst.
    ///
    /// Call it repeatedly, for example once per new block.
    pub async fn run_session(
        &self,
        sync_server: &Server,
        broadcast_server: &Server,
    ) -> Result<SessionOutcome, Error> {
        let tip = {
            let mut conn = self.migration_connection()?;
            let db = WalletDb::from_connection(&mut conn, self.network(), SystemClock, UnwrapErr(SysRng));
            db.chain_height()
                .map_err(|e| Error::Wallet(e.to_string()))?
                .map(u32::from)
                .unwrap_or(0)
        };

        if tip >= self.next_crossing_not_before() {
            match self.try_broadcast(broadcast_server).await? {
                Broadcasted::Sent(outcome) => return Ok(outcome),
                Broadcasted::NoMigration => {
                    return Ok(match self.plan_migration()? {
                        Some(preview) => SessionOutcome::FollowUpNeeded { preview },
                        None => SessionOutcome::Complete,
                    });
                }
                Broadcasted::NothingDue => {}
            }
        }

        self.sync(sync_server).await?;
        let proved = self.prove_ready()?;
        Ok(SessionOutcome::Synced { proved })
    }

    /// Broadcasts the one migration transaction that is proved and due, if any.
    async fn try_broadcast(&self, server: &Server) -> Result<Broadcasted, Error> {
        let network = self.network();
        let mut conn = self.migration_connection()?;
        let (account, scanned, tip) = {
            let db = WalletDb::from_connection(&mut conn, network, SystemClock, UnwrapErr(SysRng));
            let (account, _, _) = account_of(&db)?;
            (account, scanned_height(&db)?, chain_height(&db)?)
        };
        let targets = DuenessTargets::new(scanned + 1, tip + 1);
        let config = AdvanceConfig::new(REORG_SETTLE_DEPTH);
        let mut rng = UnwrapErr(SysRng);

        let mut store = PoolMigrations::for_account(network, SystemClock, &mut conn, account)
            .map_err(|e| Error::Wallet(e.to_string()))?;
        let Some(mut state) = store.get_migration().map_err(|e| Error::Wallet(e.to_string()))? else {
            return Ok(Broadcasted::NoMigration);
        };
        if state.is_terminal() {
            return Ok(Broadcasted::NoMigration);
        }
        let advance = advance_migration(&mut store, &mut state, targets, &config, &mut rng)
            .map_err(|e| Error::Wallet(e.to_string()))?;
        let AdvanceStep::Broadcast { id } = advance.step() else {
            return Ok(Broadcasted::NothingDue);
        };
        let id = *id;
        let crossing = matches!(kind_of(&state, id), Some(MigrationTxKind::Transfer { .. }));

        let tx = store
            .take_transaction_for_broadcast(UnwrapErr(SysRng), &state, id)
            .map_err(|e| Error::Wallet(e.to_string()))?;
        let txid = tx.txid();
        let mut raw = service::RawTransaction::default();
        tx.write(&mut raw.data)?;

        let mut client = server.connect().await?;
        let response = client.send_transaction(raw).await?.into_inner();
        if response.error_code != 0 {
            state.report_broadcast_failure(id, tip);
            store.replace_migration(&state).map_err(|e| Error::Wallet(e.to_string()))?;
            return Err(Error::Rejected(format!(
                "code {}: {}",
                response.error_code, response.error_message
            )));
        }
        state.mark_broadcast(id);
        store.replace_migration(&state).map_err(|e| Error::Wallet(e.to_string()))?;

        let mut next = self.next_crossing_not_before();
        if crossing {
            next = u32::from(tip) + draw_delay(&mut rng);
            self.set_next_crossing_not_before(next)?;
        }
        Ok(Broadcasted::Sent(SessionOutcome::Broadcast {
            id: u32::from(id),
            crossing,
            txid: txid.to_string(),
            next_crossing_not_before: next,
        }))
    }

    /// Proves every migration transaction that is ready. Proving is local and emits nothing a
    /// network observer can see, so there is no reason to space it out.
    fn prove_ready(&self) -> Result<Vec<u32>, Error> {
        let network = self.network();
        let mut conn = self.migration_connection()?;
        let (account, fvk, scanned, tip, preparation_anchor) = {
            let mut db =
                WalletDb::from_connection(&mut conn, network, SystemClock, UnwrapErr(SysRng));
            let (account, ufvk, _) = account_of(&db)?;
            let fvk = ufvk
                .orchard()
                .cloned()
                .ok_or_else(|| Error::Wallet("the account has no Orchard viewing key".into()))?;
            let scanned = scanned_height(&db)?;
            let tip = chain_height(&db)?;
            let anchor = newest_checkpoint_at_or_below(
                &mut db,
                scanned - (PREPARATION_ANCHOR_CONFIRMATIONS - 1),
            )?;
            (account, fvk, scanned, tip, anchor)
        };
        let targets = DuenessTargets::new(scanned + 1, tip + 1);
        let config = AdvanceConfig::new(REORG_SETTLE_DEPTH);
        let mut rng = UnwrapErr(SysRng);

        let (advance, mut state) = {
            let mut store = PoolMigrations::for_account(network, SystemClock, &mut conn, account)
                .map_err(|e| Error::Wallet(e.to_string()))?;
            let Some(mut state) = store.get_migration().map_err(|e| Error::Wallet(e.to_string()))?
            else {
                return Ok(vec![]);
            };
            if state.is_terminal() {
                return Ok(vec![]);
            }
            let advance = advance_migration(&mut store, &mut state, targets, &config, &mut rng)
                .map_err(|e| Error::Wallet(e.to_string()))?;
            (advance, state)
        };

        let AdvanceStep::Prove { transactions } = advance.step() else {
            return Ok(vec![]);
        };
        let mut proved = vec![];
        for target in transactions {
            let id = target.id();
            let outcome = {
                let mut db =
                    WalletDb::from_connection(&mut conn, network, SystemClock, UnwrapErr(SysRng));
                let mut prover =
                    WalletMigrationProver::new(&mut db, UnwrapErr(SysRng), account, fvk.clone());
                match target.kind() {
                    MigrationTxKind::Transfer { .. } => {
                        prove_transfer(&network, &mut prover, &mut state, id, scanned, &mut rng)
                    }
                    MigrationTxKind::Preparation { .. } => {
                        let Some(anchor) = preparation_anchor else {
                            // The wallet holds no checkpoint deep enough yet; a later sync will.
                            continue;
                        };
                        prove_preparation(&mut prover, &mut state, id, anchor)
                    }
                }
                .map_err(|e| Error::Wallet(format!("proving piece {} failed: {e}", u32::from(id))))?
            };
            let mut store = PoolMigrations::for_account(network, SystemClock, &mut conn, account)
                .map_err(|e| Error::Wallet(e.to_string()))?;
            match outcome {
                ProveOutcome::Proved(proven) => {
                    store
                        .store_proved_transaction(&mut state, proven)
                        .map_err(|e| Error::Wallet(e.to_string()))?;
                    proved.push(u32::from(id));
                }
                ProveOutcome::NotYetProvable | ProveOutcome::MarkedUnsatisfiable { .. } => {
                    store.replace_migration(&state).map_err(|e| Error::Wallet(e.to_string()))?;
                }
            }
        }
        Ok(proved)
    }
}

enum Broadcasted {
    Sent(SessionOutcome),
    NothingDue,
    NoMigration,
}

/// Draws the delay before the next crossing: exponential with mean [`MEAN_DELAY_BLOCKS`] from a
/// CSPRNG, redrawn above [`MAX_DELAY_BLOCKS`], and at least one block so two crossings never
/// share a block.
pub fn draw_delay<R: Rng>(rng: &mut R) -> u32 {
    loop {
        // A uniform draw in (0, 1).
        let u = ((rng.next_u64() >> 11) as f64 + 1.0) / ((1u64 << 53) as f64 + 1.0);
        let d = (-u.ln() * MEAN_DELAY_BLOCKS).round() as u32;
        if d <= MAX_DELAY_BLOCKS {
            return d.max(1);
        }
    }
}

/// A wallet handle over a borrowed connection, so the migration store can share it.
type BorrowedDb<'a> = WalletDb<&'a mut rusqlite::Connection, FerryNetwork, SystemClock, UnwrapErr<SysRng>>;

/// The wallet's single account: its id, viewing key and (for a seed-derived account) its
/// ZIP 32 index.
fn account_of<W>(db: &W) -> Result<(AccountUuid, UnifiedFullViewingKey, Option<zip32::AccountId>), Error>
where
    W: WalletRead<AccountId = AccountUuid>,
    W::Error: std::fmt::Display,
{
    let id = *db
        .get_account_ids()
        .map_err(|e| Error::Wallet(e.to_string()))?
        .first()
        .ok_or_else(|| Error::Wallet("the wallet has no account".into()))?;
    let account = db
        .get_account(id)
        .map_err(|e| Error::Wallet(e.to_string()))?
        .ok_or_else(|| Error::Wallet("the wallet's account is missing".into()))?;
    let ufvk = account
        .ufvk()
        .cloned()
        .ok_or_else(|| Error::Wallet("the account has no viewing key".into()))?;
    let index = account.source().key_derivation().map(|d| d.account_index());
    Ok((id, ufvk, index))
}

fn open_connection(path: &Path) -> Result<rusqlite::Connection, Error> {
    let conn = rusqlite::Connection::open(path).map_err(|e| Error::Wallet(e.to_string()))?;
    rusqlite::vtab::array::load_module(&conn).map_err(|e| Error::Wallet(e.to_string()))?;
    Ok(conn)
}

fn chain_height<W: WalletRead>(db: &W) -> Result<BlockHeight, Error>
where
    W::Error: std::fmt::Display,
{
    db.chain_height()
        .map_err(|e| Error::Wallet(e.to_string()))?
        .ok_or_else(|| Error::Wallet("the wallet has not synced yet".into()))
}

fn scanned_height<W: WalletRead>(db: &W) -> Result<BlockHeight, Error>
where
    W::Error: std::fmt::Display,
{
    Ok(db
        .block_fully_scanned()
        .map_err(|e| Error::Wallet(e.to_string()))?
        .ok_or_else(|| Error::Wallet("the wallet has not scanned any blocks yet".into()))?
        .block_height())
}

/// The newest height at or below `deepest` that the wallet holds an Orchard tree checkpoint for.
fn newest_checkpoint_at_or_below(
    db: &mut BorrowedDb<'_>,
    deepest: BlockHeight,
) -> Result<Option<BlockHeight>, Error> {
    db.with_orchard_tree_mut(|tree| {
        let store = tree.store();
        let mut depth = 0;
        loop {
            match store.get_checkpoint_at_depth(depth).map_err(ShardTreeError::Storage)? {
                Some((height, _)) if height <= deepest => return Ok(Some(height)),
                Some(_) => depth += 1,
                None => return Ok(None),
            }
        }
    })
    .map_err(|e: ShardTreeError<_>| Error::Wallet(format!("reading the Orchard tree failed: {e}")))
}

fn kind_of(state: &MigrationState, id: MigrationTransferId) -> Option<MigrationTxKind> {
    state
        .transactions()
        .iter()
        .find(|t| t.id() == id)
        .map(|t| t.kind())
}

fn spendable_orchard<W>(db: &W, account: <W as InputSource>::AccountId) -> Result<u64, Error>
where
    W: WalletRead + InputSource<AccountId = <W as WalletRead>::AccountId>,
    <W as WalletRead>::Error: std::fmt::Display,
    <W as InputSource>::Error: std::fmt::Display,
{
    let tip = chain_height(db)?;
    let notes = db
        .select_unspent_notes(
            account,
            &[ShieldedPool::Orchard],
            TargetHeight::from(u32::from(tip) + 1),
            &[],
            LockFilter::Unfiltered,
        )
        .map_err(|e| Error::Wallet(e.to_string()))?;
    Ok(notes.orchard().iter().map(|n| n.note().value().inner()).sum())
}

fn plan_error<E: std::fmt::Display>(e: MigrationError<E>) -> Error {
    Error::Wallet(e.to_string())
}

fn preview<W>(
    network: &FerryNetwork,
    backend: &PlanBackend<'_, W>,
    db: &W,
) -> Result<Option<PlanPreview>, Error>
where
    W: WalletRead + InputSource<AccountId = <W as WalletRead>::AccountId>,
    <W as WalletRead>::Error: std::fmt::Display,
    <W as InputSource>::Error: std::fmt::Display,
{
    let mut rng = UnwrapErr(SysRng);
    let plan = match plan_migration(network, backend, &mut rng) {
        Ok(plan) => plan,
        Err(MigrationError::NothingToMigrate) => return Ok(None),
        Err(e) => return Err(plan_error(e)),
    };
    let spendable = spendable_orchard(db, backend.account)?;
    Ok(Some(preview_of(&plan, spendable, chain_height(db)?)))
}

fn preview_of(
    plan: &zcash_pool_migration::engine::MigrationPlan,
    spendable: u64,
    planned_at: BlockHeight,
) -> PlanPreview {
    let crossings: Vec<u64> = plan.crossing_values().iter().map(|z| z.into_u64()).collect();
    let value_migrated = plan.value_migrated().into_u64();
    let left_in_orchard = plan.residual().into_u64();
    let heights: Vec<u32> = plan
        .schedule()
        .iter()
        .map(|s| u32::from(s.broadcast_height()))
        .collect();
    PlanPreview {
        crossings,
        value_migrated,
        left_in_orchard,
        fees: spendable.saturating_sub(value_migrated + left_in_orchard),
        preparation_transactions: plan.preparation_tx_count(),
        preparation_layers: plan.preparation_layer_count(),
        planned_at: u32::from(planned_at),
        first_crossing_height: heights.iter().copied().min().unwrap_or(0),
        last_crossing_height: heights.iter().copied().max().unwrap_or(0),
    }
}

fn progress_of(state: &MigrationState, next_crossing_not_before: u32) -> Progress {
    let crossing_values: Vec<u64> = state
        .denominations()
        .crossing_values()
        .iter()
        .map(|z| z.into_u64())
        .collect();
    let pieces = state
        .transactions()
        .iter()
        .map(|t| {
            let (kind, amount) = match t.kind() {
                MigrationTxKind::Transfer { crossing } => {
                    ("crossing", crossing_values.get(crossing).copied())
                }
                MigrationTxKind::Preparation { .. } => ("preparation", None),
            };
            PieceProgress {
                id: u32::from(t.id()),
                kind,
                amount,
                state: t.state().as_ref().to_owned(),
                scheduled_height: u32::from(t.scheduled_height()),
            }
        })
        .collect();
    Progress {
        status: state.status().as_ref().to_owned(),
        active: !state.is_terminal(),
        pieces,
        next_crossing_not_before,
    }
}

/// The read-only view `plan_migration` needs: the spendable Orchard notes and the chain tip.
struct PlanBackend<'a, W: WalletRead + InputSource> {
    wallet: &'a W,
    account: <W as InputSource>::AccountId,
}

impl<W> MigrationBackend for PlanBackend<'_, W>
where
    W: WalletRead + InputSource,
    <W as WalletRead>::Error: std::fmt::Display,
    <W as InputSource>::Error: std::fmt::Display,
{
    type Error = Error;

    fn spendable_orchard_note_values(&self) -> Result<Vec<Zatoshis>, Self::Error> {
        let tip = chain_height(self.wallet)?;
        let notes = self
            .wallet
            .select_unspent_notes(
                self.account,
                &[ShieldedPool::Orchard],
                TargetHeight::from(u32::from(tip) + 1),
                &[],
                LockFilter::Unfiltered,
            )
            .map_err(|e| Error::Wallet(e.to_string()))?;
        notes
            .orchard()
            .iter()
            .map(|n| {
                Zatoshis::from_u64(n.note().value().inner())
                    .map_err(|e| Error::Wallet(format!("note value out of range: {e}")))
            })
            .collect()
    }

    fn chain_tip_height(&self) -> Result<BlockHeight, Self::Error> {
        chain_height(self.wallet)
    }

    fn scheduling_params(&self) -> SchedulingParams {
        SchedulingParams::new_with_default_distributions(self.wallet.anchor_retention_interval())
    }
}

/// The migration store used while committing, before anything is persisted. It holds no chain
/// view, so it cannot judge spendability; commit never asks it to.
#[derive(Default)]
struct InMemoryStore {
    state: Option<MigrationState>,
}

impl From<Option<MigrationState>> for InMemoryStore {
    fn from(state: Option<MigrationState>) -> Self {
        Self { state }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("the commit-time migration store has no view of the chain")]
struct NoChainView;

impl PoolMigrationRead for InMemoryStore {
    type Error = NoChainView;

    fn get_migration(&self) -> Result<Option<MigrationState>, Self::Error> {
        Ok(self.state.clone())
    }

    fn check_step_satisfiability(
        &self,
        _tx: &MigrationTransaction,
        _settle: ReorgSettleDepth,
    ) -> Result<StepSatisfiability, Self::Error> {
        Err(NoChainView)
    }

    fn mined_height(&self, _txid: TxId) -> Result<Option<BlockHeight>, Self::Error> {
        Err(NoChainView)
    }
}

impl PoolMigrationWrite for InMemoryStore {
    fn replace_migration(&mut self, state: &MigrationState) -> Result<(), Self::Error> {
        self.state = Some(state.clone());
        Ok(())
    }

    fn update_transaction(
        &mut self,
        _id: MigrationTransferId,
        _tx_state: MigrationTxState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn store_proved_transaction(
        &mut self,
        state: &mut MigrationState,
        proven: ProvedTransaction,
    ) -> Result<(), Self::Error> {
        proven.apply(state);
        self.replace_migration(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_follow_zip_318_bounds() {
        let mut rng = UnwrapErr(SysRng);
        let draws: Vec<u32> = (0..20_000).map(|_| draw_delay(&mut rng)).collect();
        assert!(draws.iter().all(|&d| (1..=MAX_DELAY_BLOCKS).contains(&d)));
        let mean = draws.iter().map(|&d| f64::from(d)).sum::<f64>() / draws.len() as f64;
        // Exponential with mean 66, truncated at 576: the sample mean sits close to 66.
        assert!((60.0..72.0).contains(&mean), "mean {mean}");
    }
}

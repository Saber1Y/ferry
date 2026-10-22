//! FERRY's desktop shell. Everything that touches keys, the chain or the schedule lives in
//! `ferry-core`; this file exposes it to the window as Tauri commands and runs the scheduler in
//! the background while a migration is in progress.

mod desktop;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ferry_core::{
    Balances, FerryNetwork, HistoryEntry, PlanPreview, Progress, Server, SessionOutcome, Wallet,
};
use secrecy::SecretString;
use serde::Serialize;
#[cfg(target_os = "macos")]
use tauri::RunEvent;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tokio::sync::Mutex;

/// The hosted demo chain (regtest, fake coins) set up by `deploy/demo-server`.
const DEMO_SYNC_HOST: &str = "sync.ferry.162-62-231-147.sslip.io";
const DEMO_SEND_HOST: &str = "send.ferry.162-62-231-147.sslip.io";
const DEMO_WALLETS_URL: &str = "https://wallets.ferry.162-62-231-147.sslip.io/demo-wallet";

/// Settings the app runs with. The hackathon build runs against regtest chains only (the hosted
/// demo chain, or one on this computer); mainnet is not enabled yet.
struct Config {
    network: FerryNetwork,
    network_name: &'static str,
    /// ZIP 318: sync from one server, broadcast through a different one.
    sync_server: Server,
    broadcast_server: Server,
    /// Where the hosted demo chain hands out demo wallets, if this network has one.
    wallets_url: Option<&'static str>,
    wallet_dir: PathBuf,
    /// How often the scheduler runs a session. A session does nothing visible unless a block
    /// arrived or something became due, so this only bounds how late FERRY notices.
    session_interval: Duration,
}

/// What the scheduler last did, for the progress screen.
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchedulerState {
    running: bool,
    /// The last session's outcome.
    last: Option<SessionOutcome>,
    last_error: Option<String>,
    /// Set when a run finished and a follow-up run needs the user's signature.
    follow_up: Option<PlanPreview>,
}

struct AppState {
    config: Config,
    /// Serialises wallet operations: one sync, restore, commit or session at a time.
    busy: Mutex<()>,
    scheduler: Mutex<SchedulerState>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    network: &'static str,
    sync_server: String,
    broadcast_server: String,
    default_birthday: u32,
    /// Where to fetch a demo wallet, on the hosted demo chain.
    demo_wallets_url: Option<&'static str>,
    has_wallet: bool,
    balances: Option<Balances>,
    progress: Option<Progress>,
    scheduler: SchedulerState,
}

/// Errors cross into the window as plain messages.
type CmdResult<T> = Result<T, String>;

async fn status_of(state: &AppState) -> CmdResult<Status> {
    let config = &state.config;
    let (has_wallet, balances, progress) = match Wallet::open(&config.wallet_dir, config.network) {
        Ok(wallet) => (
            true,
            wallet.balances().map_err(|e| e.to_string())?,
            wallet.migration_progress().map_err(|e| e.to_string())?,
        ),
        Err(_) => (false, None, None),
    };
    Ok(Status {
        network: config.network_name,
        sync_server: config.sync_server.to_string(),
        broadcast_server: config.broadcast_server.to_string(),
        default_birthday: config.network.lowest_birthday(),
        demo_wallets_url: config.wallets_url,
        has_wallet,
        balances,
        progress,
        scheduler: state.scheduler.lock().await.clone(),
    })
}

fn open_wallet(config: &Config) -> CmdResult<Wallet> {
    Wallet::open(&config.wallet_dir, config.network).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_status(state: tauri::State<'_, Arc<AppState>>) -> CmdResult<Status> {
    status_of(&state).await
}

/// Restores the wallet from a recovery phrase, then syncs it.
///
/// The phrase is moved into zeroizing memory on arrival and dropped as soon as the account
/// exists. FERRY keeps only what it needs to watch and prove: nothing that can spend is stored.
#[tauri::command]
async fn restore_wallet(
    state: tauri::State<'_, Arc<AppState>>,
    phrase: String,
    birthday: u32,
) -> CmdResult<Status> {
    let phrase = SecretString::new(phrase);
    {
        let _guard = state.busy.lock().await;
        let config = &state.config;
        let wallet = Wallet::restore(
            &config.wallet_dir,
            config.network,
            phrase,
            birthday,
            &config.sync_server,
        )
        .await
        .map_err(|e| e.to_string())?;
        wallet
            .sync(&config.sync_server)
            .await
            .map_err(|e| e.to_string())?;
    }
    status_of(&state).await
}

#[tauri::command]
async fn sync_wallet(state: tauri::State<'_, Arc<AppState>>) -> CmdResult<Status> {
    {
        let _guard = state.busy.lock().await;
        let config = &state.config;
        open_wallet(config)?
            .sync(&config.sync_server)
            .await
            .map_err(|e| e.to_string())?;
    }
    status_of(&state).await
}

/// Every transaction FERRY has sent for this wallet, newest first.
#[tauri::command]
async fn get_history(state: tauri::State<'_, Arc<AppState>>) -> CmdResult<Vec<HistoryEntry>> {
    open_wallet(&state.config)?
        .history()
        .map_err(|e| e.to_string())
}

/// Previews the next migration run for the consent screen. Nothing is changed.
#[tauri::command]
async fn plan_migration(state: tauri::State<'_, Arc<AppState>>) -> CmdResult<Option<PlanPreview>> {
    let _guard = state.busy.lock().await;
    open_wallet(&state.config)?
        .plan_migration()
        .map_err(|e| e.to_string())
}

/// Signs a whole migration run with the phrase (wiped when the call returns) and starts the
/// scheduler. Calling it is the user's consent to the plan they were shown (ZIP 318).
#[tauri::command]
async fn commit_migration(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    phrase: String,
) -> CmdResult<Status> {
    let phrase = SecretString::new(phrase);
    {
        let _guard = state.busy.lock().await;
        open_wallet(&state.config)?
            .commit_migration(phrase)
            .map_err(|e| e.to_string())?;
        state.scheduler.lock().await.follow_up = None;
    }
    desktop::set_start_at_login(&app, true);
    start_scheduler(app.clone(), state.inner().clone()).await;
    refresh_tray(&app, &state).await;
    status_of(&state).await
}

/// Updates the tray's status line from the wallet's migration progress.
async fn refresh_tray(app: &AppHandle, state: &AppState) {
    let progress = open_wallet(&state.config)
        .and_then(|w| w.migration_progress().map_err(|e| e.to_string()))
        .ok()
        .flatten();
    let follow_up = state.scheduler.lock().await.follow_up.is_some();
    desktop::set_tray_status(app, &desktop::status_line(progress.as_ref(), follow_up));
}

/// A follow-up run is pending when this wallet has migrated before (FERRY has sent for it) and
/// balance above dust is still in Orchard.
fn pending_follow_up(config: &Config) -> Option<PlanPreview> {
    let wallet = open_wallet(config).ok()?;
    if wallet.history().ok()?.is_empty() {
        return None;
    }
    wallet.plan_migration().ok().flatten()
}

/// ZIP 318 catch-up: if crossings became overdue while FERRY was closed or the computer slept,
/// hold the next one back by a fresh random delay instead of sending it right away.
async fn defer_overdue(state: &AppState) {
    let _guard = state.busy.lock().await;
    if let Ok(wallet) = open_wallet(&state.config) {
        match wallet.defer_overdue_crossings(&state.config.sync_server).await {
            Ok(Some(height)) => log::info!("overdue crossings: next one not before block {height}"),
            Ok(None) => {}
            Err(e) => log::warn!("could not check for overdue crossings: {e}"),
        }
    }
}

/// Starts the background scheduler if it is not already running.
async fn start_scheduler(app: AppHandle, state: Arc<AppState>) {
    {
        let mut scheduler = state.scheduler.lock().await;
        if scheduler.running {
            return;
        }
        scheduler.running = true;
        scheduler.last_error = None;
    }
    tauri::async_runtime::spawn(async move {
        defer_overdue(&state).await;
        let mut last_tick = Instant::now();
        loop {
            // A tick far later than scheduled means the computer slept (or the process was
            // suspended): treat it like a relaunch and space out whatever became overdue.
            if last_tick.elapsed() > state.config.session_interval + Duration::from_secs(300) {
                defer_overdue(&state).await;
            }
            last_tick = Instant::now();

            let result = {
                let _guard = state.busy.lock().await;
                match open_wallet(&state.config) {
                    Ok(wallet) => wallet
                        .run_session(&state.config.sync_server, &state.config.broadcast_server)
                        .await
                        .map_err(|e| e.to_string()),
                    Err(e) => Err(e),
                }
            };

            let stop = {
                let mut scheduler = state.scheduler.lock().await;
                match result {
                    Ok(outcome) => {
                        scheduler.last_error = None;
                        let stop = match &outcome {
                            SessionOutcome::Complete => true,
                            SessionOutcome::FollowUpNeeded { preview } => {
                                scheduler.follow_up = Some(preview.clone());
                                true
                            }
                            _ => false,
                        };
                        scheduler.last = Some(outcome);
                        stop
                    }
                    // A failed session (server unreachable, a rejected broadcast) is reported and
                    // retried on the next tick: nothing is lost, the transactions stay signed.
                    Err(e) => {
                        scheduler.last_error = Some(e);
                        false
                    }
                }
            };
            let _ = app.emit("ferry://session", ());
            refresh_tray(&app, &state).await;
            if stop {
                let follow_up = {
                    let mut scheduler = state.scheduler.lock().await;
                    scheduler.running = false;
                    scheduler.follow_up.is_some()
                };
                if follow_up {
                    desktop::notify(
                        &app,
                        "FERRY needs you",
                        "This run has finished. Sign a short follow-up run to move what is left.",
                    );
                } else {
                    desktop::notify(&app, "Everything has crossed", "Your ZEC is in Ironwood.");
                    desktop::set_start_at_login(&app, false);
                }
                let _ = app.emit("ferry://session", ());
                refresh_tray(&app, &state).await;
                break;
            }
            tokio::time::sleep(state.config.session_interval).await;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first: a second launch just shows the running FERRY.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            desktop::show_window(app)
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            desktop::build_tray(app.handle())?;
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // FERRY_WALLET_DIR lets a demo point the app at a prepared wallet folder.
            // Which network to run against: the hosted demo chain by default in release
            // builds, this computer's regtest chain in development. FERRY_NETWORK overrides.
            let network_name = match std::env::var("FERRY_NETWORK").as_deref() {
                Ok("local") => "local",
                Ok("demo") => "demo",
                _ if cfg!(debug_assertions) => "local",
                _ => "demo",
            };
            let (sync_server, broadcast_server, wallets_url) = match network_name {
                // The hosted demo chain (deploy/demo-server): fake coins, served over HTTPS.
                "demo" => (
                    Server::new(DEMO_SYNC_HOST, 443),
                    Server::new(DEMO_SEND_HOST, 443),
                    Some(DEMO_WALLETS_URL),
                ),
                _ => (
                    Server::new("127.0.0.1", 9067),
                    Server::new("127.0.0.1", 9077),
                    None,
                ),
            };
            // FERRY_WALLET_DIR lets a demo point the app at a prepared wallet folder.
            let wallet_dir = match std::env::var_os("FERRY_WALLET_DIR") {
                Some(dir) => PathBuf::from(dir),
                None => app.path().app_data_dir()?.join("wallets").join(network_name),
            };
            let config = Config {
                // Both chains use regtest/heights.env: upgrades through NU6.2 at 1, NU6.3 at 300.
                network: FerryNetwork::regtest(1, 300),
                network_name,
                sync_server,
                broadcast_server,
                wallets_url,
                wallet_dir,
                session_interval: Duration::from_secs(3),
            };
            let state = Arc::new(AppState {
                config,
                busy: Mutex::new(()),
                scheduler: Mutex::new(SchedulerState::default()),
            });
            app.manage(state.clone());

            // Resume a migration that was in progress when the app last closed (ZIP 318:
            // reconcile on launch). The scheduler spaces overdue crossings out; it never bursts.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let in_progress = Wallet::open(&state.config.wallet_dir, state.config.network)
                    .and_then(|w| w.migration_progress())
                    .map(|p| p.is_some_and(|p| p.active))
                    .unwrap_or(false);
                if in_progress {
                    start_scheduler(handle.clone(), state.clone()).await;
                } else if let Some(preview) = pending_follow_up(&state.config) {
                    // A run finished earlier and left balance for a follow-up run the user has
                    // not signed yet: come back up in that state rather than forgetting it.
                    state.scheduler.lock().await.follow_up = Some(preview);
                }
                refresh_tray(&handle, &state).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            restore_wallet,
            sync_wallet,
            plan_migration,
            commit_migration,
            get_history
        ])
        // Closing the window keeps FERRY in the tray, so a migration keeps running; quitting is
        // done from the tray menu.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {
            // macOS: clicking the Dock icon brings the hidden window back. (Elsewhere the tray
            // menu's "Show FERRY" does.)
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = _event {
                desktop::show_window(_app);
            }
        });
}

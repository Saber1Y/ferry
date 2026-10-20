//! FERRY's desktop shell. Everything that touches keys, the chain or the schedule lives in
//! `ferry-core`; this file only exposes it to the window as Tauri commands.

use std::path::PathBuf;

use ferry_core::{Balances, FerryNetwork, Server, Wallet};
use secrecy::SecretString;
use serde::Serialize;
use tauri::Manager;
use tokio::sync::Mutex;

/// Settings the app runs with. The hackathon build targets the local regtest chain from
/// `regtest/setup-demo.sh`; mainnet is not enabled yet.
struct Config {
    network: FerryNetwork,
    network_name: &'static str,
    /// ZIP 318: sync from one server, broadcast through a different one.
    sync_server: Server,
    broadcast_server: Server,
    wallet_dir: PathBuf,
}

struct AppState {
    config: Config,
    /// Serialises wallet operations: one sync, restore or migration step at a time.
    busy: Mutex<()>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    network: &'static str,
    sync_server: String,
    broadcast_server: String,
    has_wallet: bool,
    balances: Option<Balances>,
}

/// Errors cross into the window as plain messages.
type CmdResult<T> = Result<T, String>;

fn status_of(config: &Config) -> CmdResult<Status> {
    let (has_wallet, balances) = match Wallet::open(&config.wallet_dir, config.network) {
        Ok(wallet) => (true, wallet.balances().map_err(|e| e.to_string())?),
        Err(_) => (false, None),
    };
    Ok(Status {
        network: config.network_name,
        sync_server: config.sync_server.to_string(),
        broadcast_server: config.broadcast_server.to_string(),
        has_wallet,
        balances,
    })
}

#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> CmdResult<Status> {
    status_of(&state.config)
}

/// Restores the wallet from a recovery phrase, then syncs it.
///
/// The phrase is moved into zeroizing memory on arrival and dropped as soon as the account
/// exists. FERRY keeps only what it needs to watch and prove: nothing that can spend is stored.
#[tauri::command]
async fn restore_wallet(
    state: tauri::State<'_, AppState>,
    phrase: String,
    birthday: u32,
) -> CmdResult<Status> {
    let phrase = SecretString::new(phrase);
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
    status_of(config)
}

#[tauri::command]
async fn sync_wallet(state: tauri::State<'_, AppState>) -> CmdResult<Status> {
    let _guard = state.busy.lock().await;
    let config = &state.config;
    let wallet = Wallet::open(&config.wallet_dir, config.network).map_err(|e| e.to_string())?;
    wallet
        .sync(&config.sync_server)
        .await
        .map_err(|e| e.to_string())?;
    status_of(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // FERRY_WALLET_DIR lets a demo point the app at a prepared wallet folder.
            let wallet_dir = match std::env::var_os("FERRY_WALLET_DIR") {
                Some(dir) => PathBuf::from(dir),
                None => app.path().app_data_dir()?.join("wallets").join("regtest"),
            };
            let config = Config {
                // Matches regtest/heights.env.
                network: FerryNetwork::regtest(1, 300),
                network_name: "regtest",
                sync_server: Server::new("127.0.0.1", 9067),
                broadcast_server: Server::new("127.0.0.1", 9077),
                wallet_dir,
            };
            app.manage(AppState {
                config,
                busy: Mutex::new(()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_status, restore_wallet, sync_wallet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

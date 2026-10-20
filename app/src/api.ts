import { invoke } from "@tauri-apps/api/core";

/** Balances in zatoshis, as of the wallet's last sync (mirrors ferry_core::Balances). */
export interface Balances {
  chain_tip: number;
  scanned_to: number;
  orchard_spendable: number;
  orchard_total: number;
  ironwood_spendable: number;
  ironwood_total: number;
}

export interface Status {
  network: string;
  syncServer: string;
  broadcastServer: string;
  hasWallet: boolean;
  balances: Balances | null;
}

export const getStatus = () => invoke<Status>("get_status");

/** The phrase is handed to the Rust side immediately; callers must clear their copy. */
export const restoreWallet = (phrase: string, birthday: number) =>
  invoke<Status>("restore_wallet", { phrase, birthday });

export const syncWallet = () => invoke<Status>("sync_wallet");

/** Tauri rejects with the command's error string. */
export const errorMessage = (e: unknown): string =>
  typeof e === "string" ? e : e instanceof Error ? e.message : "Something went wrong.";

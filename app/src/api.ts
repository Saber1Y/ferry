import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Balances in zatoshis, as of the wallet's last sync (mirrors ferry_core::Balances). */
export interface Balances {
  chain_tip: number;
  scanned_to: number;
  orchard_spendable: number;
  orchard_total: number;
  ironwood_spendable: number;
  ironwood_total: number;
}

/** What a migration run would do (mirrors ferry_core::PlanPreview). Amounts in zatoshis. */
export interface PlanPreview {
  crossings: number[];
  value_migrated: number;
  left_in_orchard: number;
  fees: number;
  preparation_transactions: number;
  preparation_layers: number;
  planned_at: number;
  first_crossing_height: number;
  last_crossing_height: number;
}

export interface PieceProgress {
  id: number;
  kind: "preparation" | "crossing";
  amount: number | null;
  state: string;
  scheduled_height: number;
}

export interface Progress {
  status: string;
  active: boolean;
  pieces: PieceProgress[];
  next_crossing_not_before: number;
}

export type SessionOutcome =
  | { kind: "synced"; proved: number[] }
  | { kind: "broadcast"; id: number; crossing: boolean; txid: string; next_crossing_not_before: number }
  | { kind: "followUpNeeded"; preview: PlanPreview }
  | { kind: "complete" };

export interface SchedulerState {
  running: boolean;
  last: SessionOutcome | null;
  lastError: string | null;
  followUp: PlanPreview | null;
}

export interface Status {
  network: string;
  syncServer: string;
  broadcastServer: string;
  defaultBirthday: number;
  /** On the hosted demo chain: where to fetch a demo wallet. */
  demoWalletsUrl: string | null;
  /** Networks the window can switch to; empty in development. */
  networks: string[];
  hasWallet: boolean;
  balances: Balances | null;
  progress: Progress | null;
  scheduler: SchedulerState;
}

/** One transaction FERRY sent (mirrors ferry_core::HistoryEntry). */
export interface HistoryEntry {
  sent_at: number;
  sent_at_height: number;
  txid: string;
  kind: "preparation" | "crossing";
  amount: number | null;
  mined_height: number | null;
}

export const getStatus = () => invoke<Status>("get_status");

export const getHistory = () => invoke<HistoryEntry[]>("get_history");

/** Switches network in place; each network keeps its own wallet. */
export const setNetwork = (network: string) => invoke<Status>("set_network", { network });

/** The phrase is handed to the Rust side immediately; callers must clear their copy. */
export const restoreWallet = (phrase: string, birthday: number) =>
  invoke<Status>("restore_wallet", { phrase, birthday });

export const syncWallet = () => invoke<Status>("sync_wallet");

/** Asks the hosted demo chain for an unused demo wallet (fake coins). */
export async function fetchDemoWallet(url: string): Promise<{ phrase: string; birthday: number }> {
  const response = await fetch(url, { cache: "no-store" });
  const body = await response.json().catch(() => ({}));
  if (!response.ok) throw body.error ?? `the demo server answered ${response.status}`;
  return body;
}

export const planMigration = () => invoke<PlanPreview | null>("plan_migration");

/** Signs the whole run; the phrase is wiped on the Rust side when the call returns. */
export const commitMigration = (phrase: string) => invoke<Status>("commit_migration", { phrase });

/** Fires after every scheduler session. */
export const onSession = (handler: () => void): Promise<UnlistenFn> =>
  listen("ferry://session", handler);

/** Tauri rejects with the command's error string. */
export const errorMessage = (e: unknown): string =>
  typeof e === "string" ? e : e instanceof Error ? e.message : "Something went wrong.";

/** Error strings from Rust start lowercase; sentences on screen should not. */
export const sentence = (s: string) => `${s.charAt(0).toUpperCase()}${s.slice(1)}`;

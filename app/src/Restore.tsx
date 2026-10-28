import { useState, type FormEvent } from "react";
import {
  errorMessage,
  fetchDemoWallet,
  restoreWallet,
  sentence,
  setNetwork,
  type Status,
} from "./api";

const VALID_WORD_COUNTS = [12, 15, 18, 21, 24];

const NETWORK_LABELS: Record<string, { name: string; detail: string }> = {
  demo: { name: "Demo chain", detail: "fake coins" },
  mainnet: { name: "Mainnet", detail: "real ZEC" },
};

export default function Restore({
  network,
  networks,
  defaultBirthday,
  demoWalletsUrl,
  onRestored,
}: {
  network: string;
  networks: string[];
  defaultBirthday: number;
  demoWalletsUrl: string | null;
  onRestored: (s: Status) => void;
}) {
  const [switching, setSwitching] = useState(false);
  const [fetchingDemo, setFetchingDemo] = useState(false);
  const [phrase, setPhrase] = useState("");
  const [birthday, setBirthday] = useState(String(defaultBirthday));
  const [understood, setUnderstood] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const words = phrase.trim() ? phrase.trim().split(/\s+/).length : 0;
  const wordsOk = VALID_WORD_COUNTS.includes(words);
  const birthdayNum = Number(birthday);
  const birthdayOk = Number.isInteger(birthdayNum) && birthdayNum >= defaultBirthday;
  const canSubmit = wordsOk && birthdayOk && understood && !busy;

  async function useDemoWallet() {
    if (!demoWalletsUrl) return;
    setFetchingDemo(true);
    setError(null);
    try {
      const demo = await fetchDemoWallet(demoWalletsUrl);
      setPhrase(demo.phrase);
      setBirthday(String(demo.birthday));
    } catch (err) {
      setError(`Could not get a demo wallet: ${errorMessage(err)}.`);
    } finally {
      setFetchingDemo(false);
    }
  }

  async function switchNetwork(next: string) {
    if (next === network || switching) return;
    setSwitching(true);
    setError(null);
    try {
      await setNetwork(next);
    } catch (err) {
      setError(`Could not switch network: ${errorMessage(err)}.`);
      setSwitching(false);
    }
  }

  async function submit(e: FormEvent) {
    e.preventDefault();
    if (!canSubmit) return;
    const secret = phrase;
    // Clear our copy before the round trip; the Rust side moves it into zeroizing memory.
    setPhrase("");
    setBusy(true);
    setError(null);
    try {
      onRestored(await restoreWallet(secret, birthdayNum));
    } catch (err) {
      setError(
        `${sentence(errorMessage(err))}. For safety your phrase was cleared, so please type it again.`,
      );
      setBusy(false);
    }
  }

  return (
    <section className="page" aria-labelledby="restore-title">
      <p className="eyebrow">Step 1 of 3 · Connect your wallet</p>
      <h1 id="restore-title" className="title">
        Bring your wallet aboard.
      </h1>
      <p className="lede">
        Use the recovery phrase of the wallet whose Orchard funds you want to move (for example
        Zodl: Settings, Recovery phrase). Your wallet keeps working as before; FERRY keeps only a
        view-only copy and asks for the phrase again just once, to sign.
      </p>

      {networks.length > 1 && (
        <div className="network-switch" role="group" aria-label="Network">
          {networks.map((n) => (
            <button
              key={n}
              type="button"
              className={n === network ? "network-option network-option-active" : "network-option"}
              aria-pressed={n === network}
              onClick={() => switchNetwork(n)}
              disabled={switching || busy}
            >
              <span className="network-name">{NETWORK_LABELS[n]?.name ?? n}</span>
              <span className="network-detail">{NETWORK_LABELS[n]?.detail}</span>
            </button>
          ))}
        </div>
      )}

      {network === "mainnet" && (
        <div className="notice notice-mainnet" role="note">
          <strong>Mainnet is experimental.</strong> FERRY has migrated wallets end to end on demo
          chains; on mainnet, start with a small amount you can afford to lose. It syncs from{" "}
          <span className="mono">zec.rocks</span> and broadcasts through{" "}
          <span className="mono">zcash.mysideoftheweb.com</span>, two different operators, without
          Tor, so they can see your IP address.
        </div>
      )}

      {demoWalletsUrl && (
        <div className="demo-offer">
          <div>
            <p className="demo-offer-title">Trying FERRY?</p>
            <p className="hint">
              This build runs on a demo chain with fake coins. Get a demo wallet that holds
              Orchard funds and migrate it for real.
            </p>
          </div>
          <button
            type="button"
            className="button button-quiet"
            onClick={useDemoWallet}
            disabled={fetchingDemo || busy}
          >
            {fetchingDemo ? "Getting one…" : "Get a demo wallet"}
          </button>
        </div>
      )}

      <div className="notice notice-warn">
        <strong>Seed mode, said plainly.</strong> While FERRY sets up your wallet, it holds your
        recovery phrase in memory and could spend your funds. The phrase is never written to disk,
        and it is wiped as soon as the wallet is created. Only do this on your own computer, and do not start a migration in your other wallet at the same time: both would try to move the same funds.
      </div>

      <form className="form" onSubmit={submit}>
        <label className="field">
          <span className="field-label">
            Recovery phrase
            <span className={`count ${wordsOk ? "count-ok" : ""}`}>{words} words</span>
          </span>
          <textarea
            className="input phrase"
            value={phrase}
            onChange={(e) => setPhrase(e.target.value)}
            placeholder="Type or paste your 24 words, separated by spaces"
            rows={4}
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
            spellCheck={false}
            disabled={busy}
          />
          {words > 0 && !wordsOk && (
            <span className="hint">Recovery phrases have 12, 15, 18, 21 or 24 words.</span>
          )}
        </label>

        <label className="field">
          <span className="field-label">Wallet birthday (block height)</span>
          <input
            className="input mono birthday"
            inputMode="numeric"
            value={birthday}
            onChange={(e) => setBirthday(e.target.value.replace(/[^0-9]/g, ""))}
            disabled={busy}
          />
          <span className="hint">
            {network === "mainnet"
              ? "The block your wallet was created at (your wallet app shows it, often under Advanced or Wallet birthday). Funds received before it are never scanned. The default is the start of Orchard, which misses nothing but can take hours to scan."
              : "The block your wallet was created at. Funds received before it are never scanned, so they would be left behind. If you are unsure, keep the default: FERRY scans from the start, which is slower but misses nothing."}
          </span>
        </label>

        <label className="check">
          <input
            type="checkbox"
            checked={understood}
            onChange={(e) => setUnderstood(e.target.checked)}
            disabled={busy}
          />
          <span>I understand FERRY holds my phrase in memory while it sets up.</span>
        </label>

        {error && (
          <div className="notice notice-error" role="alert">
            {error}
          </div>
        )}

        <div className="actions">
          <button className="button button-primary" type="submit" disabled={!canSubmit}>
            {busy ? "Restoring and scanning the chain…" : "Restore wallet"}
          </button>
          {busy && <span className="spinner" aria-hidden="true" />}
        </div>
      </form>
    </section>
  );
}

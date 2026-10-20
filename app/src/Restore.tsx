import { useState, type FormEvent } from "react";
import { errorMessage, restoreWallet, type Status } from "./api";

const VALID_WORD_COUNTS = [12, 15, 18, 21, 24];

export default function Restore({ onRestored }: { onRestored: (s: Status) => void }) {
  const [phrase, setPhrase] = useState("");
  const [birthday, setBirthday] = useState("1");
  const [understood, setUnderstood] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const words = phrase.trim() ? phrase.trim().split(/\s+/).length : 0;
  const wordsOk = VALID_WORD_COUNTS.includes(words);
  const birthdayNum = Number(birthday);
  const birthdayOk = Number.isInteger(birthdayNum) && birthdayNum >= 1;
  const canSubmit = wordsOk && birthdayOk && understood && !busy;

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
      const message = errorMessage(err);
      setError(
        `${message.charAt(0).toUpperCase()}${message.slice(1)}. ` +
          "For safety your phrase was cleared, so please type it again.",
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
        FERRY reads your Orchard balance and, later, signs the whole migration in one sitting. It
        starts from your recovery phrase.
      </p>

      <div className="notice notice-warn">
        <strong>Seed mode, said plainly.</strong> While FERRY sets up your wallet, it holds your
        recovery phrase in memory and could spend your funds. The phrase is never written to disk,
        and it is wiped as soon as the wallet is created. Only do this on your own computer.
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
            The block your wallet was created at. Funds received before it are never scanned, so
            they would be left behind. If you are unsure, keep the default: FERRY scans from the
            start, which is slower but misses nothing.
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

import { useState } from "react";
import { errorMessage, syncWallet, type Status } from "./api";
import { DUST_ZATOSHIS, blockHeight, zec, zecText } from "./format";

function Amount({ zatoshis, tone }: { zatoshis: number; tone: "orchard" | "ironwood" }) {
  const { whole, fraction } = zec(zatoshis);
  return (
    <p className={`amount amount-${tone}`}>
      <span className="amount-whole">{whole}</span>
      {fraction && <span className="amount-fraction">.{fraction}</span>}
      <span className="amount-unit">ZEC</span>
    </p>
  );
}

export default function Home({
  status,
  onStatus,
}: {
  status: Status;
  onStatus: (s: Status) => void;
}) {
  const [syncing, setSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const b = status.balances;

  async function sync() {
    setSyncing(true);
    setError(null);
    try {
      onStatus(await syncWallet());
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setSyncing(false);
    }
  }

  if (!b) {
    return (
      <section className="page">
        <h1 className="title">Your wallet is aboard.</h1>
        <p className="lede">It has not been scanned yet.</p>
        <button className="button button-primary" onClick={sync} disabled={syncing}>
          {syncing ? "Scanning…" : "Scan the chain"}
        </button>
      </section>
    );
  }

  const toMove = b.orchard_spendable >= DUST_ZATOSHIS;
  const pending = b.orchard_total - b.orchard_spendable;

  return (
    <section className="page" aria-labelledby="home-title">
      <p className="eyebrow">Your crossing</p>
      <h1 id="home-title" className="title">
        {toMove
          ? `${zecText(b.orchard_spendable)} ZEC is still on the Orchard side.`
          : b.ironwood_total > 0
            ? "Everything has crossed to Ironwood."
            : "There is nothing to move."}
      </h1>
      <p className="lede">
        {toMove
          ? "Since NU6.3, Orchard accepts no new value. FERRY moves it across in standard amounts, one at a time and hours apart, so nobody watching the chain can add them up to your balance. Most crossings finish within a couple of days."
          : "Your Orchard balance is below 0.01 ZEC, the dust ZIP 318 leaves behind."}
      </p>

      <div className="crossing" role="group" aria-label="Balances on each side">
        <div className="bank bank-orchard">
          <p className="bank-name">Orchard</p>
          <p className="bank-role">the old pool</p>
          <Amount zatoshis={b.orchard_spendable} tone="orchard" />
          {pending > 0 && (
            <p className="bank-note">+ {zecText(pending)} waiting for confirmations</p>
          )}
        </div>
        <svg className="river" viewBox="0 0 120 120" aria-hidden="true">
          <rect className="river-water" x="34" y="0" width="52" height="120" rx="2" />
          <path className="river-wave" d="M38 34 q 5 -4 10 0 t 10 0 t 10 0 t 10 0" />
          <path className="river-wave" d="M42 86 q 5 -4 10 0 t 10 0 t 10 0 t 10 0" />
          <path className="river-route" d="M6 60 L 106 60" />
          <path className="river-arrow" d="M100 54 L 108 60 L 100 66" />
        </svg>
        <div className="bank bank-ironwood">
          <p className="bank-name">Ironwood</p>
          <p className="bank-role">the new pool</p>
          <Amount zatoshis={b.ironwood_total} tone="ironwood" />
        </div>
      </div>

      {toMove && (
        <div className="next">
          <p className="next-label">Next</p>
          <p>
            Planning the crossing (pieces, schedule, fees) is the next part of FERRY being built.
            It is not in this build yet.
          </p>
        </div>
      )}

      {error && (
        <div className="notice notice-error" role="alert">
          {error}
        </div>
      )}

      <footer className="statusbar">
        <span>
          Scanned to block <span className="mono">{blockHeight(b.scanned_to)}</span> of{" "}
          <span className="mono">{blockHeight(b.chain_tip)}</span>
        </span>
        <button className="button button-quiet" onClick={sync} disabled={syncing}>
          {syncing ? "Syncing…" : "Sync now"}
        </button>
      </footer>
    </section>
  );
}

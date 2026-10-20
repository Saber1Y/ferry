import { useState } from "react";
import { errorMessage, sentence, syncWallet, type SessionOutcome, type Status } from "./api";
import { blockHeight, zecText } from "./format";

const STATE_LABEL: Record<string, string> = {
  awaiting_signature: "Unsigned",
  signed: "Waiting",
  proved: "Ready",
  broadcast: "Sent",
  mined: "Landed",
};

function describe(outcome: SessionOutcome | null): string {
  if (!outcome) return "Starting…";
  switch (outcome.kind) {
    case "synced":
      return outcome.proved.length
        ? `Synced and proved ${outcome.proved.length} ${outcome.proved.length === 1 ? "piece" : "pieces"}.`
        : "Synced. Nothing was due.";
    case "broadcast":
      return outcome.crossing
        ? `Sent a crossing. The next one waits until block ${blockHeight(outcome.next_crossing_not_before)}.`
        : "Sent a preparation transaction.";
    case "followUpNeeded":
      return "This run is finished. A follow-up run needs your signature.";
    case "complete":
      return "Finished.";
  }
}

export default function Crossing({
  status,
  onStatus,
  onFollowUp,
}: {
  status: Status;
  onStatus: (s: Status) => void;
  onFollowUp: () => void;
}) {
  const [syncing, setSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const progress = status.progress!;
  const { scheduler, balances } = status;

  const crossings = progress.pieces.filter((p) => p.kind === "crossing");
  const preparations = progress.pieces.filter((p) => p.kind === "preparation");
  const landed = crossings.filter((p) => p.state === "mined");
  const moved = landed.reduce((sum, p) => sum + (p.amount ?? 0), 0);
  const prepsDone = preparations.filter((p) => p.state === "mined").length;
  const preparing = prepsDone < preparations.length;
  // While preparing, the bar tracks preparation; afterwards, crossings landed.
  const share = preparing
    ? preparations.length
      ? prepsDone / preparations.length
      : 0
    : crossings.length
      ? landed.length / crossings.length
      : 0;

  async function sync() {
    setSyncing(true);
    setError(null);
    try {
      onStatus(await syncWallet());
    } catch (e) {
      setError(sentence(errorMessage(e)));
    } finally {
      setSyncing(false);
    }
  }

  return (
    <section className="page" aria-labelledby="crossing-title">
      <p className="eyebrow">Step 3 of 3 · Crossing</p>
      <h1 id="crossing-title" className="title">
        {!progress.active
          ? "This run is finished."
          : preparing
            ? `Preparing your pieces: ${prepsDone} of ${preparations.length} done.`
            : `${landed.length} of ${crossings.length} pieces have landed in Ironwood.`}
      </h1>
      <p className="lede">
        {progress.active
          ? preparing
            ? "FERRY is cutting your balance into the exact amounts, with private send-to-self transactions. Then the pieces cross one at a time, a random while apart. Leave this computer on; you do not need to do anything."
            : "FERRY sends one piece at a time and waits a random while in between. Leave this computer on; you do not need to do anything."
          : `${zecText(moved)} ZEC crossed in this run.`}
      </p>

      <div
        className="meter"
        role="progressbar"
        aria-label={preparing ? "Preparation" : "Crossings landed"}
        aria-valuemin={0}
        aria-valuemax={preparing ? preparations.length : crossings.length}
        aria-valuenow={preparing ? prepsDone : landed.length}
      >
        <div className="meter-fill" style={{ width: `${share * 100}%` }} />
      </div>
      <p className="meter-caption">
        <span className="mono">{zecText(moved)} ZEC</span> moved ·{" "}
        {prepsDone} of {preparations.length} preparation transactions done
      </p>

      {scheduler.followUp && (
        <div className="notice notice-warn">
          This run is finished. A follow-up run can move another{" "}
          <span className="mono">{zecText(scheduler.followUp.value_migrated)} ZEC</span>.{" "}
          <button className="link" onClick={onFollowUp}>
            Review and sign the follow-up run
          </button>
        </div>
      )}

      <ol className="ledger" aria-label="Crossings">
        {crossings.map((p) => (
          <li key={p.id} className={`ledger-row state-${p.state}`}>
            <span className="mono ledger-amount">{p.amount != null ? `${zecText(p.amount)} ZEC` : "—"}</span>
            <span className="ledger-state">{STATE_LABEL[p.state] ?? p.state}</span>
            <span className="ledger-when mono">not before {blockHeight(p.scheduled_height)}</span>
          </li>
        ))}
      </ol>

      {(scheduler.lastError || error) && (
        <div className="notice notice-error" role="alert">
          {sentence(scheduler.lastError ?? error ?? "")}. FERRY will retry; your signed pieces are safe.
        </div>
      )}

      <footer className="statusbar">
        <span>
          {scheduler.running ? <span className="live" aria-hidden="true" /> : null}
          {describe(scheduler.last)}
          {balances && (
            <>
              {" "}
              · block <span className="mono">{blockHeight(balances.chain_tip)}</span>
            </>
          )}
        </span>
        <button className="button button-quiet" onClick={sync} disabled={syncing}>
          {syncing ? "Syncing…" : "Sync now"}
        </button>
      </footer>
    </section>
  );
}

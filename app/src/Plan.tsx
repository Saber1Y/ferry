import { useEffect, useState, type FormEvent } from "react";
import { commitMigration, errorMessage, planMigration, sentence, type PlanPreview, type Status } from "./api";
import { DUST_ZATOSHIS, zecText } from "./format";

/** Mainnet's target block spacing (75 seconds), used to turn block heights into time. */
const BLOCK_SECONDS = 75;

function duration(blocks: number): string {
  const hours = (blocks * BLOCK_SECONDS) / 3600;
  if (hours < 1.5) return "about an hour";
  if (hours < 36) return `about ${Math.round(hours)} hours`;
  return `about ${Math.round(hours / 24)} days`;
}

export default function Plan({
  followUp,
  onCancel,
  onCommitted,
}: {
  /** Set when this is a follow-up run for balance an earlier run left behind. */
  followUp: PlanPreview | null;
  onCancel: () => void;
  onCommitted: (s: Status) => void;
}) {
  const [plan, setPlan] = useState<PlanPreview | null | undefined>(followUp ?? undefined);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [phrase, setPhrase] = useState("");
  const [agreed, setAgreed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // A follow-up arrives with its plan; a first run asks for a fresh one.
    if (followUp) return;
    planMigration()
      .then(setPlan)
      .catch((e) => setLoadError(sentence(errorMessage(e))));
  }, [followUp]);

  const words = phrase.trim() ? phrase.trim().split(/\s+/).length : 0;
  const canSign = !!plan && words >= 12 && agreed && !busy;

  async function sign(e: FormEvent) {
    e.preventDefault();
    if (!canSign) return;
    const secret = phrase;
    setPhrase("");
    setBusy(true);
    setError(null);
    try {
      onCommitted(await commitMigration(secret));
    } catch (err) {
      setError(
        `${sentence(errorMessage(err))}. For safety your phrase was cleared, so please type it again.`,
      );
      setBusy(false);
    }
  }

  if (loadError) {
    return (
      <section className="page">
        <div className="notice notice-error" role="alert">
          {loadError}
        </div>
        <button className="button button-quiet" onClick={onCancel}>
          Back
        </button>
      </section>
    );
  }
  if (plan === undefined) return <p className="muted">Planning the crossing…</p>;
  if (plan === null) {
    return (
      <section className="page">
        <h1 className="title">There is nothing to move.</h1>
        <button className="button button-quiet" onClick={onCancel}>
          Back
        </button>
      </section>
    );
  }

  const followUpNeeded = plan.left_in_orchard >= DUST_ZATOSHIS;
  const spanBlocks = Math.max(plan.last_crossing_height - plan.planned_at, 0);

  return (
    <section className="page" aria-labelledby="plan-title">
      <p className="eyebrow">
        {followUp ? "Follow-up run · Step 2 of 3" : "Step 2 of 3 · Review the crossing"}
      </p>
      <h1 id="plan-title" className="title">
        {zecText(plan.value_migrated)} ZEC crosses in {plan.crossings.length}{" "}
        {plan.crossings.length === 1 ? "piece" : "pieces"}.
      </h1>
      <p className="lede">
        {followUp
          ? "The last run finished. This run moves what it left behind."
          : "Each piece is a standard amount that many other wallets also send, so the pieces cannot be added up to your balance."}
      </p>

      <ul className="pieces" aria-label="Pieces">
        {plan.crossings.map((c, i) => (
          <li key={i} className="piece">
            <span className="mono">{zecText(c)}</span> <span className="piece-unit">ZEC</span>
          </li>
        ))}
      </ul>

      <dl className="facts">
        <div>
          <dt>Time</dt>
          <dd>
            {duration(spanBlocks)} at mainnet block times, while this computer is on. Pieces go out
            one at a time, at random intervals.
          </dd>
        </div>
        <div>
          <dt>Fees</dt>
          <dd className="mono">{zecText(plan.fees)} ZEC</dd>
        </div>
        <div>
          <dt>Preparation</dt>
          <dd>
            {plan.preparation_transactions} private send-to-self{" "}
            {plan.preparation_transactions === 1 ? "transaction" : "transactions"} first, to cut
            your balance into exactly these amounts.
          </dd>
        </div>
        <div>
          <dt>Left in Orchard</dt>
          <dd>
            <span className="mono">{zecText(plan.left_in_orchard)} ZEC</span>
            {followUpNeeded
              ? ". The pieces are fixed amounts, so this run cannot cover everything. A short follow-up run moves the rest; you will sign once more when this one finishes."
              : ", which is dust ZIP 318 leaves behind."}
          </dd>
        </div>
        <div>
          <dt>Why these amounts</dt>
          <dd>
            Since NU6.3, Orchard funds cannot be used for ordinary payments, only moved out, so
            FERRY moves all of it. The pieces are the standard sizes every migrating wallet uses;
            amounts you picked yourself would stand out on the chain and point back to you.
          </dd>
        </div>
        <div>
          <dt>Strangers see</dt>
          <dd>Standard-size crossings at random times, the same shape as everyone else's.</dd>
        </div>
        <div>
          <dt>This computer sees</dt>
          <dd>Your whole wallet history, through its viewing key. That is why it must be yours.</dd>
        </div>
      </dl>

      <form className="form sign" onSubmit={sign}>
        <p className="sign-title">Sign the whole run once</p>
        <label className="field">
          <span className="field-label">
            Recovery phrase
            <span className="count">{words} words</span>
          </span>
          <textarea
            className="input phrase"
            value={phrase}
            onChange={(e) => setPhrase(e.target.value)}
            placeholder="FERRY needs it for a few seconds to sign every piece, then wipes it"
            rows={3}
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
            spellCheck={false}
            disabled={busy}
          />
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={agreed}
            onChange={(e) => setAgreed(e.target.checked)}
            disabled={busy}
          />
          <span>I agree to this plan and its schedule.</span>
        </label>
        {error && (
          <div className="notice notice-error" role="alert">
            {error}
          </div>
        )}
        <div className="actions">
          <button className="button button-primary" type="submit" disabled={!canSign}>
            {busy ? "Signing every piece…" : "Sign and start"}
          </button>
          {busy ? (
            <span className="spinner" aria-hidden="true" />
          ) : (
            <button type="button" className="button button-quiet" onClick={onCancel}>
              Not now
            </button>
          )}
        </div>
      </form>
    </section>
  );
}

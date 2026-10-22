import { useEffect, useState } from "react";
import { errorMessage, getHistory, onSession, sentence, type HistoryEntry } from "./api";
import { blockHeight, zecText } from "./format";

const when = (seconds: number) =>
  new Date(seconds * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });

function CopyTxid({ txid }: { txid: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      className="txid"
      title={txid}
      onClick={() => {
        navigator.clipboard.writeText(txid).then(() => {
          setCopied(true);
          setTimeout(() => setCopied(false), 1400);
        });
      }}
    >
      <span className="mono">
        {txid.slice(0, 10)}…{txid.slice(-6)}
      </span>
      <span className="txid-action">{copied ? "Copied" : "Copy"}</span>
    </button>
  );
}

export default function History() {
  const [entries, setEntries] = useState<HistoryEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const load = () =>
      getHistory()
        .then(setEntries)
        .catch((e) => setError(sentence(errorMessage(e))));
    load();
    const unlisten = onSession(load);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  if (error) {
    return (
      <div className="notice notice-error" role="alert">
        {error}
      </div>
    );
  }
  if (!entries) return <p className="muted">Reading history…</p>;

  const crossings = entries.filter((e) => e.kind === "crossing");
  const landed = crossings.filter((e) => e.mined_height != null);
  const moved = landed.reduce((sum, e) => sum + (e.amount ?? 0), 0);

  return (
    <section className="page" aria-labelledby="history-title">
      <p className="eyebrow">History</p>
      <h1 id="history-title" className="title">
        {crossings.length
          ? `${zecText(moved)} ZEC has crossed in ${landed.length} ${landed.length === 1 ? "piece" : "pieces"}.`
          : "Nothing has crossed yet."}
      </h1>
      <p className="lede">
        Every transaction FERRY sent for this wallet. This record stays on this computer; on the
        chain, nothing ties these transactions to each other.
      </p>

      {entries.length > 0 && (
        <ol className="history" aria-label="Sent transactions">
          {entries.map((e) => (
            <li key={e.txid} className={`history-row history-${e.kind}`}>
              <span className="history-when">{when(e.sent_at)}</span>
              <span className="history-what">
                {e.kind === "crossing" && e.amount != null ? (
                  <>
                    <span className="mono">{zecText(e.amount)} ZEC</span> crossed to Ironwood
                  </>
                ) : (
                  "Preparation, a private send to yourself"
                )}
              </span>
              <span className="history-block">
                {e.mined_height != null ? (
                  <>
                    landed in <span className="mono">{blockHeight(e.mined_height)}</span>
                  </>
                ) : (
                  "on its way"
                )}
              </span>
              <CopyTxid txid={e.txid} />
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

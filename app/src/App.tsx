import { useCallback, useEffect, useState } from "react";
import { errorMessage, getStatus, onSession, setNetwork, type Status } from "./api";
import Crossing from "./Crossing";
import History from "./History";
import Home from "./Home";
import Plan from "./Plan";
import Restore from "./Restore";

type View = "main" | "plan" | "history";

const NETWORK_LABELS: Record<string, string> = { demo: "Demo chain", mainnet: "Mainnet" };

export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [view, setView] = useState<View>("main");
  const [switching, setSwitching] = useState(false);
  const [switchError, setSwitchError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    getStatus()
      .then(setStatus)
      .catch((e) => setLoadError(errorMessage(e)));
  }, []);

  useEffect(() => {
    refresh();
    // The scheduler announces every session; refetch so progress stays live.
    const unlisten = onSession(refresh);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [refresh]);

  const migrating = !!status?.progress?.active || !!status?.scheduler.followUp;
  const crossing = !!status?.scheduler.running;

  async function switchNetwork(next: string) {
    if (!status || next === status.network || switching) return;
    setSwitching(true);
    setSwitchError(null);
    try {
      setStatus(await setNetwork(next));
      setView("main");
    } catch (e) {
      setSwitchError(errorMessage(e));
    } finally {
      setSwitching(false);
    }
  }

  let page: React.ReactNode = <p className="muted">Opening FERRY…</p>;
  if (status && !status.hasWallet) {
    page = (
      <Restore
        // A new network brings its own defaults (the birthday), so start the form afresh.
        key={status.network}
        network={status.network}
        defaultBirthday={status.defaultBirthday}
        demoWalletsUrl={status.demoWalletsUrl}
        onRestored={setStatus}
      />
    );
  } else if (status && view === "history") {
    page = <History />;
  } else if (status && view === "plan") {
    page = (
      <Plan
        followUp={status.scheduler.followUp}
        onCancel={() => setView("main")}
        onCommitted={(s) => {
          setStatus(s);
          setView("main");
        }}
      />
    );
  } else if (status && migrating && status.progress) {
    page = <Crossing status={status} onStatus={setStatus} onFollowUp={() => setView("plan")} />;
  } else if (status) {
    page = <Home status={status} onStatus={setStatus} onPlan={() => setView("plan")} />;
  }

  return (
    <div className="shell">
      <aside className="rail">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true" />
          FERRY
        </div>
        <p className="rail-tagline">Orchard to Ironwood, the private way.</p>
        {status?.hasWallet && (
          <nav className="rail-nav" aria-label="Sections">
            <button
              className={view !== "history" ? "rail-link rail-link-active" : "rail-link"}
              aria-current={view !== "history" ? "page" : undefined}
              onClick={() => setView("main")}
            >
              Crossing
            </button>
            <button
              className={view === "history" ? "rail-link rail-link-active" : "rail-link"}
              aria-current={view === "history" ? "page" : undefined}
              onClick={() => setView("history")}
            >
              History
            </button>
          </nav>
        )}
        {status && (
          <dl className="rail-facts">
            <div>
              <dt>Network</dt>
              <dd>
                {status.networks.length > 1 ? (
                  <>
                    <div className="network-switch" role="group" aria-label="Network">
                      {status.networks.map((n) => (
                        <button
                          key={n}
                          type="button"
                          className={
                            n === status.network
                              ? "network-option network-option-active"
                              : "network-option"
                          }
                          aria-pressed={n === status.network}
                          onClick={() => switchNetwork(n)}
                          disabled={switching || crossing}
                        >
                          {NETWORK_LABELS[n] ?? n}
                        </button>
                      ))}
                    </div>
                    {crossing && (
                      <p className="network-hint">Switch once this migration has finished.</p>
                    )}
                    {switchError && <p className="network-hint">{switchError}</p>}
                  </>
                ) : (
                  <span className="badge">{status.network}</span>
                )}
              </dd>
            </div>
            <div>
              <dt>Sync server</dt>
              <dd className="mono">{status.syncServer}</dd>
            </div>
            <div>
              <dt>Broadcast server</dt>
              <dd className="mono">{status.broadcastServer}</dd>
            </div>
          </dl>
        )}
        <p className="rail-note">
          Syncing and broadcasting go through different servers, so neither can link what your
          wallet looks at to what it sends.
        </p>
      </aside>

      <main className="stage">
        {loadError && (
          <div className="notice notice-error" role="alert">
            FERRY could not start: {loadError}
          </div>
        )}
        {!loadError && page}
      </main>
    </div>
  );
}

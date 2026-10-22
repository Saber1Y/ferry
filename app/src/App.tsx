import { useCallback, useEffect, useState } from "react";
import { errorMessage, getStatus, onSession, type Status } from "./api";
import Crossing from "./Crossing";
import History from "./History";
import Home from "./Home";
import Plan from "./Plan";
import Restore from "./Restore";

type View = "main" | "plan" | "history";

export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [view, setView] = useState<View>("main");

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

  let page: React.ReactNode = <p className="muted">Opening FERRY…</p>;
  if (status && !status.hasWallet) {
    page = <Restore defaultBirthday={status.defaultBirthday} onRestored={setStatus} />;
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
                <span className="badge">{status.network}</span>
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

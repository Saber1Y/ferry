import { useEffect, useState } from "react";
import { errorMessage, getStatus, type Status } from "./api";
import Home from "./Home";
import Restore from "./Restore";

export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    getStatus()
      .then(setStatus)
      .catch((e) => setLoadError(errorMessage(e)));
  }, []);

  return (
    <div className="shell">
      <aside className="rail">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true" />
          FERRY
        </div>
        <p className="rail-tagline">Orchard to Ironwood, the private way.</p>
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
        {!status && !loadError && <p className="muted">Opening FERRY…</p>}
        {status && !status.hasWallet && <Restore onRestored={setStatus} />}
        {status && status.hasWallet && <Home status={status} onStatus={setStatus} />}
      </main>
    </div>
  );
}

# FERRY build TODO

Deadline: **Oct 28, 2026, 23:59 UTC**.
Track: Core & Tooling.
Full reasoning and evidence: [docs/BUILD-PLAN.md](docs/BUILD-PLAN.md).

Legend: `[x]` done and verified, `[ ]` to do, **KILL** = decision point.

## Done (Oct 11-20)

- [x] Confirm the problem: Orchard closed to new value after NU6.3 (ZIP 258), 349,078 ZEC still in Orchard on Oct 20
- [x] Find a real naive migration on mainnet that leaks its amount (block 3,524,073, tx `85914dd5…`)
- [x] Confirm the engine exists: `zcash_pool_migration` (0.1.0 stable, 0.2.0-pre.1)
- [x] Confirm sign-now, prove-later works on v6 transactions (ZIP 318, ZIP 374, `pczt` Prover role)
- [x] Run librustzcash `pool_migration_prove_chain_sim` on this Mac: 10/10 passed, real proofs (commit `8cdf170`)
- [x] Confirm the Tauri stack versions (tauri 2.12.4, autostart, notification, single-instance plugins)
- [x] Docker Desktop running (29.5.3, arm64)
- [x] Create private repo `Saber1Y/ferry`

## Oct 20: local chain

- [ ] `regtest/zebrad.toml`: Regtest, NU5 to NU6.2 at height 1, NU6.3 at a set height (e.g. 200), RPC on localhost
- [ ] `regtest/chain.sh`: start / stop / mine N / rpc, our own script (Logos's has no licence, do not copy)
- [ ] Run `zebrad` 7.0.0-rc.0 (arm64 Linux build) in Docker, confirm `getblockcount` answers
- [ ] Build `lightwalletd` with Go, run two instances (A = sync, B = broadcast)
- [ ] Generate two demo wallets (Ada + a "naive" user) and their unified addresses

## Oct 21: prove one crossing on a real node (**KILL**)

- [ ] Mine Orchard coinbase to both demo wallets before the NU6.3 height, then mine past it
- [ ] `harness/`: a small Rust binary on `zcash_client_sqlite` + `zcash_pool_migration`, pinned to the tested librustzcash revision
- [ ] Harness: sync Ada from server A, see her Orchard balance
- [ ] Harness: plan, consent (flag), sign all with the seed, persist
- [ ] Harness: mine to the scheduled height, prove one crossing, broadcast through server B
- [ ] Confirm the node accepts it and the explorer view shows a standard denomination
- [ ] **KILL:** if no crossing is accepted by end of Oct 21, switch to the fallback CLI (see BUILD-PLAN.md)

## Oct 22: Tauri app shell

- [ ] Scaffold Tauri 2 + Vite + React + TypeScript
- [ ] Move harness logic into `src-tauri/src/core/`
- [ ] Settings: network (regtest), server A and B addresses
- [ ] Seed mode screen: warning, phrase + birthday input, derive view-only key, wipe phrase from memory
- [ ] Sync progress and "Orchard: X ZEC must move to Ironwood"
- [ ] Check: how the engine treats a piece whose scheduled height has passed

## Oct 23: plan, sign, schedule

- [ ] Plan + consent screen: pieces, timeline, fees, what strangers / this laptop can see
- [ ] Sign everything in one pass, then wipe the phrase, show confirmation
- [ ] Encrypted store (SQLite) with the key in the macOS Keychain
- [ ] Scheduler background task: wake on new blocks, run `advance_migration`, prove, broadcast via B, record result
- [ ] Never sync and broadcast in the same session

## Oct 24: desktop behaviour + observer

- [ ] Tray icon with "N of M moved", window can close while the scheduler keeps running
- [ ] Autostart at login, single instance
- [ ] Pause / resume, and spread missed pieces over fresh random delays after sleep (no burst)
- [ ] Notifications: "done" and "needs you" only
- [ ] Progress screen and Stop & wipe
- [ ] `observer/`: read all crossings in a block range, try to group them by wallet; naive run vs FERRY run

## Oct 25: end-to-end run + builds

- [ ] Full demo run on regtest, screen-recorded
- [ ] Tamper test: change a destination in a signed transaction, node rejects it
- [ ] GitHub Actions with `tauri-action`: macOS, Windows, Linux builds on tag
- [ ] Fix whatever broke

## Oct 26: submission material

- [ ] README (privacy boundary first, unverified items listed)
- [ ] `evidence/claims.yaml`: every claim mapped to a txid, log, or test
- [ ] Demo video (3-4 minutes)

## Oct 27-28: buffer and submit

- [ ] Security pass: no secrets in repo, phrase never written to disk, store encrypted
- [ ] Make the repo public
- [ ] Submit on thezecathon.com before 23:59 UTC Oct 28
- [ ] Stretch: judge playground on a VPS (regtest, fake coins, pre-funded demo seeds)

## Out of scope (named as future work in the README)

- Keystone QR signing mode
- Zallet mode
- Tor broadcast (`arti-client`)
- Headless build for always-on machines
- Code-signed installers

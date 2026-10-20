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

## Oct 20: local chain (done)

- [x] Zebra regtest config: upgrades through NU6.2 at height 1, NU6.3 at 300 (rendered by `chain.sh` from `heights.env`)
- [x] `regtest/chain.sh`: start / stop / mine / rpc / reset
- [x] `zebrad` 7.0.0-rc.0 (arm64) running in Docker
- [x] `lightwalletd` v0.5.5 built with Go, two instances (A = sync :9067, B = broadcast :9077)
- [x] Ada and Bob (naive) wallets, built by `regtest/setup-demo.sh`; Bob's one-shot migration recorded in run 2

## Oct 21: prove one crossing on a real node (**KILL**, passed Oct 20)

- [x] 750 REG of Orchard coinbase mined to Ada before NU6.3, then mined past it
- [x] `harness/`: zcash-devtool patch (broadcast command + checkpoint-safe preparation anchor) instead of a new binary
- [x] Sync Ada from server A: Orchard 750 REG
- [x] Plan (500 + 200), commit, 12 transactions signed in one pass
- [x] All 10 preparations and both crossings proved and broadcast through server B
- [x] Node accepted both crossings (mined at 745): orchard out 500.00015 / 200.00015, ironwood in 500 / 200
- [x] **KILL criterion passed on Oct 20.** Evidence: `evidence/regtest-run-1/`

## Found in regtest runs 1 and 2 (fix in FERRY)

- [x] `chain.sh` lost freshly mined blocks on zebrad restart; now waits for the block backup
- [ ] Seed mode must ask for the wallet birthday carefully (funds below the birthday are never scanned, so they would silently not migrate)

- [x] Scheduler: one action per session, at most one broadcast, random delay between crossings. Run 2: 13 crossings in 13 different blocks (`evidence/regtest-run-2/`)
- [x] 49.99 REG left in Orchard: the engine defers pieces when fees do not fit; fixed with automatic follow-up runs (final: 749.98 Ironwood, 0.00925 Orchard)
- [ ] Driver: stop when the migration completes (done in `drive.sh`)

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

# FERRY - move your ZEC to Ironwood privately, from your own laptop

Zecathon 2026 build plan.
Checked 2026-10-11, reworked around a Tauri desktop app on 2026-10-20.
Deadline: Oct 28, 2026, 23:59 UTC (8 days left).
Track: Core & Tooling ("Zebra, librustzcash, indexers, wallet SDKs, provers. The unglamorous work everything else is built on.").
One submission, one track, so it is entered here only, and it also competes for the $20K grand prize.

## The problem, in one paragraph

After the June 2026 Orchard soundness bug, NU6.3 activated on mainnet at block 3,428,143.
Since then, "No new value may enter the Orchard pool" and ordinary payments inside Orchard are disabled (ZIP 258).
Every holder has to move funds across to the new Ironwood pool.
Doing it naively reveals your balance.
ZIP 318 defines the private way: split funds into standard amounts, pre-sign everything, then send each piece separately at random times (mean 66 blocks, about 82 minutes, apart; never more than 576 blocks, about 12 hours), proved against shared anchors.
That works only if something is running at each scheduled time.
Phone wallets cannot do that reliably, because iOS and Android stop background apps, so Zodl's migration depends on you opening the app.
Hardware-wallet and cold-storage holders have nothing at all.
**349,078 ZEC was still in the Orchard pool on Oct 20** (361,806 on Oct 11, so about 1,400 ZEC a day is moving).

## What FERRY is

**A desktop app for Mac, Windows and Linux** that runs your Orchard to Ironwood migration the private way, from your own laptop.

- You install it like any app.
- You connect your wallet, review the plan, and sign everything once.
- Then FERRY sits in your menu bar or system tray, typically for a day or two, sending each piece at its scheduled time while your laptop is on.
  Regtest run 2 moved 750 REG in 13 crossings over about 2,000 blocks, which is about 42 hours at mainnet's 75-second block time.
- It holds your view-only key, never your spending key (except briefly in seed mode, below).
- Every transaction is already signed to your own Ironwood address, so FERRY cannot send your money anywhere else.

This is the "background executor that runs the migrations at the scheduled times" Kris Nuttycombe asked for in [zcash/zallet#481](https://github.com/zcash/zallet/issues/481), which is still open.

### What we deploy, and what we do not

| What | Who runs it | Why |
|---|---|---|
| **FERRY desktop app** (`.dmg`, `.exe`/`.msi`, `.AppImage`/`.deb`) | Each user, on their own computer | It holds their view-only key, so it must be on their machine |
| **Download page + GitHub Releases** | Us | Installers built by GitHub Actions (`tauri-action`). This is the whole "deployment" for real users |
| **Zcash servers (lightwalletd)** | Already public and community-run, for example `zec.rocks` and `na.zec.rocks` | FERRY syncs from one and broadcasts through a different one |
| **Judge playground** (stretch) | Us, on a cheap VPS | A regtest chain with fake coins and pre-funded demo wallets. No real keys |
| **A central FERRY server** | **Nobody, by design** | It would hold every user's viewing key: one database of everyone's full history |

### Laptop behaviour

- **Laptop on:** FERRY runs in the tray and sends pieces when they are due. It starts at login (autostart) so a reboot does not stop the migration.
- **Laptop asleep or off:** nothing is sent and nothing breaks. Each pre-signed piece stays valid for 1 to 2 months (ZIP 318 expiry rule), so a missed day only makes the migration take longer.
- **Laptop wakes up:** FERRY never sends missed pieces all at once, because "sent right after the lid opened" is itself a clue. It spreads them over fresh random delays.
- **Optional "keep awake while plugged in":** prevents sleep only on mains power, only while a piece is due soon.
- **Always-on machines** (Mac mini, home server, Start9/Umbrel) run the same app or a future headless build, and give the best timing privacy.

### Who can sign

| Mode | Who it is for | How it works | Status for the hackathon |
|---|---|---|---|
| **Seed mode** | Most people, including Zodl phone users | Type the recovery phrase into FERRY. It signs every transaction in memory, then the phrase is wiped. It is never written to disk. | **Built** (used in the demo) |
| **Keystone mode** | Hardware-wallet owners | Animated QR signing rounds, each under 14 KiB for firmware 3.1.0 | Future work (named in README) |
| **Zallet mode** | Node wallet users | Export unsigned PCZTs to Zallet, import signed ones | Future work |

Seed mode is honest about its trade-off: for a few minutes FERRY holds full spending power, like any desktop wallet.
The app says so before you type anything.

## Verification: every claim checked

| # | Claim | Status | Evidence |
|---|---|---|---|
| 1 | Orchard is closed to new value; holders must migrate | **Confirmed** | ZIP 258 consensus rules: "No new value may enter the Orchard pool"; Orchard cross-address transfers disabled. |
| 2 | NU6.3 is active on mainnet | **Confirmed** | Activation 3,428,143; mainnet tip 3,524,090 on Oct 20. |
| 3 | NU6.3 is active on testnet | **Confirmed** | Activation 4,134,000; testnet explorer shows 4,331,234 (its index is stale since Sep 7, but it is already past activation). |
| 4 | Lots of value still needs to migrate | **Confirmed** | mainnet.zcashexplorer.app: Orchard pool 361,806.56 ZEC (Oct 11), 349,078.04 ZEC (Oct 20). |
| 5 | A migration engine exists to build on | **Confirmed** | `zcash_pool_migration` in librustzcash: 0.1.0 stable and 0.2.0-pre.1 on crates.io. Plans denominations, preparation, scheduling, signing rounds, PCZTs. |
| 6 | Proving and broadcasting are left to the app (the gap FERRY fills) | **Confirmed** | Engine docs: "the consuming application broadcasts the transactions and reports results back". `advance_migration` is "the API a consuming application uses". |
| 7 | Transactions can be signed now and proved later | **Confirmed** | ZIP 318: "After NU6.3, anchors are no longer committed to by signature digests". v6 puts the anchor in the proof, so the prover picks it at broadcast time. |
| 8 | A pre-signed transaction cannot be altered by the executor | **Confirmed** | The signature covers the transaction effects. Only the anchor and witness are set later (PCZT Updater role, ZIP 374). |
| 9 | Hardware wallets can sign migrations | **Confirmed, with a bug** | Engine has a Keystone signing budget (96 actions per round). Keystone firmware 3.0.4 accepts migration PCZTs. Firmware 3.1.0 rejects anything over 14 KiB ([keystone3-firmware#2294](https://github.com/KeystoneHQ/keystone3-firmware/issues/2294)). |
| 10 | We can demo with real transactions | **Confirmed via regtest** | You cannot get new Orchard funds on testnet or mainnet (claim 1). Zebra regtest has a configurable `nu6_3` activation height: mine Orchard coinbase first, then activate. Logos's wallet test does exactly this ("mines Orchard coinbase before NU6.3"). |
| 11 | Nobody has built a standalone executor | **Confirmed (as of Oct 11)** | GitHub search: Zodl (iOS migration screens) and Logos (`logos-zcash-wallet-core-module`) run migration inside their own wallets. No desktop or hardware-wallet executor found. |
| 12 | The desktop stack exists | **Confirmed** | crates.io, Oct 20: `tauri` 2.12.4, `tauri-plugin-autostart` 2.7.0, `tauri-plugin-notification` 2.7.0, `tauri-plugin-single-instance` 2.5.4, `keepawake` 0.6.1, `arti-client` 0.47.0. |
| 13 | A naive migration really is visible | **Confirmed** | Block 3,524,073, tx `85914dd5…`: 0.01996887 ZEC out of Orchard, 0.01971887 ZEC into Ironwood, readable by anyone in the `/raw` JSON. |

### Corrections to my earlier pitch

- **Wrong: "hide the wallet from the executor".**
  The `pczt` crate says the prover requires "the full viewing key that received the note being spent" plus a witness.
  So whoever proves can see the whole account's history (but cannot spend).
  That is why FERRY runs on the user's own computer.
- **Overstated: "fix the Keystone 14 KiB bug".**
  The engine already batches signing into rounds.
  Keystone mode only has to choose a smaller round budget so each animated QR stays under 14 KiB on firmware 3.1.0.
- **Changed: "home server".**
  A laptop that is on a few hours a day is enough.
  An always-on machine is better for timing privacy, but not required.
- **New constraint:** the demo must run on regtest, because no new wallet can get Orchard funds on public networks.

## User flow

### Persona

Ada holds 40.37 ZEC in Orchard in Zodl on her phone.
She uses a MacBook every day.
She has not migrated because Zodl wants her to keep opening the app until it finishes, and she has read that a careless migration reveals her balance.

### 1. Install (2 minutes)

1. Ada downloads FERRY from the download page and drags it into Applications.
2. On first launch FERRY asks her to choose an app passphrase or let it use the macOS Keychain, which encrypts everything FERRY stores (viewing key, signed transactions, schedule).
3. FERRY asks to start at login, so the migration survives reboots.

### 2. Connect the wallet (2 minutes, then a background sync)

1. She picks **Seed mode** and reads the warning: "For the next few minutes FERRY can spend your funds. After signing, your phrase is wiped."
2. She types her recovery phrase and her wallet birthday (Zodl shows it in settings), so syncing does not start from the beginning of the chain.
3. FERRY derives the view-only key, keeps only that, and syncs from server A.
4. It shows **"Orchard: 40.37 ZEC must move to Ironwood"**.

### 3. Review the plan and consent (1 minute, ZIP 318 requires this)

FERRY asks the engine for a plan and shows:

- **Pieces:** 40.37 ZEC becomes standard amounts (for example 20 + 10 + 10 + 0.2 + 0.1 + 0.05 + 0.02), plus dust under 0.01 that stays in Orchard.
- **Runs:** the engine plans one run at a time and drops the smallest pieces when the fees do not fit the balance (it never swaps in smaller pieces, so the amounts stay a function of the balance alone).
  When that happens, FERRY says so up front: "Most of your balance moves in this run. About X ZEC is left for a short second run, which you will sign once more when this one finishes."
  Seen in regtest run 1: a round 750 left no fee slack, so run 1 crossed 500 + 200 and run 2 covers the remaining 49.99.
- **Timeline:** preparation (a few hours), then one crossing about every 82 minutes on average (never more than 12 hours apart), finishing in a day or two if the laptop stays on. Every hour the laptop is off adds to that.
- **Fees:** total network fees.
- **What a stranger can see:** a few standard-size crossings at random times, the same shape as everyone else's.
- **What they cannot see:** that the crossings belong to one person, or her total.
- **What this laptop can see:** her full account history, which is why it has to be her own computer.

Ada taps **Start migration**. This is her consent.

### 4. Sign everything (seconds in seed mode)

1. FERRY builds and signs every preparation and crossing transaction in one pass.
2. It wipes the recovery phrase from memory.
3. It shows **"All N transactions signed. FERRY now holds only your view-only key."**

### 5. Preparation (automatic, a few hours)

1. FERRY broadcasts the preparation transactions, which split her notes into the exact amounts and send only to herself.
2. It waits for them to be mined, then for the next shared anchor boundary (every 144 blocks, about 3 hours).

### 6. Crossing (automatic, hours to a couple of days)

At each scheduled height, while the laptop is on, FERRY:

1. Picks a shared boundary anchor for that piece at random (recency-weighted, per the engine).
2. Installs the anchor and witness, and produces the proof on the laptop.
3. Broadcasts through server B, never in the same session as a sync from server A.
4. Records the result and moves on.

Ada closes the lid, travels, comes back.
FERRY picks up where it left off without bursting.

### 7. Progress

- **Menu-bar icon:** "4 of 7 moved".
- **Window:** progress, the next crossing window (approximate on purpose), and her own transaction list.
- **Notifications:** only "done" and "needs you".

### 7b. Follow-up run (only when the engine deferred some pieces)

1. When the run finishes, FERRY plans the remaining balance and shows it, for example "49.98 ZEC in 11 pieces".
2. In seed mode Ada enters her phrase once more; FERRY signs the follow-up run and wipes it again.
3. The follow-up run then proceeds like steps 5 to 7.

### 8. Done

1. **"40.36 ZEC is now in Ironwood. 0.008 ZEC dust remains in Orchard."**
2. Zodl on her phone shows the Ironwood balance as usual.
3. **Wipe** deletes the viewing key and all migration data, and FERRY removes itself from login items.

### Edge cases

| Situation | What FERRY does |
|---|---|
| Laptop off for weeks, a piece reaches its expiry | Marks it "needs you". Ada re-enters her phrase (or scans with Keystone, later) to re-sign that piece only. |
| Laptop wakes after missed pieces | Spreads them over fresh random delays, never sends them back to back. |
| Ada spends from Orchard in Zodl mid-migration | Detects the conflicting note, stops those pieces, offers to re-plan the rest. |
| No internet | Waits. Nothing is lost. |
| FERRY quit from the tray | Asks "Pause migration?" and resumes on next launch. |
| Ada wants to stop | **Stop & wipe** deletes unbroadcast transactions. Pieces already sent are already in Ironwood. |

## Privacy boundary (goes first in the README)

- **Strangers on-chain** see standard-size crossings at random times with shared anchors, the same as every wallet following ZIP 318. Crossing amounts are public by design: ZIP 209 reveals the net value moving between pools.
- **Ada's laptop** holds her full viewing key, encrypted at rest. Anyone with access to her unlocked laptop could read her history. FERRY's data folder is excluded from cloud backups.
- **Seed mode** holds her spending key in memory for the minutes it takes to sign, then wipes it. It is never written to disk.
- **Lightwalletd operators** see syncs on one server and broadcasts on another, never in the same session. Without Tor (hackathon build) they still see her IP address. Tor via `arti-client` is named future work.
- **What FERRY cannot do:** change a destination, or send funds anywhere except her own Ironwood address.

## See the leak yourself (mainnet, read-only)

Any Orchard to Ironwood transaction publishes how much crossed, because ZIP 209 tracks each pool's balance.
On mainnet.zcashexplorer.app, open a transaction and then its `/raw` JSON:
`orchard.valueBalance` is what left Orchard and `ironwood.valueBalance` (negative) is what entered Ironwood.

A real example found on 2026-10-20:
block 3,524,073, transaction `85914dd575cd79422ade1d23a38a80731b2e61d3e371ade36cb6a1fe6f42e152`.
0.01996887 ZEC left Orchard and 0.01971887 ZEC entered Ironwood (0.00025 fee).
An odd amount like that, moved in one transaction, is almost certainly someone's entire Orchard balance, now public.

What else the naive way leaks:
- The lightwalletd operator saw that wallet's IP sync and then broadcast, so it can tie the amount to a person.
- The anchor in the proof shows how recently the wallet synced.
- If someone splits into odd amounts and sends them close together, the pieces still add up to the balance.

What ZIP 318 changes: standard amounts that many wallets share, random times, shared anchors, a broadcast server different from the sync server.
An observer sees many identical 10 ZEC crossings and cannot tell which belong together.

## Architecture

```
ferry/
  src-tauri/                 The desktop app's Rust side (Tauri 2).
    src/core/                Wraps zcash_pool_migration + zcash_client_sqlite:
                             seed or view-only import, sync, plan, sign (seed mode),
                             advance_migration loop (install anchor + witness, prove,
                             broadcast, report).
    src/scheduler.rs         Background task inside the app process: wakes on new blocks,
                             runs due pieces, reschedules after sleep without bursting.
    src/net.rs               Two lightwalletd endpoints: sync (A) and broadcast (B).
    src/store.rs             Encrypted SQLite, key in the OS keychain (or passphrase).
    tauri.conf.json          Tray icon, autostart, notifications, single instance.
  ui/                        Vite + React + TypeScript screens: onboarding, connect wallet,
                             plan + consent, signing, progress, wipe, settings.
  observer/                  Small tool that reads all crossings in a block range and tries
                             to group them by wallet (naive run vs FERRY run).
  regtest/                   Our own scripts: zebrad regtest (NU6.3 at a set height),
                             Orchard coinbase to demo wallets first, two lightwalletd.
  .github/workflows/         tauri-action builds for macOS, Windows, Linux on each tag.
  evidence/                  Run logs, txids, claims.yaml, proof page.
```

Notes:
- Proving runs on the laptop.
  Halo 2 needs no downloaded parameter files (no trusted setup), so there is no big first-run download.
- The app is one process: the window can be closed while the tray icon and scheduler keep running.

## Testing it all locally (this Mac)

Everything runs on one machine with fake coins, on a private regtest chain:

| Piece | How it runs here | Status |
|---|---|---|
| Migration engine, real proofs, simulated chain | `cargo test --release` of librustzcash's `pool_migration_prove_chain_sim` (commit `8cdf170`) | **Passed on this Mac, Oct 20: 10 of 10 tests in 100 s, including `migration_proves_end_to_end_against_a_funded_wallet`. First build took 9 min 47 s.** |
| Local chain with NU6.3 | `zebrad` 7.0.0-rc.0 regtest. Releases ship Linux binaries only, so run the arm64 Linux build in Docker, or compile on macOS. Config: NU5 to NU6.2 at height 1, `"NU6.3"` at a later height, mining with `generate`. | Config pattern confirmed |
| Two lightwalletd servers | Build with Go 1.26 (installed). MIT licensed. | Not started |
| Ada's Orchard funds | Mine Orchard coinbase to her address before the NU6.3 height | Confirmed allowed |
| FERRY desktop app | `cargo tauri dev` against the local chain | To build |
| Observer | Reads crossings from the local chain | To build |

Machine notes: 16 GB RAM and 8 cores are fine.
Free disk is about 24 GB, which is tight: Rust build folders can take 10 to 15 GB.
Docker is installed but its daemon is not running.

Demo time trick: on regtest we mine blocks on demand, so a day or two of crossings passes in minutes.
The scheduling delays and anchor bucket are parameters, shortened in a clearly labelled "regtest demo" setting.

## Demo (real transactions, regtest)

1. The regtest chain mines Orchard coinbase to Ada before NU6.3, then activates NU6.3.
2. **Observer, naive run:** a second wallet moves its whole balance in one transaction. The observer prints its exact balance.
3. **FERRY run:** Ada installs FERRY, connects with seed mode, reviews the plan, consents, signs. The phrase is wiped on screen.
4. **Time-lapse:** blocks are mined, FERRY broadcasts pieces at scheduled heights. The laptop is "slept" (app paused) and resumed, with no burst.
5. **Observer, FERRY run:** standard-size crossings mixed with other demo wallets; it cannot group Ada's pieces or recover her total.
6. **Tamper test:** change a destination in a signed transaction and the node rejects it.

## 8-day plan (from Oct 20)

**In:** regtest chain, Tauri app with seed mode, real prove-and-broadcast scheduler, tray + autostart, sleep/wake rescheduling, encrypted store, observer, release builds.
**Out (named as future work):** Keystone QR mode, Zallet mode, Tor, headless build for always-on boxes, code signing.

| Day | Milestone | Kill criterion |
|---|---|---|
| Oct 20 | Engine simulation test passes on this Mac. Docker running. `zebrad` regtest up with NU6.3 at a set height. | |
| Oct 21 | Orchard coinbase mined to demo wallets before NU6.3. Two lightwalletd up. A tiny Rust harness proves one migration transfer and the local node accepts it. | If no transfer is accepted by end of Oct 21, fall back (below). |
| Oct 22 | Tauri app scaffold. Seed-mode import, sync from server A, Orchard balance on screen. | |
| Oct 23 | Plan + consent screen, seed-mode signing and wipe, encrypted store, scheduler running `advance_migration`, broadcast via server B. | |
| Oct 24 | Tray icon, autostart, notifications, pause/resume and sleep rescheduling, progress, wipe. Observer tool. | |
| Oct 25 | Full demo run end to end, recorded. Release builds from GitHub Actions for macOS, Windows, Linux. | |
| Oct 26 | README (privacy boundary first), claims.yaml, demo video. | |
| Oct 27-28 | Buffer and submit early. Stretch: judge playground on a VPS. | Deadline Oct 28 23:59 UTC. |

## Fallback

If proving or broadcasting on regtest does not work by Oct 21, ship **`ferry` as a Zallet companion CLI** (`ferry plan`, `ferry sign`, `ferry run`) against the engine's in-memory chain simulation.
That is honest about being simulated, still answers zallet#481, and fits Core & Tooling.

## Risks

- **The engine is new** (0.1.0 stable, 0.2.0 in pre-release). APIs may change. Pin the git revision we test against.
- **ZODL may ship the same thing.** The issue is tracked internally (Linear COR-1358). Our wedge: a desktop app any wallet's users can use, shipped now.
- **Mainnet is not part of the demo.** Say so plainly.
- **Windows and Linux builds** come from CI only and are not tested on real machines in this window. Say so.
- **Unsigned builds** trigger "unidentified developer" warnings on macOS and SmartScreen on Windows. Normal for a hackathon, stated in the README.
- **Rescheduling after sleep** may need our own logic on top of the engine's schedule. Check on Oct 22.

## Still to confirm (Oct 20-22)

1. ~~The engine simulation test passes on this Mac.~~ Done: 10/10 passed on Oct 20.
2. Zodl's wallet birthday and recovery phrase restore cleanly in `zcash_client_sqlite` (so seed mode works for Zodl users).
3. How the engine handles a piece whose scheduled height has passed, to design the no-burst rescheduling.

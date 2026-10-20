# Regtest run 1: a full ZIP 318 migration on a real local node (2026-10-20)

**Network:** local regtest, not mainnet or testnet.
**Coins:** fake (REG).

## Setup

- `zebrad` 7.0.0-rc.0 (`zfnd/zebra:7.0.0-rc.0`, arm64) in Docker, regtest.
  Upgrades through NU6.2 at height 1, NU6.3 (Ironwood) at height 300.
- Two `lightwalletd` v0.5.5 servers: A (127.0.0.1:9067) for sync, B (127.0.0.1:9077) for broadcast.
- `zcash-devtool` at `0b3436b` with `harness/zcash-devtool-ferry.patch`, built with `regtest_support`.
- Ada's wallet: mined 120 Orchard coinbase rewards of 6.25 REG at heights 3-122 (750 REG), before NU6.3.

## What happened

| Step | Height | Result |
|---|---|---|
| Chain pools before migration | 332 | orchard 750, ironwood 0 |
| `migration plan` | 333 | crossings 500 + 200, 2 funding notes, 3 preparation layers, 10 preparation transactions |
| `migration commit` (in-process signing) | 333 | 12 transactions signed in one pass |
| Preparation 0 proved by hand, broadcast via B | 343 | `403cce9842a88f07ef1ddbc2e90fe297ba96ed96647772ac7b1dcde055be1331`, mined at 344: v6, 16 Orchard actions, value balance 0.0008 (fee only) |
| Preparations 1-9, by `regtest/drive.sh` | 349-519 | all proved and broadcast via B (see `drive.log`) |
| Crossings 0 and 1 proved | 589 | |
| Crossing 0 broadcast via B | 744 | `f33627f4bcc77a8a712aba12511e0029305540c5943a7aca6ba06ad2afabe397`, mined at 745: v6, orchard out 500.00015, ironwood in 500.0, expiry 69120 |
| Crossing 1 broadcast via B | 744 | `f6711110f2f76015c5bcc255405f365e27f1709b319429ff848a0c33e695db4d`, mined at 745: v6, orchard out 200.00015, ironwood in 200.0 |
| Chain pools after | 1104 | orchard 49.9917, ironwood 700.0 |
| Ada's wallet after | 1104 | Ironwood 700.00 REG, Orchard 49.9917 REG |

## What this proves

- Sign once, then prove and broadcast later against the drawn anchors, works on a real node with v6 transactions.
- Sync (server A) and broadcast (server B) can be separated.
- Each crossing publicly shows only a canonical denomination (500, 200) plus the fixed fee.

## Problems found (to fix in FERRY)

1. **Both crossings were broadcast in the same block (744) and mined together (745).**
   The driver sends everything that is due in one pass, which links them by timing.
   FERRY's scheduler must spread due crossings over separate random delays.
2. **49.99 REG was left in Orchard.**
   The plan only contained 500 + 200 crossings, so about 50 REG was never scheduled to cross.
   ZIP 318 leaves only dust below 0.01 behind.
   Cause not yet investigated (denomination strategy or fee reconciliation).
3. **devtool needed two changes** (see `harness/README.md`): a broadcast step, and a preparation anchor that the wallet actually holds a checkpoint for.

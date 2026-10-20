# Regtest run 2: scheduler fix, clean funding, naive vs FERRY (2026-10-20)

**Network:** local regtest, not mainnet or testnet.
**Coins:** fake (REG).

## Setup

Built by `regtest/setup-demo.sh`, after fixing `regtest/chain.sh` to wait for zebrad's block backup before restarting (run 1's setup lost blocks on restart).

- Blocks 1-2 and 163 onward: a burn address.
- Blocks 3-122: Orchard coinbase to Ada, 750.00 REG.
- Blocks 123-162: Orchard coinbase to Bob, 250.00 REG.
- NU6.3 (Ironwood) at 300.

## Bob: the naive migration

One transaction, his whole balance: `2d2866df8bad4ab45b0f0d8ceb739f889dfcfaef0a54384257d62a35c1655242`, mined at 331.
Publicly visible: 249.99215 out of Orchard, 249.99 into Ironwood.
Anyone reading the chain learns Bob's balance to within the fee.

## Ada: FERRY's scheduler (`regtest/drive.sh`)

Rules (ZIP 318 "Background scheduling" and "Fallback on application open"):
- Each session either syncs (server A) or broadcasts (server B), never both.
- At most one broadcast per session.
- After each crossing, the next waits a fresh exponential delay (mean 66 blocks, redrawn above 576, minimum 1).

Run 1 crossed 500 + 200.
The engine deferred the rest, and a follow-up run (signed automatically in this harness) crossed the remaining 49.98.

| Mined at | Ironwood in | Transaction |
|---|---|---|
| 847 | 500.0 | `f8111cb05f7e5ca8c1fbbf0d08ab225e4173741fd8a2bf9cdd6d4415ea6f33b6` |
| 957 | 200.0 | `4fc29badaa543c11666a2c0b61fee08cb7361185bf05e1c1debceab3c789a330` |
| 1402 | 2.0 | `99c9733b20978d53a03cc1a05fe01b4ff4be78ad2997bfefe08c40beb96f9b21` |
| 1452 | 5.0 | `fcfac92a0a7d266c89306ed17944ad36cdd16e514949ac8d901b0ec6ed1e1ee7` |
| 1517 | 0.05 | `648c55a8bd388a2939840fcee62dce63155d6674c75433f461ad88b71c75e384` |
| 1587 | 0.5 | `413742fc242ee1cf2bd2e295769f9db10271f20373c55b374c3a294466ac2987` |
| 1682 | 0.01 | `8a470459c0ac2e2ebeed2c717162507b25167b59f2c3aa9263277063fdb01944` |
| 1792 | 2.0 | `20e1909189e4c96f06b30e73124552f69199f42225d40fd146da512572310e79` |
| 1892 | 0.2 | `d42fa967ee591c1a41bb4c464f528248d5a465085f91186c46754326440d033d` |
| 1897 | 0.02 | `a67f6b9aa920ab534ca6c6add55e19bebb38ca1f345103e8cb8609b743d89e98` |
| 1967 | 20.0 | `e9ed5a5bf23d656a385fd5489f28ae2ff979465162c66365cd1d10eadf1002d7` |
| 2002 | 0.2 | `8edd5af7bf703139c9f4abfe52651da4b82d458903cfdde5aa55bd40aa223ad7` |
| 2322 | 20.0 | `549397f503cbdae5d5b6b296828de21af7850f847825c042dcc9a3a0c56a0231` |

13 crossings, 13 different blocks, none shared.
Every crossing is a canonical denomination.

## End state

| | Orchard | Ironwood |
|---|---|---|
| Ada | 0.00925 | 749.98 |
| Bob | 0.00785 | 249.99 |
| Chain pools | 0.0171 | 999.97 |

The chain's pool totals equal the sum of the two wallets.
Ada's leftover dust (0.00925) is under ZIP 318's 0.01 limit.

## Not covered yet

- Signing was in-process (seed mode). In the app, the follow-up run needs the user to enter the phrase again.
- No Tor: both servers are on localhost.
- The observer tool that reads these crossings and tries to group them by wallet is still to build.

# Regtest run 4: a person migrates with the FERRY desktop app (2026-10-21)

**Network:** local regtest, not mainnet or testnet.
**Coins:** fake (REG).

The first migration started from the app's own screens by a person, with the app's background scheduler doing the rest unattended.

## Steps

1. Bob's wallet (250.00 REG in Orchard, funded before NU6.3 by `regtest/setup-demo.sh`) restored with `ferry-core` into its own folder; the app was launched on it (`FERRY_WALLET_DIR`).
2. In the app: **Plan the crossing** → reviewed the plan → pasted the phrase, ticked the agreement → **Sign and start**.
3. A block every 3 seconds (`regtest/mine-every.sh`); the app ran one scheduler session every 3 seconds.
4. All 9 preparation transactions mined, then the 12 crossings went out one per session, a random gap apart.
5. The app returned to its home screen: "Everything has crossed to Ironwood."

## Crossings (read back by `observer/observe.py`)

| Mined at | Ironwood in | Transaction |
|---|---|---|
| 3114 | 20.0 | `df71c0c6aa2de17a95d6da9e699d638b1e1dc31f0fd41c537949651e1dc0a550` |
| 3182 | 0.02 | `11f10e05bac5ee84dfa6e75268dcf49b1c747ee6a50d50397f50777b1e5b132a` |
| 3271 | 0.5 | `ffda51024398684f085a9ed0c1d99df003745720e175a88b4d361123cd96f144` |
| 3399 | 0.05 | `e65f30ad0d227d75ceec61e517ae350dd370d680aa581691a882d1933002aaf0` |
| 3441 | 200.0 | `99f736e3a00f5b997d403e6e8cdd064b5007aa5d722c217d37232afffa951448` |
| 3451 | 2.0 | `521c8f69b7722459a19148b18bb0294482bf562902796e20fb590cd3756e433b` |
| 3472 | 0.2 | `5597e448a9d7c83011cb9ba0a35c84f897082d9f0138dc068ba153690d2791e2` |
| 3546 | 0.02 | `ed01155042f34ca2d6982aa832bd521b24ddfe7d805148007d9aa88af586b38a` |
| 3647 | 0.2 | `2384f8fc9dd67eab01aac427c9efe251d29993a0fd15448978a6645c730fdbed` |
| 3771 | 5.0 | `c76dce46c7b454345648c72c15c1b3051022532ec498e3ff863c523b9b8d5a26` |
| 3792 | 2.0 | `ca98fed963e2bb6d9d94a5f487e67d167377772fc06146b02e0ec490b63470f0` |
| 3911 | 20.0 | `0743be8c765eef896d024e507dcdfee7660825dcb68af7aa7d1c2496aff72922` |

12 crossings, 12 different blocks, all standard amounts; the observer flags none of them.

## End state

Bob (read by `ferry-core`): Ironwood 249.99 REG, Orchard 0.001 REG (below the 0.01 dust limit, so no follow-up run).
About 0.009 REG paid in fees.

## Notes

- Earlier on the same chain, Bob had not migrated naively; the one-shot comparison is recorded in `evidence/regtest-run-2/` (249.99 REG in one transaction).
- The observer's caveat applies: on a chain with one migrating wallet at a time, standard amounts cannot hide a wallet in a crowd. They remove the shapes that expose it.

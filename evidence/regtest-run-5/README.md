# Regtest run 5: catch-up after a closed app, and a follow-up run signed in the app (2026-10-22)

**Network:** local regtest, not mainnet or testnet.
**Coins:** fake (REG).

## Catch-up after the app was closed

1. `regtest/setup-demo.sh` funded Ada with 750.00 REG of Orchard coinbase before NU6.3.
2. `ferry-core` restored her wallet and committed a migration: crossings of 500 and 200, which the engine scheduled at blocks 904 and 908, only 4 blocks apart.
3. The scheduler ran until all 10 preparation transactions were mined (height 880), then stopped, as if the laptop had been closed.
4. 120 blocks were mined with nothing running, to height 1000, so both crossings became overdue.
5. The app was launched. It logged `overdue crossings: next one not before block 1032` (a fresh random delay from the current height) and sent nothing at launch.
6. The crossings then went out one per session: 500 at block 1038 and 200 at block 1085, 47 blocks apart.

ZIP 318 ("Fallback on application open") requires that a wallet never send more than one overdue transfer at open, and recommends shifting or redrawing the remaining schedule. FERRY does both: nothing at open, then fresh random gaps.

## Follow-up run, signed by a person in the app

The first run left 49.99 REG in Orchard (the engine defers pieces it cannot fund in one run).
In the app, the follow-up plan (49.98 REG in 11 pieces) was reviewed and signed with Ada's phrase.
The app's scheduler sent the pieces and recorded each in its local history.

| Mined at | Ironwood in |
|---|---|
| 6776 | 2 |
| 6792 | 0.05 |
| 6833 | 0.02 |
| 6859 | 5 |
| 6869 | 0.2 |
| 7142 | 0.01 |
| 7219 | 0.2 |
| 7319 | 20 |
| 7469 | 0.5 |
| 7574 | 2 |
| 7732 | 20 |

The last crossing, `5a1cb603fc95356faba03516174c0a91909e3440410d3f68ebefdf41698fb04a`, was confirmed on the node (orchard out 20.00015, ironwood in 20).
`observer/observe.py` reports all 11 as standard amounts with nothing linking them.

## End state

Ada (read by `ferry-core`): Ironwood 749.98 REG, Orchard 0.00925 REG.
Local history: 11 crossings and 1 preparation, 49.98 REG in total.

## Found and fixed during this run

- The "has this run finished?" check sat behind the random crossing gate, so the app did not notice a finished run until the gate opened. It now runs first in every session.
- The crossing gate also delayed preparation transactions, which reveal nothing. It now applies to crossings only.
- After a restart the app forgot a pending follow-up run. It now restores it when the wallet has migration history.

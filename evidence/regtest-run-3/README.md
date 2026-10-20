# Regtest run 3: FERRY's own engine drives the whole migration (2026-10-20)

**Network:** local regtest, not mainnet or testnet.
**Coins:** fake (REG).

Run 2 used zcash-devtool with a patch. This run uses only `ferry-core`, the code the desktop app runs, through `crates/ferry-core/examples/migrate.rs` and `regtest/drive-ferry.sh`.

## Steps

1. `regtest/setup-demo.sh`: fresh chain, Ada funded with 750.00 REG of Orchard coinbase before NU6.3.
2. `ferry-core` restored Ada from her phrase into a new folder (`Wallet::restore`) and synced: Orchard 750, Ironwood 0.
3. `Wallet::plan_migration`: 500 + 200, 10 preparation transactions, 49.9917 left for a follow-up run, 0.0083 fees.
4. `Wallet::commit_migration` with the phrase: every transaction pre-signed; the phrase is checked against the wallet's viewing key first.
5. `Wallet::run_session` once per 5 mined blocks. Each session either syncs from server A and proves, or broadcasts one transaction through server B, never both. After each crossing the next waits a fresh exponential delay (mean 66 blocks, at most 576, at least 1).
6. At height 845 the session reported `followUpNeeded` with a plan for the remaining 49.98; the harness committed it with the phrase, as the app asks the user to.
7. At height 2215 the session reported `complete`.

## Crossings

| Mined at | Ironwood in | Transaction |
|---|---|---|
| 761 | 500.0 | `06010823e471d0601ca7987f1291809f1f6d4816120bf376c450c10d75be853d` |
| 821 | 200.0 | `30edb9a58232da22d4a3e60ce7506463149fdb7ed379625cbb182277ff28cd0d` |
| 1096 | 2.0 | `3a6ccdac1850c7d10d74456724925ea4648cd352589a8f463eada36bebac394b` |
| 1156 | 0.05 | `12fae4998dca7ee4ddd62302bb34ea1e3e2bed9905cb1f8cc4c749e32dab4237` |
| 1356 | 2.0 | `f1483978c54afec6bcba27e5603529c7d0de77f30ce82c52c8f1a8648ef3a58f` |
| 1501 | 0.5 | `edfa995c2ed26a5b1859280f19d7388b5dd54dc38ee45eb5f0005646c3f71061` |
| 1591 | 5.0 | `ba86eefb3a49635caa4d0c1a66a6949295031c84d7e691f96d8c9114207f303c` |
| 1696 | 20.0 | `c2cc39068cbda4ad476da1daf0a5bc6a15f178852276a90be766b32a4eb0922f` |
| 1871 | 0.2 | `fc002cf0a8c1a675288cdcacd09f3dd44986a9ba4ddce491012abebb3df875b6` |
| 2066 | 20.0 | `7bd5ee172db63ad57a72cb238bf605453bf9d0e681b2c2b233523f785bcad238` |
| 2076 | 0.01 | `4cc0fead1c7e9fb8f8f75a2cd8fac0c358f0e4175454994d409d1e4b6b0bd112` |
| 2121 | 0.02 | `613545d24b9f6b52c71516589c925ef1066ae248bde7b945d41781486fc6b7cd` |
| 2181 | 0.2 | `1c6219f1ac324bd79d7552daffd18c761c11c788501f4979c385c70f56a7a8de` |

13 crossings in 13 different blocks.

## End state

Ada (read by `ferry-core`): Ironwood 749.98 REG, Orchard 0.00925 REG.
Same result as runs 1 and 2, now produced by FERRY's own code.

## Found during this run

- Proving was 141 s per preparation in debug builds because dependencies kept their debug assertions (Halo 2 self-checks). Turning them off for dependencies brought it to 36 s.
- On regtest a wallet birthday of 1 makes the sync ask lightwalletd for the tree state at height 0, which it rejects. `FerryNetwork::lowest_birthday` raises it to 2 on regtest.
- The crossing gate is computed from the wallet's last synced height, which can lag the chain by a few blocks. Crossings still go out in separate sessions and blocks.

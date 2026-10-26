# Demo chain run 1: the released installers, on Intel macOS and Linux (2026-10-24 to 2026-10-26)

**Network:** the hosted FERRY demo chain (a regtest Zebra node behind `sync.ferry…` and `send.ferry…`), not mainnet or testnet.
**Coins:** fake.
**Builds:** the v0.1.0 release installers, unchanged, connecting to the demo servers over TLS.

Two migrations ran at the same time, each from its own demo wallet of 37.50 in Orchard, fetched with the app's **Get a demo wallet** button.

## Intel macOS build

`FERRY_0.1.0_x64.dmg`, run from the mounted DMG under Rosetta on an Apple silicon Mac, clicked through by a person: Get a demo wallet, Restore, Plan the crossing, Sign and start.

| Kind | Amount | Mined at | Transaction |
|---|---|---|---|
| preparation | | 27760 | `db2329cd70619dd8aaa4dda483755292a4083ff40e23d48513e52bbe06e7e643` |
| crossing | 20 | 27960 | `24cbfbd6bdab38d3035e721dea042c284f6db874ae006ad8936b01513bf60bbf` |
| crossing | 0.05 | 28009 | `a3ac55638b85dbf568647848c046f258d86ff5c83e99c341ef7d981722039eb2` |
| crossing | 0.02 | 28118 | `8953b69d72df30f978db8dbf0d0a76c43dd1ee4893bfeb7e2fffe3d711b0bb90` |
| crossing | 10 | 28180 | `106a82e764eb5f5303482826e54415ea62ae8acf9651f6397f160719866ea3a9` |
| crossing | 5 | 28353 | `c43c7413656314e015567f0141e356c77a046a4c308b4f26e57483bf67f0f4c4` |
| crossing | 0.2 | 29001 | `0951f2f3835c7fb75c8e3d26f080ec18e945df9721d2870297f666b19dde39e6` |
| crossing | 2 | 29139 | `a3cdce036ca08f6cfb84b9c2f73ee469832998f4bbafbaa681e5d285cdcc57d5` |
| crossing | 0.02 | 29371 | `440b6a54dbc56bcb8e36cf8b81bc8e3503a16081f123e526b2e059d8dd6f5f03` |
| crossing | 0.2 | 31831 | `c3de894346b936024d826df0fd51f5c4f26cb73988101a10610f2a3a5995112d` |

The last crossing came 2,460 blocks after the one before it: the app was closed overnight and caught up when reopened.

## Linux build

`FERRY_0.1.0_amd64.deb`, installed with `apt` in a clean Ubuntu 22.04 container (x86-64, emulated) and driven on a virtual display.
The screenshots in this folder are from that run: [launch](launch.png), [restored](restore.png), [plan](plan.png), [signed and scheduled](signed.png).

| Kind | Amount | Mined at | Transaction |
|---|---|---|---|
| preparation | | 27730 | `7b200bcdc139676e71cf211177810e9865e5809a57cbb622c942b105a550e9d4` |
| crossing | 0.2 | 28031 | `49b594be1f12c716532b8e758a1ef95d996bd8e30c114cd2091704bd9ee231f9` |
| crossing | 2 | 28087 | `fe1c3a9a68f26d62161278636c1df3ec7f22ed65a10604ddff898216bf20a3d2` |
| crossing | 0.02 | 28096 | `6b9838fe9519418571f00a5533d8a1b64b72050938286bf7317a4f82d24c3f04` |
| crossing | 0.2 | 28157 | `0f6bb79f0738db38df2710971019c557bb6c2fe93224c60fa8c0de6365193120` |
| crossing | 20 | 29002 | `e6761658ee727cd8847168e9839bc2136eae133d20706ebff19ecaf449031d7d` |
| crossing | 5 | 29173 | `70b6ae409b3dc4efaa1f507cc0f26b310cd0a08184792563f4340ec6cfb658ef` |
| crossing | 10 | 29307 | `467feb03f030eb2b47e529fdca54fdbc2ac10a661dc0b7b31105580b7e4ffde4` |
| crossing | 0.02 | 29352 | `b2f6262595125be8c392b64afdc988d60d0bf5169595fedf0364f19e1109ade7` |
| crossing | 0.05 | 29383 | `19ea801fed26999fa3869404a0c449e381e3b7085bfbfb63123ba2ec775ccb29` |

## What the chain shows

Both wallets moved 37.49 in the same nine standard amounts, each crossing in its own block.
Their crossings interleave: the Intel run's 0.2 at block 29001 sits next to the Linux run's 20 at block 29002.
Nothing on the chain says which crossing belongs to which wallet; that crowd is what ZIP 318 relies on.
On a demo chain the crowd is two wallets; on mainnet it is every wallet migrating at the same time.

All 20 transactions were confirmed with `getrawtransaction` on the demo node on 2026-10-26.

## Not covered

- The Linux build ran under x86-64 emulation in Docker, not on a Linux desktop.
- The Windows installer has not been run yet.

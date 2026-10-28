# Mainnet readiness (2026-10-28)

**Network:** Zcash mainnet, read only.
**Coins:** none moved.

FERRY 0.1.2 adds mainnet as an experimental network.
This folder records what was checked against the live chain on the day of the Zecathon deadline, and what was not.

## The servers run the consensus rules FERRY is built for

`cargo run -p ferry-core --example server_info -- main <servers>` asks each lightwalletd server for its chain and its consensus branch at the tip, and compares the branch with the one FERRY's librustzcash (rev `eb38d7a`) expects for that height on mainnet:

```text
zec.rocks:443: main tip 3533589 branch 37a5165b (matches FERRY's 37a5165b), ECC LightWalletD v0.5.3
zcash.mysideoftheweb.com:9067: main tip 3533589 branch 37a5165b (matches FERRY's 37a5165b), ECC LightWalletD v0.6.0
sync.ferry.162-62-231-147.sslip.io:443: test tip 50837 branch 37a5165b (DIFFERS FERRY's 0), ECC LightWalletD v0.0.0.0-dev
```

`37a5165b` is NU6.3's consensus branch.
The two mainnet servers, run by different operators, are the ones FERRY syncs from and broadcasts through on mainnet.
The third line is FERRY's demo chain: it is a regtest chain, so comparing it with mainnet's heights says "differs", but its own branch is the same `37a5165b`.
Every demo-chain transaction in [demo chain run 1](../demo-chain-run-1) was therefore checked by a node against NU6.3's rules, the rules mainnet enforces.

## A wallet restores and syncs on mainnet from the app

With the Linux build of 0.1.2 (`FERRY_0.1.2_amd64.deb` in a clean Ubuntu 22.04 container), the network switch in the side panel moves the app to mainnet in place ([screenshot](switch-to-mainnet.png)).
A newly generated, empty wallet was restored from its phrase with a recent birthday, and FERRY scanned it to the live tip, block 3,533,587 of 3,533,587 ([screenshot](restored-on-mainnet.png)).
It correctly reports nothing to move: the Orchard balance is below ZIP 318's 0.01 ZEC dust threshold.
Switching back to the demo chain happens in the same process, with the demo wallet form back in place.

The wallet's folder is created owner-only (`drwx------`).

## Not done

No migration has run on mainnet.
Since NU6.3, Orchard accepts no new value, so a wallet can only migrate Orchard funds it already held; a test wallet cannot be funded for it.
The first mainnet migration needs a wallet that still holds at least about 0.01 ZEC in Orchard.

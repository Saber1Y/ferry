# Harness: proving the migration loop on a real local node

Before the FERRY app exists, this harness drives a real ZIP 318 migration end to end on the local regtest chain with ECC's `zcash-devtool`, plus two small changes FERRY needs.

## The patch

`zcash-devtool-ferry.patch` applies to zcash-devtool commit `0b3436bb8d235c67cd36b800cb45a7b1b489c415`.

1. **New `migration broadcast` command.**
   devtool's `migration advance` stops at "proved and due to broadcast, but broadcasting isn't wired up in this command yet".
   The new command takes the due transaction with `PoolMigrations::take_transaction_for_broadcast`, sends it through a lightwalletd server, then records `mark_broadcast` on success or `report_broadcast_failure` on rejection.
2. **Preparation anchor fix in `migration advance`.**
   devtool anchored preparation transactions at the scanned tip, which fails with "a commitment tree has no root at the anchor checkpoint".
   The wallet does not checkpoint every block: not the block it scanned last, and not blocks without Orchard commitments inside a scanned batch.
   The patch anchors at the newest Orchard tree checkpoint at least 3 confirmations deep, matching ZIP 318's "ordinary near-tip anchor depth".

## Setup

```bash
mkdir -p .deps && cd .deps
git clone https://github.com/zcash/zcash-devtool.git
cd zcash-devtool
git checkout 0b3436bb8d235c67cd36b800cb45a7b1b489c415
git apply ../../harness/zcash-devtool-ferry.patch
cargo build --release --features regtest_support
```

Build lightwalletd v0.5.5 with Go into `.deps/bin/lightwalletd`, start Docker, then use `regtest/chain.sh` and `regtest/drive.sh`.

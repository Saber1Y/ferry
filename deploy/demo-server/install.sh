#!/usr/bin/env bash
# Install the FERRY demo chain on an Ubuntu x86_64 server, natively (no Docker):
#   ferry-zebrad     zebrad regtest, RPC on 127.0.0.1:18232, memory capped
#   ferry-lwd-sync   lightwalletd on 127.0.0.1:19067 (the app syncs here)
#   ferry-lwd-send   lightwalletd on 127.0.0.1:19077 (the app broadcasts here)
#   ferry-miner      a block every MINE_INTERVAL seconds (after funding)
#   ferry-wallets    hands out unused demo wallets on 127.0.0.1:19100
# Caddy terminates TLS in front of them (see Caddyfile.ferry).
#
# Run as root from the directory holding these files and lightwalletd-linux-amd64.
set -euo pipefail

ZEBRA_VERSION=7.0.0-rc.0
HERE="$(cd "$(dirname "$0")" && pwd)"
PREFIX=/opt/ferry
STATE=/var/lib/ferry

id ferry >/dev/null 2>&1 || useradd --system --home "$STATE" --shell /usr/sbin/nologin ferry
mkdir -p "$PREFIX/bin" "$STATE/zebrad" "$STATE/lwd-sync" "$STATE/lwd-send" /etc/ferry

# zebrad: the official release binary, verified against its published SHA-256.
if [ ! -x "$PREFIX/bin/zebrad" ]; then
  tmp=$(mktemp -d)
  base="https://github.com/ZcashFoundation/zebra/releases/download/v$ZEBRA_VERSION"
  asset="zebrad-$ZEBRA_VERSION-x86_64-unknown-linux-gnu.tar.gz"
  curl -fsSL -o "$tmp/$asset" "$base/$asset"
  curl -fsSL -o "$tmp/$asset.sha256" "$base/$asset.sha256"
  (cd "$tmp" && echo "$(cut -d' ' -f1 "$asset.sha256")  $asset" | sha256sum -c -)
  tar -xzf "$tmp/$asset" -C "$tmp"
  install -m 0755 "$(find "$tmp" -type f -name zebrad | head -1)" "$PREFIX/bin/zebrad"
  rm -rf "$tmp"
fi
install -m 0755 "$HERE/lightwalletd-linux-amd64" "$PREFIX/bin/lightwalletd"
install -m 0755 "$HERE/zebrad-config.sh" "$HERE/miner.sh" "$HERE/fund.sh" "$PREFIX/bin/"
install -m 0644 "$HERE/wallet-api.py" "$PREFIX/wallet-api.py"

cat > /etc/systemd/system/ferry-zebrad.service <<UNIT
[Unit]
Description=FERRY demo chain: zebrad (regtest)
After=network.target

[Service]
User=ferry
ExecStart=$PREFIX/bin/zebrad -c /etc/ferry/zebrad.toml start
Restart=on-failure
MemoryMax=450M
# zebrad writes recent blocks to disk every few seconds; give it time to stop cleanly.
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
UNIT

for role in sync:19067 send:19077; do
  name=${role%%:*}; port=${role##*:}
  cat > /etc/systemd/system/ferry-lwd-$name.service <<UNIT
[Unit]
Description=FERRY demo chain: lightwalletd ($name)
After=ferry-zebrad.service
Requires=ferry-zebrad.service

[Service]
User=ferry
ExecStart=$PREFIX/bin/lightwalletd --no-tls-very-insecure --grpc-bind-addr 127.0.0.1:$port --http-bind-addr 127.0.0.1:$((port + 1)) --rpchost 127.0.0.1 --rpcport 18232 --rpcuser x --rpcpassword x --data-dir $STATE/lwd-$name --log-file $STATE/lwd-$name/lightwalletd.log
Restart=on-failure
MemoryMax=150M

[Install]
WantedBy=multi-user.target
UNIT
done

cat > /etc/systemd/system/ferry-miner.service <<UNIT
[Unit]
Description=FERRY demo chain: block producer
After=ferry-zebrad.service
Requires=ferry-zebrad.service

[Service]
User=ferry
Environment=MINE_INTERVAL=10
ExecStart=$PREFIX/bin/miner.sh
Restart=always

[Install]
WantedBy=multi-user.target
UNIT

cat > /etc/systemd/system/ferry-wallets.service <<UNIT
[Unit]
Description=FERRY demo chain: demo wallet hand-out
After=network.target

[Service]
User=ferry
ExecStart=/usr/bin/python3 -I $PREFIX/wallet-api.py $STATE/demo-wallets.jsonl $STATE/demo-wallets-given.txt 127.0.0.1 19100
Restart=on-failure
MemoryMax=60M

[Install]
WantedBy=multi-user.target
UNIT

chown -R ferry:ferry "$STATE"
systemctl daemon-reload
echo "installed; next: fund.sh <wallets.jsonl>"

#!/usr/bin/env bash
# Fund every demo wallet with Orchard coinbase before NU6.3 (height 300), then mine past it.
# After NU6.3 no new value may enter Orchard, so this only works on a fresh chain.
# Run as root: fund.sh <wallets.jsonl> <burn address> [blocks per wallet]
set -euo pipefail
WALLETS="$1"; BURN="$2"; PER="${3:-6}"
rpc() { curl -s --max-time 120 -H 'content-type: application/json' \
  --data-binary "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":${2:-[]}}" http://127.0.0.1:18232/; }
height() { rpc getblockcount | sed -n 's/.*"result":\([0-9]*\).*/\1/p'; }
mine() { local n=$1; while [ "$n" -gt 0 ]; do s=$(( n > 20 ? 20 : n )); rpc generate "[$s]" > /dev/null; n=$(( n - s )); done; }
restart_mining_to() {
  # Let zebrad back up its newest blocks before stopping it (see regtest/chain.sh).
  sleep 6
  /opt/ferry/bin/zebrad-config.sh "$1"
  systemctl restart ferry-zebrad
  for _ in $(seq 1 60); do rpc getblockcount | grep -q '"result"' && return 0; sleep 1; done
  echo "zebrad did not come back" >&2; exit 1
}

systemctl stop ferry-miner ferry-lwd-sync ferry-lwd-send 2>/dev/null || true
restart_mining_to "$BURN"
[ "$(height)" -lt 2 ] && mine 2
while read -r line; do
  addr=$(echo "$line" | python3 -I -c 'import sys,json; print(json.load(sys.stdin)["address"])')
  restart_mining_to "$addr"
  mine "$PER"
done < "$WALLETS"
h=$(height)
[ "$h" -lt 300 ] || { echo "funding ran past NU6.3 ($h)" >&2; exit 1; }
restart_mining_to "$BURN"
target=$(( h + 101 > 330 ? h + 101 : 330 ))
mine $(( target - h ))
install -o ferry -g ferry -m 0600 "$WALLETS" /var/lib/ferry/demo-wallets.jsonl
systemctl start ferry-lwd-sync ferry-lwd-send ferry-miner ferry-wallets
echo "funded $(wc -l < "$WALLETS") wallets; chain at $(height); NU6.3 active since 300"

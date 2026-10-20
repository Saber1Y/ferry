#!/usr/bin/env bash
# Build the FERRY demo chain from scratch:
#   * a fresh regtest chain (NU6.3 at the height in heights.env)
#   * two wallets funded with Orchard coinbase BEFORE NU6.3, since afterwards no new value may
#     enter the Orchard pool (ZIP 258):
#       ada  - migrates with FERRY (ZIP 318 schedule)
#       bob  - migrates the naive way, his whole balance in one transaction (for the observer)
#   * mined past NU6.3 and coinbase maturity, both wallets synced
#
# Usage: setup-demo.sh            (DESTROYS the current regtest chain and wallets)
# Env: ADA_BLOCKS (default 120), BOB_BLOCKS (default 40)
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=heights.env
source "$HERE/heights.env"
DT="${DEVTOOL:-$HERE/../.deps/bin/zcash-devtool}"
CHAIN="$HERE/chain"
WALLETS="$CHAIN/wallets"
SYNC_SERVER=127.0.0.1:9067
ADA_BLOCKS="${ADA_BLOCKS:-120}"
BOB_BLOCKS="${BOB_BLOCKS:-40}"

quiet() { grep -v 'INFO' || true; }

# A regtest transparent address nobody holds a key for: starter blocks and post-NU6.3 mining go
# here, so no wallet's balance is affected by them.
burn_address() {
  python3 -I -c "
import hashlib, secrets
A = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
p = bytes([0x1D, 0x25]) + secrets.token_bytes(20)
c = p + hashlib.sha256(hashlib.sha256(p).digest()).digest()[:4]
n = int.from_bytes(c, 'big'); s = ''
while n:
    n, r = divmod(n, 58); s = A[r] + s
print('1' * (len(c) - len(c.lstrip(b'\0'))) + s)"
}

new_wallet() {
  local name=$1 w="$WALLETS/$1"
  mkdir -p "$w"
  "$DT" wallet -w "$w" init --name "$name" -i "$w/identity.txt" --birthday 2 \
    -n regtest --activation-heights "$HERE/activation-heights.toml" \
    -s "$SYNC_SERVER" --connection direct 2>&1 | quiet > /dev/null
  "$DT" wallet -w "$w" list-addresses 2>/dev/null | sed -n 's/.*Default Address: //p' > "$w/address.txt"
  [ -s "$w/address.txt" ] || { echo "could not create wallet $name" >&2; exit 1; }
  echo "$name: $(cut -c1-24 "$w/address.txt")..."
}

"$HERE/chain.sh" reset > /dev/null
BURN=$(burn_address)
mkdir -p "$CHAIN"
echo "$BURN" > "$CHAIN/burn-address.txt"

"$HERE/chain.sh" start "$BURN" > /dev/null
"$HERE/chain.sh" mine 2 > /dev/null
new_wallet ada
new_wallet bob

"$HERE/chain.sh" start "$(cat "$WALLETS/ada/address.txt")" > /dev/null
"$HERE/chain.sh" mine "$ADA_BLOCKS" > /dev/null
"$HERE/chain.sh" start "$(cat "$WALLETS/bob/address.txt")" > /dev/null
"$HERE/chain.sh" mine "$BOB_BLOCKS" > /dev/null

h=$("$HERE/chain.sh" height)
if [ "$h" -ge "$NU6_3_HEIGHT" ]; then
  echo "Orchard funding ran past NU6.3 ($h >= $NU6_3_HEIGHT); lower ADA_BLOCKS/BOB_BLOCKS" >&2
  exit 1
fi

# Past NU6.3, plus coinbase maturity (100 blocks) for the last funded block.
"$HERE/chain.sh" start "$BURN" > /dev/null
target=$(( NU6_3_HEIGHT + 30 ))
[ $(( h + 101 )) -gt "$target" ] && target=$(( h + 101 ))
"$HERE/chain.sh" mine $(( target - h )) > /dev/null

for name in ada bob; do
  "$DT" wallet -w "$WALLETS/$name" sync -s "$SYNC_SERVER" --connection direct > /dev/null 2>&1
  echo "$name: $("$DT" wallet -w "$WALLETS/$name" balance 2>/dev/null | grep -E 'Orchard Spendable' | tr -s ' ')"
done
echo "chain at height $("$HERE/chain.sh" height), NU6.3 active since $NU6_3_HEIGHT"

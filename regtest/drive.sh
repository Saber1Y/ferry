#!/usr/bin/env bash
# Drive a committed migration to completion on the local regtest chain.
# This is the prototype of FERRY's scheduler loop:
#   mine a few blocks -> sync from server A -> prove what is due -> broadcast through server B.
#
# Usage: drive.sh <wallet_dir> [max_height] [blocks_per_step]
# No `set -e`: a step that has nothing to do exits non-zero, and the loop must keep going.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
DT="${DEVTOOL:-$HERE/../.deps/zcash-devtool/target/release/zcash-devtool}"
W="${1:?usage: drive.sh <wallet_dir> [max_height] [blocks_per_step]}"
MAX_HEIGHT="${2:-1000}"
STEP="${3:-5}"
SYNC_SERVER=127.0.0.1:9067
BROADCAST_SERVER=127.0.0.1:9077

# devtool logs with ANSI colour codes; drop its INFO lines.
quiet() { grep -v 'INFO' || true; }

while :; do
  h=$("$HERE/chain.sh" height)
  "$DT" wallet -w "$W" sync -s "$SYNC_SERVER" --connection direct 2>&1 | grep -E 'Error|error' || true

  advance_out=$("$DT" migration -w "$W" advance 2>&1 | quiet)
  echo "$advance_out" | grep -E '^(Proved|Error)' | sed "s/^/[h=$h] advance: /" || true

  # Broadcast everything that is due, one at a time, through the broadcast server.
  while :; do
    b=$("$DT" migration -w "$W" broadcast -s "$BROADCAST_SERVER" --connection direct 2>&1 | quiet)
    if echo "$b" | grep -q '^Broadcast migration transaction'; then
      echo "[h=$h] $b"
    elif echo "$b" | grep -q 'Nothing is due'; then
      break
    else
      echo "[h=$h] broadcast: $b"
      break
    fi
  done

  status=$("$DT" migration -w "$W" status 2>&1 | quiet | head -1)
  if echo "$status" | grep -qiE 'status: (complete|completed|finished)'; then
    echo "[h=$h] $status"
    break
  fi
  if [ "$h" -ge "$MAX_HEIGHT" ]; then
    echo "[h=$h] reached max height; $status"
    break
  fi
  "$HERE/chain.sh" mine "$STEP" > /dev/null
done

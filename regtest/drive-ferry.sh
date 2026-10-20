#!/usr/bin/env bash
# Drive a migration with ferry-core's own scheduler (the code the desktop app runs), on the local
# regtest chain: commit once, then one `run_session` per few mined blocks until complete.
#
# Usage: drive-ferry.sh <ferry_wallet_dir> <phrase_file> [max_height] [blocks_per_step]
# The phrase file is only for this regtest harness: the app asks the user instead.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
MIGRATE="${MIGRATE:-$HERE/../target/debug/examples/migrate}"
W="${1:?usage: drive-ferry.sh <ferry_wallet_dir> <phrase_file> [max_height] [blocks_per_step]}"
PHRASE="${2:?phrase file}"
MAX_HEIGHT="${3:-6000}"
STEP="${4:-5}"

if [ -z "${NO_COMMIT:-}" ]; then
  echo "[h=$("$HERE/chain.sh" height)] commit: $("$MIGRATE" "$W" commit < "$PHRASE")"
fi

while :; do
  h=$("$HERE/chain.sh" height)
  out=$("$MIGRATE" "$W" session 2>&1)
  case "$out" in
    *'"kind":"synced","proved":[]'*) [ -n "${VERBOSE:-}" ] && echo "[h=$h] $out" ;;
    *'"kind":"complete"'*) echo "[h=$h] $out"; exit 0 ;;
    *'"kind":"followUpNeeded"'*)
      echo "[h=$h] $out"
      echo "[h=$h] commit: $("$MIGRATE" "$W" commit < "$PHRASE")"
      ;;
    *) echo "[h=$h] $out" ;;
  esac
  if [ "$h" -ge "$MAX_HEIGHT" ]; then
    echo "[h=$h] reached max height"
    exit 1
  fi
  "$HERE/chain.sh" mine "$STEP" > /dev/null
done

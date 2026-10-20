#!/usr/bin/env bash
# Mine one regtest block every N seconds, standing in for real block production while the FERRY
# app runs against the local chain. Stop with Ctrl-C.
#
# Usage: mine-every.sh [seconds] (default 3)
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
INTERVAL="${1:-3}"
while :; do
  "$HERE/chain.sh" mine 1 > /dev/null
  echo "height $("$HERE/chain.sh" height)"
  sleep "$INTERVAL"
done

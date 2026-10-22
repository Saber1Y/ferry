#!/usr/bin/env bash
# Produce one regtest block every MINE_INTERVAL seconds.
set -u
while :; do
  curl -s --max-time 60 -H 'content-type: application/json' \
    --data-binary '{"jsonrpc":"2.0","id":1,"method":"generate","params":[1]}' \
    http://127.0.0.1:18232/ > /dev/null || true
  sleep "${MINE_INTERVAL:-10}"
done

#!/usr/bin/env bash
# Drive a committed migration to completion on the local regtest chain.
# This is the prototype of FERRY's scheduler. Each loop iteration is one "session" (ZIP 318,
# "Background scheduling" and "Fallback on application open"):
#
#   * A session either broadcasts or synchronizes, never both: a sync tells the light wallet
#     server about the wallet, so it must not be correlatable with a broadcast.
#   * A session broadcasts at most one migration transaction: several pool-crossing transactions
#     sent together leak the balance.
#   * After each pool-crossing (transfer) broadcast, the next one waits a fresh delay drawn from
#     an exponential distribution (mean MEAN_DELAY blocks, redrawn above MAX_DELAY), so a backlog
#     of overdue transfers is spread out instead of sent in a burst. This shifts the remaining
#     schedule, as ZIP 318 recommends when a window is missed.
#
# Usage: [IDENTITY=<age identity file>] drive.sh <wallet_dir> [max_height] [blocks_per_step]
# With IDENTITY set, follow-up runs for deferred balance are planned and signed automatically.
# Env: MEAN_DELAY (default 66), MAX_DELAY (default 576), the ZIP 318 provisional values.
#
# No `set -e`: a step that has nothing to do exits non-zero, and the loop must keep going.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
DT="${DEVTOOL:-$HERE/../.deps/zcash-devtool/target/release/zcash-devtool}"
W="${1:?usage: drive.sh <wallet_dir> [max_height] [blocks_per_step]}"
MAX_HEIGHT="${2:-1000}"
STEP="${3:-5}"
MEAN_DELAY="${MEAN_DELAY:-66}"
MAX_DELAY="${MAX_DELAY:-576}"
SYNC_SERVER=127.0.0.1:9067
BROADCAST_SERVER=127.0.0.1:9077
GATE_FILE="$W/ferry-next-crossing-height"

# devtool logs with ANSI colour codes; drop its INFO lines.
quiet() { grep -v 'INFO' || true; }

# Exponential delay in whole blocks from a CSPRNG, redrawn while above MAX_DELAY.
draw_delay() {
  python3 -I -c "
import math, secrets
mean, cap = $MEAN_DELAY, $MAX_DELAY
while True:
    u = (secrets.randbits(53) + 1) / (2**53 + 1)
    d = round(-math.log(u) * mean)
    if d <= cap:
        print(d)
        break
"
}

next_crossing_height() { cat "$GATE_FILE" 2>/dev/null || echo 0; }

# One sync-and-prove session. Proving is local; only the sync talks to a server (A).
sync_session() {
  "$DT" wallet -w "$W" sync -s "$SYNC_SERVER" --connection direct 2>&1 | grep -E 'Error|error' || true
  "$DT" migration -w "$W" advance 2>&1 | quiet | grep -E '^(Proved|Error)' | sed "s/^/[h=$h] sync session: /" || true
}

while :; do
  h=$("$HERE/chain.sh" height)
  session=sync

  if [ "$h" -ge "$(next_crossing_height)" ]; then
    # The crossing gate is open: this session may broadcast one due transaction, and nothing else.
    b=$("$DT" migration -w "$W" broadcast -s "$BROADCAST_SERVER" --connection direct 2>&1 | quiet)
    if echo "$b" | grep -q '^Broadcast migration transfer'; then
      d=$(draw_delay)
      # At least one block, so two crossings can never share a block.
      [ "$d" -lt 1 ] && d=1
      echo $((h + d)) > "$GATE_FILE"
      echo "[h=$h] broadcast session: $b (next crossing no earlier than $((h + d)), delay $d)"
      session=broadcast
    elif echo "$b" | grep -q '^Broadcast migration preparation'; then
      echo "[h=$h] broadcast session: $b"
      session=broadcast
    elif echo "$b" | grep -q 'No migration in progress'; then
      # The engine plans one run at a time. When a balance leaves no slack for fees, the
      # smallest denominations are deferred and stay in Orchard. ZIP 318 only leaves dust below
      # 0.01 ZEC, so plan and commit follow-up runs until nothing migratable is left.
      plan=$("$DT" migration -w "$W" plan 2>&1 | quiet)
      if [ -n "${IDENTITY:-}" ] && echo "$plan" | grep -q 'Crossing values: \[Z'; then
        echo "[h=$h] run complete, follow-up run for the deferred balance: $(echo "$plan" | grep 'Crossing values')"
        "$DT" migration -w "$W" commit -i "$IDENTITY" 2>&1 | quiet | sed "s/^/[h=$h] /"
      else
        echo "[h=$h] migration complete"
        exit 0
      fi
    elif ! echo "$b" | grep -q 'Nothing is due'; then
      echo "[h=$h] broadcast session: $b"
    fi
  fi

  if [ "$session" = sync ]; then
    sync_session
  fi

  if [ "$h" -ge "$MAX_HEIGHT" ]; then
    echo "[h=$h] reached max height; $("$DT" migration -w "$W" status 2>&1 | quiet | head -1)"
    break
  fi
  "$HERE/chain.sh" mine "$STEP" > /dev/null
done

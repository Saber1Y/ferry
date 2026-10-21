#!/usr/bin/env python3
"""FERRY observer: what someone watching the chain learns about Orchard to Ironwood migrations.

Reads a node's JSON-RPC (zebrad) the way any outside observer can, with no keys. For every
transaction that moves value out of Orchard and into Ironwood it records the block and the
amount crossed (public by design: ZIP 209 tracks each pool's balance), then applies the
heuristics an observer would use to link crossings to one wallet:

  * non-canonical amount: ZIP 318 crossings are n x 10^k ZEC with n in {1, 2, 5}. Anything else
    (like 249.99) is almost certainly someone's whole balance moved at once.
  * same block: crossings mined together are likely from one wallet.
  * close in time: crossings a few blocks apart are suspicious.

Usage: python3 -I observer/observe.py [--rpc http://127.0.0.1:18232] [--from HEIGHT] [--to HEIGHT]
Standard library only.
"""

import argparse
import json
import urllib.request
from decimal import Decimal

CANONICAL_MULTIPLIERS = {1, 2, 5}
SMALLEST = Decimal("0.01")  # MAX_RESIDUAL_VALUE, the smallest ZIP 318 denomination
LARGEST = Decimal("10000")  # the largest ZIP 318 denomination
CLOSE_BLOCKS = 3  # crossings this close together are flagged as possibly linked


def rpc(url, method, params=None):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or []})
    req = urllib.request.Request(url, body.encode(), {"content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as resp:
        out = json.loads(resp.read(), parse_float=Decimal)
    if out.get("error"):
        raise RuntimeError(f"{method}: {out['error']}")
    return out["result"]


def is_canonical(amount: Decimal) -> bool:
    """Whether `amount` is a ZIP 318 denomination: n x 10^k ZEC, n in {1, 2, 5}."""
    if amount < SMALLEST or amount > LARGEST:
        return False
    k = SMALLEST
    while k <= LARGEST:
        if amount in {Decimal(n) * k for n in CANONICAL_MULTIPLIERS}:
            return True
        k *= 10
    return False


def crossings(url, start, end):
    """Every Orchard to Ironwood crossing in [start, end]: (height, txid, amount into Ironwood)."""
    found = []
    for height in range(start, end + 1):
        block = rpc(url, "getblock", [str(height), 2])
        for tx in block["tx"]:
            orchard = (tx.get("orchard") or {}).get("valueBalance", Decimal(0))
            ironwood = (tx.get("ironwood") or {}).get("valueBalance", Decimal(0))
            # Value leaving Orchard is a positive Orchard value balance; value entering Ironwood is
            # a negative Ironwood value balance.
            if orchard > 0 and ironwood < 0:
                found.append((height, tx["txid"], -ironwood))
    return found


def analyse(found):
    by_block = {}
    for height, txid, amount in found:
        by_block.setdefault(height, []).append(txid)
    rows = []
    previous_height = None
    for height, txid, amount in found:
        flags = []
        if not is_canonical(amount):
            flags.append("odd amount: likely a whole balance")
        if len(by_block[height]) > 1:
            flags.append(f"shares block with {len(by_block[height]) - 1} other crossing(s)")
        if previous_height is not None and 0 < height - previous_height <= CLOSE_BLOCKS:
            flags.append(f"{height - previous_height} block(s) after the previous crossing")
        rows.append((height, txid, amount, flags))
        previous_height = height
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--rpc", default="http://127.0.0.1:18232")
    ap.add_argument("--from", dest="start", type=int, help="first height (default: NU6.3 activation)")
    ap.add_argument("--to", dest="end", type=int, help="last height (default: chain tip)")
    ap.add_argument("--json", action="store_true", help="print machine-readable output")
    args = ap.parse_args()

    info = rpc(args.rpc, "getblockchaininfo")
    nu63 = next(
        (u["activationheight"] for u in info["upgrades"].values() if u.get("name") == "NU6.3"),
        None,
    )
    start = args.start or nu63 or 1
    end = args.end or info["blocks"]

    rows = analyse(crossings(args.rpc, start, end))

    if args.json:
        print(json.dumps(
            [{"height": h, "txid": t, "amount": str(a), "flags": f} for h, t, a, f in rows],
            indent=2,
        ))
        return

    print(f"Orchard to Ironwood crossings, blocks {start} to {end}: {len(rows)}\n")
    print(f"{'block':>7}  {'ZEC into Ironwood':>18}  txid              observer's view")
    for height, txid, amount, flags in rows:
        view = "; ".join(flags) if flags else "standard amount, nothing links it"
        print(f"{height:>7}  {amount:>18}  {txid[:16]}  {view}")

    exposed = [r for r in rows if any(f.startswith("odd amount") for f in r[3])]
    linkable = [r for r in rows if r[3] and r not in exposed]
    quiet = [r for r in rows if not r[3]]
    print()
    print(f"Exposed (odd amount, a balance in plain sight): {len(exposed)}")
    print(f"Linkable by timing or shared blocks:            {len(linkable)}")
    print(f"Standard and unlinked:                          {len(quiet)}")
    print()
    print("Note: these heuristics read each crossing's shape. Standard amounts only hide a wallet")
    print("among the other wallets migrating at the same time, which a local regtest chain with one")
    print("or two wallets does not have. On mainnet that crowd is every ZIP 318 wallet.")


if __name__ == "__main__":
    main()

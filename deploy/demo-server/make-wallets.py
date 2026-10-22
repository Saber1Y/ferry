#!/usr/bin/env python3
"""Generate demo wallets for the hosted FERRY demo chain (regtest, fake coins).

Writes one JSON object per line to OUT: {"phrase": ..., "address": ...}. The phrases are random
24-word BIP-39 phrases; addresses are each wallet's default regtest unified address, derived
offline by zcash-devtool. The output is git-ignored.

Usage: python3 -I make-wallets.py <count> <bip39 english wordlist> <zcash-devtool> <out file>
"""

import hashlib
import json
import secrets
import subprocess
import sys


def phrase(words):
    entropy = secrets.token_bytes(32)  # 256 bits -> 24 words
    checksum = hashlib.sha256(entropy).digest()[0]  # first 8 bits
    bits = int.from_bytes(entropy, "big") << 8 | checksum
    return " ".join(words[(bits >> (11 * i)) & 0x7FF] for i in reversed(range(24)))


def main():
    count, wordlist, devtool, out = int(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4]
    words = open(wordlist).read().split()
    assert len(words) == 2048
    with open(out, "w") as f:
        for _ in range(count):
            p = phrase(words)
            derived = subprocess.run(
                [devtool, "wallet", "derive-address", "--mnemonic", p, "--network", "regtest"],
                capture_output=True, text=True, check=True,
            ).stdout
            address = next(
                line.split(":", 1)[1].strip()
                for line in derived.splitlines()
                if line.startswith("Unified Address:")
            )
            f.write(json.dumps({"phrase": p, "address": address}) + "\n")
    print(f"wrote {count} wallets to {out}")


if __name__ == "__main__":
    main()

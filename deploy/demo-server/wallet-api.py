#!/usr/bin/env python3
"""Hand out unused demo wallets for the hosted FERRY demo chain (fake regtest coins).

GET /demo-wallet  -> {"phrase": ..., "birthday": 2, "remaining": N}  (each wallet once)
GET /health       -> {"remaining": N}

Usage: python3 -I wallet-api.py <wallets.jsonl> <given.txt> <host> <port>
"""

import json
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

WALLETS, GIVEN, HOST, PORT = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
LOCK = threading.Lock()


def load():
    wallets = [json.loads(l) for l in open(WALLETS) if l.strip()]
    try:
        given = set(open(GIVEN).read().split())
    except FileNotFoundError:
        given = set()
    return wallets, given


class Handler(BaseHTTPRequestHandler):
    def send(self, code, body):
        data = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("content-type", "application/json")
        self.send_header("access-control-allow-origin", "*")
        self.send_header("cache-control", "no-store")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        with LOCK:
            wallets, given = load()
            free = [w for w in wallets if w["address"] not in given]
            if self.path == "/health":
                return self.send(200, {"remaining": len(free)})
            if self.path != "/demo-wallet":
                return self.send(404, {"error": "not found"})
            if not free:
                return self.send(410, {"error": "all demo wallets have been handed out"})
            wallet = free[0]
            with open(GIVEN, "a") as f:
                f.write(wallet["address"] + "\n")
            return self.send(200, {"phrase": wallet["phrase"], "birthday": 2, "remaining": len(free) - 1})

    def log_message(self, *args):
        pass  # no request logging: nothing about visitors is kept


ThreadingHTTPServer((HOST, PORT), Handler).serve_forever()

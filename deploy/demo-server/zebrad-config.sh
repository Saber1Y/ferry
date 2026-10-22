#!/usr/bin/env bash
# Write /etc/ferry/zebrad.toml mining to $1. Upgrade heights match regtest/heights.env.
set -euo pipefail
cat > /etc/ferry/zebrad.toml <<TOML
[mining]
miner_address = "$1"

[network]
network = "Regtest"
listen_addr = "127.0.0.1:18233"
cache_dir = false

[network.testnet_parameters.activation_heights]
Overwinter = 1
Sapling = 1
Blossom = 1
Heartwood = 1
Canopy = 1
NU5 = 1
NU6 = 1
"NU6.1" = 1
"NU6.2" = 1
"NU6.3" = 300

[rpc]
listen_addr = "127.0.0.1:18232"
enable_cookie_auth = false

[state]
cache_dir = "/var/lib/ferry/zebrad"

[tracing]
use_color = false
TOML

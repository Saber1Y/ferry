# FERRY desktop app

Tauri 2 shell (`src-tauri/`) over `ferry-core`, with Vite + React + TypeScript screens (`src/`).

## Run against the local regtest chain

```bash
./regtest/setup-demo.sh
cd app && npm install && npm run tauri dev
```

The hackathon build targets regtest only: sync server `127.0.0.1:9067`, broadcast server `127.0.0.1:9077`, NU6.3 at 300 (see `regtest/heights.env`).
Set `FERRY_WALLET_DIR` to point the app at an existing wallet folder; otherwise it uses the OS app-data folder.

## What works in this build

- Restore a wallet from a recovery phrase and birthday (seed mode), then sync.
- Home screen: Orchard balance still to move, Ironwood balance, sync status.

Planning, signing and the scheduler are not in the app yet (see `../TODO.md`).

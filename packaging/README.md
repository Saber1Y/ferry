# Packaging

FERRY's installers for macOS, Windows and Linux are all built on one Apple silicon Mac, without CI:

```bash
packaging/build.sh macos-arm
packaging/build.sh macos-intel
XWIN_ACCEPT_LICENSE=1 packaging/build.sh windows
packaging/build.sh linux
```

Each one copies its installer to `.deps/dist/`.
The GitHub Actions workflow in `.github/workflows/release.yml` builds the same installers on hosted runners when Actions is available.

## One-time setup

| Target | Needs |
|---|---|
| `macos-arm` | Rust, Node 24, `npm ci` in `app/` |
| `macos-intel` | `rustup target add x86_64-apple-darwin` |
| `windows` | `rustup target add x86_64-pc-windows-msvc`, `brew install llvm`, `cargo install --locked cargo-xwin`, Docker |
| `linux` | Docker that can run `linux/amd64` containers |

## Windows

`cargo-xwin` cross-compiles with LLVM against Microsoft's CRT and Windows SDK, which it downloads on first use.
Those are under Microsoft's license, so the build only runs when `XWIN_ACCEPT_LICENSE=1` says you accept it.

The installer is made by NSIS.
`packaging/bin/makensis` stands in for it and runs Debian 13's NSIS 3.11 in Docker, because Homebrew's makensis 3.13 crashes on Tauri's installer script and Ubuntu 22.04's NSIS 3.08 is too old for it.

The app links the Visual C++ runtime statically, so it needs only DLLs that ship with Windows 10 and 11.

## Linux

`packaging/linux-in-docker.sh` builds in an x86-64 Ubuntu 22.04 container, so the `.deb` depends on the libraries of Ubuntu 22.04 and later.
Docker volumes (`ferry-linux-build`, `ferry-linux-home`, `ferry-linux-apt`) keep the toolchain and build cache, so a second build only recompiles what changed.
The first build compiles everything under emulation and takes about an hour.

## Signing

None of the installers are signed.
macOS asks the user to confirm the first launch, and Windows shows SmartScreen.

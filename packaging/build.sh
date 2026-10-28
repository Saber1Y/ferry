#!/usr/bin/env bash
# Builds FERRY's installers on one Apple silicon Mac, without CI.
#
# Usage:
#   packaging/build.sh macos-arm      FERRY_<v>_aarch64.dmg   (native)
#   packaging/build.sh macos-intel    FERRY_<v>_x64.dmg       (cross-compiled; needs the x86_64-apple-darwin target)
#   packaging/build.sh windows        FERRY_<v>_x64-setup.exe (cargo-xwin + LLVM; NSIS runs in Docker)
#   packaging/build.sh linux          FERRY_<v>_amd64.deb and .AppImage (x86-64 Ubuntu 22.04 in Docker)
#   packaging/build.sh all
#
# Installers are copied to .deps/dist/. See packaging/README.md for the one-time setup of each target.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
DIST="$ROOT/.deps/dist"
mkdir -p "$DIST"

macos() {
  local target=$1
  (cd "$ROOT/app" && npm run tauri build -- --target "$target" --bundles dmg)
  cp "$ROOT/target/$target/release/bundle/dmg/"*.dmg "$DIST/"
}

windows() {
  if [ "${XWIN_ACCEPT_LICENSE:-}" != "1" ]; then
    echo "The Windows build downloads Microsoft's CRT and Windows SDK, which are under Microsoft's license." >&2
    echo "Read it, then run again with XWIN_ACCEPT_LICENSE=1 to accept it." >&2
    exit 1
  fi
  local llvm
  llvm="$(brew --prefix llvm)/bin"
  [ -x "$llvm/clang-cl" ] || { echo "LLVM not found: brew install llvm" >&2; exit 1; }
  command -v cargo-xwin > /dev/null || { echo "cargo-xwin not found: cargo install --locked cargo-xwin" >&2; exit 1; }
  # Our makensis shim first on PATH: Homebrew's makensis crashes on Tauri's installer script.
  (cd "$ROOT/app" && PATH="$HERE/bin:$llvm:$PATH" \
    npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis)
  cp "$ROOT/target/x86_64-pc-windows-msvc/release/bundle/nsis/"*-setup.exe "$DIST/"
}

linux() {
  # The volumes keep the toolchain, downloads and build cache between runs; the host's cargo
  # registry is shared so crates are not downloaded twice.
  docker run --rm --platform linux/amd64 \
    -v "$ROOT:/src:ro" \
    -v "$HERE/linux-in-docker.sh:/linux-in-docker.sh:ro" \
    -v "$DIST:/out" \
    -v ferry-linux-build:/build \
    -v ferry-linux-home:/root \
    -v ferry-linux-apt:/var/cache/apt/archives \
    -v "$HOME/.cargo/registry:/root/.cargo/registry" \
    -v "$HOME/.cargo/git:/root/.cargo/git" \
    ubuntu:22.04 bash /linux-in-docker.sh
}

case "${1:-}" in
  macos-arm) macos aarch64-apple-darwin ;;
  macos-intel) macos x86_64-apple-darwin ;;
  windows) windows ;;
  linux) linux ;;
  all) macos aarch64-apple-darwin; macos x86_64-apple-darwin; windows; linux ;;
  *) sed -n '2,11p' "$0"; exit 1 ;;
esac
ls -la "$DIST"

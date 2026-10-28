#!/usr/bin/env bash
# Runs inside ubuntu:22.04 (linux/amd64), started by `packaging/build.sh linux`.
# /src is the repo (read-only), /out receives the installers.
# Every download retries: builds have run on networks that drop out for minutes at a time.
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
retry() { for i in $(seq 1 8); do "$@" && return 0; echo "retry $i: $*" >&2; sleep 15; done; return 1; }

# Keep downloaded .debs in the apt volume between runs (the image deletes them by default).
rm -f /etc/apt/apt.conf.d/docker-clean
echo 'Acquire::Retries "10";' > /etc/apt/apt.conf.d/80-retries
PACKAGES="curl ca-certificates build-essential pkg-config git file rsync libwebkit2gtk-4.1-dev
  libappindicator3-dev librsvg2-dev patchelf xdg-utils libssl-dev"
if ! dpkg -s $PACKAGES > /dev/null 2>&1; then
  retry sh -c 'apt-get update -qq && apt-get install -y -qq '"$(echo $PACKAGES)"' > /dev/null'
fi

RUST=1.96.0
if [ ! -x "$HOME/.cargo/bin/rustup" ]; then
  retry sh -c "curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain $RUST"
fi
. "$HOME/.cargo/env"

# Node from nodejs.org, kept in the home volume: apt's fallback would be Ubuntu's Node 12.
NODE=node-v24.21.0-linux-x64
if [ ! -x "$HOME/$NODE/bin/node" ]; then
  retry curl -fsSL -C - -o "$HOME/$NODE.tar.xz" "https://nodejs.org/dist/v24.21.0/$NODE.tar.xz"
  tar -xJf "$HOME/$NODE.tar.xz" -C "$HOME"
  rm "$HOME/$NODE.tar.xz"
fi
export PATH="$HOME/$NODE/bin:$PATH"

rsync -a --delete --exclude target --exclude node_modules --exclude .deps --exclude regtest/chain \
  --exclude deploy/demo-server/out --exclude site/.vercel --exclude 'site/.env*' /src/ /build/
cd /build
export CARGO_NET_RETRY=10
retry cargo fetch
cd /build/app
retry npm ci --no-audit --no-fund

npm run tauri build -- --bundles deb
cp /build/target/release/bundle/deb/*.deb /out/

# AppImage tools are AppImages themselves. Containers have no FUSE, so they unpack themselves instead.
export APPIMAGE_EXTRACT_AND_RUN=1
# Under x86-64 emulation the kernel only recognises a program when bytes 8-10 of its header are zero,
# but AppImages put their "AI\2" marker there. Tauri clears it on linuxdeploy, not on this plugin,
# which otherwise fails with "Exec format error". The plugin is downloaded during the first attempt.
clear_appimage_marker() {
  local plugin="$HOME/.cache/tauri/linuxdeploy-plugin-appimage.AppImage"
  [ -f "$plugin" ] && printf '\0\0\0' | dd of="$plugin" bs=1 seek=8 conv=notrunc status=none
  return 0
}
appimage() { clear_appimage_marker && npm run tauri build -- --bundles appimage; }
retry appimage
cp /build/target/release/bundle/appimage/*.AppImage /out/

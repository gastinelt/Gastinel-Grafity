#!/usr/bin/env bash
# Vérifications CI headless (Phases 2 & 3) — à exécuter dans Dockerfile.ci (Alpine).
set -euo pipefail

echo "== [1/5] Toolchain"
rustc --version
cargo --version

echo "== [2/5] Vulkan Lavapipe"
vulkaninfo --summary 2>/dev/null | tee /tmp/vulkaninfo.txt | grep -E "deviceName|driverName" || true
grep -qi "llvmpipe" /tmp/vulkaninfo.txt || { echo "ERREUR : llvmpipe introuvable"; exit 1; }

echo "== [3/5] Tests (hors-ligne)"
cargo test --offline --workspace

echo "== [4/5] Build release musl dynamique"
cargo build --offline --release --bin gastinel-desktop
BIN="${CARGO_TARGET_DIR:-target}/release/gastinel-desktop"
file "$BIN"
readelf -d "$BIN" | grep NEEDED
readelf -l "$BIN" | grep -q "ld-musl-x86_64.so.1" \
    || { echo "ERREUR : le binaire n'utilise pas l'interpréteur musl dynamique"; exit 1; }
if readelf -d "$BIN" | grep -q "libc.so.6"; then
    echo "ERREUR : dépendance glibc détectée"; exit 1
fi
echo "OK : binaire musl lié dynamiquement (dlopen libvulkan possible)"

echo "== [5/5] Weston headless"
mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"
weston --backend=headless --socket=gastinel-ci >/tmp/weston.log 2>&1 &
WPID=$!
for _ in $(seq 1 50); do [ -S "$XDG_RUNTIME_DIR/gastinel-ci" ] && break; sleep 0.1; done
[ -S "$XDG_RUNTIME_DIR/gastinel-ci" ] || { cat /tmp/weston.log; echo "ERREUR : weston headless"; exit 1; }
kill "$WPID"; wait "$WPID" 2>/dev/null || true
echo "OK : weston headless démarre"

echo "== CI : tout est vert"

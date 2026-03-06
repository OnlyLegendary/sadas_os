#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_DIR="$ROOT_DIR/system/state"

mkdir -p "$STATE_DIR"

echo "==> Building Rust workspace"
cargo build --workspace

echo "==> Building Sadas UEFI ESP layout"
if ! cargo run -p sadas-qemu-runner -- --uefi --build-only; then
  echo "==> UEFI build prerequisites missing; falling back to legacy boot image"
  cargo run -p sadas-qemu-runner -- --legacy --build-only
fi

echo "==> Seeding state defaults"
cat > "$STATE_DIR/permissions.db" <<'PERMS'
# app_id network mic camera files
* deny deny deny prompt
PERMS

cat > "$STATE_DIR/apps.db" <<'APPS'
# app_id install_source
org.sadas.shell base
org.sadas.settings base
APPS

cat > "$STATE_DIR/updates.db" <<'UPDATES'
slot=A
rollback_available=false
UPDATES

echo "Build complete. Prefer UEFI ESP at $ROOT_DIR/target/esp; legacy image at $ROOT_DIR/target/sadas_boot.img"

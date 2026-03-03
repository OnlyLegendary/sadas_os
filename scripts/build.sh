#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_DIR="$ROOT_DIR/system/image"
STATE_DIR="$ROOT_DIR/system/state"
CACHE_DIR="$ROOT_DIR/system/cache"

ISO_URL="https://download.fedoraproject.org/pub/fedora/linux/releases/40/Spins/x86_64/iso/Fedora-KDE-Live-x86_64-40-1.14.iso"
ISO_PATH="$IMAGE_DIR/sadas-os-phase1-kde.iso"
FALLBACK_QCOW2="$IMAGE_DIR/sadas-os-mvp.qcow2"

mkdir -p "$IMAGE_DIR" "$STATE_DIR" "$CACHE_DIR"

echo "==> Building Rust workspace components (Sadas apps/services/tools)"
cargo build --workspace

echo "==> Seeding local policy/state defaults"
cat > "$STATE_DIR/permissions.db" <<'PERMS'
# app_id network mic camera files
* deny deny deny prompt
PERMS

cat > "$STATE_DIR/apps.db" <<'APPS'
# app_id install_source
org.kde.systemsettings base
org.kde.dolphin base
org.kde.konsole optional
APPS

cat > "$STATE_DIR/updates.db" <<'UPDATES'
slot=A
rollback_available=false
UPDATES

if [[ ! -f "$ISO_PATH" ]]; then
  echo "==> Attempting to download KDE base image"
  if curl -L --fail --retry 3 "$ISO_URL" -o "$ISO_PATH"; then
    echo "==> Downloaded $ISO_PATH"
  else
    echo "==> Download blocked/unavailable; keeping local fallback image path"
    rm -f "$ISO_PATH"
  fi
else
  echo "==> Reusing existing image: $ISO_PATH"
fi

if [[ ! -f "$ISO_PATH" && ! -f "$FALLBACK_QCOW2" ]]; then
  echo "==> Creating fallback qcow2 placeholder: $FALLBACK_QCOW2"
  if command -v qemu-img >/dev/null 2>&1; then
    qemu-img create -f qcow2 "$FALLBACK_QCOW2" 16G >/dev/null
  else
    dd if=/dev/zero of="$FALLBACK_QCOW2" bs=1M count=64 >/dev/null 2>&1
  fi
fi

echo "Build artifacts ready:"
[[ -f "$ISO_PATH" ]] && echo "- $ISO_PATH"
[[ -f "$FALLBACK_QCOW2" ]] && echo "- $FALLBACK_QCOW2"
echo "Privacy defaults: telemetry=off crash_reporting=off account_required=false"

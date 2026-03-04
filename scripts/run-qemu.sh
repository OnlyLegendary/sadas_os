#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ISO_PATH="$ROOT_DIR/system/image/sadas-os-phase1-kde.iso"
FALLBACK_QCOW2="$ROOT_DIR/system/image/sadas-os-mvp.qcow2"
OVMF_CODE="${OVMF_CODE:-/usr/share/OVMF/OVMF_CODE.fd}"
OVMF_VARS_TEMPLATE="${OVMF_VARS_TEMPLATE:-/usr/share/OVMF/OVMF_VARS.fd}"
OVMF_VARS_RUNTIME="$ROOT_DIR/system/cache/OVMF_VARS.fd"

mkdir -p "$ROOT_DIR/system/cache"

if [[ ! -f "$ISO_PATH" && ! -f "$FALLBACK_QCOW2" ]]; then
  echo "No boot artifact found. Run ./scripts/build.sh first."
  exit 1
fi

if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
  echo "qemu-system-x86_64 not found on PATH."
  echo "Install QEMU and rerun ./scripts/run-qemu.sh"
  exit 0
fi

QEMU_ACCEL="tcg"
if [[ -e /dev/kvm ]]; then
  QEMU_ACCEL="kvm:tcg"
fi

QEMU_COMMON=(
  -machine "q35,accel=$QEMU_ACCEL"
  -m 4096
  -smp 4
  -device virtio-net-pci,netdev=net0
  -netdev user,id=net0
)

if [[ -f "$ISO_PATH" ]]; then
  if [[ -f "$OVMF_CODE" && -f "$OVMF_VARS_TEMPLATE" ]]; then
    cp "$OVMF_VARS_TEMPLATE" "$OVMF_VARS_RUNTIME"
    echo "Launching KDE ISO in UEFI mode"
    exec qemu-system-x86_64 \
      "${QEMU_COMMON[@]}" \
      -device virtio-vga-gl \
      -display gtk,gl=on \
      -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
      -drive if=pflash,format=raw,file="$OVMF_VARS_RUNTIME" \
      -cdrom "$ISO_PATH" \
      -boot d
  fi

  echo "Launching KDE ISO in BIOS fallback mode"
  exec qemu-system-x86_64 \
    "${QEMU_COMMON[@]}" \
    -device virtio-vga \
    -display gtk \
    -cdrom "$ISO_PATH" \
    -boot d
fi

echo "Launching fallback qcow2 artifact"
exec qemu-system-x86_64 \
  "${QEMU_COMMON[@]}" \
  -drive file="$FALLBACK_QCOW2",format=qcow2

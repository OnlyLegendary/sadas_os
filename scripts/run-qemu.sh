#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Launching Sadas UEFI boot in QEMU"
if ! cargo run -p sadas-qemu-runner -- --uefi --run; then
  echo "==> UEFI run failed; falling back to legacy boot"
  cargo run -p sadas-qemu-runner -- --legacy --run
fi

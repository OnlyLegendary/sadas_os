#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="$ROOT_DIR/target/sadas_boot.img"

if [[ ! -f "$IMAGE" ]]; then
  echo "Boot image not found. Run ./scripts/build.sh first."
  exit 1
fi

echo "==> Launching Sadas boot image in QEMU"
cargo run -p sadas-qemu-runner -- --run

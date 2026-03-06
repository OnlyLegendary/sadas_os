# Booting Sadas OS (Prompt 1 baseline)

This phase provides a standalone x86_64 boot artifact and runner flow.

## What gets built

- `target/sadas_boot.img` (raw boot sector image)
- kernel boot log transcript emitted over text output in QEMU

## Build

```bash
./scripts/build.sh
```

Equivalent direct command:

```bash
cargo run -p sadas-qemu-runner -- --build-only
```

## Run in QEMU

```bash
./scripts/run-qemu.sh
```

Equivalent direct command:

```bash
cargo run -p sadas-qemu-runner -- --run
```

Expected serial output includes:
- `[sadas][INFO] sadas: hello from kernel`
- scheduler tick logs
- `phase1 scheduler loop entered`

## Real hardware status

Real hardware UEFI boot is **not complete yet** in this baseline. The next prompts add:
- UEFI loader artifact (`BOOTX64.EFI`)
- USB image generation
- installer path for ESP + kernel/initfs

## Troubleshooting

- `QEMU not found...`
  - Install QEMU and ensure `qemu-system-x86_64` is on PATH.
  - Or set `QEMU_SYSTEM_X86_64=/full/path/to/qemu-system-x86_64(.exe)`.

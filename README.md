# Sadas OS

Sadas OS is a standalone Rust OS effort focused on privacy-first defaults and a Windows-friendly UX.

Current implementation target (Prompt 1 baseline):
- x86_64 boot path in QEMU
- serial/text boot logs from kernel
- no-std kernel + scheduler skeleton
- deterministic build/run scripts

## Repository layout

- `crates/` kernel, boot, logging, sysapi and core runtime crates
- `tools/qemu-runner/` boot image generator + QEMU launcher
- `scripts/` build/run entrypoints
- `system/` config/state defaults and install assets
- `docs/` boot and roadmap documentation

## Quick start

```bash
./scripts/build.sh
./scripts/run-qemu.sh
```

## Required checks

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

## Notes

- `scripts/build.sh` now produces `target/sadas_boot.img` via `sadas-qemu-runner --build-only`.
- `scripts/run-qemu.sh` launches the same artifact through `sadas-qemu-runner --run`.
- If QEMU is not installed, the runner exits cleanly with guidance.

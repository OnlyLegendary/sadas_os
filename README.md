# Sadas OS

Sadas OS is a from-scratch Rust operating system project designed to become a complete, polished desktop platform while staying privacy-first.

## Product goals

- **Privacy by default** with capability controls, strict policy levels, and secure IPC channels.
- **Familiarity** through a complete shell/compositor/service stack and predictable workflows.
- **Customization** with sleek UI themes, frosted glass mode, and obsidian glass mode.
- **Runs well on old and new hardware** using runtime tiers, latency-aware scheduling, and frame-budget-aware UX profiles.

## Current operational stack

- `crates/boot`: hardened boot chain model (secure/measured boot and fallback slot) plus a QEMU boot-stub transcript model.
- `crates/vm`: VM profile and canonical user/kernel address layout definitions.
- `crates/drivers`: broad driver matrix (storage, network, input, media, graphics, sensors, power, printer).
- `crates/installer`: secure installer profile and stage pipeline.
- `crates/kernel`: microkernel core, priority scheduler, latency hints, capability audit log, secure IPC checks.
- `crates/platform`: low-level primitives such as entropy.
- `crates/logging`: reusable no_std logging core with global backend and serial COM1 backend.
- `crates/sysapi`: syscall ABI and shared product enums for privacy/theme/window control.
- `crates/services`: policy model for vault, broker, compositor, shell, updater, compatibility layer.
- `crates/ui`: UI profile tuning, frosted/obsidian glass palettes, and desktop layout helpers.
- `crates/init`: startup orchestration for boot, VM, installer, runtime budgets, UI, services, and driver matrix.
- `tools/sadasctl`: planning tool with boot/VM/driver/installer inspection commands.

## Quick checks

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo run -p sadas-init
cargo run -p sadasctl -- feature-matrix
cargo run -p sadasctl -- boot-stub-demo
cargo run -p sadasctl -- demo-shell
```

## What is still needed for a true shipping OS

1. Real bootloader executable and firmware integration.
2. Working page tables, process isolation, and memory reclaim.
3. Real hardware drivers and userspace daemon interfaces.
4. GUI installer frontend, recovery image, and signed OTA pipeline.


## Booting in QEMU

A minimal BIOS boot artifact is available and prints `sadas: hello from kernel`.

```bash
make run
```

See full cross-platform instructions in `docs/boot.md`.

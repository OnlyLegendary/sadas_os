# Sadas OS

Sadas OS is a from-scratch Rust operating system project designed to compete at product level with mainstream desktop systems while staying privacy-first.

## Product goals

- **Privacy by default** with capability controls, strict policy levels, and secure IPC channels.
- **Familiarity** through a complete shell/compositor/service stack and predictable workflows.
- **Customization** with theme modes, density, animation levels, and adaptive rendering.
- **Runs well on old and new hardware** using runtime tiers and frame-budget-aware UX profiles.

## Current full-stack components

- `crates/kernel`: microkernel core, priority scheduler, capability audit log, secure IPC checks.
- `crates/platform`: low-level primitives such as entropy.
- `crates/sysapi`: syscall ABI and shared product-level enums.
- `crates/services`: policy model for vault, broker, compositor, shell, updater, compatibility layer.
- `crates/ui`: UI profile tuning and render-budget helpers.
- `crates/init`: boot-time orchestration of kernel, services, and user experience defaults.
- `tools/sadasctl`: system planning and competitive feature inspection.

## Quick checks

```bash
cargo check --workspace
cargo test -p sadas-kernel
cargo run -p sadas-init
cargo run -p sadasctl -- feature-matrix
```

## Delivery path to production

1. UEFI boot + memory manager + process isolation completion.
2. Filesystem, driver stack, and hardware acceleration.
3. Native app SDK + compatibility subsystem.
4. Signed OTA updates + enterprise policy controls.
5. Installer/recovery tools and long-term support channels.

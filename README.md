# Sadas OS

Sadas OS is a from-scratch Rust operating system project focused on:

- **Privacy by default**: strict capabilities, deny-by-default IPC, and auditable policies.
- **Familiarity**: predictable user workflows and service naming.
- **Customization**: UI themes, density, and behavior controls as first-class system features.
- **Efficiency everywhere**: adaptive runtime profiles that scale from legacy hardware to modern devices.

> This repository provides a full fresh OS architecture starting point (kernel, platform primitives, syscall ABI, services, UI stack, init process, and tooling) and intentionally does **not** depend on Linux distribution components.

## Repository layout

- `crates/kernel`: capability-oriented microkernel scheduler, budgets, and IPC checks.
- `crates/platform`: low-level hardware primitives such as entropy source handling.
- `crates/sysapi`: stable syscall ABI shared by kernel, services, and UI.
- `crates/services`: privacy policies for vault, permissions, shell, compositor, and sync.
- `crates/ui`: UI profile system tuned for legacy/modern hardware.
- `crates/init`: first userspace process orchestrating full system bring-up.
- `tools/sadasctl`: host-side planning, privacy inspection, and device-tier recommendation.
- `docs/roadmap.md`: staged plan to reach bootable hardware targets.

## Quick checks

```bash
cargo check --workspace
cargo run -p sadas-init
cargo run -p sadasctl -- plan
cargo run -p sadasctl -- device-profile 2048 2
```

## Vision milestones

1. Boot pipeline: UEFI + handcrafted loader + kernel ELF handoff.
2. Virtual memory + user/kernel isolation.
3. Capability-based security and per-process namespaces.
4. Native compositing desktop with deep customization.
5. Reproducible builds and signed system bundles.

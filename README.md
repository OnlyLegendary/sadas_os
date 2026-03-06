# Sadas OS

Sadas OS is a from-scratch Rust operating system project designed to become a complete, polished desktop platform while staying privacy-first.

## Product goals

- **Privacy by default** with capability controls, strict policy levels, and secure IPC channels.
- **Familiarity** through a complete shell/compositor/service stack and predictable workflows.
- **Customization** with sleek UI themes, frosted glass mode, and obsidian glass mode.
- **Runs well on old and new hardware** using runtime tiers, latency-aware scheduling, and frame-budget-aware UX profiles.

## Current baseline (what exists in this repo today)

- x86_64 boot flow targeting **UEFI-first** development.
- Rust UEFI bootloader crate (`sadas-bootloader-uefi`) that produces `BOOTX64.EFI`.
- Shared `BootInfo` handoff contract (memory map / framebuffer / ACPI RSDP / initfs / cmdline fields).
- QEMU runner with both legacy and UEFI modes.
- Unified console layer (`sadas-console`) with serial COM1 and framebuffer text rendering.
- Memory scaffolding (`sadas-memory`) for frame allocation, heap init, and page mapping APIs.
- x86_64 arch baseline crate (`sadas-arch-x86_64`) for IDT/exceptions, MADT parsing, APIC, and timer setup.
- Minimal userspace scaffolding (`sadas-syscall`, `sadas-exec`, `userland/*`) for process isolation and shell bring-up.

## Current operational stack (higher-level crate map)

- `crates/boot`: boot chain model and artifact layout.
- `crates/vm`: VM profile and canonical user/kernel address layout definitions.
- `crates/drivers`: driver matrix (storage, network, input, graphics, etc.) — WIP.
- `crates/installer`: installer pipeline — WIP.
- `crates/kernel`: kernel core, scheduling, and syscall dispatch — WIP.
- `crates/platform`: low-level primitives.
- `crates/sysapi`: syscall ABI and shared enums.
- `crates/services`: policy model and service interfaces — WIP.
- `crates/ui`: UI profile tuning — WIP.
- `crates/init`: startup orchestration — WIP.
- `tools/sadasctl`: inspection/utility tool.

## Console layer

- Serial console initializes COM1 (`0x3F8`) early and is always available.
- Framebuffer console uses GOP-provided framebuffer metadata from `BootInfo` when present.
- Logs flow through unified backend to serial + framebuffer with simple rate limiting.
- Kernel panic path prints panic context and halts CPU.

## Memory layer

- `sadas-memory::frame`: frame allocator seeded from UEFI-style descriptors and reserved ranges.
- `sadas-memory::heap`: lock-based global heap initializer for kernel allocations.
- `sadas-memory::paging`: page-alignment-checked mapping API for identity and explicit mappings.

## Minimal userspace and process model

- Syscall ABI crate: `crates/syscall` (numbers and stubs; expanding over time).
- ELF loader crate: `crates/exec` with minimal ELF64 header parsing (WIP).
- Userspace packages:
  - `userland/init` (`/bin/init` model)
  - `userland/shell` with builtins: `help ls cat echo reboot shutdown ps`
  - `userland/ping` and `userland/httpget` (network plumbing WIP)

## Boot in QEMU (UEFI)

```bash
./scripts/build.sh
./scripts/run-qemu.sh
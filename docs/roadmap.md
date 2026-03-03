# Sadas OS Roadmap

## Phase 0: Core scaffolding (done)

- Rust workspace created with independent OS layers.
- No Linux distro dependencies.
- Microkernel building blocks established (tasks, scheduler, capabilities, IPC authorization).
- Initial service policy model and UI profile primitives included.

## Phase 1: Boot to shell

- Add `boot` crate for UEFI + framebuffer initialization.
- Load kernel as an ELF image and hand off memory map.
- Bring up serial and frame logger.
- Start `init` in ring 3 with a minimal userspace runtime.

## Phase 2: Privacy-first runtime

- Implement sealed vault daemon (`vaultd`) with hardware-backed keys.
- Add permission broker with per-app declarative policy.
- Add auditable event log with privacy-preserving aggregation.

## Phase 3: Familiar UX + customization

- Ship baseline shell that mirrors common desktop shortcuts.
- Add declarative personalization engine and hot-swappable themes.
- Keep 30Hz/60Hz/120Hz profiles to ensure smooth experience across hardware generations.

## Phase 4: Hardware expansion

- NVMe, USB HID, audio, Wi-Fi drivers.
- Multi-core scheduler and NUMA-aware memory allocator.
- TPM-backed secure boot measurement chain.

# Sadas OS

Sadas OS is a standalone Rust OS effort focused on privacy-first defaults and a Windows-friendly UX.

## Current baseline

- x86_64 boot flow targeting UEFI-first development.
- Rust UEFI bootloader crate (`sadas-bootloader-uefi`) that produces `BOOTX64.EFI`.
- Shared `BootInfo` handoff contract (memory map / framebuffer / ACPI RSDP / initfs / cmdline fields).
- QEMU runner with both legacy and UEFI modes.
- Unified console layer (`sadas-console`) with serial COM1 and framebuffer text rendering.
- Hardened memory scaffolding (`sadas-memory`) for frame allocation, heap init, and page mapping APIs.
- x86_64 arch baseline crate (`sadas-arch-x86_64`) for IDT/exceptions, MADT parsing, APIC, and timer setup.
- Minimal userspace scaffolding (`sadas-syscall`, `sadas-exec`, `userland/*`) for process isolation and shell bring-up.

## Console layer

- Serial console initializes COM1 (`0x3F8`) early and is always available.
- Framebuffer console uses GOP-provided framebuffer metadata from `BootInfo` when present.
- Logs flow through unified backend to serial + framebuffer with simple rate limiting.
- Kernel panic path prints panic context and halts CPU.

## Memory layer

- `sadas-memory::frame`: frame allocator seeded from UEFI-style descriptors and reserved ranges.
- `sadas-memory::heap`: lock-based global heap initializer for kernel allocations.
- `sadas-memory::paging`: page-alignment-checked mapping API for identity and explicit mappings.
- Kernel now emits debug memory stats during early boot (`tracked/free/allocated`).

## Minimal userspace and process model

- Syscall ABI crate: `crates/syscall` with `write/read/exit/spawn/wait/open/close/readdir/stat/mmap` numbers.
- ELF loader crate: `crates/exec` with minimal ELF64 header parsing.
- Userspace packages:
  - `userland/init` (`/bin/init` model)
  - `userland/shell` with builtins: `help ls cat echo reboot shutdown ps`
- Kernel process table tracks PID + isolated address-space IDs and dispatches core syscall stubs.

## Boot in QEMU (UEFI)

```bash
./scripts/build.sh
./scripts/run-qemu.sh
```

Direct runner commands:

```bash
cargo run -p sadas-qemu-runner -- --uefi --build-only
cargo run -p sadas-qemu-runner -- --uefi --run
```

Notes:
- UEFI QEMU flow requires OVMF (`OVMF_CODE.fd` and `OVMF_VARS.fd`).
- UEFI bootloader compilation requires Rust target `x86_64-unknown-uefi`.

## Make USB

After `./scripts/build.sh`, `target/esp/` contains the ESP layout:
- `EFI/BOOT/BOOTX64.EFI`
- `kernel.elf`
- `initfs.cpio`
- `cmdline.txt`

To prepare a USB manually:
1. Partition USB with a FAT32 ESP.
2. Mount it and copy all files from `target/esp/` to the FAT32 root.
3. Ensure `EFI/BOOT/BOOTX64.EFI` exists on the USB.


## Persistent storage path

- `sadas-block` provides a block layer with backends modeled as `VirtioBlk` (QEMU) and `Ahci` (real-hardware path target).
- `sadas-fs` provides a simple reliable root filesystem image mounted as `/` at `system/state/rootfs.img` in the current host-model runtime.
- ESP is kept separate and can be mounted as `/efi` (optional) while root remains on the persistent image.
- Shell builtins (`ls`, `cat`, `echo` via boot writes) read/write files persisted across reboot simulations by remounting the same image.


## Networking baseline

- `sadas-net` adds pragmatic device-first networking primitives with backends for `VirtioNet` (QEMU) and `E1000e` (real-hardware target).
- Packet support in this phase includes Ethernet framing, ARP packet encoding, IPv4 packet encoding, ICMP echo generation (ping), DHCP lease stub flow, and minimal TCP HTTP GET helper.
- New userland utilities:
  - `sadas-user-ping` (`/bin/ping` model)
  - `sadas-user-httpget` (`/bin/httpget` model)

Examples:

```bash
cargo run -p sadas-user-ping
cargo run -p sadas-user-httpget -- example.com /
```

## Required checks

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

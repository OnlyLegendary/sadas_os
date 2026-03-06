# Sadas OS

Sadas OS is a standalone Rust OS effort focused on privacy-first defaults and a Windows-friendly UX.

## Current baseline

- x86_64 boot flow targeting UEFI-first development.
- Rust UEFI bootloader crate (`sadas-bootloader-uefi`) that produces `BOOTX64.EFI`.
- Shared `BootInfo` handoff contract (memory map / framebuffer / ACPI RSDP / initfs / cmdline fields).
- QEMU runner with both legacy and UEFI modes.

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

## Required checks

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

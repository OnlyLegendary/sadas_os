# Sadas Boot (UEFI-first)

## Components

- `crates/bootloader-uefi`: UEFI application entry (`efi_main`) and boot-path diagnostics.
- `crates/boot-protocol`: stable `BootInfo` ABI shared with kernel.
- `tools/qemu-runner`: builds ESP layout and runs QEMU with OVMF.

## BootInfo contract

`BootInfo` contains:
- memory map pointer/count/entry size/version
- framebuffer address/size/stride/format + width/height
- ACPI RSDP pointer
- initfs address/size
- cmdline address/length

## Build UEFI artifacts

```bash
cargo run -p sadas-qemu-runner -- --uefi --build-only
```

Produces `target/esp/` with:
- `EFI/BOOT/BOOTX64.EFI`
- `kernel.elf`
- `initfs.cpio`
- `cmdline.txt`

## Run in QEMU + OVMF

```bash
cargo run -p sadas-qemu-runner -- --uefi --run
```

The runner uses:
- `QEMU_SYSTEM_X86_64` if set (else PATH + Windows fallbacks)
- `OVMF_CODE`/`OVMF_VARS` env vars if set (else common Linux paths)

## Current status

This prompt adds the UEFI crate, ESP layout, and QEMU UEFI wiring.
Kernel ELF loading, memory map handoff, GOP setup, and ExitBootServices are scaffolded with explicit error logs and remain to be wired fully in subsequent prompts.

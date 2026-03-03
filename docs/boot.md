# Booting Sadas OS in QEMU

This repository includes a minimal bootable BIOS image that prints:

`sadas: hello from kernel`

## Prerequisites

- Rust toolchain (stable)
- QEMU (`qemu-system-x86_64`) installed and available on `PATH`

## One-command run

From repo root:

```bash
make run
```

This command runs `cargo run -p sadas-qemu-runner`, which:
1. Generates a 512-byte bootable BIOS image (`target/sadas_boot.img`).
2. Boots it in QEMU.
3. Prints `sadas: hello from kernel` from boot code.

## Platform notes

### Linux
- Install QEMU via your package manager (`qemu-system-x86` package name varies).
- Run `make run`.

### macOS
- Install QEMU via Homebrew: `brew install qemu`.
- Ensure `qemu-system-x86_64` is in PATH.
- Run `make run`.

### Windows
- Install QEMU (official installer or package manager).
- Ensure `qemu-system-x86_64.exe` is in PATH.
- Run:
  - `cargo run -p sadas-qemu-runner`
  - or `make run` if GNU Make is installed.


## Kernel entry handoff ABI

The kernel entrypoint is defined in `crates/kernel` as:

```rust
#[no_mangle]
extern "C" fn kmain(boot_info_ptr: u64) -> !
```


The generated boot image uses a tiny stage-0 loader that performs minimal setup and then jumps to a `kmain` routine in the boot sector. This keeps the handoff explicit while the Rust `kmain` symbol in `crates/kernel` is now defined with the same ABI for upcoming linkage work.

Current calling convention/arguments:
- `boot_info_ptr` is reserved for future boot metadata.
- The minimal boot path passes `0` (no metadata yet).
- `kmain` is responsible for printing `sadas: hello from kernel`.
- `kmain` currently emits three log lines through the logging layer: INFO, WARN, and ERROR.

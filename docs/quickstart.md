# Sadas OS Quickstart

## Build

```bash
./scripts/build.sh
```

This compiles the workspace, generates `target/sadas_boot.img`, and seeds state defaults.

## Run

```bash
./scripts/run-qemu.sh
```

This launches QEMU via `sadas-qemu-runner`.

## Phase goal checks

- Boot image generated.
- QEMU invocation works when QEMU is installed.
- Kernel boot logs are emitted.

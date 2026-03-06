# Sadas OS Quickstart (UEFI)

## Build

```bash
./scripts/build.sh
```

## Run in QEMU (UEFI)

```bash
./scripts/run-qemu.sh
```

## Alternative runner modes

```bash
cargo run -p sadas-qemu-runner -- --legacy --run
cargo run -p sadas-qemu-runner -- --uefi --run
```

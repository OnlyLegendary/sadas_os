# Sadas OS

Sadas OS is a privacy-first, no-bloat, Windows-friendly product OS effort.

Phase 1 uses a pragmatic base for a real desktop demo:
- Linux kernel
- Wayland + KDE Plasma
- Flatpak app tooling
- QEMU x86_64 UEFI as primary demo target

## Repo layout

- `system/` image artifacts, defaults, branding, persistent state
- `scripts/` build/run entrypoints
- `docs/` quickstart + privacy + roadmap docs
- `apps/` Rust UI apps (`sadas-store`, `sadas-settings`)
- `services/` Rust services (`permission-broker`, `privacy-firewall`, `sadas-core`)
- `crates/` kernel/platform/sysapi and lower-level Rust components
- `tools/` developer tools

## Phase 1 quickstart

```bash
./scripts/build.sh
./scripts/run-qemu.sh
```

Detailed instructions:
- `docs/quickstart.md`
- `docs/privacy.md`

## Required checks

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

# Sadas OS Phase 1 Quickstart (KDE in QEMU)

This phase uses a pragmatic product base: **Fedora KDE Live** (Linux kernel + Wayland/KDE + Flatpak tooling), plus Sadas Rust components built in this repository.

## Prerequisites

- Linux host with QEMU:
  - `qemu-system-x86_64`
  - OVMF firmware (`/usr/share/OVMF/OVMF_CODE.fd`, `/usr/share/OVMF/OVMF_VARS.fd`) for UEFI boot
- Network access on first build (downloads the base ISO)

## Build from a clean clone

```bash
./scripts/build.sh
```

Expected result:
- Builds Rust workspace binaries.
- Seeds local privacy/update state in `system/state/`.
- Produces bootable image artifact:
  - `system/image/sadas-os-phase1-kde.iso`

## Run in QEMU

```bash
./scripts/run-qemu.sh
```

Expected result:
1. QEMU launches and boots the UEFI image.
2. KDE Plasma desktop appears.
3. You can open **System Settings** and **Dolphin File Manager**.
4. Flatpak tooling is present (via Discover/Flatpak packages in base image).

## Phase-1 demo checklist

- [ ] Boots to graphical KDE desktop without manual terminal steps.
- [ ] Settings app opens.
- [ ] File Manager opens.
- [ ] Sadas privacy defaults are seeded under `system/configs/privacy-defaults.conf`.

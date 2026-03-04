# Sadas OS Privacy Stance (Phase 1)

## Default data collection

**None by default.**

Sadas OS Phase 1 ships with these defaults:
- Telemetry: `off`
- Crash reporting: `off`
- Online account required: `false`
- App network access: `deny` until user allows
- Mic/camera access: `deny` until user allows
- File access: `prompt`

Source of truth:
- `system/configs/privacy-defaults.conf`
- `system/state/permissions.db`

## User control model

Per-app permissions are designed to be user-controlled from Settings and brokered by Sadas services.

Current repo components:
- `services/permission-broker` (permission control surface)
- `services/privacy-firewall` (first-pass allow/deny checks)
- `apps/sadas-settings` (privacy toggles)

## Flatpak + sandboxing

Sadas app story is Flatpak-first for user apps. The repository includes:
- Flatpak remote configuration (`system/configs/flatpak-remotes.conf`)
- Sadas Store placeholder launcher (`system/branding/sadas-store.desktop`)
- Rust Store MVP (`apps/sadas-store`) for curated install flows

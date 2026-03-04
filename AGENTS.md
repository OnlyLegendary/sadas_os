# Sadas OS (Product OS) — Codex Instructions

## Mission
Build Sadas OS as a privacy-first, no-bloat, Windows-friendly desktop OS that normal people can use without terminal.

## Non-negotiables (run and report every task)
- cargo fmt --all
- cargo check --workspace
- cargo test --workspace (or explain why a crate can’t be tested yet)
- One QEMU boot command that demonstrates the phase goal

## Product rules
- No telemetry by default. No account required.
- “Not scary”: no terminal required for normal use; avoid Linux jargon in UI.
- Base image stays small; everything else is optional install.
- Use sandboxed apps (Flatpak) and permission prompts.

## Engineering rules
- Phases only. Each phase ends in a runnable, user-visible system.
- No “architecture prose” unless matched by code in the repo.
- Keep diffs reviewable. Prefer incremental PR-sized changes.

## Platform
- Primary: x86_64 UEFI + QEMU
- Desktop: Wayland + KDE (Windows-like)
- Updates: atomic/rollback (choose a practical approach and document it)

## Definition of done for any feature
- Works in QEMU
- Documented in docs/
- Build + checks pass

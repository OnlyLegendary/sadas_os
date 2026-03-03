# Sadas OS Contributor Rules

This file defines default working rules for the entire repository.

## Scope
- Applies to all files under this repository root.

## Project goals
- Keep Sadas OS modular, auditable, and privacy-first.
- Prefer deterministic behavior and small, composable abstractions.
- Keep no-std crates lean and dependency-light.

## Required commands before marking work "done"
Run these from repo root:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test -p sadas-kernel
```

If any command fails, fix the cause or document why it cannot be resolved.

## Formatting and style
- Rust formatting is enforced via `cargo fmt --all`.
- Avoid unnecessary refactors when touching unrelated code.
- Prefer explicit types and small functions over large multipurpose blocks.
- Keep public APIs documented through descriptive naming.

## Safe-change policy
- Keep behavior unchanged unless a failing check requires a fix.
- For hygiene/CI tasks, use minimal edits.
- Add tests only when they directly validate the change.

## Structure expectations
- Core runtime logic belongs in `crates/*` domain crates.
- Host/developer tooling belongs in `tools/`.
- Long-form plans/docs belong in `docs/`.
- Keep cross-crate dependencies intentional and minimal.

## Definition of done
A change is done when:
1. Code is formatted.
2. Workspace compiles.
3. Kernel tests pass.
4. Any new operational rules are documented.

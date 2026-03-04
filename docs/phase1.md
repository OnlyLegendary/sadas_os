# Phase 1 – Boot + Kernel Foundation

## Scope implemented in this phase

- Boot path enters a kernel-style entry contract and prints kernel-owned log lines.
- Kernel has a no_std `kmain` entrypoint and level-based logging via `sadas-logging`.
- Timer/preemptive scheduler skeleton exists in kernel code (`PreemptiveScheduler`) and is testable.
- Boot output includes timer-driven task-switch lines (`idle`/`worker`) representing scheduler handoff.

## Demo script (Phase 1)

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
make run
```

### Expected QEMU output (serial)

At minimum:
- `[sadas][INFO] sadas: hello from kernel`
- `[sadas][WARN] sadas: pic/pit timer initialized`
- alternating tick lines for `worker` and `idle`
- `[sadas][ERROR] sadas: phase1 scheduler loop entered`

## Acceptance checklist

- [x] QEMU boot command exists (`make run`).
- [x] Kernel entrypoint (`kmain`) is no_std and documented.
- [x] Serial logging backend active in kernel.
- [x] Timer/scheduler skeleton is implemented in kernel code and validated by unit tests.
- [x] Repo checks pass (`fmt`, `check`, `test -p sadas-kernel`).

## Notes

- In this environment, `qemu-system-x86_64` may be unavailable; build/test commands still validate phase code.
- Next phase should replace skeleton timer wiring with true hardware interrupt plumbing and context-switch state.

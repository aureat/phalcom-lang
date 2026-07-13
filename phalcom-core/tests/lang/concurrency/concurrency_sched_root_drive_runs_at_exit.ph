// area: concurrency
// spec: U-SCHED plan.md §5 (root-drive pump), implementation-spec.md §5
// status: PASS
// `main` never calls `System.runScheduled` — the scheduled fiber's side
// effect is still observed because `VM::run`'s root-drive pump drains
// `VM::ready_queue` once the top-level program's own activation ends.

System.schedule({ System.print("ran-at-exit") })

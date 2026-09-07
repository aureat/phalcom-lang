# CONC001.C2 — Fiber scheduling overview

Two independently-dispatchable units, grouped here because both close gaps
in the `Fiber`/`Future` concurrency track opened by
the Future plan's DEC-FUT-SCHED ruling ([`CONC001.C4.P1`](../C4-future-library/P1-future-library-and-await.md#9-blocked-on-decision-register)):

| Plan | Mission | Dispatch status | Depends on |
|---|---|---|---|
| [`CONC001.C3.P1`](../C3-fiber-reflection/P1-fiber-reflection-and-terminal-state.md) ([supporting spec](../C3-fiber-reflection/fiber-reflection-spec.md)) | `Fiber#isDone`/`Fiber#error` — pure reads, no scheduler dependency | **dispatch-ready, no blockers** | landed `Fiber` substrate |
| [`CONC001.C2.P1`](P1-ready-queue-and-root-drive.md) ([supporting spec](ready-queue-and-root-drive-spec.md)) | native ready-queue + `System.schedule`/`nextScheduled` + root-drive pump | **dispatch-ready, no blockers** | landed `Fiber` substrate |

## Why grouped, not merged

They are **not** sequentially dependent on each other — P4's
write-set (`primitive/fiber.rs`, `universe/primitives.rs` `fiber_cls`
block) and P3's write-set (`vm/mod.rs`, `vm/dispatch.rs`,
`primitive/system.rs`, `universe/primitives.rs` `system_cls` block) are
disjoint and can dispatch **in parallel**. They are grouped in one folder
because both exist for the same reason: [P2 §9](../C4-future-library/P1-future-library-and-await.md#9-blocked-on-decision-register)
(**DEC-FUT-SCHED**) split `Future` Slice B's one "needs a scheduler"
blocker into exactly these two independent, unblockable-today
preconditions, so neither has to wait on the other, and neither blocks on
a ruling only `Future` itself needed.

## Downstream

Both are preconditions for **`Future` Slice B** (`async`/`await`,
`../C4-future-library/P1-future-library-and-await.md` §7 build order steps 4–7) — not built
by either unit here. Once both land:

```
Fiber (landed) ──▶ P4 ───────────────┐
                                     ├──▶ Future Slice B (async/await/then-pending)
Fiber (landed) ──▶ P3 ───────────────┘
```

`System.sleep(_)`/timers are **not** part of U-SCHED's core slice — see
[CONC001.C2.P1 §4](P1-ready-queue-and-root-drive.md#4-timerssleep--explicitly-deferred-not-this-units-scope)
for why (fairness, `open-questions.md §15`, is OPEN).

## Reviewer note

Both plans are **Reviewer ON**. P4 is small and
low-collision (pure reads, no state change). U-SCHED is the
higher-risk unit of the pair — SPINE (`vm/dispatch.rs`, `vm/mod.rs`,
`vm/bootstrap.rs`, just split out of the former monolithic `vm.rs` this
session) — serialize its dispatch against any other in-flight unit
touching those files; see [P3 §3.1](P1-ready-queue-and-root-drive.md#31-collision-risk).

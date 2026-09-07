# U-SCHED-FIBER — combined work plan: Fiber-surface completion + scheduler seam

Two independently-dispatchable units, grouped here because both close gaps
in the `Fiber`/`Future` concurrency track opened by
[U-FUTURE's DEC-FUT-SCHED ruling](../../../forge/units/U-FUTURE/plan.md#9-blocked-on-decision-register):

| Unit | Mission | Dispatch status | Depends on |
|---|---|---|---|
| [U-FIBER-REFLECT](reflect/plan.md) ([implementation-spec](reflect/implementation-spec.md)) | `Fiber#isDone`/`Fiber#error` — pure reads, no scheduler dependency | **dispatch-ready, no blockers** | landed `Fiber` (U-FIBER) only |
| [U-SCHED](U-SCHED/plan.md) ([implementation-spec](U-SCHED/implementation-spec.md)) | native ready-queue + `System.schedule`/`nextScheduled` + root-drive pump | **dispatch-ready, no blockers** | landed `Fiber` (U-FIBER) only |

## Why grouped, not merged

They are **not** sequentially dependent on each other — U-FIBER-REFLECT's
write-set (`primitive/fiber.rs`, `universe/primitives.rs` `fiber_cls`
block) and U-SCHED's write-set (`vm/mod.rs`, `vm/dispatch.rs`,
`primitive/system.rs`, `universe/primitives.rs` `system_cls` block) are
disjoint and can dispatch **in parallel**. They are grouped in one folder
because both exist for the same reason: [U-FUTURE/plan.md §9](../../../forge/units/U-FUTURE/plan.md#9-blocked-on-decision-register)
(**DEC-FUT-SCHED**) split `Future` Slice B's one "needs a scheduler"
blocker into exactly these two independent, unblockable-today
preconditions, so neither has to wait on the other, and neither blocks on
a ruling only `Future` itself needed.

## Downstream

Both are preconditions for **`Future` Slice B** (`async`/`await`,
`../../../forge/units/U-FUTURE/plan.md` §7 build order steps 4–7) — not built
by either unit here. Once both land:

```
Fiber (landed) ──▶ U-FIBER-REFLECT ──┐
                                     ├──▶ Future Slice B (async/await/then-pending)
Fiber (landed) ──▶ U-SCHED ──────────┘
```

`System.sleep(_)`/timers are **not** part of U-SCHED's core slice — see
[U-SCHED/plan.md §4](U-SCHED/plan.md#4-timerssleep--explicitly-deferred-not-this-units-scope)
for why (fairness, `open-questions.md §15`, is OPEN).

## Reviewer note

Both units are **Reviewer ON**. U-FIBER-REFLECT is small and
low-collision (pure reads, no state change). U-SCHED is the
higher-risk unit of the pair — SPINE (`vm/dispatch.rs`, `vm/mod.rs`,
`vm/bootstrap.rs`, just split out of the former monolithic `vm.rs` this
session) — serialize its dispatch against any other in-flight unit
touching those files; see [U-SCHED/plan.md §3.1](U-SCHED/plan.md#31-collision-risk).

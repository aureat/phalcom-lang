# Confirmed runtime errors

Reproduced defects in landed `main` code. Each entry here has a **repro that
fires under `target/debug/phalcom`** — nothing is listed on the strength of an
argument. A finding enters only after its repro is observed; it leaves only by
being fixed or disproved at a cited `file:line`.

Distinct from [`docs/forge/DEFERRED.md`](../forge/DEFERRED.md) (out-of-scope
observations noticed in passing) and from `perf-log/` (measured performance
facts): this directory is runtime **correctness** defects — crashes and wrong
results — with a standing repro.

## Method

Found by an adversarial Fable 5 audit (one read-only auditor per subsystem
lens), then **gated**: every finding was reproduced under the debug binary and
isolated with a control before being written down; fix *directions* are recorded
but marked unverified, because in this codebase a reproduced diagnosis is still
not a verified fix — prior rounds reproduced the defect but prescribed the wrong
fix. The auditor's word confirms nothing; the machine does.

## Open

| ID | Title | Severity | Confirmed |
|----|-------|----------|-----------|
| [E002](E002-fiber-floor-upvalue-crash.md) | Fiber-floor failure capture drops the live stack without closing open upvalues | **blocker** (crash) | 2026-07-19 |
| [E003](E003-schedule-pump-arity.md) | `System.schedule` pump resumes an arity-1 entry with zero args, failing the run | minor | 2026-07-19 |
| [E005](E005-nonlocal-return-some-wrapped.md) | A non-local `return` through `bool_if_true`/`bool_if_false` comes back `Some`-wrapped | **major** (silent wrong answer) | 2026-07-20 |

E001, E002 and E004(c) are the **same family**: a value held live across a re-entrant /
parked interpreter boundary that the root/unwind scan does not cover. E001 is
fixed and E002 is not, and the difference is instructive: E001's recovery path
had a root enumeration to extend, while the fiber-failure path has no unwind at
all to hook a per-cell step onto (see
[`docs/learn/concurrency/fiber-failure.md`](../learn/concurrency/fiber-failure.md)).
The family's shape, stated once: **a participant is removed from the machinery on
one exit path and not the other** — cells closed by `unwind_to` but not by the
fiber floor (E002), a handle rooted for one re-entrant call but not the next
(E001), a waiter unregistered on `await`'s root branch but not its raising branch
(E004(c)). Each was found by asking which arm of a conditional the cleanup lives
inside.

## Fixed

| ID | Title | Fixed at | Verified |
|----|-------|----------|----------|
| [E001](E001-gc-ensure-temp-root-uaf.md) | `block_ensure` frees the protected block's pending result if the cleanup collects | `cdd2117` — `VM::push_temp_root` + `collect_roots` enumeration | 2026-07-19 (all repros + control + the error-carrying path) |
| [E004](E004-await-cannot-suspend.md) | `Future#await` could never suspend a fiber; its own `.attempt()` wrapper tripped the restricted-yield guard | `f479189` — `Fiber#isRoot` predicate + bare yield, pump quiescence check, `drain` skips finished waiters | 2026-07-19 (all 3 repros + both controls + clean-checkout suite) |

E004 is the sharpest illustration of this directory's method note: its diagnosis was reproduced and
correct, and **its fix direction was still wrong**. The recorded prescription for E004(c) — unregister
the waiter on `await`'s raising branch — is not implementable once (a) is fixed, because catching that
raise requires the very native frame whose removal was the fix. The repair that worked guards at
`drain` instead, and covers strictly more. Reproduce, then re-derive.

## Refuted (documented non-errors)

Kept so the disproof is not re-litigated.

### R001 · Reflective `perform` does **not** corrupt the fiber floor — refuted 2026-07-19

A HIGH-confidence audit finding claimed a fiber-switch primitive dispatched
reflectively (`send_dynamic`/`invoke_method_object` increment
`native_reentry_depth` *after* the initial `call_method`,
`phalcom-core/src/vm/send.rs:223-235,272-278`) would switch under a native frame
at depth 0 and corrupt the parked floor, where the spec requires
`CannotYieldAcrossNativeFrame`.

**Disproved by reproduction.** On a fiber that yields, `f.perform(#call())`,
`f.perform(Symbol.new("call()"))`, and a direct `f.call()` control all produce
identical, correct output (`1` / `isDone == false`). The premise was also wrong:
yielding back through `call()` is legal, not an error — the two-way-channel
golden (`tests/lang/concurrency/concurrency_fiber_two_way_channel.ph`) depends on
it. A plausible, well-argued blocker that a three-line control deleted.

# Raw source snapshot: performance

> Captured: 2026-09-08
> Source area: performance specification and optimization-program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/spec/current/performance.md ---
# Performance Strategy

> Normative specification of Phalcom's performance discipline: the laws that
> govern optimization work, the tier model and its committed sequence, the
> success target, and the standing invariants every optimization must preserve.
> Realises [ADR-0051](../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md).
> This document governs *how* the runtime is made fast; it does not itself change
> any observable semantics.

Related: [ADR-0010](../../adr/0010-tagged-value-enum.md) (`Value` repr;
NaN-boxing deferral), [ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md)
(selector encoding; the inline-cache seam),
[ADR-0018](../../adr/0018-sacred-selector-inliner-and-override-guard.md) (sacred-selector
inliner + deopt guard), [ADR-0050](../../adr/0050-non-moving-mark-sweep-collector.md)
+ [memory-management.md](memory-management.md) (the collector),
[method-lookup.md](method-lookup.md) (dispatch), `docs/forge/DEFERRED.md` (the
deferred perf cluster), `docs/forge/units/U-HOTPATH`, `U-GC` (existing units).

---

## 1. The laws (normative)

Every optimization to the Phalcom runtime or compiler is subject to these. They
bind implementers and reviewers alike; a unit that violates one is not mergeable.

- **P1 — Measure before you touch.** No optimization lands without (a) a
  reproducible in-repo benchmark, (b) a profile that attributes the cost to a
  named mechanism, and (c) a recorded before/after number. A performance change
  with no before/after measurement is **not done**. Claims of "the bottleneck is
  X" are hypotheses to falsify with a profile, never facts to act on directly.

- **P2 — Behavior-invariant, or it is a spec change.** Optimization keeps the
  golden `.ph` corpus byte-identical and `./scripts/verify.sh` green; the
  behavior floor stays `+0`. A change that alters *any* observable — output,
  error, evaluation order, dispatch result — is a semantic change and takes its
  own ADR + spec amendment. It is never smuggled in as a performance edit.

- **P3 — Every fast path equals the slow path exactly.** A specialized path (an
  arithmetic superinstruction, an inline-cache hit, an unboxed operation) that
  differs from the generic message send on *any* input — numeric overflow, a
  runtime method override, a subclass, a side effect, a `doesNotUnderstand` — is
  a correctness defect, not an optimization. Each fast path carries a **guard that
  provably implies the slow path**, and a **deopt that reconstructs exact
  interpreter state** (the discipline of [ADR-0018](../../adr/0018-sacred-selector-inliner-and-override-guard.md)).

- **P4 — Name what it precludes.** Every optimization states, in its unit plan,
  what future optimization or invariant it forecloses. A local win must not create
  a global regret: the non-moving collector must not block a future moving one; an
  inline-cache layout must not block NaN-boxing; an added superinstruction must not
  exhaust the opcode budget a later one needs.

- **P5 — Reconcile with the locked contract.** The 16-byte tagged `Value` API
  ([ADR-0010](../../adr/0010-tagged-value-enum.md)), the `ClassId`-keyed
  inline-cache seam ([ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md)),
  comma-canonical selector encoding, and the handle/arena heap
  ([ADR-0009](../../adr/0009-handle-arena-heap.md)) are **locked**. Inline-cache
  *population* and NaN-boxing are **deferred-but-sanctioned**: implemented behind
  those surfaces, never redesigning them. No optimization reopens a locked
  question or builds atop an open one without flagging it.

## 2. The cost model

The strategy targets a specific, verified model of where time goes. Optimizations
are justified against it, not against intuition.

- **The send is the atom of cost.** An allocation-heavy or dispatch-heavy program
  (the 1M-fiber Skynet benchmark is the archetype) spends its time in millions of
  message sends, each paying a fixed dispatch tax and often a per-send allocation.
  The fiber **switch** is already O(1) (`mem::take` of three containers,
  `fiber.rs:29-51`) and is **not** a target — the sends around it are.

- **Two independent per-send cost classes.**
  1. *Fixed dispatch tax* — an `IndexMap<Symbol, ObjRef>` hash probe walked per
     superclass level on every send (`lookup_method_in_hierarchy`, `class.rs:65`),
     with no inline cache. Addressed by Tier 3.
  2. *Per-send allocation* — a `Vec<Value>` built for arguments on every primitive
     send (`vm.rs:626`), i.e. on every arithmetic op and every fiber operation.
     Addressed by Tier 2.

- **Representation sets the ceiling.** A 16-byte tagged `Value`, a 256-byte
  `Object` slot, and boxed-everything cap peak throughput below the target
  *regardless* of dispatch quality. The representation wins — `Box`-fat-variants
  (Tier 4) and NaN-boxing (Tier 6) — raise that ceiling.

- **Whole-process lifetime is the unit of measurement.** Phalcom starts instantly
  and has a flat peak (pure interpreter, no warmup). Benchmarks compare
  whole-process wall-clock (and peak RSS), not steady state — a fair comparison
  against a warmup-paying JIT'd rival, and the honest measure for CLI/short-script
  workloads.

## 3. The success target (normative)

- **Target: Wren parity — within ~2× of Wren on the Skynet benchmark.** Realistic
  for a pure interpreter equipped with an inline cache and NaN-boxed values.
- **CPython parity is an intermediate checkpoint**, expected to be reached by

--- docs/implementation/PERF001-hotpath/PROGRAM.md ---
---
id: PERF001
category: PERF
kind: optimization
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# PERF001 — hotpath

This program owns pending hot-path optimization and execution-cost work.

--- docs/implementation/PERF002-inline-caches/PROGRAM.md ---
---
id: PERF002
category: PERF
kind: optimization
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# PERF002 — inline caches

This program owns inline-cache implementation and optimization records.

--- docs/implementation/PERF003-runtime-performance/PROGRAM.md ---
---
id: PERF003
category: PERF
kind: optimization
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# PERF003 — runtime performance

This program owns runtime performance instrumentation and performance-one
implementation records.

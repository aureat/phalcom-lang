# Performance Strategy

> Normative specification of Phalcom's performance discipline: the laws that
> govern optimization work, the tier model and its committed sequence, the
> success target, and the standing invariants every optimization must preserve.
> Realises [ADR-0051](../../adr/0051-performance-strategy-measure-first-tiered-optimization.md).
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

- **P5 — Reconcile with the locked contract.** The tagged `enum Value` API
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
  Tiers 1–3 (inline cache + arithmetic fast path) *without* NaN-boxing — CPython
  is a switch-loop interpreter with boxed integers and no inline cache, a lower
  bar than Wren.
- The **final ~2× to Wren** is what Tier 6 (NaN-boxing) exists to close, and it is
  **gated on a measured shortfall after Tier 5**, not shipped speculatively.

## 4. The tier model (committed sequence)

Each tier is independently shippable and measured against the Tier 0 baseline.
**Re-measure after every tier** — the ranking below is a hypothesis Tier 0's
profile may re-order, and an allocation-heavy target may close the gap before the
dispatch tier ships.

### Tier 0 — Instrumentation (prerequisite; blocks all others)

Build the harness before touching any hot path. Reproduce Skynet in-repo, add
criterion micro-benches (bare send, arithmetic send, fiber spawn/yield), profile
for a by-mechanism attribution, and commit a `BASELINE.md` recording Phalcom vs
Wren vs CPython. Deliverable: the harness plus the profile that ratifies or
re-ranks §2's cost model. Unit: `U-BENCH`.

### Tier 1 — Cheap behavior-invariant wins

Feature-gate the per-opcode `tracing` span + stack `debug!` (`vm.rs:1214-1216`)
so release builds pay nothing, and land `U-HOTPATH` (register-hoist interpreter
state, precompute dispatch-derived selectors, branch-free `class-of`). Lowest
risk, measured first. Units: tracing-gate (may be a micro-fix) + `U-HOTPATH`.

### Tier 2 — Kill per-send allocation

Adopt a primitive **in-place stack ABI** (`DEFERRED.md`): a primitive reads and
writes a `&mut [Value]` stack window and returns a status, with no `Vec` allocation
and no `CallFrame` push — removing the `vm.rs:626` allocation from every arithmetic
and fiber op. Add a guarded arithmetic fast path for `Number ⊕ Number` (P3: deopt
to the real send on non-Number or override). Unit: `U-PRIM-ABI`.

### Tier 3 — Dispatch structural (the fixed-tax lever)

Carve a dense **selector-only interner** out of the mixed `Symbol(u32)` space,
then populate a **monomorphic inline cache** keyed `(ClassId, SelectorId)` per call
site — the locked seam of ADR-0012. Prefer per-class own-method arrays + chain-walk
(design B) over flatten-on-reopen (design A, rejected — it fights the dynamic object
model). Add operand-free superinstructions (`LOAD_LOCAL_0..15`, `LOAD_FIELD_THIS`)
after an opcode-budget check. Unit: `U-IC`.

### Tier 4 — Memory: bounded and dense

Land the non-moving mark-sweep collector
([ADR-0050](../../adr/0050-non-moving-mark-sweep-collector.md)) to bound the
unbounded heap (on an allocation-bound benchmark this alone may recover much of the
gap), `Box` the fat `Object` variants (256 B → ~24–32 B), and pool fiber operand
and frame `Vec`s across fiber death. Non-moving is required so Tier 3's IC tags and
`==` identity survive collection (§5, I3). Unit: `U-GC` (existing).

### Tier 5 — Compile-time & startup

Cache the compiled core so `core.ph` is not re-lexed/parsed/compiled on every
`VM::new` (`vm.rs:279,309-313`) — the largest single startup win. Deduplicate
`add_constant` (`chunk.rs:27`) and the per-literal compile-time heap `Str`
allocation, resolve locals/upvalues through a hashmap instead of a linear reverse
scan (`compiler/lib.rs:475`), and cut lexer allocations (borrow `&str` tokens; skip
the `scan_number` underscore-strip when there is no `_`). Unit: `U-COMPILE`.

### Tier 6 — Ceiling-raisers (measured-gate)

Only against a measured shortfall to target after Tier 5:
- **NaN-boxing** (`Value` 16 B → 8 B, native doubles) behind the locked `Value`
  API — the peak-throughput ceiling-raiser.
- **Generational collection** (write barrier funnelled through mutation
  choke-points; see [memory-management.md](memory-management.md) §7).
- **Threaded dispatch / computed-goto** — gate on measured branch-mispredict cost.

## 5. Invariants (P-series enforcement)

These are the standing checks a reviewer verifies for any optimization unit.

- **I1 — Golden byte-identity.** For a behavior-invariant unit, the entire golden
  `.ph` corpus and `tests/invariants.rs` produce byte-identical output before and
  after. A single changed golden is a behavior leak to investigate, not to
  rebaseline (enforces P2).

- **I2 — Inline-cache coherence.** Any inline cache invalidates on class reopen,
  method (re)definition, and `superclass=` via a class-epoch bump. A cache that can
  serve a stale method after a hierarchy mutation is unsound. A megamorphic site
  (receiver-class count past the cache's capacity) falls back gracefully to the
  dictionary walk — no cliff, no incorrectness (the inline-cache ⊗ mutable-hierarchy
  and ⊗ polymorphism hazards).

- **I3 — Non-moving handles under collection.** While the collector is non-moving
  ([ADR-0050](../../adr/0050-non-moving-mark-sweep-collector.md)), a surviving
  object keeps its handle, so inline-cache tags keyed on `ClassId`/`ObjRef` and
  `==` object identity remain valid across a collection. A future moving collector
  must introduce handle-indirection *before* it may relocate, or it breaks the IC
  and identity (P4).

- **I4 — Fast-path equivalence.** Every speculative fast path ships with a golden
  test proving it equals the slow send on its guard-miss inputs (non-Number
  operands, overridden operator, subclass receiver), and its deopt restores exact
  state (enforces P3).

- **I5 — No `unsafe` without review.** Optimization introduces no `unsafe` without
  an independent reviewer sign-off; the `miri` lane stays green. The register-hoist
  borrow tension is resolved with indexed/`ChunkId` access, not raw pointers,
  absent such sign-off.

- **I6 — Defined errors under resource pressure.** Stack-depth, allocation, and
  recursion caps convert runaway or hostile input into a defined diagnostic, never
  UB or a raw `panic!` (the dynamic-power ⊗ untrusted-input hazard). These caps are
  currently UNSPECIFIED (see §6) and are owed before the VM is exposed to untrusted
  bytecode.

## 6. Feature flags (measured, gated behind Cargo features)

Cargo features in `phalcom-core/Cargo.toml`, both off by default (`default = []`)
so a plain build/release pays nothing:

- **`vm-trace`** — the per-opcode `tracing` span + stack `debug!`s in the
  dispatch loop (`vm/dispatch.rs`, Tier 1). Measured cost with every subscriber
  at `LevelFilter::OFF`: 18.2% of arith wall-clock (perf-log 003) — not
  avoidable at runtime without compiling the callsites out, hence the gate.
  Enable only to debug the dispatch loop; never for benchmarking or release.
- **`fiber-pool`** — recycles a finished fiber's `stack`/`frames` `Vec`s into a
  bounded `VM::fiber_pool` free-list instead of allocating fresh ones per
  `Fiber.new` (U-GC step 5, Tier 4's "pool fiber operand and frame `Vec`s
  across fiber death"). Measured **net negative** in whole-process A/B
  benchmarking (perf-log, 2026-07-14): pool bookkeeping cost exceeds the
  allocations it avoids. Kept as a flag, off by default, so the experiment can
  be re-run/re-measured later without reconstructing it.

## 7. What stays open

- **Resource caps are unspecified.** The concrete stack-depth / allocation /
  recursion limits (and the diagnostic each raises) are not yet defined; I6 states
  the *obligation*, not the numbers. A dedicated unit owes them before untrusted
  input is in scope.
- **Tier 6 is gated, not scheduled.** NaN-boxing, generational collection, and
  threaded dispatch are designed-open behind committed surfaces; each ships only
  against a measured need, and the measurement is taken after Tier 5, not now.
- **The tier ranking is provisional.** §2's cost model and §4's ordering are the
  current best hypothesis; Tier 0's attribution profile is authoritative and may
  re-order the tiers. This document is amended if it does.

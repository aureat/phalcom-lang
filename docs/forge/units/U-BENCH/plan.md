# U-BENCH — performance instrumentation & baseline (Tier 0)

Status: **PLANNED** (dispatch-ready). Tier 0 of the performance strategy
([performance.md](../../../spec/v0.2/performance.md) §4,
[ADR-0051](../../../adr/0051-performance-strategy-measure-first-tiered-optimization.md)).
**Blocks every other perf tier** — law P1 (measure before you touch) is
unsatisfiable without it. Write-set is test/bench/doc infrastructure only; it
touches no runtime hot path, so it can land in parallel with unrelated units
(no `vm.rs`/`heap.rs` write — but confirm no collision on `benchmarks/`,
[[phalcom-concurrent-session-hazards]]).

## Role
Turn the oral "~29× slower than Wren on Skynet" into a **reproducible, attributed,
committed** measurement, and stand up the harness every later tier re-measures
against. Nothing here changes runtime behavior; it makes the runtime *measurable*.

## Spec anchor
[performance.md](../../../spec/v0.2/performance.md) §3 (target), §4 Tier 0, law P1.
No surface semantics change, so no ADR is amended.

## Preconditions (verify on HEAD before building)
- Confirm no `skynet.ph`, no criterion harness, no `benches/`, and no committed
  baseline exist (they did not at authoring; a concurrent session may have added
  one — reconcile, don't duplicate).
- Confirm `benchmarks/math/run.sh` is the only existing runner and is deliberately
  outside `cargo test` (staging corpus). U-BENCH's harness is *separate* — a
  perf/attribution harness, not a correctness corpus.
- Confirm a Wren build is available for reference (the tree references
  `docs/wren/`); if not, the baseline records "Wren: not-locally-built, upstream
  figure only" rather than inventing a number.

## Design
- **Reproduce the gap in-repo.** Add `benchmarks/vm/skynet.ph` (the recursive
  1M-fiber program) plus two isolating micro-programs: an allocation-bound loop
  and a dispatch-bound loop (same send count, no fiber spawn) — so the profile can
  separate allocation cost from dispatch cost per §2's cost model.
- **Criterion micro-benches** under a `benches/` target: bare send, arithmetic
  send (`1+2` in a loop), fiber spawn/yield. These give per-mechanism numbers that
  survive as regression tripwires for later tiers.
- **Attribution profile.** Produce a flamegraph / `samply` profile of Skynet and a
  by-mechanism cost table. This is the deliverable that **ratifies or re-ranks**
  the Part II / §2 suspect ordering (tracing span, args `Vec`, dispatch lookup,
  unbounded heap). The ranking is a hypothesis until this table exists.
- **Commit `BASELINE.md`** (under `benchmarks/vm/` or `docs/forge/`): Phalcom vs
  Wren vs CPython wall-clock and peak RSS for Skynet, the criterion numbers, and
  the attribution table. Whole-process lifetime, not steady state (§2).
- **A one-command runner** (`benchmarks/vm/run.sh` or a cargo alias) that
  reproduces every number, so "re-measure after each tier" (P1) is one command.

## Write-set (STOP-and-report if outside)
- `benchmarks/vm/` — new: `skynet.ph`, micro-programs, `run.sh`, `BASELINE.md`.
- `benches/` (or a `[[bench]]` target in `phalcom-core/Cargo.toml`) — criterion
  benches. If adding criterion needs a `[dev-dependencies]` edit, that is in-scope;
  a change to any `src/*.rs` runtime file is **not** — STOP-and-report.
- **No `src/` runtime edit. No golden change.** Floor: **+0**.

## Build order
1. `skynet.ph` + micro-programs; confirm they run and reproduce a visible gap.
2. `run.sh` one-command runner; record raw Phalcom/Wren/CPython numbers.
3. Criterion benches (send / arith / fiber).
4. Attribution profile → cost table.
5. `BASELINE.md` committing all of it. Commit per green step.

## Tests / verification
- The programs must **execute** (this is the gate — a benchmark that errors is not
  a baseline). Skynet uses the while-loop form if range-with-variable-bounds still
  does not parse (noted in prior sessions).
- `cargo build && cargo test` stays green (no runtime change); `cargo doc` clean.
- The baseline is *descriptive*, not a pass/fail gate — but the criterion benches
  become regression tripwires from Tier 1 on.

## Decisions to flag (DEC-BENCH)
- **DEC-BENCH-A — criterion vs a custom timing harness.** Criterion gives
  statistics + regression detection but adds a dev-dependency and warmup semantics
  that must not mask whole-process cost. Recommend criterion for micro-benches +
  a plain wall-clock `run.sh` for whole-process Skynet.
- **DEC-BENCH-B — Wren/CPython reference policy.** Require a local Wren/CPython
  build for an apples-to-apples number, or record upstream figures with a
  provenance note? Recommend local build where feasible; annotate provenance
  either way (no oral numbers — the very problem this unit closes).
- **DEC-BENCH-C — where `skynet.ph` lives.** `benchmarks/vm/` (new) vs
  `benchmarks/` root. Recommend `benchmarks/vm/` to separate perf programs from the
  math-identity correctness corpus.

## What must this not preclude (P4)
- The harness must measure **whole-process wall-clock and peak RSS**, not just
  steady-state throughput — later tiers include a *memory* tier (U-GC) whose win is
  bounded RSS, invisible to a throughput-only harness.
- It must be **re-runnable unchanged after each tier** so the same numbers are
  comparable across the roadmap; no baseline that bakes in a specific build config.

## Return shape (implementer)
commit SHA(s) · the reproduced Skynet gap (Phalcom/Wren/CPython, wall-clock + RSS)
· the attribution cost table and whether it **ratifies or re-ranks** the §2 suspect
order · criterion numbers (send/arith/fiber) · confirmation of zero `src/` runtime
edit and zero golden change · verify + `cargo doc` tails · write-set confirm.

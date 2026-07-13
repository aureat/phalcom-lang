# 51. Performance strategy: measure-first, tiered, behavior-invariant

- Status: Proposed
- Date: 2026-07-13
- Related: [ADR-0009](0009-handle-arena-heap.md) (handle heap),
  [ADR-0010](0010-tagged-value-enum.md) (`Value` repr; NaN-boxing deferral),
  [ADR-0012](0012-selector-signature-encoding-and-dispatch.md) (selector encoding;
  inline-cache seam), [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)
  (sacred-selector inliner + deopt guard), [ADR-0030](0030-fibers-and-futures-cooperative-concurrency.md)
  (fibers own heap-resident stacks), [ADR-0050](0050-non-moving-mark-sweep-collector.md)
  (non-moving collector); `docs/spec/v0.2/performance.md` (normative);
  `docs/forge/DEFERRED.md` (the deferred perf cluster);
  `docs/forge/units/U-HOTPATH`, `U-GC` (existing units this strategy sequences).

> **Provisional number.** `0051` was the next free slot at authoring time on a
> tree with live concurrent sessions (tail was `0050`). If a concurrent ADR
> claims `0051`, renumber this one — no cross-file index is edited by this ADR.

## Context

Phalcom is a pure bytecode interpreter with no JIT. A 1M-fiber Skynet
microbenchmark reportedly ran ~29× slower than Wren. That figure is **oral**: it
is recorded in no committed file, and the tree contains no `skynet.ph`, no
criterion harness, and no committed baseline. "Phalcom is 29× slower" is
therefore an *unverified hypothesis*, not a measured fact.

At the same time, a three-lens verification pass over the current tree
established the mechanisms a performance effort would plausibly target — these
are confirmed code facts:

- **No inline cache exists.** Every `Invoke` resolves through an
  `IndexMap<Symbol, ObjRef>` hash probe walked per superclass level
  (`lookup_method_in_hierarchy`, `class.rs:65`). ADR-0012 reserves the
  `ClassId`-keyed IC slot but it is specced-but-unpopulated.
- **Every primitive send allocates.** The `Primitive` arm of `call_method`
  builds `self.stack[recv+1..].to_vec()` (`vm.rs:626`) — a heap `Vec` per send,
  on every arithmetic op and every fiber `call`/`yield`/`resume`.
- **Arithmetic goes through full dispatch.** `1 + 2` compiles to an ordinary
  `Invoke` (`compiler/lib.rs:2190`); the only bytecode-level fast paths are the
  Bool/Block sacred selectors (`inliner.rs`).
- **The interpreter loop pays per-opcode overhead.** A `tracing` span plus a
  `debug!("Stack before: {:?}", self.stack)` are constructed unconditionally in
  the loop body (`vm.rs:1214-1216`), and the current chunk is re-derived from the
  heap arena every opcode.
- **The heap is unbounded.** ADR-0009 deferred reclamation; a 1M-fiber run leaks
  every `FiberObject`. Slots are 256 B (sized to the fattest variant).
- **The fiber *switch* is already O(1)** — `mem::take` of three containers
  (`fiber.rs:29-51`), no stack copy. The cost is the per-send tax paid millions
  of times around the switch, not the switch itself.
- **Startup recompiles the core.** `core.ph` is re-lexed/parsed/compiled on every
  `VM::new` (`vm.rs:279,309-313`); no compiled-module cache.

The risk this ADR guards against is **optimizing the wrong thing** — pouring
effort into a dispatch rewrite when the measured bottleneck on an
allocation-heavy benchmark might be allocation, or shipping a speculative fast
path that silently changes semantics.

## Decision

Adopt a **measure-first, tiered, behavior-invariant** performance strategy,
targeting **Wren parity (~2× on Skynet)**. The strategy is a standing policy plus
an ordered sequence; the normative detail (laws, invariants, tier contract) is in
`docs/spec/v0.2/performance.md`.

1. **Measure-first is policy, not advice.** No optimization lands without (a) a
   reproducible in-repo benchmark, (b) a profile attributing the cost to a named
   mechanism, and (c) a recorded before/after number. The very first unit
   (`U-BENCH`, Tier 0) builds the harness, reproduces the gap, and commits a
   `BASELINE.md` — turning the oral 29× into a measured, attributed figure. Every
   later tier re-measures; the tier ranking is a hypothesis the profile may
   re-order.

2. **Behavior-invariant is the default gate.** Optimization work keeps the golden
   `.ph` corpus byte-identical and `verify.sh` green; floor stays `+0`. A change
   that alters an observable is a *spec change* — it gets its own ADR + spec — not
   a performance sneak. Any speculative fast path (arithmetic, IC hit) must equal
   the generic send on every input and deopt to exact state (the
   speculative-optimization ⊗ observable-semantics discipline of ADR-0018).

3. **A tiered sequence, cheapest-and-safest first.** The committed order (detail
   in `performance.md` §Tiers):
   - **Tier 0 — instrumentation** (`U-BENCH`): harness + baseline. Blocks all.
   - **Tier 1 — cheap invariant wins** (`U-HOTPATH` + feature-gating the
     per-opcode tracing).
   - **Tier 2 — kill per-send allocation** (`U-PRIM-ABI`: in-place stack ABI +
     arithmetic fast path).
   - **Tier 3 — dispatch structural** (`U-IC`: selector-only interner +
     monomorphic inline cache at the `ClassId` seam + superinstructions).
   - **Tier 4 — memory** (`U-GC`: the non-moving collector + `Box`-fat-variants +
     fiber-stack pool).
   - **Tier 5 — compile-time & startup** (`U-COMPILE`: compiled-core cache,
     `add_constant` dedup, hashmap scope resolution, lexer allocation cuts).
   - **Tier 6 — ceiling-raisers, measured-gate**: NaN-boxing, generational GC,
     threaded dispatch — shipped only against a measured shortfall to target.

4. **NaN-boxing is in scope but gated.** Wren parity ultimately needs the
   representation win (`Value` 16 B → 8 B, native doubles), but it ships *only* if
   Tiers 1–5 leave a material gap. CPython parity is expected to fall out of
   Tiers 1–3 without it. NaN-boxing stays behind the locked `Value` API
   (ADR-0010), so this is a population, not a redesign.

5. **Nothing in the locked contract is reopened.** The tagged `enum Value` API,
   the `ClassId`-keyed IC seam, comma-canonical selector encoding, and the
   handle/arena heap are locked. This strategy *populates* the deferred-sanctioned
   optimizations (IC, NaN-boxing) rather than altering the surfaces they sit
   behind.

6. **Memory safety is a constant, every tier.** No `unsafe` without an
   independent reviewer sign-off (the U-HOTPATH register-hoist has a genuine
   borrow-checker tension — resolve with indexed/`ChunkId` access, not raw
   pointers). The `miri` lane stays in verify. Resource caps (stack depth,
   allocation, recursion) — which the overlay currently leaves UNSPECIFIED — must
   convert runaway or hostile input into a *defined error*, never UB or a raw
   `panic!`.

## Consequences

- **The 29× stops being a rumor.** After Tier 0 the gap is reproducible,
  attributed, and diffed against Wren and CPython in-repo. Effort is spent against
  a profile, not a guess.
- **Correctness is protected by construction.** The behavior-invariant default
  means a perf regression shows up as a golden diff, and every fast path carries a
  guard-implies-slow-path obligation. Optimization cannot silently change
  semantics.
- **The tiers interlock cleanly.** Tier 4's non-moving collector keeps Tier 3's IC
  tags and `==` identity valid across collection (ADR-0050, U-GC §9), so IC
  population needs no GC-invalidation story. Tier 6's NaN-boxing sits behind the
  same `Value` accessor the collector already traces through, so it touches
  neither the collector nor the IC key.
- **Sequencing constraint acknowledged.** `U-HOTPATH`, `U-GC`, `U-PRIM-ABI`, and
  `U-IC` all single-write `vm.rs`/`heap.rs`; they cannot share a parallel wave and
  must serialize under worktree isolation
  ([[phalcom-concurrent-session-hazards]]).
- **A defined stopping point.** "Wren parity (~2×)" is the success metric; the
  gate on Tier 6 means the effort stops when measured, not when the roadmap is
  mechanically exhausted.

## Alternatives considered

- **Add a JIT (baseline or tracing).** Rejected for now. Wren — the parity target
  — is itself a pure interpreter; its speed is IC + NaN-boxing + superinstructions,
  none of which need a JIT. A JIT is a large warmup/deopt machinery whose payoff is
  steady-state peak, and Phalcom's start-instant profile (CLI, short scripts) is
  exactly where a tiering JIT loses. Revisit only after the interpreter is
  IC-and-NaN-box-complete and a measured workload still demands peak beyond it.
- **Dispatch-first ordering (rewrite lookup before touching allocation).**
  Rejected as the *default* order. Skynet is allocation-heavy (a `Vec` per send on
  an unbounded heap of 1M fibers); the profile may show allocation dominating
  dispatch. Tier 0 settles the order empirically; the committed sequence front-loads
  the cheap allocation and tracing wins precisely because they may dominate.
- **Ship the optimizations speculatively (skip the harness).** Rejected — this is
  the failure mode the ADR exists to prevent. Without attribution, effort risks the
  wrong mechanism, and without a baseline a "win" is unfalsifiable.
- **Optimize the fiber switch (assume Skynet is switch-bound).** Rejected on
  evidence: the switch is already O(1) `mem::take` (`fiber.rs:29-51`). The cost is
  the surrounding sends. Targeting the switch would be optimizing a non-bottleneck.
- **Pursue full Wren parity unconditionally, NaN-boxing up front.** Rejected as
  premature. NaN-boxing is invasive to land and validate; gating it on a measured
  post-IC shortfall avoids paying that cost if IC + arithmetic fast paths already
  reach the target.

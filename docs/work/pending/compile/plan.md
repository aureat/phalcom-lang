# U-COMPILE — compile-time & startup optimization (Tier 5)

Status: **PLANNED** (dispatch-ready). Tier 5 of the performance strategy
([performance.md](../../../spec/current/performance.md) §4 Tier 5,
[ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)).
Single-writer on the compiler front-end (`chunk.rs`, `compiler/lib.rs`, `vm.rs`
core-load path, and optionally `phalcom-ast`) → **worktree-isolate**; the lexer
sub-change collides broadly with `phalcom-ast` work — serialize
([[phalcom-concurrent-session-hazards]]). **Requires U-BENCH** (P1). Independent of
the runtime tiers (touches compile/startup, not the send hot path), so it may land
in any order among them.

## Role
Cut the cost the user named explicitly — lex/parse/compile time and startup —
**without changing compiled behavior**. The compiled bytecode is byte-identical;
it is produced faster and, for the core, cached. Ordered cheapest-and-highest-value
first; the invasive lexer refactor is last and optional.

## Spec anchor
[performance.md](../../../spec/current/performance.md) §4 Tier 5, law P2. Compile-time
optimization is behavior-invariant by construction (same bytecode out) — no ADR
amendment. The one thing to guard is that dedup/caching does not change what the
compiler emits or the diagnostics/spans it attaches.

## Preconditions (verify on HEAD)
- Confirm `core.ph` is `include_str!`-embedded and re-lexed/parsed/compiled on
  every `VM::new` (`vm.rs:279,309-313`) — no compiled-module cache.
- Confirm `add_constant` (`chunk.rs:27`) unconditionally pushes without dedup, and
  that string literals allocate a fresh heap `Str` at compile time
  (`compiler/lib.rs:2060-2063`) before the non-dedup add.
- Confirm locals/upvalues resolve by linear reverse scan per lookup
  (`compiler/lib.rs:475-477,516-524`).
- Confirm the lexer owns a `String` per identifier (`lexer.rs:288`) and
  unconditionally allocates in `scan_number`'s `replace('_','')` (`lexer.rs:216`).

## Design (ordered: highest-value / lowest-invasion first)
### Change 1 — Compiled-core cache (largest startup win)
Skip re-lex/parse/compile of `core.ph` on every `VM::new`. Options (DEC-COMPILE-A):
in-process memoization (compile once per process, share across `VM::new`), a
build-time-serialized bytecode blob, or a lazy `OnceCell`. The cache **must
invalidate when `core.ph` changes** (a build-time content hash), or a stale cache
silently runs old core — a correctness trap dressed as a perf win.

### Change 2 — `add_constant` dedup + compile-time string dedup
Give the compiler a constant-interning map keyed on constant identity: a repeated
literal / selector / symbol returns the existing index instead of appending a
duplicate (`chunk.rs:27`). Dedup the per-literal compile-time heap `Str` allocation
the same way (one `Str` object per distinct string content, not per occurrence).
Behavior-invariant: the same constants are referenced, the pool is just smaller.
**Guard:** dedup keys on *content*, and `Str` constants are heap handles — dedup by
string content, not handle identity (two equal literals must collapse).

### Change 3 — Hashmap scope resolution
Replace the linear reverse scan in `resolve_local_in` / `add_upvalue`
(`compiler/lib.rs:475,516`) with a per-scope name→slot hashmap, so variable
resolution is O(1) not O(n) per lookup. Behavior-invariant: same slot resolved.

### Change 4 — Lexer allocation cuts (last; the `&str` borrow is optional/invasive)
- **Cheap, non-invasive:** skip the `scan_number` `replace('_','')` allocation when
  the slice contains no `_` (`lexer.rs:216`).
- **Invasive, optional (DEC-COMPILE-C):** borrow `&'input str` tokens instead of
  owning a `String` per identifier (`lexer.rs:288`). This threads a lifetime
  through `Token`/`Lexeme`/the AST — a **large `phalcom-ast` refactor** that
  conflicts with the "non-invasive low-level code" goal. Recommend **deferring**
  this to its own unit unless U-BENCH shows lexing dominating compile time; ship
  only the cheap `scan_number` cut here.

## Write-set (STOP-and-report if outside)
- `phalcom-core/src/vm.rs` — the core-load/cache path (Change 1).
- `phalcom-core/src/chunk.rs` — `add_constant` dedup (Change 2).
- `phalcom-core/src/compiler/lib.rs` — constant-interning map, compile-time `Str`
  dedup, hashmap scope resolution (Changes 2–3).
- `phalcom-ast/src/lexer.rs` — `scan_number` alloc skip (Change 4 cheap part).
  The `&str`-borrow (Change 4 invasive part) touches `token.rs`/`ast.rs`/`parser.rs`
  — **out of this unit's scope unless DEC-COMPILE-C says otherwise; STOP-and-report**.
- **Floor: +0** (no surface change).

## Build order
1. Change 3 (hashmap scope resolution) — self-contained, prove golden-clean.
2. Change 2 (`add_constant` + string dedup) — prove golden-clean **and** that the
   constant pool shrank (a measurable count, not just "looks smaller").
3. Change 4 cheap part (`scan_number` skip) — golden-clean.
4. Change 1 (compiled-core cache) — highest value, but the invalidation logic is
   the riskiest; land last with the content-hash guard. Commit per green step.

## Tests / verification
- **Primary gate = zero golden diff** (I1): the compiled program behaves
  identically. For Change 1, additionally assert that **editing `core.ph` busts the
  cache** (a test that changes core content and confirms the new behavior is picked
  up — the invalidation guard).
- For Change 2, assert the constant pool has **no duplicate entries** for a program
  with repeated literals (a direct count assertion, the behavioral proxy for the
  dedup).
- `cargo build && cargo test && cargo clippy --workspace` green; `cargo doc` clean.
- **Re-run U-BENCH startup timing** — record the `VM::new` / whole-process startup
  delta (Change 1 is the headline number). WORKTREE-VERIFY each SHA.

## Decisions to flag (DEC-COMPILE)
- **DEC-COMPILE-A — core-cache mechanism.** In-process memoization (simplest, wins
  only for multi-`VM::new` in one process) vs build-time serialized bytecode
  (wins the first-startup cost too, but needs a stable bytecode serialization +
  content-hash invalidation, and interacts with U-IC's IC-slot storage — DEC-IC-C).
  Recommend starting with in-process memoization; escalate to a serialized blob only
  if U-BENCH shows first-startup compile cost dominates.
- **DEC-COMPILE-B — constant dedup key.** By `Value` structural equality (collapses
  equal numbers/symbols/strings) — confirm `Str` constants dedup by **content**, and
  that dedup never merges two constants the bytecode relies on being distinct slots.
- **DEC-COMPILE-C — lexer `&str` borrow: now, deferred, or never.** Recommend
  **deferred to its own unit** (large `phalcom-ast` churn, tension with the
  non-invasive goal); ship only the `scan_number` cheap cut here.

## What must this not preclude (P4)
- **Bytecode serialization (future).** If Change 1 uses in-process memoization, the
  door to a serialized core blob must stay open — do not bake per-process pointers
  into the cached chunk. Coordinate the cache format with U-IC's IC-slot storage
  (DEC-IC-C side table) so a serialized chunk and an IC side table compose.
- **Diagnostic spans.** Constant dedup and the `scan_number` change must not corrupt
  source spans (`chunk.rs` parallel `Vec<SourceRange>`); an optimization that
  degrades error locations violates errors-as-UX.
- **The deferred lexer borrow** must remain a clean future unit — do not half-migrate
  token ownership here.

## Return shape (implementer)
commit SHA(s) · core-cache mechanism chosen (DEC-COMPILE-A) + the invalidation
guard + a test proving cache-bust on core edit · constant-pool dedup landed + the
no-duplicate count assertion (DEC-COMPILE-B) · scope-resolution now O(1) · lexer
`scan_number` cut (and confirmation the `&str` borrow was deferred, DEC-COMPILE-C) ·
**confirmation of zero golden diff** · U-BENCH startup-time delta · any `unsafe`
(expect none) · floor delta (exp 0) · verify + `cargo doc` tails · write-set confirm.

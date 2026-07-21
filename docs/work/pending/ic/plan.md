# U-IC — selector-only interner + monomorphic inline cache + superinstructions (Tier 3)

Status: **PLANNED** (dispatch-ready). Tier 3 of the performance strategy
([performance.md](../../../spec/current/performance.md) §4 Tier 3,
[ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)).
Populates the inline-cache seam ADR-0012 already reserves. Single-writer on
`vm.rs` + `class.rs` + `interner.rs` → **worktree-isolate**; serialize against
`U-HOTPATH`, `U-PRIM-ABI`, `U-GC` ([[phalcom-concurrent-session-hazards]]).
**Requires U-BENCH** (P1) and is cleanest **after U-GC** (I3 — the non-moving
collector keeps IC tags valid), though either order is sound since the collector is
non-moving by construction.

## Role
Replace the fixed per-send dispatch tax — an `IndexMap` hash probe walked per
superclass level on **every** send — with a monomorphic inline cache that costs a
class-identity compare on a hit. This is the structural Wren-parity lever, and it
is the **most correctness-sensitive** unit in the roadmap: an inline cache that
misses a hierarchy mutation serves a stale method.

## Spec anchor
[ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) (the
`ClassId`-keyed IC seam — **locked**; this unit *populates* it, does not redesign
it), [performance.md](../../../spec/current/performance.md) §4 Tier 3 + invariant I2,
[method-lookup.md](../../../spec/current/method-lookup.md),
[ADR-0041](../../../adr/0041-hierarchy-stability-policy.md) (what mutations the
cache must invalidate on). Behavior-invariant (P2): identical methods resolved,
faster. No new surface, so no ADR amendment — but the epoch-invalidation contract
(I2) is load-bearing and reviewer-audited.

## Preconditions (verify on HEAD)
- Confirm `lookup_method_in_hierarchy` (`class.rs:65`) still walks `superclass`
  handles doing `methods.get(&selector)` per level, and that no IC is populated
  (only comment stubs at `vm.rs:1578,1630`, `bytecode.rs:92`).
- Confirm `Symbol(u32)` (`interner.rs:10`) is still a **mixed** space
  (vars/fields/selectors) — the reason a raw-`Symbol`-indexed per-class row is
  sparse and a selector-only space is needed first.
- Enumerate the hierarchy-mutation sites that must bump an epoch: class reopen,
  method (re)definition, `superclass=` ([ADR-0041](../../../adr/0041-hierarchy-stability-policy.md),
  the `superclass=` open question). Missing one is the classic IC soundness bug.

## Design
### Change 1 — Selector-only interner (prerequisite)
Carve a dense `SelectorId` space out of the mixed `Symbol` space so a cache/array
can index by selector without massive sparsity. Selectors are interned into their
own contiguous id space at `encode_selector` time; the mixed `Symbol` space is
untouched for vars/fields.

### Change 2 — Monomorphic inline cache, keyed `(ClassId, SelectorId)`
- Each `Invoke` call site carries a cache slot `{ ClassId, resolved ObjRef,
  class-epoch }` (storage per DEC-IC-C). On a send: if the receiver's `ClassId` and
  the class-epoch match the slot, use the cached method — no lookup. On a miss,
  run the full `lookup_method_in_hierarchy`, refill the slot.
- **Underlying storage: per-class own-method arrays + chain-walk (design B)** —
  removes the per-level *hash* while keeping the superclass walk, with **no
  flatten-on-reopen invalidation** (design A, flatten own+inherited, is **rejected**:
  it re-flattens the whole subtree on class reopen / conditional re-parent, fighting
  the dynamic object model). The IC sits on top of B and self-heals on mutation.
- **I2 obligation (mandatory):** every hierarchy mutation (Change-2 preconditions)
  bumps the owning class's epoch, so a stale slot misses and refills. A cache that
  can serve a method removed or overridden after caching is **unsound** — this is
  the unit's primary review gate.

### Change 3 — Operand-free superinstructions
Add `LOAD_LOCAL_0..15` and `LOAD_FIELD_THIS` (Wren uses 0..8; **0..15 for
Phalcom**) folding the operand fetch into the opcode for the hottest local/
receiver-field reads. Additive opcodes — **opcode-budget check first** (`u8` = 256
slots, `bytecode.rs`). Emit them from the compiler where a `GetLocal 0..15` /
receiver-field read is compiled.

## Write-set (STOP-and-report if outside)
- `phalcom-core/src/interner.rs` — the selector-only id space.
- `phalcom-core/src/class.rs` — per-class own-method arrays (design B), the class
  **epoch** field + bump on mutation, and its consumption in lookup.
- `phalcom-core/src/vm.rs` — `Invoke` IC slot check/refill; superinstruction arms.
- `phalcom-core/src/bytecode.rs` — the IC slot representation + the new
  superinstruction opcodes (budget-checked).
- `phalcom-core/src/compiler/lib.rs` — emit superinstructions; thread `SelectorId`.
- `phalcom-core/src/value.rs` — `lookup_method` if it routes through the new path.
- **Floor: +0.**

## Build order
1. Selector-only interner (Change 1) — no behavior change, prove golden-clean.
2. Class epoch + design-B own-method arrays (Change 2 storage), still full-lookup
   on every send — prove golden-clean and epoch bumps on every mutation site.
3. Populate the IC slot + hit/miss path — prove golden-clean **and** correct under
   a reopen/override stress test (see Tests).
4. Superinstructions (Change 3) last — opcode-budget-checked. Commit per green step.

## Tests / verification
- **Primary gate = zero golden diff** (I1) across the full corpus.
- **IC coherence stress (I2) — the load-bearing test:** define a class, send in a
  loop (populate the cache), then at runtime (a) reopen the class and override the
  method, (b) add a method, (c) change `superclass=` if supported — and assert the
  next send resolves the **new** binding, not the cached one. A monomorphic-cache
  bug passes the static corpus and fails only here; this test is mandatory.
- **Megamorphic fallback:** a site hammered with many receiver classes must fall
  back to the dict walk with correct results, no cliff to incorrectness (I2).
- `cargo build && cargo test && cargo clippy --workspace` green; `cargo doc` clean.
  **Re-run U-BENCH** — record the dispatch-tax delta (send micro-bench, Skynet).
  WORKTREE-VERIFY each SHA ([[clean-checkout-verify-each-commit]]).

## Decisions to flag (DEC-IC)
- **DEC-IC-A — epoch granularity.** Per-class epoch (fine-grained, only affected
  subtree invalidates) vs a single global method-dictionary version (simplest,
  invalidates all caches on any mutation). Recommend **per-class**, bumped up the
  affected subtree, to avoid a global cache flush on every method define — but
  global is an acceptable v1 if per-class proves fiddly (measure the mutation
  frequency in U-BENCH).
- **DEC-IC-B — SuperSend IC.** `SuperSend` is currently uncached (DEC-INH-F,
  `DEFERRED.md`). Fold it into this unit or keep it uncached for a follow-on?
  Recommend keep `SuperSend` uncached in v1 (statically-known target, lower
  frequency); flag as a follow-on.
- **DEC-IC-C — IC slot storage.** Inline in the `Invoke` bytecode operand vs a
  side table indexed by instruction offset. Recommend the side table — keeps
  `Bytecode` `Copy`/small and does not couple the cache to bytecode serialization
  (relevant to U-COMPILE's core cache).
- **DEC-IC-D — monomorphic vs polymorphic (PIC).** Ship monomorphic (1 entry) in
  v1; leave the slot shape able to grow to a small poly cache later. Recommend
  monomorphic v1 with a slot layout that does not preclude PIC (P4).

## What must this not preclude (P4)
- **NaN-boxing (Tier 6).** The IC key is `ClassId` (a handle-derived id), **not**
  raw `Value` bits — so a future NaN-boxed `Value` does not change the cache key.
  Keep it that way.
- **Non-moving GC (I3).** The cache stores `ClassId`/`ObjRef`; it is valid only
  while handles are stable. Document that a future *moving* collector must
  invalidate or indirect the IC — the non-moving collector (U-GC) keeps it valid
  for free.
- **Polymorphic upgrade.** The monomorphic slot layout must be extensible to a
  small PIC without a bytecode format change (DEC-IC-D).
- **The dynamic object model.** Design B (chain-walk, no flatten) is chosen
  precisely so class reopen / conditional re-parent stay cheap; do not adopt design
  A under performance pressure.

## Return shape (implementer)
commit SHA(s) · selector-only interner landed · epoch granularity chosen
(DEC-IC-A) + **every mutation site that bumps it** · IC slot storage (DEC-IC-C) +
hit/miss/refill path · SuperSend disposition (DEC-IC-B) · superinstruction range +
opcode-budget headroom (DEC-IC-D) · **confirmation of zero golden diff** + the IC
coherence stress test + megamorphic fallback test · U-BENCH dispatch-tax delta ·
any `unsafe` (expect none) · floor delta (exp 0) · verify + `cargo doc` tails ·
write-set confirm.

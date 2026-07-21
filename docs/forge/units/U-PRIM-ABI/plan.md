# U-PRIM-ABI — in-place primitive stack ABI + arithmetic fast path (Tier 2)

Status: **PLANNED** (dispatch-ready). Tier 2 of the performance strategy
([performance.md](../../../spec/current/performance.md) §4 Tier 2,
[ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)).
Single-writer on `vm.rs` + `primitive/*` → **worktree-isolate**; serialize against
`U-HOTPATH`, `U-IC`, `U-GC` (all single-write `vm.rs`)
([[phalcom-concurrent-session-hazards]]). **Requires U-BENCH first** (P1 — the win
is an allocation cut whose size must be measured).

## Role
Remove the per-send heap allocation from the primitive path and give the hottest
arithmetic its own fast path — attacking §2's *per-send allocation* cost class and
the allocation half of arithmetic dispatch, **without changing any observable
behavior**.

## Spec anchor
[performance.md](../../../spec/current/performance.md) §4 Tier 2, laws P2/P3, invariant
I4. The in-place ABI is behavior-invariant (no ADR). The arithmetic fast path is a
**speculative optimization** and is governed by P3 + I4 (guard-implies-slow-path +
exact deopt); it amends no surface semantics, so no ADR — but it is the part a
reviewer scrutinizes hardest.

## Preconditions (verify on HEAD)
- Confirm the `Primitive` arm of `call_method` (`vm.rs:620,626`) still builds
  `self.stack[recv+1..].to_vec()` per send — the allocation this unit removes.
- Confirm arithmetic still compiles to an ordinary `Invoke` (`compiler/lib.rs:2190`)
  and that only Bool/Block sacred selectors are inlined (`inliner.rs:5`).
- Enumerate the primitive surface that must migrate to the new ABI (every `fn` in
  `primitive/*.rs` reachable through the `Primitive` arm) — the migration is the
  bulk of the work, not the ABI design.

## Design
### Change 1 — In-place stack-window ABI (the allocation cut)
Wren primitives read/write `args[0]` in place and return a status; a hit drops
`stackTop` with **no `CallFrame` push and no args `Vec`** (`wren_primitive.h`,
`wren_vm.c`). Adopt the same:
- A primitive receives a `&mut [Value]` window into the operand stack
  (receiver + args) and returns a status (e.g. `PrimResult { Return, Call, Error }`),
  writing its result into the window's base slot.
- The `Primitive` arm of `call_method` passes the window and adjusts the stack from
  the status — **no `to_vec()`, no frame push** for the native fast path.
- Behavior-invariant: identical values in, identical value/error out; only the
  transient `Vec` and frame push disappear.

### Change 2 — Guarded arithmetic fast path (the dispatch cut for `Number ⊕ Number`)
- For the common binary operators on two `Number`s, emit a fast path (a guarded
  superinstruction, or fold into the Tier 3 IC — see DEC-PRIM-B) that computes the
  `f64` result directly.
- **P3/I4 obligation:** the guard tests both operands are `Number` *and* the
  operator is not overridden on `Number` (epoch/override check), and **deopts to
  the exact generic send** on any miss — non-Number operand, overridden operator,
  subclass receiver. The fast path must produce byte-identical results to the slow
  send on every input, including edge cases (`0.0`/`-0.0`, `NaN`, overflow —
  Phalcom is flat-`f64`, [ADR-0042](../../../adr/0042-flat-number-defer-integer-float-split.md)).

## Write-set (STOP-and-report if outside)
- `phalcom-core/src/vm.rs` — `call_method` `Primitive` arm (window + status
  handling); the arithmetic fast-path emit/guard if it lives in the loop.
- `phalcom-core/src/primitive/*.rs` — migrate each primitive to the window ABI.
  This is the largest surface; keep the signature change mechanical.
- `phalcom-core/src/primitive/number.rs` — the arithmetic fast-path bodies.
- `phalcom-core/src/compiler/lib.rs` + `bytecode.rs` — **only if** Change 2 uses a
  new superinstruction opcode (see DEC-PRIM-B); if so, opcode-budget check required.
- **Floor: +0** (no new surface; existing primitives, faster ABI).

## Build order
1. Introduce the `PrimResult` status + window ABI on **one** primitive end-to-end;
   prove zero golden diff.
2. Migrate the rest mechanically, committing in green batches.
3. Add the arithmetic fast path **last** (riskiest — P3), behind its guard, with
   the equivalence tests. Commit per green step ([[commit-frequently]]).

## Tests / verification
- **Primary gate = zero golden diff** (I1): full lang golden corpus +
  `tests/invariants.rs` byte-identical before/after.
- **Fast-path equivalence (I4):** add goldens proving `Number ⊕ Number` fast path
  equals the slow send on guard-miss inputs — non-Number operand, an overridden
  `+(_)` on Number, a Number subclass — and on `f64` edge cases.
- `cargo build && cargo test && cargo clippy --workspace` green; `cargo doc` clean.
- **Re-run U-BENCH** — record the allocation-cut delta on the arith and fiber
  micro-benches and on Skynet (P1). WORKTREE-VERIFY each SHA
  ([[clean-checkout-verify-each-commit]]).

## Decisions to flag (DEC-PRIM)
- **DEC-PRIM-A — status ABI shape.** `bool` (Wren-style, hit/miss) vs a richer
  `enum PrimResult { Return, Call, Error }`. Recommend the enum — Phalcom primitives
  can re-enter the interpreter (fiber ops) and raise typed errors, which a bare
  `bool` cannot express without side channels.
- **DEC-PRIM-B — arithmetic fast path: superinstruction now vs fold into Tier 3
  IC.** A standalone guarded superinstruction ships the win in this unit but adds
  opcodes; folding into the IC (Tier 3) avoids new opcodes but couples the win to
  `U-IC`. Recommend the standalone superinstruction **iff** U-BENCH shows arithmetic
  dispatch dominating; else defer the dispatch half to U-IC and ship only the
  allocation cut here.
- **DEC-PRIM-C — migration batching.** All primitives in one unit vs core
  arithmetic/comparison first and the long tail in a follow-on. Recommend
  arithmetic + fiber + comparison first (the measured hot set), tail in a follow-on
  if the diff grows unwieldy.

## What must this not preclude (P4)
- The window ABI must not assume a fixed `Value` width — it must keep working when
  `Value` becomes NaN-boxed (Tier 6). Pass the window as `&mut [Value]`, never raw
  bytes.
- It must not block the Tier 3 IC: a primitive hit must still be cacheable by
  `(ClassId, SelectorId)`. Keep the primitive reachable through the same lookup the
  IC will populate.
- The arithmetic guard must not hard-code "Number is never reopened" — it checks
  the override epoch, so a future `Number` extension deopts correctly (I2).

## Return shape (implementer)
commit SHA(s) · the ABI status type chosen (DEC-PRIM-A) · count of primitives
migrated + any left for follow-on (DEC-PRIM-C) · arithmetic fast-path route taken
(superinstruction vs deferred to U-IC, DEC-PRIM-B) + its guard + deopt points ·
**confirmation of zero golden diff** + the fast-path equivalence goldens added ·
U-BENCH allocation-cut delta (arith/fiber/Skynet) · any `unsafe` (expect none) ·
floor delta (exp 0) · verify + `cargo doc` tails · write-set confirm.

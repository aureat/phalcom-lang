# U-HOTPATH — dispatch-loop hot-path optimizations (behavior-invariant)

Status: **PLANNED** (dispatch-ready). Single-writer on `vm.rs` + `value.rs` → **worktree-isolate**;
serialize against any in-flight unit whose write-set touches `vm.rs`/`value.rs`
([[phalcom-concurrent-session-hazards]]). Ports three Wren perf techniques judged *take-now* in the
2026-07-13 Wren analysis; the three v0.3 items (symbol-indexed table, micro-superinstructions,
primitive in-place ABI) are registered in [DEFERRED.md](../../DEFERRED.md), not here.

## Role
Cut per-send and per-instruction overhead in the interpreter **without changing any observable
behavior**. Three independent changes, all guarded by the existing golden corpus + `tests/invariants.rs`
(any output/behavior diff = regression, not a feature).

## Spec anchor — none required (behavior-invariant optimization)
No surface semantics change, so no spec § / ADR is amended (a new ADR is only owed when behavior
changes — `documentation-and-adrs`). **The golden `.ph` corpus and `tests/invariants.rs` ARE the
anchor:** the unit is correct iff every existing golden stays byte-identical. Grounded in Wren
precedent (`wren_vm.c` LOAD_FRAME/STORE_FRAME, `wren_vm.h` `wrenGetClassInline`) + the analysis above.

## Change 1 — Register-hoist interpreter state into loop locals (biggest win)
Wren keeps `ip`/`stackStart`/`fn` in `register` locals, re-syncing only on call push/pop
(`wren_vm.c:832-862`, credited "a large speed boost").

- **Current:** `run_until_inner` (vm.rs:1182) `match opcode` (vm.rs:1208) re-derives the current
  chunk every instruction — e.g. `Invoke` does
  `self.heap.closure(closure_id).callable.chunk.constants[selector_idx]` (vm.rs:1566), a full
  heap-borrow + pointer chase per op.
- **Change:** at frame entry, hoist the current chunk's `code`/`constants` (and `ip`) into loop
  locals; access them directly in the arms; re-sync (LOAD_FRAME/STORE_FRAME analog) only on
  call / return / fiber-switch.
- **Rust design risk (call out in return shape):** a borrowed slice of `self.heap` cannot coexist
  with `&mut self.stack` mutation (borrow checker). Resolve by one of: (a) index-based access with a
  cached `ChunkId`/base index rather than a borrow, (b) `Rc<Chunk>`/raw `*const` snapshot refreshed at
  frame boundaries, or (c) split the heap so chunk code lives in a region not aliased by the value
  stack. Pick the least-unsafe that measures; **no `unsafe` without a reviewer sign-off.**

## Change 2 — Kill String allocation on dispatch-derived-selector paths
Every derived-selector probe currently builds a `String` and re-interns on a semi-hot path:

- `value.rs:146` — `format!("init {}", selector_str)` + `interner.find` in `lookup_method`'s
  class-init fallback (runs for class receivers after the primary miss).
- `vm.rs:1589-1592` — `decode_selector(...)` (allocates a `String` + `Vec` for name/labels) then
  `format!("{name}(*)")` + `intern` for the variadic probe, on every `Invoke` **miss**.
- **Change:** precompute the derived `Symbol`s at class finalization
  (`FinalizeClass` / `install_core`) and store them (e.g. a small per-class map or a memoized interner
  result), so the runtime path does a `Symbol` lookup, not a `String` build. The **cold** `dNU`
  branch (`decode_selector` for `Message` reification) may keep allocating — only the *variadic
  fast-probe* and the *init fallback* must go alloc-free.
- Behavior-invariant: identical selectors resolved, just without the transient allocation.

## Change 3 — Branch-free class-of, common case first
Wren force-inlines `wrenGetClassInline` testing `IS_NUM`/`IS_OBJ` first (`wren_vm.h:209-237`), hit on
every `CODE_CALL`.

- **Current:** `Value::class` (value.rs:95) matches arms in `Nil,Bool,Number,Symbol,Obj` order; the
  `Obj` arm double-indirects (`heap.get` then variant match).
- **Change:** order the match arms most-frequent-first (`Obj`/`Number` are the hot cases), keep the
  fn `#[inline]`. Optionally fast-path `Number`/`Obj` ahead of the singleton arms. No semantic change
  (a `match` is exhaustive regardless of arm order) — purely a codegen/predictor hint. Confirm the
  compiler doesn't already reorder (LLVM may); keep the change only if it measures or is a wash.

## Write-set (STOP-and-report if outside)
- `phalcom-core/src/vm.rs` — `run_until_inner` loop-local hoist, `Invoke` derived-selector probe,
  `call_method` (only if the hoist touches frame push/pop).
- `phalcom-core/src/value.rs` — `Value::class` arm order; `lookup_method` init-fallback alloc removal.
- `phalcom-core/src/class.rs` — **only if** Change 2 stores precomputed derived `Symbol`s on the
  class row (finalize-time). If it needs `universe.rs`/`heap.rs` beyond a field add → STOP-and-report.
- `phalcom-core/tests/` — **no new goldens** (behavior-invariant); optionally a `criterion` dispatch
  micro-bench if a bench harness exists (else skip, note it).
- **Floor: +0** (no primitive, no surface change).

## Tests / verification
- **Primary gate = zero golden diff.** `cargo test` (full lang golden corpus + `tests/invariants.rs`)
  must stay byte-identical before/after. A single changed golden means a behavior leak — investigate,
  don't rebaseline.
- `cargo build && cargo test && cargo clippy --workspace` green; `cargo doc` clean (docs mandatory —
  [[rust-doc-mandatory]]).
- WORKTREE-VERIFY at each commit SHA on a throwaway checkout ([[clean-checkout-verify-each-commit]]) —
  the in-tree gate can hide a partial stage.
- Commit per green change, not one batch ([[commit-frequently]]): Change 3 (trivial) → Change 2 →
  Change 1 (riskiest, last).

## Reviewer
ON — independent `phalcom-reviewer`; writer ≠ approver. Reviewer confirms: **no observable-behavior
change** (goldens byte-identical), the register-hoist re-syncs correctly on **every** frame
transition (call / return / non-local return / fiber-switch — a stale hoisted `ip`/chunk after a
fiber switch is the classic bug), no `unsafe` without justification, and the alloc-removal resolves
the *same* selectors as before (variadic probe + init fallback parity).

## Return shape (implementer)
commit SHA(s) · what was hoisted + the borrow-checker resolution chosen (index / Rc / raw-ptr / heap
split) · frame-transition re-sync points covered (incl. fiber-switch) · alloc sites removed
(value.rs:146, vm.rs:1592) + where derived `Symbol`s are now precomputed · `Value::class` arm-order
change (kept/dropped + why) · **confirmation of zero golden diff** · any `unsafe` introduced (expect
none) · floor delta (exp 0) · verify + `cargo doc` tails · write-set confirm.

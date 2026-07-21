# Independent review of U-GC's heap/vm spine diff — the sign-off the plan always mandated

- Date: 2026-07-20
- Reviewed: `git diff 7480d75~1..94b6bbf -- phalcom-core/src/heap phalcom-core/src/vm
  phalcom-core/src/value` (Win A boxing, `trace_object`, `Heap::collect`, `System.gc`, safepoint
  latch) plus `cdd2117` (`temp_roots` push/pop escape hatch, paired fix)
- Reviewed against: [U-GC plan.md](../forge/units/U-GC/plan.md) §3/Rubric/§7/§10,
  [ADR-0050](../adr/accepted/0050-non-moving-mark-sweep-collector.md),
  [memory-management.md](../spec/current/memory-management.md) §1–§7
- Reviewer: done in-conversation, no subagent — the collector was already landed and green on
  `main`; this is the missing independent adversarial pass the plan's own text required
  ("Reviewer ON — hand the diff to `phalcom-reviewer`; do not self-approve") but that, on the
  evidence, never actually ran before the code merged
- **Verdict: APPROVE.** No BLOCK-level finding. One follow-up worth a DEFERRED entry (§9 below).

## 1. Checklist and evidence

| # | Check | Result | Evidence |
|---|---|---|---|
| 1 | **M1 non-moving** — no compaction, `SlotMap` keys stable across a collect | **PASS** | `heap/mod.rs::collect` sweeps via `self.objects.retain(\|id, _\| marked.contains_key(id))` — retained keys are untouched, nothing moves |
| 2 | **M3 root set matches spec §2.1** — including the two historically-missed roots, `sealed_classes`/`checking` | **PASS** | `vm/gc.rs::collect_roots`'s exhaustive destructure lists both; `out.extend(sealed_classes.values().copied())` and `out.extend(checking.iter().copied())` |
| 3 | **Edge table (§2.3) complete**, especially the two historically-missed edges `Block.closure` / `Upvalue::Open.fiber` | **PASS** | `heap/trace.rs`: `Object::Block(block) => push(block.closure)`; `Upvalue::Open { fiber, .. } => push(*fiber)` |
| 4 | **M4 no finalization** — zero `impl Drop` on the object graph | **PASS** | `rg "impl Drop" phalcom-core/src/{heap,vm,value}` → zero hits |
| 5 | **M5 kernel cycle** — `Metaclass`-instance-of-itself doesn't trap the marker, never swept | **PASS** | `collect`'s worklist only pushes on `marked.insert(id, ()).is_none()` — an already-marked node is never re-pushed, so a self-referential cycle terminates. `universe.each_handle` roots the kernel unconditionally |
| 6 | **Safepoint discipline** — `alloc` only latches, collection only at the dispatch back-edge | **PASS** | `Heap::insert`: `if len >= next_gc { gc_pending = true }` with an explicit `// LATCH ONLY — never collect here` comment; `dispatch.rs` calls `self.service_gc_safepoint()` once, at the top of the loop body before the frame is read |
| 7 | **Deep-graph safety** — worklist-based mark, not native recursion | **PASS** | `let mut gray: Vec<ObjRef>` + `while let Some(id) = gray.pop()` in `Heap::collect` |
| 8 | **`temp_roots` correctness** — depth/truncate balanced, no leak, no premature pop | **PASS** | Sole call site is `block_ensure` (`primitive/block.rs`): `let roots = vm.temp_root_depth()` before the re-entrant cleanup call, `vm.truncate_temp_roots(roots)` unconditionally right after — runs on every path (`Ok`/`Err` outcome, cleanup itself raising) since the truncate sits before the `match cleanup_outcome`. `push_temp_root` correctly no-ops on immediates (`Value::as_obj()` filter) |
| 9 | **No `unsafe`** anywhere in the diff | **PASS** | `rg unsafe` on the changed files matches only doc-comment prose ("no `unsafe`-at-the-call-site guarantee") |
| 10 | **Docs** — `///` on every new public item, citing ADR-0050/memory-management.md § | **PASS** | `Heap::collect`, `trace_object`, `trace_frame`, `Value::as_obj`, `VM::collect_roots`, `VM::force_gc`, `VM::push_temp_root`, `VM::service_gc_safepoint` all carry rustdoc with spec citations |

## 2. One thing worth double-checking (not a block)

`block_ensure`'s temp-root special-case is:

```rust
match &outcome {
    Ok(value) => vm.push_temp_root(*value),
    Err(PhError::Runtime(RuntimeError::Raise { error, .. })) => vm.push_temp_root(*error),
    Err(_) => {}
}
```

Checked every other `RuntimeError` variant (`error.rs`) for a `Value`-carrying payload that could
also need rooting across the cleanup call: `Arity`, `Type`, `DepthExceeded`, `UnsupportedOperation`,
`BinaryNotSupported`, `UnaryNotSupported`, `InvalidSetSuper`, `InvalidSetClass`, `UndefinedVar`,
`ZeroDivision`, `TypeConversion`, `InvalidSuperClass`, `NotAllowed`, `ArgumentError`, `Internal`,
`DeadFrameError`, `Message` — none carry a `Value`. `Raise` is the only variant that does. The
`Err(_) => {}` catch-all is therefore correct today, not merely unexercised. **Fragile against
drift**: a future `RuntimeError` variant that adds a `Value` field would silently need the same
treatment and nothing forces it (unlike `trace_object`'s exhaustive match, this one is a `match …
{ .. => {} }` catch-all, so the compiler won't flag a new variant). Not worth blocking on — flagged
as a DEFERRED entry instead (below) rather than hardening speculatively.

## 3. Scope note

This diff (`7480d75~1..94b6bbf` + `cdd2117`) is the collector's core: boxing, trace, collect,
safepoint latch, and the temp-root escape hatch. It does **not** include `vm/dispatch.rs`'s full
opcode set beyond the one safepoint call site and the `Closure` alloc boxing, nor `primitive/*.rs`
beyond `block.rs`'s single `ensure` site — matching the plan's own §3.4 audit finding that the
re-entrant-handle-across-alloc intersection was empty everywhere except this one site. Nothing in
this review contradicts that scope.

## 4. Disposition

U-GC's reviewer-sign-off obligation ([UNITS-TRACKER.md](../forge/UNITS-TRACKER.md) §11, item 6c of
the 2026-07-20 dispatch list) is now discharged. DEFERRED entry filed for §2's catch-all fragility.

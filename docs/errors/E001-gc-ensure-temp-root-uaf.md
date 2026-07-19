# E001 · `block_ensure` frees the protected block's pending result if the cleanup collects

- **Status:** **FIXED** at `cdd2117` (*"fix(vm): commit temp_roots GC escape-hatch"*) — verified
  2026-07-19: `VM::push_temp_root` (`phalcom-core/src/vm/gc.rs:148`) exists, `collect_roots`
  enumerates `temp_roots`, and `block_ensure` roots both the value and the `Raise` error before the
  cleanup call (`phalcom-core/src/primitive/block.rs:318-319`). Repros A and C and the control now
  run clean, as does the error-carrying path §Defect flagged as ungated. The mechanism is
  depth-and-truncate, not push/pop, so a caller need not count its own pushes; no leak or over-pop
  was constructible under nesting. **The §Defect text below is preserved as written on 2026-07-19
  and its "there is no `temp_roots`" claim is false at HEAD** — read it as the record of the bug,
  not of the tree.
- *Originally:* OPEN — confirmed 2026-07-19 (reproduced under `target/debug/phalcom`, isolated by control)
- **Severity:** blocker — hard crash of a valid program (`dangling ObjRef` panic, defined, not UB)
- **Subsystem:** garbage collector / native-window rooting
- **Related:** [E002](E002-fiber-floor-upvalue-crash.md) (same family — value held live across a boundary the root scan misses)

## Defect

`block_ensure` (`phalcom-core/src/primitive/block.rs:303-322`) runs the protected
block, keeps its result **only in a Rust `outcome` local** — `run_until_inner`
*pops* the drained activation's result off the VM stack before returning it to
native code (`phalcom-core/src/vm/dispatch.rs:495`) — then runs the cleanup block
re-entrantly.

`collect_roots` (`phalcom-core/src/vm/gc.rs:32-110`) enumerates stack / frames /
fibers / universe only. **There is no `temp_roots`** — `push_temp_root` has zero
occurrences in the tree, though [ADR-0050 §7](../adr/accepted/0050-non-moving-mark-sweep-collector.md)
mandates one for exactly this window ("a primitive that holds a freshly allocated
handle across a call that re-enters the interpreter … protects it with
`vm.push_temp_root(h)`"). A collection during cleanup sweeps the pending result;
`block_ensure` returns a stale `ObjRef` → panic at `phalcom-core/src/heap/mod.rs:188`.
Slotmap generational keys (Invariant M6) make it a defined panic, not aliasing UB.

## Reproduction

Panics under `target/debug/phalcom`:

```phalcom
// A — explicit collect
let r = { "fresh" + "string" }.ensure { System.gc }
System.print(r)          // -> panic: dangling ObjRef ObjRef(NNNNv1)  @ heap/mod.rs:188
```

```phalcom
// C — NO System.gc: an allocating cleanup trips the automatic safepoint.
//     Proves the bug is reachable in ordinary code, not just via System.gc.
class Trash {}
let r = { "fresh" + "string" }.ensure {
  var i = 0
  while (i < 9000) { Trash.new(); i = i + 1 }
}
System.print(r)          // -> panic: dangling ObjRef
```

**Control** (isolates the collection as the trigger):

```phalcom
let r = { "fresh" + "string" }.ensure { 0 }
System.print(r)          // -> "freshstring"  (clean)
```

The error-carrying path (`{ throw E.new(_) }.ensure { System.gc }`, where the
`Raise{error}` Value is the unrooted handle) is the same class — **plausible, not
yet independently gated.**

## Why the shipped "zero temp-roots needed" audit missed it

`docs/forge/units/U-GC/IMPL-SPEC-steps-3-5.md:119-139` searched for functions
containing **both an allocation and a re-entrant call**, found only
`bool_if_true`/`bool_if_false`, and concluded the hazard class was empty.
`block_ensure` **allocates nothing** — the hazardous handle is the value
*returned by* the first re-entrant call, held across the *second*. The correct
predicate is **"a native fn holds a handle across a re-entrant interpreter
call,"** allocator-agnostic. Re-run the native-window audit with that predicate:
other chaining primitives (`.on(_)`, `whileTrue`) may share it.

## Doc debt (fix in the same change, per the ADR/STATUS two-way-sync rule)

- `docs/forge/units/U-GC/IMPL-SPEC-steps-3-5.md:137-139` — "do not build `VM::temp_roots`" (refuted by this entry).
- `phalcom-core/src/vm/gc.rs:119-121` — claims automatic safepoint triggering is off until step 4. **False at HEAD** — repro C fires it; `tests/gc.rs:238` `automatic_safepoint_fires` already passes. Half of step 4 shipped; the doc says neither did.
- `docs/spec/v0.2/memory-management.md` §2.1 line 70 — lists a `VM::temp_roots` root that does not exist.

## Fix direction (as recorded before the fix landed — superseded, kept for the record)

Root `outcome` (and the `Raise` error Value) for the cleanup call's duration —
push onto `vm.stack` around the second `block_call` and pop after, **or** land
the ADR-0050 §7 `temp_roots` mechanism and use it here. Then: re-derive from
code, re-run repros A/C + the full suite, add A/C as negative-lane fixtures
(`tests/gc.rs` has **no** native-window-collection test — a standing coverage
gap), correct the three docs above in the same change, commit narrow on `main`.
Re-derive rather than trusting this direction — see the method note in
[README](README.md).

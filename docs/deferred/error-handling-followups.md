# Deferred: error-handling follow-ups (unowned)

Split out of the U-TRACE audit (see [`tracing.md`](tracing.md)). These are the items that
audit surfaced which are **not** in U-TRACE's scope and currently have no owning unit.

Ranked by severity. All file:line references verified 2026-07-19 against `main`.

---

## 1. BLOCKER — `Map`/`Set` reentrant `hash`/`==` invalidates a live slot index

**Severity:** highest item on this list. Failure mode includes *silent* data corruption,
which outranks the panic.

`locate()` sends Phalcom `hash`/`==` reentrantly (via `VM::send_dynamic`,
`primitive/mod.rs:338-369`) to disambiguate same-bucket candidates — i.e. it runs arbitrary
user `.ph` code **while a slot index into the very `MapObject`/`SetObject` being operated on
is still live in scope**.

- `phalcom-core/src/primitive/map.rs:54-66` (`locate`), consumed at `:80`, `:108`, `:137`
- `phalcom-core/src/primitive/set.rs:47-59` (`locate`), consumed at `:110`
- `phalcom-core/src/heap/map.rs:106-108` (`set_value_at`) and `:128-154` (`remove_at`) —
  both doc'd "Panics if `slot` is out of range"; the precondition is **admitted but unenforced**

User code that mutates the same map from `==` triggers `remove_at`'s `swap_remove` plus an
index rebuild. The outer slot index is then stale: either out of range (indexing panic) or —
worse — pointing at a different, swapped-in entry. The second case is memory-safe and
**surfaces no error at all**; the map is simply wrong from then on.

Trigger sketch:

```phalcom
class K {
  hash() { 0 }
  ==(other) {
    m.remove(k2)   // side effect: shrinks + reindexes the SAME map mid-locate()
    true
  }
}
let m = Map.new()
let k1 = K.new()
let k2 = K.new()
m.at(k1, put: 1)
m.at(k2, put: 2)
m.at(k1, put: 99)   // locate(k1) -> == side-effects remove k2 -> stale slot
```

`is_mutable_collection_key` (`primitive/mod.rs:379-384`) does not help: it rejects
`List`/`Map`/`Set` as *keys*, and does nothing about a key's `==`/`hash` mutating a
collection as a side effect.

**Direction (not a ruling):** re-validate or re-`locate` the slot after the reentrant send
returns, and/or make `set_value_at`/`remove_at` fallible (`Option`) rather than
indexing-panic. Detecting same-object structural mutation across a reentrant `hash`/`==`
would be the stronger fix. Needs a decision on which.

---

## 2. `error-handling.md` §6 documents three examples that cannot work today

**Cheapest item here — a doc annotation, not a code change.**

Every non-`Raise` `RuntimeError` is wrapped, on catch, into a **generic base `Error`
instance** (`primitive/block.rs:253-263`, mirrored in `vm/dispatch.rs:370-379`
`capture_error_value`), never the specific subclass. There are no `DeadFrameError` /
`RangeError` / `TypeError` / `ZeroDivision` kernel classes — only `Error`,
`MessageNotUnderstood`, and `CannotYieldAcrossNativeFrame`.

So the worked examples at `docs/spec/v0.2/error-handling.md:143-146` are false:

```phalcom
{ obj.frobnicate() }.on(MessageNotUnderstood) { e => … }   // works (reified via Raise)
{ list[99] }.on(RangeError) { e => … }                     // handler NEVER fires
{ escapedBlock.call() }.on(DeadFrameError) { e => … }      // handler NEVER fires
```

`isA(DeadFrameError)` is always `false` on the wrapped instance, so the error keeps
unwinding uncaught with no compile-time signal that the handler is dead.

This is a **recorded, deliberate deferral** — `docs/forge/units/U-ERR/plan.md:277-281`
explicitly scopes reification of the remaining native variants out. The defect is that the
spec doc was never marked to match, so it currently reads as ground truth.

**Do now:** annotate those three examples as aspirational/unimplemented so the spec stops
lying. **Defer:** the reification itself, which is U-ERR's territory and wants an ADR.

---

## 3. `block_on` rustdoc contradicts `block_on`'s behavior

`phalcom-core/src/primitive/block.rs:226-227` states that any non-`Raise` `Err`
(`DeadFrameError`, a future fiber `abort` payload, …) is *"re-propagated unchanged"* and that
*"`on` catches only `Raise`"*.

The code does the opposite: non-`Raise` errors are wrapped into a synthetic `Error` instance
(`block.rs:257-261`) and run through the **same** `isA` match as a real `Raise`
(`block.rs:272-278`). So `{ 1 + "x" }.on(Error) { e => … }` — a bare `RuntimeError::Type`
from `primitive/number.rs` arithmetic, not wrapped in `Raise` — **is** caught by a catch-all.

The behavior matches `error-handling.md` §6's *intent* ("every built-in failure is an Error
subclass … catchable"). It's the rustdoc that is wrong, and it is wrong in the direction that
misleads anyone reading the source as ground truth for what `on` catches.

**Do now:** fix the comment. Cheap, no behavior change.

---

## Not recorded here

The GC `ensure` temp-root UAF and the F1 fiber-floor upvalue-close crash are already tracked
elsewhere. They appear in [`tracing.md`](tracing.md) only as a **sequencing hazard**:
`runtime_error` dereferences `heap.closure(frame.closure)` while walking frames, so a stale
`ObjRef` from either bug turns the traceback itself into the crash site (panic at
`heap/mod.rs:188`, "dangling ObjRef") — converting a clean error into a panic. Fix those
before the traceback runs on every error path.

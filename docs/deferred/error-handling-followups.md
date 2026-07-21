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

**Reproduced 2026-07-20, both modes** (see
[`../spec/current/traceback/verification-2026-07-20.md`](../spec/current/traceback/verification-2026-07-20.md)
§2c). Clarification the original phrasing undersells: the panic mode is `map_raw_get`'s raw
`.expect("slot from locate() is live")` at `primitive/map.rs:80` — a **Rust process abort,
uncatchable by `on(_)`** — not a catchable `RuntimeError`. The corruption mode reproduced as: a
key silently removed, a neighboring key's value overwritten with the value destined for the
removed key, `size` shrunk, exit 0. The naive trigger sketch above recurses into PDR-0007's
native-reentrancy limit; a reentrancy-guard flag in `==` is needed to reach the actual defect.

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

**RULED 2026-07-20 — reentrancy lock, raise at the mutation site.** The collection is flagged
for the duration of `locate()`'s reentrant `hash`/`==` sends; structural mutation of it while
flagged raises a catchable `Error` with `kind: #concurrentMutation` **at the mutation call**
inside the user's `==`/`hash` — precise blame, both failure modes eliminated. Fallible raw
accessors (`set_value_at`/`remove_at` → `Option`) land underneath as defense-in-depth. The
language rule this states: *a key's `hash`/`==` may not structurally mutate the collection
being operated on* — an extension of the existing key/collection contract
(`is_mutable_collection_key`, collection law 4). Rejected: re-locate (order-dependent
semantics, livelock bound), version-counter-at-outer-op (blame one frame removed),
fallible-accessors-alone (leaves silent corruption). Unit scheduling:
[`../spec/current/traceback/plan.md`](../spec/current/traceback/plan.md) §G0 — ranked above the traceback
track. Scope: `locate()` only; iteration-during-mutation stays a separate question.

---

## 2. `error-handling.md` §6 documents three examples that cannot work today

**Cheapest item here — a doc annotation, not a code change.**

Every non-`Raise` `RuntimeError` is wrapped, on catch, into a **generic base `Error`
instance** (`primitive/block.rs:253-263`, mirrored in `vm/dispatch.rs:370-379`
`capture_error_value`), never the specific subclass. There are no `DeadFrameError` /
`RangeError` / `TypeError` / `ZeroDivision` kernel classes — only `Error`,
`MessageNotUnderstood`, and `CannotYieldAcrossNativeFrame`.

So the worked examples at `docs/spec/current/error-handling.md:143-146` are false:

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

**DONE 2026-07-20:** examples annotated in `error-handling.md` §6 with the ruled `e.kind`
replacement form. The reification question itself is **closed by PDR-0010 §2** (ratified
2026-07-20): `kind` Symbol on `Error`, no kernel classes minted; lands with traceback plan
units T3/T6. This item is resolved.

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

**DONE 2026-07-20:** rustdoc rewritten to match the wrap-and-probe behavior — including the
second stale bullet (non-matching `Raise` "frames untouched", false since PDR-0007 moved the
unwind before the probe). This item is resolved.

---

## 4. No compile-error renderer exists — every compile-error span is carried and dropped

**Added 2026-07-20**, surfaced by U-CLASSCLOSE's two-span diagnostic work (PDR-0002). Appended
rather than inserted to avoid renumbering items a concurrent session may be citing — **by severity
it belongs second**, above items 2 and 3: it is a user-facing defect across every compile error,
not a doc or comment fix.

**Compile-error spans never reach the user.** `cmd_run` calls
`vm.compile_closure(module, &source)?` (`phalcom-core/bin/phalcom/cli.rs:160`); the `?` propagates
the `CompilerError` through `anyhow::Result` to `main`, which prints its `Display` text and
nothing else. So the `SourceRange` carried by every span-bearing `CompilerError` variant —
`DestructuringWithoutInitializer`, `ConstructStaticCollision`, `BreakOutsideLoop`,
`ContinueOutsideLoop`, `ThrowNonError`, and now `class.already_defined`'s pair — is **carried and
dropped**. The user gets a message with no file, no line, and no caret.

**Only parse errors render.** `print_parse` fires from `compile_closure`'s `map_err`
(`phalcom-core/src/interpret.rs:145`), which is the parse path only. So Phalcom renders a source
frame for a syntax error and nothing for a compile error — an inconsistency no one chose.

**miette is unused repo-wide.** `use miette` / `miette::` appears in **zero** `.rs` files, despite
miette being a declared workspace dependency and `CLAUDE.md` naming "thiserror + miette" as the
convention. `CompilerError` derives `thiserror::Error` only. **Any doc or decision that says
"rendered as miette labels" is describing something that does not exist** — PDR-0002 did, and was
amended for it (`bb4f365`).

This is the same shape as [`tracing.md`](tracing.md)'s finding, one layer up: the renderer exists
(`phalcom-core/src/diagnostics.rs`, `color_print`-based `print_line_information` — caret span plus
one line of context either side), and the call site doesn't reach it.

**Consequence for anyone adding a diagnostic:** adding a span to a compiler error is *not* a
user-visible change on its own. Adding one costs a struct field; making it appear costs wiring
this path. Price them separately. U-CLASSCLOSE hit this directly — PDR-0001 ruling 2 asked for
both spans of a duplicate-class error, and the shipped compromise puts both **locations in the
message text** (*"first declared at 3:1"*) precisely because there was nothing to render into.
That variant already carries both `SourceRange`s, so literal two-label rendering later is a pure
rendering change with no re-derivation.

**Why it wants its own unit, not a bolt-on.** Wiring `cmd_run`'s compile-error path changes how
**every** compile error prints, and the negative-lane golden harness asserts on
`format!("{stdout}\n{stderr}")` as a **substring** (`phalcom-core/tests/support/mod.rs`). Every
negative sidecar in the corpus — ~90 files, each currently exactly one line — would need
re-checking against the new output. That is a gated migration, not an afternoon. It would also
incidentally revive the five dead spans above, which is the actual prize.

---

## Not recorded here

The GC `ensure` temp-root UAF and the F1 fiber-floor upvalue-close crash are already tracked
elsewhere. They appear in [`tracing.md`](tracing.md) only as a **sequencing hazard**:
`runtime_error` dereferences `heap.closure(frame.closure)` while walking frames, so a stale
`ObjRef` from either bug turns the traceback itself into the crash site (panic at
`heap/mod.rs:188`, "dangling ObjRef") — converting a clean error into a panic. Fix those
before the traceback runs on every error path.

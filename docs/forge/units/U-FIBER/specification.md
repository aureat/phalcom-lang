# U-FIBER — Specification: cooperative `Fiber` (bare) on the restricted re-entrant loop

> **Status:** Normative surface (deepens the ratified spec). Unit-scoped, full-detail
> specification for U-FIBER. It **extends** [`concurrency.md`](../../../spec/current/concurrency.md)
> §1 (Draft 0.1, execution model ratified by
> [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)) — that
> document stays the surface index; this one adds the `FiberObject` state machine, the
> operational semantics of `call`/`yield`/`try`/`abort` (the O(1) switch + the typed
> `ControlFlow` signal), the restricted-yield guard at the bytecode level, the
> failure/fiber-floor unwind, the error surface, cross-feature seams (the
> `for`-generator seam with [[U-ITER]](../U-ITER/specification.md); the
> `yield`/resumer/result-slot seam with the later [[U-FUTURE]](../U-FUTURE/specification.md)),
> worked examples, and conformance points.
>
> **Governing sources.** [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)
> §1–§7 (the execution model — the whole thing); [`concurrency.md`](../../../spec/current/concurrency.md)
> §1 (the surface); [`forward-compat.md`](../../../spec/current/core/forward-compat.md) §7
> (the code-grounded foreclosure audit, D1–D7 + the re-entrant-loop finding);
> [ADR-0009](../../../adr/0009-handle-arena-heap.md) (arena heap — `Object::Fiber`, no
> native fiber stacks); [ADR-0010](../../../adr/0010-tagged-value-enum.md) (tagged
> `Value` — no `Value::Fiber` arm); [ADR-0013](../../../adr/0013-closure-upvalues-and-frame-token-return.md)
> (frame-token non-local return, fiber-local by construction);
> [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md) (one unwind primitive —
> the seam the fiber-floor capture layers over); [ADR-0033](../../../adr/0033-amend-fiber-execution-trampolined-block-callsite.md)
> (the Deferred `.each { yield }` lift — this unit only *builds the switch signal* it
> depends on).
>
> **Ratified constraints honored verbatim (2026-07-12).** v0.2 ships **bare `Fiber`
> only** — `new`/`call`/`try`/`yield`/`current`/`abort`; **no `Future`, no scheduler, no
> `System` hooks** (those are [[U-FUTURE]](../U-FUTURE/specification.md), a **Deferred**
> post-v0.2 track). Fiber switch uses a **typed `ControlFlow` signal**, never a
> `frames.len()` delta (ADR-0030 §5). `next_frame_generation` **stays VM-global**
> (ADR-0030 §6, D4). **No `Value::Fiber` arm** — `Object::Fiber` via `Value::Obj(ObjRef)`
> (ADR-0030 §2, D2). ADR-0033 stays **Deferred** — U-FIBER does not trampoline the block
> call-site. **Zero `unsafe`** (the whole point of Option A over stackful C).

---

## 1. Surface

*Deepens [`concurrency.md`](../../../spec/current/concurrency.md) §1.* `Fiber` is an
independently-suspendable call stack — the **sole** concurrency primitive. Both the
instance and class sides are ordinary metaclass methods (ADR-0030 §Consequences, D6).

### 1.1 Interface (v0.2 bare set)

| Signature | Side | Meaning |
|---|---|---|
| `@constructor
new(_)` | class | wrap a [`Function`](../../../spec/current/functions.md) as a not-yet-started (`suspended`) fiber |
| `call` / `call(_)` | instance | resume; the argument becomes the value of the suspended `yield` (or the entry's parameter on first resume). Returns the next yielded/returned value. **Re-raises** if the fiber fails. |
| `try` / `try(_)` | instance | like `call`, but a failure yields the captured `Error` value (or `None`) instead of propagating |
| `isDone` | instance | `true` once `done` or `failed` |
| `error` | instance | the captured `Error` as `Option`, if `failed` (`None` otherwise) |
| `yield(_)` | **class** | suspend the *current* fiber, handing the value to its resumer. Returns the value passed to the next `call` |
| `current` | **class** | the fiber now running |
| `abort(_)` | **class** | raise an `Error` out of the current fiber to its resumer (fails the current fiber) |

`Fiber.yield`/`current`/`abort` are **class-side** because they always act on the
*running* fiber — you cannot yield/abort a named fiber (ADR-0030 §1; concurrency.md §1).

### 1.2 The canonical generator (ADR-0030 §4)

```phalcom
let counter = Fiber.new {
  let n = 0
  while (true) { Fiber.yield(n); n = n + 1 }
}
counter.call()   // 0
counter.call()   // 1
counter.call()   // 2
```

The `while (true)` lowers to an **inlined** `Jump`/`Loop` skeleton (ADR-0018) — no frame
push, no native frame — so this generator suspends freely (§3).

---

## 2. `FiberObject` — structure and state machine

### 2.1 Structure (ADR-0030 §2, D2)

A `FiberObject` is **one arena variant** (`Object::Fiber(FiberObject)` in `heap.rs`),
reached through `Value::Obj(ObjRef)` exactly as native `List` is — **no new `Value::Fiber`
arm**. It owns:

- **`stack: Vec<Value>`** — its own operand stack (not the caller's);
- **`frames: Vec<CallFrame>`** — its own call-frame stack. Because `CallFrame.stack_offset`
  is **frame-relative** (`frame.rs:75`, D3), a per-fiber stack starting at 0 needs no
  rebasing;
- **`status`** — the state machine below;
- **`resumer: Option<ObjRef>`** — the fiber to hand control back to on `yield`/return/
  failure (a *dynamic caller chain*, not a fixed parent);
- **a result slot** — the last yielded/returned `Value`, or the captured `Error`;
- **the entry closure** — the `Function` the fiber runs when first resumed.

> **Keep the resumer link and result slot general — not generator-specific.** The later
> [[U-FUTURE]](../U-FUTURE/specification.md) layer suspends `await` through *exactly*
> these two fields (`await` = "add `current` to the future's waiters, then
> `Fiber.yield`"). This is the [[U-FUTURE]](../U-FUTURE/specification.md#the-fiber-seam)
> seam — do not special-case them for the generator idiom.

### 2.2 Status state machine

```
                    Fiber.new(fn)
                         │
                         ▼
                    ┌─────────┐   call/try (resumer := caller; caller→suspended)
                    │suspended│ ───────────────────────────────────────────────┐
                    └─────────┘                                                 ▼
                         ▲                                               ┌──────────┐
       yield(v)          │  yield(v): result:=v; current:=resumer        │ running  │
   (result:=v,           └───────────────────────────────────────────── │(= current)│
    caller resumes) ◄──────────────────────────────────────────────────  └──────────┘
                                                                            │  │  │
                    entry returns v ──────────► ┌──────┐                    │  │  │
                    (result:=v, resumer resumes)│ done │ ◄──────────────────┘  │  │
                                                └──────┘                        │  │
                    entry raises e / abort(e) ─► ┌────────┐                     │  │
                    (result:=e captured,         │ failed │ ◄───────────────────┘  │
                     resumer resumes)            └────────┘                        │
                                                                                   │
                    (any further call/try on done|failed → error) ◄────────────────┘
```

**Transition rules (normative):**

1. **`new`** → `suspended`, `resumer = None`, entry stored, `frames`/`stack` empty.
2. **`call`/`try`** on a `suspended` fiber: set `resumer := currentFiber`; the resumer
   goes `suspended`; the callee goes `running` and becomes `current` (§4.1).
3. **`yield(v)`** (running fiber): `result := v`; the fiber goes `suspended`; `current :=
   resumer`; the resumer goes `running` and receives `v` as the value of its `call` (§4.2).
4. **entry returns `v`**: the fiber goes `done`, `result := v`; `current := resumer`; the
   resumer receives `v` (a returned value is indistinguishable at the call-site from a
   yielded one — the caller detects end-of-iteration via `isDone`).
5. **entry raises `e` / `abort(e)`**: the fiber goes `failed`, `result := e` (captured);
   `current := resumer`; under `call` the resumer **re-raises** `e`, under `try` it
   **receives** `e`/`None` (§5).
6. **`call`/`try` on a `done` or `failed` fiber** → a runtime error (you cannot resume a
   finished fiber). *(Exact error spelling: implementer sub-decision D-FIB-2, §implementation-spec.)*
7. **`yield`/`abort` with no `resumer`** (the root/main fiber, which is `running` at
   top level and has no resumer) → a runtime error — "cannot yield the root fiber."

**The root fiber** is the main program; it is `suspended` only while a callee runs. It is
never `done`/`failed` under normal termination and has no resumer.

---

## 3. Execution model — restricted yield (Option A)

*Deepens [`concurrency.md`](../../../spec/current/concurrency.md) §1 "Execution model";
grounded in [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §4
and [`forward-compat.md`](../../../spec/current/core/forward-compat.md) §7.2.*

### 3.1 The crown-jewel hazard — native-stack frames ⊗ suspendable control {#the-crown-jewel}

The VM dispatch loop is **not a flat trampoline**. Pure Phalcom→Phalcom sends are
trampolined — `call_method`'s Closure arm pushes a `CallFrame` and the single `run_until`
loop drains it with no native recursion. But **every path where a Rust primitive needs a
synchronous `Value` back from Phalcom re-enters `run_until` recursively, growing the
native Rust stack** (`forward-compat.md` §7.2):

- `block_call` → `vm.run_until(base_frames)` (`primitive/block.rs:153`);
- `send_dynamic`/`perform` (`vm.rs:595`), `forward_does_not_understand` (`vm.rs:558`);
- transitively, **every combinator that calls a block** — `List.each`, `map`, `reduce` —
  because they bottom out in `block_call` (`core.ph` `each` → `f.call(self.at(i))`).

When the running fiber is *inside* such a primitive, native Rust frames sit between the
fiber's entry and the `Fiber.yield` call site, and those frames **are** the fiber's
suspended position — you cannot repoint `current` and return through them without
destroying it.

### 3.2 The restriction (ADR-0030 §4) {#restricted-yield}

`Fiber.yield` integrates with the **top-level** `run_until` only. If a yield is attempted
while a re-entrant native primitive sits between the fiber floor and the yield, the VM
raises a **catchable `CannotYieldAcrossNativeFrame`** rather than corrupting the
suspended position.

| | Shape | Result |
|---|---|---|
| **✅ Suspends freely** | pure sends + **inlined** control flow (`while`/`ifTrue:` → `Jump`/`Loop`, one chunk) — the `counter` generator (§1.2); and, once [[U-ITER]](../U-ITER/specification.md) lands, `for (x in coll) { Fiber.yield(x) }` | yield reaches the top loop |
| **✗ Foreclosed** | the **callback generator** `Fiber.new { list.each { x => Fiber.yield(x) } }` — `yield` under `each`'s native `block_call` | `CannotYieldAcrossNativeFrame` |

**This is a guard, not a wall.** The residue (`.each { yield }`, a stored-block generator,
a user native combinator that yields) is the **Deferred**
[ADR-0033](../../../adr/0033-amend-fiber-execution-trampolined-block-callsite.md) lift —
de-recursing the block call-site (Option B), purely additive, breaking no program that ran
under A, to land *with* the typed switch signal U-FIBER builds (§4.3). U-FIBER **does not**
implement ADR-0033. The common generator ergonomic is delivered for v0.2 by
[[U-ITER]](../U-ITER/specification.md)'s `for` (ADR-0035), which lowers to an inlined
`while` and suspends freely — see the seam at §7.1.

---

## 4. Operational semantics

### 4.1 The O(1) fiber switch (ADR-0030 §3, D3)

The VM's "current stack / current frames" relocate behind a **`current: ObjRef`** pointer
into the running `FiberObject`. `call`/`yield` swap **which fiber the dispatch loop reads**
— never copying stacks. Concretely (spec-level; exact bookkeeping is
implementation-spec §3):

- On resume (`call`), the resumer's live `stack`/`frames` are stored back into its
  `FiberObject`, `current` is repointed to the callee, and the callee's `stack`/`frames`
  become the VM's live vectors. The transferred argument lands where the callee's `yield`
  (or its entry parameter) expects it.
- Because `stack_offset` is frame-relative (D3), per-fiber stacks starting at 0 need **no
  rebasing** — the switch is a pointer swap, O(1).

### 4.2 `call` / `yield` reconcile through a TYPED signal, not a length delta (ADR-0030 §5, D5) {#the-typed-signal}

Today the `call_method` **Primitive arm** detects a non-local return by a `frames.len()`
**shrink heuristic** (`vm.rs:442-469`: snapshot `frames_before`, run `native_fn`, then
`if self.frames.len() >= frames_before` → ordinary return, else a `ReturnNonLocal` fired
inside). **A fiber switch also moves `frames.len()`** (different fiber, different depth) —
conflating the two would misread a swap as a return.

U-FIBER therefore replaces the length heuristic with an **explicit typed `ControlFlow`
value** out of the primitive. `Fiber.call`/`Fiber.yield` return a **switch** variant;
`Bytecode::ReturnNonLocal` continues to signal a non-local return; an ordinary primitive
returns its value. The dispatch loop honors each distinctly:

- **switch** → the loop repoints to `current` and resumes it at its saved `ip`
  (delivering the transferred value at the resumer's call-site slot);
- **non-local return** → the eager fiber-local unwind (`ReturnNonLocal`, unchanged);
- **ordinary return** → land the value in the receiver slot (unchanged).

`call`/`yield` **do not start nested `run_until`s** — they return the switch to the
*existing* top-level loop, which is exactly what "integrate with the top-level loop only"
(§3.2) means. This is the distinct **third cause** `forward-compat.md` §7.3 told every
pre-fiber unit to leave room for, and the exact dependency **ADR-0033** waits on.

### 4.3 The `Yield` opcode + the restricted-yield guard (ADR-0030 §4) {#yield-guard}

A `Yield` opcode is added (its disasm needs no arm — the disassembler prints via derived
`Debug`, `bin/phalcom/disasm.rs:18`). `Fiber.yield(_)` (class-side) drives it. The guard:

- **Legal** iff the yield executes directly under the fiber's **floor** `run_until` — i.e.
  no re-entrant native `run_until` (a `block_call`/`send_dynamic`/`forward_dnu`) sits
  between it and the floor. Detection is a **native re-entrancy marker** the VM records at
  each fiber resume and checks at yield (exact representation — a depth counter vs. a
  sentinel — is implementer sub-decision D-FIB-3).
- **Illegal** → raise `CannotYieldAcrossNativeFrame` (a catchable surface `Error`; §6).

**Keep the restriction lift-by-deletion** (ADR-0030 §Consequences): do not bake the guard
anywhere it cannot later be removed by simply deleting it (the additive A→B path for
ADR-0033).

### 4.4 Non-local return stays fiber-local for free (ADR-0030 §6, D4)

Once `self.frames` is the *current fiber's* vector, `ReturnNonLocal` (`vm.rs:1154`)
searches only that fiber: `self.frames.get(token.frame_index)` with a `generation` check
(`vm.rs:1174-1177`). A token whose home is on **another** fiber fails the check →
`DeadFrameError` (`vm.rs:1179`) — exactly concurrency.md §3's "`return` across a fiber
boundary raises `DeadFrameError`." **This falls out with no new code**, provided the
**load-bearing invariant** holds:

> **`next_frame_generation` MUST stay VM-global** (`vm.rs:72`), never relocated into
> `FiberObject` (ADR-0030 §6, D4). A per-fiber counter would let fiber B's
> `(frame_index, generation)` collide with a live frame in fiber A → a silent non-local
> return into the **wrong fiber**. The global monotonic counter is the only thing making
> a cross-fiber token globally non-matching.

---

## 5. Failure and the fiber-floor unwind

*Grounded in [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §6,
[`forward-compat.md`](../../../spec/current/core/forward-compat.md) §7.1 D7,
[ADR-0008](../../../adr/0008-layered-exceptions-and-result.md), and the landed U-CORE-6
unwind.*

A fiber that raises (via `throw`→`raise`, `abort(_)`, or any `RuntimeError`) must **capture
its `Error` into its result slot and resume its resumer — never terminate the host**
(concurrency.md §1 pt 4; the §2 hazard "do not special-case `throw` as host-process
termination").

### 5.1 DEC-FIB-A — the landed U-CORE-6 unwind is NOT floor-parameterised {#dec-fib-a}

**Verified against HEAD (the live-risk resolution of DEC-FIB-A).** The U-CORE-6 raise
(landed at commit `85c4e1d`) returns `Err(RuntimeError::Raise { error, rendered })`
(`error.rs:86-94`) which propagates **via Rust `?` through the ordinary `PhResult`
channel**. `run_until` (`vm.rs:818`) drains to `base_frames` **only on a normal `Return`**
(`vm.rs:820-826`, `:1149-1151`); it has **no error catch** and does **not** truncate
`self.frames` on `Err` — the error rides Rust's own stack unwinding straight out of every
nested `run_until` to the top-level driver (`run` → exit 70, `interpret.rs`). The only
"floor" in the code is the *normal-return* drain count; **there is no fiber floor and no
error-catch point.**

**⟹ U-FIBER OWNS THE FIX (verify-on-HEAD confirmed).** No modification to U-CORE-6's
`error_raise`/`RuntimeError::Raise` is required. Instead, at the **fiber-resume boundary**
— the `run_until` the `call`/`try` primitive owns for the fiber body — U-FIBER must:

```
result = run_fiber_body(fiber)               # a run_until scoped to this fiber's frames
match result:
    Ok(v)  → fiber.status = done;   fiber.result = v;   resume_with(resumer, v /*value*/)
    Err(PhError::Runtime(RuntimeError::Raise { error, .. })) →
             fiber.status = failed; fiber.result = error
             fiber.frames.clear()             # abandon the fiber's frames (never popped on Err)
             under call → re-raise error to the resumer
             under try  → deliver error/None to the resumer
    Err(other terminal RuntimeError) → same capture path, wrapping `other` as a surface Error
```

The **fiber floor is exactly this Rust-level `run_until` boundary** that the resume
primitive owns. Because errors unwind native frames correctly via Rust `?` (unlike
`yield`, which cannot), the capture works even when a re-entrant `block_call` was on the
stack when the error fired — the error simply unwinds those native frames back to the
fiber floor, where U-FIBER catches it.

> **⚠ BLOCKED / verify-on-HEAD at dispatch.** Re-read the landed U-CORE-6 unwind on HEAD
> before Phase 1. If a later change introduces a floor-parameterisable stop-point in
> `run_until`, U-FIBER's catch may simplify to reusing it. As of HEAD `9d3b7e1`, no such
> parameter exists — **U-FIBER owns the fiber-floor capture** (implementation-spec §0
> Phase 0 D7).

### 5.2 `try` vs `call` on failure

- **`call`** on a failing fiber **re-raises** the captured `Error` into the resumer's
  unwind (the resumer may itself `try`/`catch` it — U-CORE-6 unwind).
- **`try`** on a failing fiber **does not propagate** — it returns the captured `Error`
  value (or `None`), leaving the resumer running. This is the seam the resumer uses to
  keep the host alive across a callee failure (§9 C-FIB-4).

---

## 6. Error surface

| Situation | Error | Catchable? | Where |
|---|---|---|---|
| `Fiber.yield` under a native `block_call`/re-entrant primitive | **`CannotYieldAcrossNativeFrame`** (a surface `Error`) | **yes** (ordinary U-CORE-6 unwind / `try`) | §4.3 guard |
| non-local `return` whose home frame is on another fiber | **`DeadFrameError`** | yes | `ReturnNonLocal` (§4.4) |
| `call`/`try` on a `done`/`failed` fiber | runtime error (D-FIB-2) | yes | resume primitive (§2, rule 6) |
| `yield`/`abort` on the root fiber (no resumer) | runtime error | yes | §2 rule 7 |
| fiber entry raises / `abort(_)` | captured into the fiber's result slot; `call` re-raises, `try` delivers | yes | §5 |

`CannotYieldAcrossNativeFrame` is a **real, catchable `Error`** users can hit — the spec
documents the restriction and the `for`-loop workaround (§7.1). It is **never** a Rust
`panic!` or host abort (the "every panic on input is a robustness bug" posture).

---

## 7. Cross-feature interactions

### 7.1 `Fiber` ⊗ `for` — the generator seam (with [[U-ITER]](../U-ITER/specification.md))

*Cross-links [[U-ITER §7.1]](../U-ITER/specification.md#fiber-generator-seam),
[ADR-0035](../../../adr/0035-iteration-protocol-cursor.md) §5, ADR-0030 §4.*

The cursor protocol needs **no** `Fiber`; conversely, `for` is the idiomatic v0.2
generator body because it lowers to an inlined `while` (no `block_call`) and so **suspends
freely** where `.each { yield }` cannot:

```phalcom
Fiber.new { for (x in coll) { Fiber.yield(x) } }   // ✅ inlined while — suspends
Fiber.new { coll.each { x => Fiber.yield(x) } }    // ✗ CannotYieldAcrossNativeFrame
```

U-FIBER and U-ITER are **independent to build** (either order); the interaction is proven
by U-ITER's PENDING `for_generator_suspends` fixture, which graduates to PASS once **both**
land ([[U-ITER §9 C-ITER-8]](../U-ITER/specification.md#9-conformance-points-machine-checkable),
§9 C-FIB-6 here).

### 7.2 `Fiber` ⊗ `Future` — the resumer/result-slot seam (with [[U-FUTURE]](../U-FUTURE/specification.md)) {#future-seam}

*Cross-links [[U-FUTURE §The Fiber seam]](../U-FUTURE/specification.md#the-fiber-seam);
[`concurrency.md`](../../../spec/current/concurrency.md) §2; ADR-0030 §1.*

`Future`/`async`/`await` are a **Deferred post-v0.2 track** ([[U-FUTURE]](../U-FUTURE/specification.md)),
**not** built here. But U-FIBER must leave them layerable: `await` will be defined as "add
`current` to the future's waiters, then `Fiber.yield` to the scheduler," so the
**`resumer` link and the result slot must stay general** (§2.1), not generator-specialized.
`Future` adds **no** VM mechanism beyond `Fiber` + a ready-queue (ADR-0030 §Consequences).

### 7.3 `Fiber` ⊗ GC roots (ADR-0030 §7, D1)

**Invariant to encode now, even pre-GC:** a reachable, non-`done`/`failed` `FiberObject`'s
`stack` and `frames` are **GC roots** — **not only** the `current` fiber's. A collector
that scanned only `current` would free objects held solely by a parked fiber. Keeping the
stacks **inside the arena object** (§2.1) is what lets a future collector reach them; **do
not stash fiber stacks in native memory.** No native fiber stacks means nothing new for a
moving collector to scan — ADR-0009's moving-ready arena claim is preserved intact.

### 7.4 `Fiber` ⊗ the metaclass tower (ADR-0030 D6)

`Fiber` is an ordinary heap class; `yield(_)`/`current`/`abort(_)` are class-side
(metaclass) methods like any other. It must pass `verify_invariants()` (the parallel rule)
at bootstrap (`universe.rs:485`) — no apex/tower change.

---

## 8. Worked examples

### 8.1 Counter generator (ADR-0030 §4 canonical) — §1.2 above.

### 8.2 Resume value

```phalcom
let echo = Fiber.new { let x = Fiber.yield(0); Fiber.yield(x + 1) }
echo.call()     // 0            (runs to the first yield)
echo.call(10)   // 11           (10 becomes the value of the first yield; yields 10+1)
```

### 8.3 Failure capture — host survives

```phalcom
let bad = Fiber.new { throw SomeError.new("boom") }
let e = bad.try()          // e = the captured Error (not re-raised)
System.print(bad.isDone)   // true
System.print("host still running")   // prints — the host was NOT terminated
```

### 8.4 Restricted-yield guard (negative)

```phalcom
let g = Fiber.new { [1, 2].each { x => Fiber.yield(x) } }
g.call()   // raises CannotYieldAcrossNativeFrame (catchable) — rewrite with `for`
```

### 8.5 Fiber-local non-local return

```phalcom
// A block whose home method lives on fiber A, invoked on fiber B, that does a
// non-local `return`, raises DeadFrameError (not a cross-fiber unwind). See §4.4.
```

---

## 9. Conformance points (machine-checkable)

| ID | Requirement | How verified |
|---|---|---|
| **C-FIB-1** | `counter` generator (§1.2): successive `call`s yield `0,1,2,…`. | golden |
| **C-FIB-2** | resume value (§8.2): the argument to `call(_)` becomes the value of the suspended `yield`. | golden |
| **C-FIB-3** | restricted-yield guard (§8.4): `.each { yield }` raises a **catchable** `CannotYieldAcrossNativeFrame` (not a host abort). | negative golden |
| **C-FIB-4** | failure capture (§8.3, D7): a fiber that `throw`s ends `failed`, `try` yields the `Error`, the host keeps running. | golden |
| **C-FIB-5** | fiber-local non-local return (§4.4): a `return` token from fiber A used in fiber B → `DeadFrameError`; **`next_frame_generation` stays VM-global**. | golden + `invariants.rs` |
| **C-FIB-6** | *(cross-unit, PENDING → graduates)* [[U-ITER]](../U-ITER/specification.md)'s `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }` suspends and yields `1,2,3`. | pending golden |
| **C-FIB-7** | the fiber switch is a **typed signal**, not a `frames.len()` delta — existing non-local-return goldens stay green after the swap. | regression |
| **C-FIB-8** | **no `unsafe`** anywhere in the fiber machinery; no native fiber stacks. | grep + review |
| **C-FIB-9** | `Fiber` passes `verify_invariants()` (parallel rule); floor census bumped (no new ADR). | `invariants.rs` + census |

---

## 10. Non-goals and reserved shapes

- **`Future` / `async` / `await` / scheduler / `System` hooks** — the **Deferred**
  post-v0.2 [[U-FUTURE]](../U-FUTURE/specification.md) track. U-FIBER ships **none** of it,
  but keeps the resumer/result-slot general so it layers cleanly (§7.2).
- **ADR-0033 (`CallBlock` trampoline)** — Deferred; U-FIBER builds the typed switch signal
  ADR-0033 depends on (§4.2) but does **not** trampoline the block call-site (§3.2).
- **`ensure`-on-abandoned-fiber + resource limits** — post-v0.2 layer over the same
  fiber-local unwind (`experimental/fiber-ensure-and-limits.md`); U-FIBER's fiber-floor
  capture (§5) is its seam, but ships none of it.
- **Stackful coroutines (Option C)** / **preemptive/multithreaded fibers** — rejected
  (ADR-0030 §Alternatives): they add `unsafe` and permanently constrain the GC. Do not
  reopen.
- **Full trampoline (Option B) now** — Deferred; A→B is additive (lift-by-deletion).

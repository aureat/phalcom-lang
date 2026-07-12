# U-CORE-6 — Error root + wire dNU → `MessageNotUnderstood` (Implementation Spec)

> **Status:** Normative work order. Authored for a `phalcom-implementer` to
> execute end-to-end. Grounded against **HEAD `4e2ec73`** (U10 landed; U-CORE-0
> 7/7). Where a fact was verified against source, the `file:line` is cited; verify
> line numbers before editing (concurrent forge sessions shift them).
>
> **Scope in one line:** the *minimal reification slice* of [ADR-0008](../../../../adr/0008-layered-exceptions-and-result.md) —
> reify `Error` (root) + `MessageNotUnderstood` (`< Error`) as surface classes with
> `message`/`raise`, and rewire the existing native miss path (U8's
> `object_does_not_understand` + `Message` reification) to **raise a surface
> `MessageNotUnderstood`** carrying the reified `Message` through the **unified
> unwind** — *not* the native `RuntimeError::MessageNotUnderstood`.

---

## §0. Prerequisites + scope gate

### Already landed (do not rebuild)

| Dep | What it gives U-CORE-6 | Ground truth |
|---|---|---|
| **U8** | Overridable `doesNotUnderstand(_)` hook + `Message` reification (`VM::new_message`) + `forward_does_not_understand` miss forward. | `primitive/object.rs:140` (`object_does_not_understand`), `vm.rs:467` (`new_message`), `vm.rs:510` (`forward_does_not_understand`) |
| **U10** | The **unified unwind** substrate: `Bytecode::ReturnNonLocal` + frame-token eager unwind + `RuntimeError::DeadFrameError`. This is the *Return-token* payload of ADR-0008's one unwind primitive; U-CORE-6 adds the sibling *Raise* payload. | `vm.rs:1068` (`ReturnNonLocal` handler), `error.rs:138` (`DeadFrameError`) |
| **U-CORE-0** | Q2 ruling (confirm ADR-0008, do not redesign), the census, the invariant ledger, the forward-compat gate. | [`decisions.md`](../../core/decisions.md) §Q2, [`floor-census.md`](../../core/floor-census.md), [`invariant-requirements.md`](../../core/invariant-requirements.md) §U-CORE-6, [`forward-compat.md`](../../core/forward-compat.md) §2 |
| **Class-tower machinery** | `make_core_class` (create a kernel row + parallel-rule metaclass), Phase-E field stamping (`Some`/`Message`), `add_class!` globals, the reopen path. | `universe.rs:492`, `vm.rs:142-158`, `vm.rs:323-352` |

### Explicitly OUT of scope (RESERVE, do not implement)

Per [`decisions.md`](../../core/decisions.md) §Q2 and [ADR-0008](../../../../adr/0008-layered-exceptions-and-result.md),
the following are **later units** — U-CORE-6 must *reserve* their shapes (keep them
layerable) but ship **none** of them:

- **`Result` / `Ok` / `Err`** and the bridges `{…}.attempt()`, `result.unwrap()`,
  `option.okOr(_)`, `result.ok()` ([values-and-absence.md](../../values-and-absence.md) §4).
  Later **Result unit**; must mirror `Option`/`Some`/`None` (abstract root + two
  concrete subclasses), ADR-0008/[ADR-0007](../../../../adr/0007-option-as-abstract-with-some-none.md).
- **The full handling protocol** — `blk.on(ErrorClass){…}`, `blk.ensure{…}`, and the
  `try`/`catch`/`finally` sugar over it ([error-handling.md](../../error-handling.md) §2).
  Later **error-syntax / block-protocol unit**.
- **Surface `DeadFrameError` / `TypeError` / `ArgumentError` / `RangeError` classes.**
  The runtime *already raises these natively today* as `RuntimeError` variants —
  they stay native this unit (see §1, "native errors that remain unreified"). Only
  the dNU miss is reified.
- **`throw`-as-compile-error** (`throw "oops"` rejected at compile time,
  error-handling.md §1). That is a parser/compiler check owned by the error-syntax
  unit. This unit delivers the *mechanism* (`raise` lives only on `Error`), not the
  syntactic rejection (§4, R-INV-6.3).

### Non-negotiables carried in

- **ADR-0019 floor is frozen.** This unit adds **two** native primitives
  (`Error#message`, `Error#raise`) — that is an **ADR-0019 amendment** (§2, §6,
  drafted). Every other capability is `.ph`/plumbing.
- **No truthiness ([ADR-0021](../../../../adr/0021-no-truthiness-enforcement.md)).** Nothing here reintroduces surface `nil`.
- **No sacred-selector contact.** `message`/`raise` are not sacred (floor-census §5);
  the inliner is untouched.

---

## §1. What exists vs what is missing (grounded)

### The miss path today (verified)

1. `Bytecode::Invoke` misses the exact-selector probe (and the variadic probe) →
   `forward_does_not_understand(receiver_idx, selector, …)` (`vm.rs:1023`).
2. `forward_does_not_understand` (`vm.rs:510`) truncates the args, synthesizes a
   4-slot `Message` via `new_message` (`vm.rs:467`), pushes it, looks up
   `doesNotUnderstand(_:)` and dispatches it via `call_method`. A user override is
   invoked here; otherwise the default resolves on `Object`.
3. The default `object_does_not_understand` (`primitive/object.rs:140`) **returns
   `Err(RuntimeError::MessageNotUnderstood { selector, receiver })`** (`error.rs:82`).
4. That `PhError` propagates via `?` up `call_method` → `run_until` → `run_in_module`
   → `interpret_source`'s `inspect_err`, which calls `runtime_error` (`vm.rs:632`)
   to print `err.to_string()` + a source-mapped trace; `run_file` maps
   `PhError::Runtime(_)` → exit **70** (`interpret.rs:125`).

**What is missing:** steps 3–4 surface a *native* `RuntimeError` string, not a
*surface* `Error` object. There is **no `Error` / `MessageNotUnderstood` class**, no
`message`/`raise` protocol, and the unwind carries no catchable `Value`. `Error`,
`MessageNotUnderstood`, `DeadFrameError`, `TypeError`, `ArgumentError`, `RangeError`
are catalogued ([object-model.md](../../object-model.md) §4, lines 160-165) but
**absent from the tower** (`universe.rs` `create_core_classes` has no `Error` row;
[catalog-delta.md](../../core/catalog-delta.md) §2.7 marks all six ❌/❌).

### The unified-unwind gap (the load-bearing design point)

ADR-0008: "the VM's unwind carries either a `Return` (frame-token) or a
`Raise(error)` payload." U10 built the **Return** payload
(`Bytecode::ReturnNonLocal` + eager frame-token unwind). The **Raise** payload does
not exist yet as a value-carrying channel: today an error is a `PhError` string that
propagates via Rust `?` and, uncaught, renders + exits. U-CORE-6 introduces the
Raise payload as a **surface-`Error`-carrying** `PhError` that propagates through
the *same* `PhResult`/`?` channel an uncaught error already uses — so that later
`on`/`ensure` (block protocol) and a fiber's result-slot capture (forward-compat §1)
intercept a real `Value`, not a Rust string.

### Native errors that remain unreified (reserved, note only)

These `RuntimeError` variants (`error.rs:63`) still raise natively this unit; the
later error unit reifies each to its surface `< Error` class. **Do not touch them:**
`Arity` (→ `ArgumentError`), `Type` (→ `TypeError`), `ZeroDivision`, `DeadFrameError`
(→ surface `DeadFrameError`), `InvalidSetClass`/`InvalidSetSuper`, `UndefinedVar`,
list-index type errors, etc. Their corpus fixtures in `tests/lang/runtime-errors/`
must stay **byte-identical**.

---

## §2. The native/`.ph` split + exact insertion points

**Decision: mirror `Message` exactly — Rust-created rows, Phase-E stamped field
layout, native construction in the miss path, native accessors.** This is a
deliberate architect call (see §6 D2): the alternative — a `.ph` `message` getter
over the field — trips the compiler's **read-before-write** check (`compiler/lib.rs:84`;
a getter that *reads* `_message` without any in-class *assignment* is rejected), and
would couple the Rust miss-path to `.ph` field-declaration order. `Message` already
solves exactly this problem the native way (floor-census §2.14); `Error`/`MNU` follow it.

| Concern | Native (Rust) | `.ph` |
|---|---|---|
| `Error`, `MessageNotUnderstood` **class rows** | ✅ `create_core_classes` (`make_core_class`) | optional empty reopen for surface visibility (see below) |
| Field layout (`_message`; MNU adds `_reifiedMessage`) | ✅ stamped in `VM::new` Phase E (mirrors `Some`/`Message`) | — |
| `Error#message` (getter → slot 0) | ✅ `error_message` **primitive** *(ADR-0019 amendment)* | — |
| `Error#raise` (unwind primitive) | ✅ `error_raise` **primitive** *(ADR-0019 amendment)* | — |
| Building the `MessageNotUnderstood` on a miss | ✅ rewritten `object_does_not_understand` builds it directly | — |
| `Raise` unwind payload | ✅ new `RuntimeError::Raise { error, rendered }` (**plumbing, not a floor binding**) | — |
| Globals `Error` / `MessageNotUnderstood` | ✅ `add_class!` in `install_core` | — |

> **Floor delta: 73 → 75** (two new installed bindings: `message`, `raise` on
> `Error`). The `RuntimeError::Raise` variant is *plumbing the primitive returns*,
> not an installed `(class, selector)` binding, so it does **not** count. R-INV-0.1
> census + R-INV-6.5 update in lockstep (§4).

### Insertion points (exact)

1. **`error.rs`** — add the unwind payload and retire the native miss variant.
   - `use crate::value::Value;` (verified no cycle; `Value` implements `Debug`
     manually at `value.rs:293` and `Clone`+`Copy` at `value.rs:30`, so the
     `#[derive(Error, Debug, Clone)]` on `RuntimeError` still holds).
   - Add:
     ```rust
     /// The surface-`Error` unwind payload — the `Raise(error)` half of ADR-0008's
     /// single unwind primitive (the sibling of U10's `Return`/`ReturnNonLocal`).
     /// `error` is a surface `Error` subclass instance (catchable, `isA(Error)`);
     /// `rendered` is a snapshot of its `message` for the uncaught-render path.
     #[error("{rendered}")]
     Raise { error: Value, rendered: String },
     ```
   - **Remove** `RuntimeError::MessageNotUnderstood` (`error.rs:82`). Grep first:
     its only constructor is `object_does_not_understand`; corpus fixtures match on
     the *stdout string*, not the variant name, so the `rendered` string (below)
     keeps them green. If any non-dNU site references it, STOP and report.

2. **`universe.rs` `create_core_classes`** (`universe.rs:93`) — after the
   `message_class` row (`universe.rs:173`), add:
   ```rust
   let error_class = make_core_class(heap, "Error", object_class, metaclass_class);
   let message_not_understood_class =
       make_core_class(heap, "MessageNotUnderstood", error_class, metaclass_class);
   ```
   `MNU`'s superclass is `error_class`, so `error_class` **must** be created first
   (mirror the `Option → Some/None` ordering, `universe.rs:149-151`). Add both to
   the `CoreClasses { … }` literal and to the `struct CoreClasses` (`universe.rs:512`)
   with rustdoc.

3. **`vm.rs` `VM::new`** — after the `Message` stamp (`vm.rs:150-158`), stamp the
   two error layouts (same idiom as `Some`, `vm.rs:142-148`):
   ```rust
   { // Error: one field `_message` at slot 0.
       let error_class = vm.universe.classes.error_class;
       let msg_sym = vm.interner.intern("_message");
       vm.heap.class_mut(error_class).field_slots.insert(msg_sym, 0);
       vm.heap.class_mut(error_class).field_count = 1;
   }
   { // MessageNotUnderstood < Error: inherits `_message` (slot 0), adds
     // `_reifiedMessage` (slot 1). Subclass fields append after superclass
     // (compiler/lib.rs:713 offset rule) — keep 0/1 consistent with that.
       let mnu = vm.universe.classes.message_not_understood_class;
       let msg_sym = vm.interner.intern("_message");
       let reified_sym = vm.interner.intern("_reifiedMessage");
       vm.heap.class_mut(mnu).field_slots.insert(msg_sym, 0);
       vm.heap.class_mut(mnu).field_slots.insert(reified_sym, 1);
       vm.heap.class_mut(mnu).field_count = 2;
   }
   ```

4. **`vm.rs` `install_core`** (`vm.rs:317`) — add two `add_class!` lines after
   `add_class!(message_class);` (`vm.rs:352`):
   ```rust
   add_class!(error_class);
   add_class!(message_not_understood_class);
   ```
   This binds the globals **and** inserts them into `self.classes`, which routes any
   `core.ph` reopen through the **existing-class** path (`compiler/lib.rs:734`,
   `Bytecode::Constant`) rather than `create_class` — so the reopen never re-applies
   a computed `ClassLayout` and never clobbers the Phase-E `field_count` (this is the
   same mechanism that keeps `Some`'s stamped `field_count = 1` alive across its
   empty `class Some {}` reopen). **Do not** reopen `Error`/`MNU` in `core.ph` with a
   *body that reads a field* — that re-introduces the read-before-write hazard. An
   empty `class Error {}` / `class MessageNotUnderstood {}` reopen is *optional and
   harmless* (surface-visibility only, like `class Some {}`); the `add_class!` global
   already makes the name resolvable, so **skipping the reopen entirely is preferred**
   (fewer moving parts; matches how `Message` ships with no `.ph` reopen).

5. **`primitive/error.rs`** *(new module)* — `error_message`, `error_raise` (§3).
   Register it in `primitive/mod.rs` (`pub mod error;`) and add human-form `Sig`
   display aliases if the file keeps them (floor-census §1.2 — display only, not
   lookup keys).

6. **`primitive/object.rs`** — rewrite `object_does_not_understand` body (§3).

7. **`universe.rs` `install_primitives`** — install the two `Error` primitives
   (place near the `Message` accessors block, `universe.rs:249-253`):
   ```rust
   let error_cls = vm.universe.classes.error_class;
   primitive!(vm, error_cls, "message", SignatureKind::Getter, error_message);
   primitive!(vm, error_cls, "raise",   SignatureKind::Method(0), error_raise);
   ```
   `raise` is `raise()` (0-arity method, interns `raise()`), matching
   object-model §4 and `throw expr === expr.raise()`; `message` is a getter (interns
   `message`).

8. **`universe.rs` `verify_invariants`** — R-INV-6.1 boot check (§4).

---

## §3. Concrete bodies

### 3.1 Rewritten `object_does_not_understand` (`primitive/object.rs:140`)

Keep the *exact* rendered string (`"{receiver} does not understand '{selector}'"`) so
`runtime_unknown_method.expected` (`3 does not understand 'wibble'`) and
`runtime_perform_unknown_selector.expected` (`3 does not understand 'bogus'`) stay
byte-identical. Change only the terminal step: build a surface `MessageNotUnderstood`
carrying the reified `Message` (already handed in as `args[0]`), then unwind via the
Raise payload.

```rust
pub fn object_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    // Selector text from the reified Message (slot 0), exactly as today.
    let selector = match message_slot(vm, &args[0], 0) {
        Some(Value::Symbol(sym)) => vm.resolve_symbol(sym).to_string(),
        _ => "<unknown>".to_string(),
    };
    let receiver_name = receiver.to_string(vm);
    let rendered = format!("{receiver_name} does not understand '{selector}'");

    // Reify the surface MessageNotUnderstood: slot 0 = message string,
    // slot 1 = the reified Message (`args[0]`, census §2.14). Built directly in
    // Rust — the Message precedent (VM::new_message), no `.ph` construct.
    let mnu_class = vm.universe.classes.message_not_understood_class;
    let field_count = vm.heap.class(mnu_class).field_count; // == 2 (Phase E)
    let mut inst = crate::instance::InstanceObject::new(mnu_class, field_count);
    inst.slots[0] = vm.alloc_string_value(rendered.clone());
    inst.slots[1] = args[0]; // the reified Message
    let mnu = Value::Obj(vm.heap.alloc(Object::Instance(inst)));

    // Raise it through the unified unwind (NOT the native RuntimeError variant).
    Err(RuntimeError::Raise { error: mnu, rendered }.into())
}
```

> The overridable-hook contract is untouched: `forward_does_not_understand`
> (`vm.rs:510`) still looks up `doesNotUnderstand(_:)` and dispatches a user override
> *before* this default is reached (R-INV-6.4). A proxy that overrides and returns a
> value never enters this function.

### 3.2 `primitive/error.rs` (new)

```rust
//! Native primitives on `Error` — the raisable root (object-model.md §4,
//! ADR-0008). `raise` is the surface half of the unified unwind; `message`
//! reads the error's `_message` slot. Both are ADR-0019 floor additions
//! (decisions.md §Q2).

/// Signature: `Error::message` — the error's message string (slot 0), surfaced
/// (`None` if unset). Native slot accessor, mirroring `Message::selector`
/// (avoids the read-before-write hazard a `.ph` getter over this field trips).
pub fn error_message(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let slot0 = match receiver {
        Value::Obj(id) => vm.heap.as_instance(*id).map(|i| i.slots[0]),
        _ => None,
    }.ok_or_else(|| RuntimeError::Type { expected: "Error", found: receiver.type_name() })?;
    Ok(vm.surface_absence(slot0))
}

/// Signature: `Error::raise()` — unwind the stack with `self` as a surface
/// `Error` (ADR-0008; `throw expr` desugars here). Returns the `Raise(error)`
/// payload; a fiber boundary (later) captures `error` into its result slot,
/// an `on(_)`/`ensure` (later) intercepts it — this unit only produces it.
pub fn error_raise(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    // Render via the receiver's own `message` protocol so a future computed
    // override is honored. Re-entrancy here is safe: the stack is healthy
    // (inside a live primitive, pre-unwind), same pattern as object_perform.
    let message_sym = vm.get_or_intern("message");
    let rendered = vm.send_dynamic(*receiver, message_sym, &[])?.to_string(vm);
    Err(RuntimeError::Raise { error: *receiver, rendered }.into())
}
```

> **Only `Error` subclasses respond to `raise`** — the primitive is installed on
> `Error` only. A non-`Error` receiver has no `raise`, so a future `throw 42`
> (`42.raise()`) misses → dNU → `MessageNotUnderstood`; this is the runtime half of
> R-INV-6.3 (the compile-time rejection of `throw 42` is error-syntax, deferred).

### 3.3 Propagation & rendering (no changes needed, confirm)

- `RuntimeError::Raise` sits inside `PhError::Runtime`, so `run_file`
  (`interpret.rs:125`) maps it to exit **70** with **no edit**.
- `runtime_error` (`vm.rs:632`) prints `err.to_string()` = `rendered` (via
  `#[error("{rendered}")]`) + the live-frame trace — byte-identical to the old
  `MessageNotUnderstood` render, **no edit**.
- The surface `error` `Value` rides along untouched for the later `on`/`ensure`/fiber
  consumers.

---

## §4. Test strategy

### `_pending` fixtures this unit relates to ([pending-retirement.md](../../core/pending-retirement.md) §4)

Neither is a **direct** flip — both need surface syntax this unit does not add:

| Fixture | Category | This unit delivers | Flips when |
|---|---|---|---|
| `errors/errors_throw_try_catch_finally` | B+C | the **raise mechanism** (`Error`/`MNU`, `raise`, unified-unwind payload) | error-syntax (`throw`/`try`/`catch`/`finally`) **+** the `on`/`ensure` block-protocol unit |
| `errors/errors_result_bridge` | B | **nothing** — `Result`/`Ok`/`Err` + `.attempt()`/`.unwrap()` are RESERVED (§0) | the later **Result unit** **+** error-syntax |

Set the acceptance bar on **new unit-local fixtures in already-supported syntax**,
not on these lexer/Result-gated ones.

### New unit-local fixtures (the acceptance bar)

1. **`tests/lang/runtime-errors/` — uncaught surface-MNU raise renders (NEGATIVE,
   plain syntax).** New `.ph` sending an unknown message to a *user* object plus
   `.expected` = the exact miss string. Proves the reified raise renders identically
   to the old native path (no user-visible regression). Also **keep**
   `runtime_unknown_method` / `runtime_perform_unknown_selector` byte-identical
   (regression guards on the `rendered` format).
   ```phalcom
   // status: NEGATIVE
   class Widget {}
   System.print(Widget.new().frobnicate())
   ```
   `.expected`: `<Widget instance> does not understand 'frobnicate'` (confirm the
   exact receiver rendering via `Value::to_string`; bless from the built binary).

2. **`tests/invariants.rs` — R-INV-6.2 surface-class assertion (corpus, Rust).**
   The `.ph`/stdout lane cannot observe the raised object without `catch`; assert it
   at the VM level (models the existing `VM::new()` + `send_dynamic` corpus style,
   `invariants.rs:39`):
   ```rust
   #[test]
   fn genuine_miss_raises_surface_message_not_understood() {
       let mut vm = VM::new();
       let bogus = vm.get_or_intern("frobnicate");
       let err = vm.send_dynamic(Value::Number(3.0), bogus, &[]).unwrap_err();
       // (a) It is the Raise payload, not the old native MessageNotUnderstood.
       let raised = match err {
           PhError::Runtime(RuntimeError::Raise { error, .. }) => error,
           other => panic!("expected Raise, got {other:?}"),
       };
       // (b) The raised object isA(Error) and is a MessageNotUnderstood.
       let cls = raised.class(&vm.heap); // or raised.class(&mut vm) per the API
       assert_eq!(cls, vm.universe.classes.message_not_understood_class);
       assert!(is_a(&vm, cls, vm.universe.classes.error_class),
               "raised object must be isA(Error)");
       // (c) It carries the reified Message in slot 1.
       // (assert slot 1 is an instance of Message)
   }
   ```
   (Use a small `is_a` helper walking the superclass chain, or reuse the runtime's
   lookup; the point is the three assertions.)

### Invariants this unit adds ([invariant-requirements.md](../../core/invariant-requirements.md) §U-CORE-6)

| # | Invariant | Where | Notes |
|---|---|---|---|
| **6.1** | `MessageNotUnderstood < Error < Object`; parallel rule holds for both new rows (extends R-INV-0.2). | **H** (`verify_invariants`) + **C** | Boot: assert `error.superclass == Object`, `mnu.superclass == Error`, and `X.class.superclass == X.superclass.class` for both. Corpus: same via handle identity + a user subclass of `Error`. |
| **6.2** | A genuine miss (dNU not overridden) raises a **surface** `MessageNotUnderstood` carrying the `Message`, `isA(Error)`, **not** native `RuntimeError`. | **C** | The Rust corpus test above. |
| **6.3** | Only `Error` subclasses are raisable — `raise` lives on `Error` only. | **C** | Assert an `Error` (or subclass) instance responds to `raise` and `3` / a `String` does not (`respondsTo` via `Symbol.new("raise()")`, no `#…` literal needed). `throw 42` compile-rejection = deferred (error-syntax). |
| **6.4** | An overriding `doesNotUnderstand(_)` still intercepts **before** the default raise. | **C** | Promote/guard `dispatch/pending/dispatch_does_not_understand` (Proxy override → `missing: frobnicate`, no raise). It is category-A green today; U-CORE-6 must keep it green. |
| **6.5** | Floor census (R-INV-0.1) updated in lockstep for the `message`/`raise` additions. | **C** | Bump the census audit's expected binding count **73 → 75** and add the two `(Error, selector)` rows; amend [`floor-census.md`](../../core/floor-census.md) §2 + §1.1 in the same change. |

**Boot vs corpus:** 6.1 → **H + C**; 6.2/6.3/6.4/6.5 → **C**.

---

## §5. Must-not-preclude ([forward-compat.md](../../core/forward-compat.md) §2 — *the* section — + §1)

| Hazard (§2/§1) | How this design clears it |
|---|---|
| §2 — a *second, non-`Error`* error channel | The miss raises a `MessageNotUnderstood` **`< Error`**; `raise` is on `Error` only. Single channel, ADR-0008-conformant. |
| §2 — wiring dNU to a non-`Error` or to **host termination** | dNU raises a surface `Error` subclass **value** that propagates through the ordinary `PhResult`/`?` channel. Uncaught → the existing top-level render/exit (unchanged behavior), **not** a special `throw`-terminates-host path. |
| §2 — forking the unwind | The Raise payload is the **sibling** of U10's `Return` payload within the *one* unwind (ADR-0008), carried by the same `PhResult` the VM already threads. No second mechanism; `ensure`-on-any-unwind and `on(_)` layer over `RuntimeError::Raise { error, .. }` later. |
| §2 — `Result` shape incompatible with `Option` | `Result`/`Ok`/`Err` are **reserved, not built**; §0 pins them to the `Option` abstract-root + two-subclass shape so the later unit mirrors it. Nothing here shapes them. |
| §2 — `ensure` as exception-only | Not built here; but because Raise is a payload of the unified unwind (not a bespoke exception path), a later `ensure` that fires on *any* unwind (Return/Raise/abort) is additive. |
| §1 — fiber captures a propagating `Error` into its result slot | The Raise payload **carries the surface `error` `Value`** (not a Rust string). A future `Fiber` boundary extracts that `Value` into its result slot; `throw` is never special-cased as host-process termination. |
| §1 — `Value` openness / frame-locality | No new `Value` arm (`Error`/`MNU` are ordinary `InstanceObject`s). The Raise propagates via `?`, touching no frame indices — it stays fiber-local when fibers arrive. |

**Reserved shapes to keep layerable (do not implement):** `Result`/`Ok`/`Err`
(abstract + two subclasses, `Option`-mirrored); `attempt`/`unwrap`/`okOr`/`ok`
bridges; `on(_)(_)`/`ensure(_)` block protocol; `try`/`catch`/`finally` sugar;
surface `DeadFrameError`/`TypeError`/`ArgumentError`/`RangeError`.

**PHASE2-INDEX ADR-0008 amendment note (folded in):** *"`MessageNotUnderstood` is the
default-dNU raise."* This unit realizes it: the default `doesNotUnderstand(_)` raises
a surface `MessageNotUnderstood` through the unified unwind.

---

## §6. Open sub-decisions + traceability

### Sub-decisions (recommended, flag if deviating)

- **D1 — Raise payload placement.** *Recommended:* `RuntimeError::Raise { error:
  Value, rendered: String }` (inside `PhError::Runtime`). Rationale: `run_file`
  already maps `PhError::Runtime(_)` → exit 70, and `#[error("{rendered}")]` gives
  the uncaught-render path for free — **zero edits** to `runtime_error`/`run_file`.
  *Alternative:* a top-level `PhError::Raise(Value)` (purer payload) — cleaner for
  `on`/fiber but forces edits to `run_file`'s match and a special-case render in
  `runtime_error`. The **load-bearing** requirement either way: `error` is the raw
  surface `Value` (never let `rendered` become what `on`/fiber reads); `rendered` is
  a display cache only.
- **D2 — native accessors vs `.ph` getter.** *Recommended:* native `message`/`raise`
  (Message precedent). Rationale: a `.ph` `message { return _message }` trips the
  read-before-write check (`compiler/lib.rs:84`) because the field is only *read*,
  never *assigned*, in the reopened body; and a `.ph` getter over a Rust-stamped slot
  couples the miss-path to `.ph` field order. This trades a strict-minimal floor
  (+1) for robustness (+2); both are ADR-0019 amendments regardless. *Migration:*
  once a real `.ph` `Error` construction path exists (user `throw`, later unit),
  `message` may move to `.ph` over a field the class also assigns.
- **D3 — `error_raise` render source.** *Recommended:* send `message` (honors a
  future computed override; re-entrancy is safe pre-unwind). *Alternative:* read
  slot 0 directly (fewer cycles, but ignores overrides). The default dNU builds
  `rendered` locally either way (no send).
- **D4 — retire `RuntimeError::MessageNotUnderstood`.** *Recommended:* remove it
  (single constructor). Grep-gate: if any non-dNU site references the variant, STOP
  and report before removing.

*No new ADR beyond the ADR-0019 amendment is required* — ADR-0008 already governs the
model, and [`decisions.md`](../../core/decisions.md) §Q2 confirms it.

### ADR-0019 amendment (draft — lift into a superseding ADR when U-CORE-6 lands)

> *Amends [ADR-0019](../../../../adr/0019-freeze-vm-blessed-primitive-floor.md).* Add to the
> frozen floor: **`Error#message`** (getter; native slot-0 accessor) and
> **`Error#raise`** (`raise()`; the surface half of the unified unwind, returning the
> VM's `Raise(error)` payload). Justification: `raise` initiates a stack unwind — it
> produces a `PhError` unwind payload **below** the `.ph` boundary and is
> underivable in Phalcom (no `.ph` construct yields a `Raise`); `message` is a native
> accessor on a Rust-built kernel instance, mirroring `Message`'s
> `selector`/`name`/`labels`/`args` (floor-census §2.14) and side-stepping the
> read-before-write check a `.ph` getter over a Rust-stamped field would trip. The
> `RuntimeError::Raise` enum variant is *plumbing the primitive returns*, not an
> installed binding. Floor count moves **73 → 75**; update
> [`floor-census.md`](../../core/floor-census.md) §1.1/§2 and the R-INV-0.1 audit in the same
> change.

### Traceability

| Claim / requirement | Source |
|---|---|
| Confirm ADR-0008; U-CORE-6 = minimal reification | [`decisions.md`](../../core/decisions.md) §Q2; [ADR-0008](../../../../adr/0008-layered-exceptions-and-result.md) |
| `Error`/`MNU`/… catalog rows (`message`, `raise`) | [object-model.md](../../object-model.md) §4 (lines 160-165) |
| Raise / handling as block protocol; `throw expr === expr.raise()`; only-`Error` throwable | [error-handling.md](../../error-handling.md) §1-2, §4 |
| One unwind primitive (Return vs Raise payloads) | [ADR-0008](../../../../adr/0008-layered-exceptions-and-result.md); U10 spec §2 |
| `Result`/`Ok`/`Err` reserved, `Option`-mirrored | [values-and-absence.md](../../values-and-absence.md) §4; [ADR-0007](../../../../adr/0007-option-as-abstract-with-some-none.md) |
| dNU/`Message` reification the raise wires to | [floor-census.md](../../core/floor-census.md) §2.14; [catalog-delta.md](../../core/catalog-delta.md) §2.7/§4.5 |
| Invariants R-INV-6.1…6.5 | [invariant-requirements.md](../../core/invariant-requirements.md) §U-CORE-6 |
| Must-not-preclude (errors + fibers) | [forward-compat.md](../../core/forward-compat.md) §2, §1 |
| Fixtures `errors_throw_try_catch_finally` / `errors_result_bridge` | [pending-retirement.md](../../core/pending-retirement.md) §4 |
| open-Q9 resolved by ADR-0008 | [open-questions.md](../../open-questions.md) §9 (`~~struck~~`, → ADR-0008) |
| Current miss path (files) | `primitive/object.rs:140`; `vm.rs:467,510,632`; `error.rs:63,82,138`; `interpret.rs:125` |
| Reopen preserves stamped `field_count` | `compiler/lib.rs:734` (existing-class path); `vm.rs:142-148` (`Some` stamp) |
| Read-before-write hazard | `compiler/lib.rs:84`, field collection `compiler/lib.rs:584-730` |

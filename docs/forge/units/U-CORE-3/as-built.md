# U-CORE-3 — Callables / `Block` Implementation Specification

> **Status:** Normative, dispatch-ready. A single work order a
> `phalcom-implementer` executes end to end. Where this document and older forge
> planning (`PHASE2-INDEX.md`, the coarse `U-STD` callable notes) disagree,
> **follow this document**; where it is silent, the design specs it cites govern.
>
> **Baseline:** HEAD `4e2ec73` (U10 landed — non-local `return` +
> `DeadFrameError`). The U-CORE-0 census docs still pin `76b5f35`; the two are
> the same tree for callable purposes (U10 added no floor bindings). Every
> `file:line` below was read against the working tree in this session — re-confirm
> before editing, the tree is a moving target with concurrent forge sessions.
>
> **Owning track:** the U-CORE-N core-library roadmap. This is the *surface
> callable / reflection layer*. The block **mechanism** (first-class closures,
> `call`/`arity`/`callWith`, upvalues, frame tokens, non-local return) already
> landed in **U4 + U10** — U-CORE-3 does **not** rebuild it. It adds `Method`
> reflection (`methodFor`/`bind`/`invokeOn`/`selector`/`holder`), executes the
> §4.1 `Method < Function` re-parent, and asserts the callable-tower invariants.

---

## 0. Prerequisites and scope gate

### 0.1 Must already be landed (all true at baseline)

| Prereq | Evidence |
|---|---|
| First-class `Block` (`Value::Obj`→`Object::Block`), `call`/`arity`/`name`/`callWith`, upvalues (U4) | `heap.rs` L81 (`Object::Block`); `primitive/block.rs` L46/L61/L85/L127; `block.rs` (`BlockObject`) L18 |
| Frame-token infra + non-local `return` + `DeadFrameError` (U10, ADR-0013) | `vm.rs` L1068 (`Bytecode::ReturnNonLocal`), L1093 (`DeadFrameError`); `frame.rs` L18/L94 |
| `Method` reified as `Value::Obj`→`Object::Method(MethodObject)`; `.class`→`Method` | `heap.rs` L73; `method.rs` L299 (`MethodObject`), L290 (`MethodKind`); `value.rs` L96 |
| Reflective-send workhorse `VM::send_dynamic`, lookup `Value::lookup_method` | `vm.rs` L547; `value.rs` L111 |
| `Symbol.new(_)`, `List.new()`/`.add(_)`/`.at(_)`/`.size` (needed by the unit-local fixture) | `universe.rs` L312 (`symbol_class_new`), L383 (`list_class_new`); `core.ph` L75–94 |

### 0.2 Explicitly OUT of scope (do not build here)

- **Iteration methods** (`each` already in `core.ph` L87; `map`/`reduce`/`filter`)
  — these are U-STD, layered over `Block#call`. U-CORE-3 is a **hard prereq** for
  them (they need `Block#call`, which already exists), but does not add them.
- **`#…` selector literals, `[…]` list literals, `::` family reference** — all
  U-LEX surface syntax. The three pending fixtures this unit *unblocks* are gated
  on U-LEX (§4.2); U-CORE-3 does not touch the lexer/parser.
- **Composite `Method#signature` object** — functions.md §3 lists `signature`
  returning "the `Signature` (selector + kind)". A first-class `Signature` value
  needs a `Signature` kernel class that does not exist and is not in the catalog.
  **Deferred** (forward work, §6). Ship `selector` (the `Symbol`) + `arity`
  instead, which cover every fixture and invariant here.
- **`Method#isPrimitive`** — trivial (`MethodObject::is_primitive()` exists,
  `method.rs` L332) but not exercised by any fixture/invariant. **Deferred to
  U-STD reflection**; note it so a later unit does not re-derive it.
- **Variadic *block literals* (`{ *args => … }`)** — block literals compile
  `BlockExpr.params: Vec<String>` with no `is_rest` (`ast.rs` L270); only
  method/getter/setter/construct defs carry `ParameterDef.is_rest` (`ast.rs`
  L67). This is why `bind` is **not** expressible in `.ph` today (§2.3) — do not
  attempt the `{ *args => self.invokeOn(receiver, args) }` form; it will not
  compile and will break `core.ph` boot.
- **The full error hierarchy.** Arity/receiver mismatches raised here surface as
  the native `RuntimeError::Arity` / `RuntimeError::Type`, **not** a surface
  `ArgumentError`/`TypeError` class. Reifying those is U-CORE-6; R-INV-3.4's
  "`ArgumentError`" today means the native `RuntimeError::Arity`.

---

## 1. What exists vs. what is missing (grounded)

### 1.1 Present today

`Function`/`Block` carry the **complete call protocol** on both rows (identical
native fns so a `Function` responds without a `Block`; floor-census §2.10):

| Selector | Rows | Native fn | `file:line` |
|---|---|---|---|
| `arity`, `name` | Function, Block | `block_arity`, `block_name` | `primitive/block.rs` L46, L61 |
| `callWith(_)` | Function, Block | `block_call_with` (forwards, does **not** yet unpack a `List`) | L127 |
| `call()`…`call(_,_,_,_)` | Function, Block | `block_call` (arities 0–4) | L85; installed `universe.rs` L351–361 |
| `whileTrue(_)` | Block | `block_while_true` ★sacred | L148 |

`resolve_callable` (`primitive/block.rs` L34) accepts **only** `Object::Block`
and bare `Object::Closure`; it type-errors on everything else, including
`Object::Method`.

`Method` today carries **one** floor binding — `new(_)` → `method_class_new`,
which always errors ("Method instances cannot be created directly",
`primitive/method.rs` L6). `MethodObject` (`method.rs` L299) already stores
everything reflection needs: `kind` (`Closure(ObjRef)`|`Primitive(fn)`),
`signature` (selector `Symbol` + `SignatureKind` + `positional_arity` + `variadic`),
`holder: Option<ClassId>`. Accessors exist in Rust: `selector()` L327,
`is_primitive()` L332, `set_holder()` L342.

### 1.2 Two gaps to close

1. **Structural (§4.1 divergence, ruled in `decisions.md` §4.1).** The catalog
   (object-model §4) and ADR-0006 make `Block` **and** `Method` siblings under
   `Function`. The code makes `Method < Object` (`universe.rs` L136:
   `make_core_class(heap, "Method", object_class, …)`) — an **ADR-0006
   violation**. Fix: re-parent to `Method < Function`, which also requires
   **moving `Method`'s `make_core_class` after `Function`'s** (currently L136
   precedes L137).

2. **Protocol (catalog-delta §2.3 "Pending").** `Method` is missing
   `bind(_)`, `invokeOn(_,_)`, `selector`, `holder`; `Object` is missing
   `methodFor(_)`; and `Method` — once re-parented under `Function` — inherits
   `arity`/`name` whose primitive bodies do not yet understand an
   `Object::Method` receiver.

---

## 2. Native-vs-`.ph` split and insertion points

### 2.1 Everything here is native. Why `.ph` is not possible.

The reflection surface reads representation **below** the `.ph` boundary
(ADR-0019 §1 derivability test), so it fails the "prefer `.ph`" default:

| Capability | Why not `.ph` |
|---|---|
| `methodFor(_)` | needs `Value::lookup_method` (`value.rs` L111) — no `.ph` primitive exposes the resolved `MethodObject` handle. `respondsTo(_)` returns only a `Bool`. |
| `invokeOn(_,_)` | needs to run **a specific `MethodObject`** against an explicit receiver through the VM (`call_method`+`run_until`); no `.ph` handle on the closure/dispatch machinery. |
| `bind(_)` | the ADR-0006 `.ph` form `{ *args => self.invokeOn(receiver, args) }` requires **variadic block literals**, which do not parse (§0.2). No fixed-arity `.ph` form is general over a method's arity. |
| `selector`, `holder` | read `MethodObject.signature.selector` / `.holder` — not exposed to `.ph`. |

**Consequence: this unit is an [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md)
amendment.** It adds **5 new floor bindings** (73 → 78). The amendment text is
drafted in §2.6; the implementer lifts it into a new superseding ADR and
re-baselines [`floor-census.md`](../../../spec/v0.2/core/floor-census.md) §2.9/§2.10 + §1.1 count in the
same change (R-INV-0.1). `core.ph` is **not** touched by this unit.

### 2.2 The §4.1 re-parent + load-order fix — CONDITIONAL

> **Conditional (decisions.md §4.1): "whichever of U-CORE-1 / U-CORE-3 lands
> first makes the change; the other asserts it."**
>
> - **If `Method < Function` is not yet in the tree** (check `universe.rs`
>   `create_core_classes`): make the change here.
> - **If U-CORE-1 already re-parented it**, do **not** edit `create_core_classes`
>   — only add the boot assertion (R-INV-3.1) and proceed to §2.3+.

The change (`universe.rs` `create_core_classes`, currently L132–138): move
`Method`'s allocation to **after** `Function`'s and re-parent it. `make_core_class`
reads `heap.class(superclass).class` for the metaclass side (L493), so `Function`
must be wired first — the reorder is what makes the parallel rule
(ADR-0002) hold automatically for `Method class`:

```rust
// BEFORE (L136–138):
let method_class   = make_core_class(heap, "Method",   object_class,   metaclass_class);
let function_class = make_core_class(heap, "Function", object_class,   metaclass_class);
let block_class    = make_core_class(heap, "Block",    function_class, metaclass_class);

// AFTER: Function first, then Method AND Block both < Function.
let function_class = make_core_class(heap, "Function", object_class,   metaclass_class);
let method_class   = make_core_class(heap, "Method",   function_class, metaclass_class);
let block_class    = make_core_class(heap, "Block",    function_class, metaclass_class);
```

No change to the `CoreClasses` struct or field order. This is a pure superclass
edit — it adds **zero** bindings; `Method`'s own method dictionary is unchanged.

### 2.3 `Method#bind(_)` representation — sub-decision SD-3.1 (decided; reviewable)

> **Brief's open item ("bind/invokeOn likely reuse the closure/dispatch
> machinery — decide"). Decision: `bind` is native, returning a new
> `Object::BoundMethod` heap arm whose surface class is `Block`.** The `.ph`
> alternative is blocked (§0.2); a deferral fallback is stated below.

`bind(receiver)` must return a value that (a) `isA(Function)` and reads as "a
`Block`" (ADR-0006, functions.md §3: "a zero-`self` `Function` (a `Block`)"), and
(b) responds to the `call` protocol so that `bound.call(args)` ≡
`method.invokeOn(receiver, args)` (R-INV-3.3). It must work for **primitive**
methods too (`3.methodFor(#+).bind(3)`), which have no `ClosureObject`, so it
cannot reuse `BlockObject` (which wraps a closure + home-frame token, `block.rs`
L18).

**Chosen representation — a new heap arm:**

```rust
// heap.rs, added to `enum Object` (alongside Block, L81):
/// A method closed over a receiver — the result of `Method#bind(_)`
/// (ADR-0006). Its surface class is `Block`; it responds to the `Function`
/// call protocol by delegating to `VM::invoke_method_object`.
BoundMethod(BoundMethodObject),

// new module boundmethod.rs (or inline):
#[derive(Debug, Clone, Copy)]
pub struct BoundMethodObject {
    pub method: ObjRef,   // the wrapped Object::Method
    pub receiver: Value,  // the bound self
}
```

- `Value::class` (`value.rs` L93–103): add `Object::BoundMethod(_) => block_class`.
- `Value::to_debug` (L159–169): `Object::BoundMethod(_) => "<bound method>"`.
- `Value::to_context` (L184–188): `Object::BoundMethod(_) => Instance { instance: *id }`
  (its heap identity, same as a `Block`).
- `Value::value_eq`: identity (`ObjRef` equality), matching `Method`/`Block`.
- The call path (`primitive/block.rs`) special-cases it **before**
  `resolve_callable` (see §3.3).

**Why an `Object` arm and not a `Value` arm:** it lives under the existing
`Value::Obj` handle; no new `Value` variant, so forward-compat §1 (the closed-enum
`Fiber` hazard) is untouched. Adding an `Object` arm ripples only through the
exhaustive `match Object::` sites in **`heap.rs`, `value.rs`, `vm.rs`** (grep
confirmed: those three files) — bounded and enumerated in the write-set (§3.6).

**Fallback (if a reviewer vetoes the new arm):** defer `bind` to the unit that
lands variadic block literals, then write it as `.ph`
`{ *args => self.invokeOn(receiver, args) }`. In that case `invokeOn` + `methodFor`
still land here; R-INV-3.3 becomes a forward invariant and the `functions_method_bind`
fixture waits on both U-LEX **and** that unit. Recommended: do it now — the
native arm is ~40 lines, keeps the reflection surface coherent, and makes
R-INV-3.3 testable this unit.

### 2.4 `arity`/`name` for a `Method` receiver — behavior completion (no new binding)

After §2.2, `Method < Function` **inherits** `arity`/`name`/`callWith`/`call`
from `Function`. Their bodies must learn the `Object::Method` (and
`Object::BoundMethod`) receiver. This adds **zero bindings** — it completes
existing floor primitives. Extend, in `primitive/block.rs`:

- `block_arity` (L46): `Object::Method(m) => Number(m.signature.positional_arity)`;
  `Object::BoundMethod(b) => arity of the wrapped method`.
- `block_name` (L61): `Object::Method(m) => resolve_symbol(m.signature.selector)`;
  `Object::BoundMethod(b) => name of the wrapped method`.

**`call`/`callWith` on an *unbound* `Method` stays an error.** An unbound method
has no receiver; applying it is meaningless (you must `bind` or `invokeOn`).
`resolve_callable` already errors on `Object::Method`; keep that, but improve the
message to `"unbound Method — use bind(_) or invokeOn(_,_)"` (a
`RuntimeError::NotAllowed`). This is a deliberate, documented semantic: `Method`
`isA Function` and answers the *reflective* protocol (`arity`/`name`/`selector`/
`holder`/`bind`/`invokeOn`) but not raw `call`.

### 2.5 New primitives — exact signatures

All installed in `Universe::install_primitives` (`universe.rs`).

| Class | Selector | `SignatureKind` | New native fn | Home file |
|---|---|---|---|---|
| `Object` | `methodFor(_)` | `Method(1)` | `object_method_for` | `primitive/object.rs` |
| `Method` | `invokeOn(_,_)` | `Method(2)` | `method_invoke_on` | `primitive/method.rs` |
| `Method` | `bind(_)` | `Method(1)` | `method_bind` | `primitive/method.rs` |
| `Method` | `selector` | `Getter` | `method_selector` | `primitive/method.rs` |
| `Method` | `holder` | `Getter` | `method_holder` | `primitive/method.rs` |

Install site for the `Object` binding — beside the reflective-send surface
(`universe.rs` L240–243):
```rust
primitive!(vm, object_cls, "methodFor", SignatureKind::Method(1), object_method_for);
```
Install site for the four `Method` bindings — replacing the lone `new(_)` block
(`universe.rs` L337–338), keeping the static `new(_)` and adding the instance
methods on `method_cls`:
```rust
let method_cls = vm.universe.classes.method_class;
primitive_static!(vm, method_cls, "new", SignatureKind::Method(1), method_class_new);
primitive!(vm, method_cls, "invokeOn", SignatureKind::Method(2), method_invoke_on);
primitive!(vm, method_cls, "bind",     SignatureKind::Method(1), method_bind);
primitive!(vm, method_cls, "selector", SignatureKind::Getter,    method_selector);
primitive!(vm, method_cls, "holder",   SignatureKind::Getter,    method_holder);
```

### 2.6 ADR-0019 amendment (draft — lift into a new superseding ADR when this lands)

> *Amends [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md).* Add to
> the frozen floor the **`Method` reflection surface**: `Object#methodFor(_)`,
> `Method#invokeOn(_,_)`, `Method#bind(_)`, `Method#selector`, `Method#holder`
> (5 bindings). Justification: each reads or drives representation below the
> `.ph` boundary — the resolved `MethodObject` handle (`Value::lookup_method`),
> the closure/dispatch machinery (`call_method`/`run_until`), and
> `MethodObject.signature`/`.holder` — none of which any existing floor primitive
> exposes; and `bind` cannot be `.ph` until variadic block literals exist. Also
> adds one **heap representation**, `Object::BoundMethod` (surface class `Block`),
> as the value `bind(_)` returns; it introduces no new `Value` arm. Behavior
> completions (no new binding): `block_arity`/`block_name`/`resolve_callable`/
> `block_call` learn the `Object::Method` and `Object::BoundMethod` receivers.
> Constraint: `invokeOn(recv, args)` runs the **exact reified method** (no
> re-dispatch) and `bound.call(args) ≡ method.invokeOn(recv, args)`
> (R-INV-3.3); an arity mismatch raises `RuntimeError::Arity` (R-INV-3.4). Floor
> count moves **73 → 78**; update [`floor-census.md`](../../../spec/v0.2/core/floor-census.md) §2.9/§2.10
> and §1.1 in the same change.

---

## 3. Concrete implementation

### 3.1 `Object#methodFor(_)` — reify a method by selector

Returns the resolved `Method` value on a hit, or the shared `None` singleton on a
miss (absence, ADR-0007 — usable as `recv.methodFor(sel).ifNone { … }`). On a hit
it returns the **bare** `Method` (not `Some(method)`), because the pending fixture
uses the result directly (`g.invokeOn(…)`, `.bind(g)`); the mild raw-vs-`None`
asymmetry is intentional and pinned by the fixture. A miss must **not** trigger
`doesNotUnderstand` — reflective lookup is a pure probe (like `respondsTo`).

```rust
// primitive/object.rs
/// `Object::methodFor(_)` — reifies the MethodObject `args[0]` (a selector
/// Symbol) resolves to on the receiver, as a `Method` value; the shared `None`
/// singleton on a miss (functions.md §3). A pure probe: never fires dNU.
pub fn object_method_for(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = *expect_value!(&args[0], Symbol);
    match receiver.lookup_method(vm, selector) {
        Some(method_id) => Ok(Value::Obj(method_id)),
        None => Ok(vm.none_value()),
    }
}
```

### 3.2 `Method#invokeOn(_,_)` and the `invoke_method_object` workhorse

`invokeOn(recv, argsList)` runs **the exact reified method** against `recv` with
the elements of `argsList` unpacked as positional arguments. It does **not**
re-dispatch by selector (unlike `perform`): the caller is responsible for
receiver compatibility (extracting `Number#+` and applying it to a `String`
misbehaves inside `number_add` — documented, intentional).

Add a workhorse to `vm.rs`, mirroring `send_dynamic` (L547) exactly except it
dispatches a **given** method handle instead of looking one up:

```rust
// vm.rs
/// Runs the exact method `method_id` against `receiver` with `args`, re-entering
/// `run_until` to recover a synchronous result — the shared engine behind
/// `Method#invokeOn(_,_)` and `Method#bind(_)`'s `call`. Mirrors `send_dynamic`'s
/// re-entrancy; the only difference is no lookup (no dNU on a "miss": the method
/// is already resolved). Validates arity up front (R-INV-3.4).
pub fn invoke_method_object(&mut self, method_id: ObjRef, receiver: Value, args: &[Value]) -> PhResult<Value> {
    // Arity check BEFORE touching the stack, so an error leaves it consistent.
    let (positional, variadic) = {
        let sig = &self.heap.method(method_id).signature;
        (sig.positional_arity as usize, sig.variadic)
    };
    let ok = if variadic { args.len() >= positional } else { args.len() == positional };
    if !ok {
        return Err(RuntimeError::Arity { signature: "invokeOn", expected: positional, found: args.len() }.into());
    }
    let receiver_idx = self.stack.len();
    self.stack.push(receiver);
    self.stack.extend_from_slice(args);
    let base_frames = self.frames.len();
    self.call_method(&receiver, method_id, args.len(), SourceRange::default())?;
    self.run_until(base_frames)
}
```

`call_method` (L383) already handles both `MethodKind::Primitive` (runs in place)
and `MethodKind::Closure` (pushes a frame, collapses the variadic rest into a
`List`); the non-local-return-inside-primitive guard (L394–421) already covers a
block inside the invoked method returning non-locally. No change to `call_method`.

```rust
// primitive/method.rs
/// `Method::invokeOn(_,_)` — applies the reified method (receiver `self`) to the
/// explicit receiver `args[0]` and the argument `List` `args[1]` (functions.md §3).
pub fn method_invoke_on(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;            // Value::Obj → Object::Method
    let target = args[0];
    let list_id = expect_list(vm, &args[1])?;
    let elements: Vec<Value> = vm.heap.list(list_id).elements().to_vec();
    vm.invoke_method_object(method_id, target, &elements)
}
```

(`expect_method` is a small helper mirroring `expect_list`/`expect_class` in
`primitive/mod.rs`; add it there.)

### 3.3 `Method#bind(_)` + the `BoundMethod` call path

```rust
// primitive/method.rs
/// `Method::bind(_)` — closes the reified method over `args[0]` as its receiver,
/// returning a `BoundMethod` (surface class `Block`) that responds to `call`
/// (ADR-0006). `bound.call(a) ≡ method.invokeOn(recv, [a])` (R-INV-3.3).
pub fn method_bind(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    let bound = BoundMethodObject { method: method_id, receiver: args[0] };
    Ok(Value::Obj(vm.heap.alloc(Object::BoundMethod(bound))))
}
```

`block_call` (`primitive/block.rs` L85) branches on `Object::BoundMethod` **first**,
before `resolve_callable` (which assumes a closure and cannot represent a bound
primitive method):

```rust
// at the top of block_call, before resolve_callable:
if let Value::Obj(id) = receiver {
    if let Object::BoundMethod(b) = vm.heap.get(*id) {
        let (method_id, target) = (b.method, b.receiver);
        return vm.invoke_method_object(method_id, target, args); // ← same engine as invokeOn
    }
}
```

Because both `invokeOn` and `bound.call` funnel through `invoke_method_object`,
R-INV-3.3 holds **by construction**, and R-INV-3.4 (arity check) is enforced once,
in one place, for both.

### 3.4 `Method#selector` and `Method#holder`

```rust
// primitive/method.rs
/// `Method::selector` — the interned selector Symbol (functions.md §3).
pub fn method_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    Ok(Value::Symbol(vm.heap.method(method_id).signature.selector))
}

/// `Method::holder` — the defining Class (or metaclass, for a class-side method);
/// the `None` singleton if unbound (functions.md §3).
pub fn method_holder(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    match vm.heap.method(method_id).holder {
        Some(class_id) => Ok(Value::Obj(class_id)), // ClassId is a type alias for ObjRef (heap.rs L59)
        None => Ok(vm.none_value()),
    }
}
```

### 3.5 `arity`/`name` completion (per §2.4) — the two match arms

In `block_arity` and `block_name`, add `Object::Method` and `Object::BoundMethod`
arms reading `MethodObject.signature` (see §2.4). Also add `Object::BoundMethod`
to `resolve_callable`'s error-avoidance is unnecessary — `block_call` already
intercepts it (§3.3) — but `block_arity`/`block_name` are reached directly, so
they need the arm.

### 3.6 Write-set (this unit only — one file per the brief is the *spec*; this is the implementer's set)

- `phalcom-core/src/universe.rs` — §2.2 re-parent + load order (conditional);
  §2.5 install 5 primitives.
- `phalcom-core/src/heap.rs` — `Object::BoundMethod` arm + a `bound_method`
  accessor if needed.
- `phalcom-core/src/boundmethod.rs` (new) — `BoundMethodObject` (or inline in
  `heap.rs`).
- `phalcom-core/src/value.rs` — `class`/`to_debug`/`to_context`/`value_eq` arms.
- `phalcom-core/src/vm.rs` — `invoke_method_object`.
- `phalcom-core/src/primitive/method.rs` — `method_invoke_on`/`method_bind`/
  `method_selector`/`method_holder`.
- `phalcom-core/src/primitive/object.rs` — `object_method_for`.
- `phalcom-core/src/primitive/block.rs` — `block_call` BoundMethod branch;
  `block_arity`/`block_name` Method+BoundMethod arms; `resolve_callable` error msg.
- `phalcom-core/src/primitive/mod.rs` — `expect_method` helper.
- `phalcom-core/tests/` — the unit-local fixture + invariant corpus (§4).
- `docs/spec/core/floor-census.md` — re-baseline (73 → 78); `docs/adr/00NN` — the
  ADR-0019 amendment. **`core.ph` is NOT modified.**

---

## 4. Test strategy

### 4.1 Unit-local acceptance fixture (supported syntax — this unit's real bar)

All three pending fixtures are U-LEX-gated (§4.2), so acceptance rides on a
**new** fixture written in already-supported syntax. Selectors are obtained via
`Symbol.new("…")` (interns to the *same* symbol the compiler produced, so
`lookup_method` hits) and argument lists via `List.new().add(_)` — **no `#…` or
`[…]` needed**. Place at `phalcom-core/tests/lang/functions/functions_method_reflection.ph`:

```phalcom
class Greeter {
  greet(name) {
    return "Hello, " + name
  }
}
let g = Greeter.new()
let m = g.methodFor(Symbol.new("greet(_:)"))
let args = List.new().add("World")
System.print(m.invokeOn(g, args))     // Hello, World  — invokeOn runs the exact method
let bound = m.bind(g)
System.print(bound.call("World"))      // Hello, World  — R-INV-3.3: ≡ invokeOn
System.print(m.arity)                  // 1             — R-INV-3.1: Method responds to arity
System.print(m.name)                   // greet(_:)     — R-INV-3.1: Method responds to name
System.print(m.selector.toString)      // greet(_:)     — reflection getter
```
`.expected`:
```
Hello, World
Hello, World
1
greet(_:)
greet(_:)
```
(`m.name` returns the full encoded selector text — the method's display name;
`m.selector` is the `Symbol`, rendered via `Symbol#toString`.)

A miss fixture: `g.methodFor(Symbol.new("nope")).isNone` → `true` (methodFor
returns `None` on a miss; `isNone` is the `core.ph` combinator, U-CORE-2).

### 4.2 `_pending` fixtures this unit *unblocks* (all U-LEX-gated — none flip on its own)

Per [`pending-retirement.md`](../../../spec/v0.2/core/pending-retirement.md) §3–§4, U-CORE-3 flips
**zero** fixtures directly; it lands the *capability*, and each fixture flips when
U-LEX adds the surface syntax:

| Fixture | Capability landed here | Still needs (U-LEX) |
|---|---|---|
| `functions/functions_method_for_invoke_on` | `methodFor`, `invokeOn` | `#+(_)` selector literal **and** `[4]` list literal |
| `functions/functions_method_bind` | `methodFor`, `bind` | `#greet(_)` selector literal |
| `messages/messages_family_reference` | the bound-callable value `::` produces (via `bind`) | `p::move` `::` family syntax + `::`→`bind` lowering (SD-3.2) |

State in the fixtures / retirement map: **"capability lands in U-CORE-3; fixture
flips once U-LEX adds `#…` / `[…]` / `::`."**

`blocks/blocks_non_local_return` was **already retired by U10** — not a U-CORE-3
flip. U-CORE-3 only guards it still passes (R-INV-3.2).

### 4.3 Invariants (from [`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md) §4, R-INV-3.x)

| # | Assertion | Surface |
|---|---|---|
| **3.1** | `Block < Function` **and** `Method < Function`; all three respond to `arity`/`name`; the parallel rule (R-INV-0.2) holds for each. | **`verify_invariants` (boot, H)** for the tower shape (`Method.superclass == Function`, `Block.superclass == Function`, and `Method class.superclass == Function class`, `Block class.superclass == Function class`); **corpus (C)** for `arity`/`name` response. |
| **3.2** | Non-local `return` from a block whose home frame is dead raises `DeadFrameError`, and it **survives** U-CORE-3's additions — including when the block escapes a method run via `invokeOn`/`bound.call`. | corpus (C) |
| **3.3** | `method.invokeOn(recv, args)` and `method.bind(recv).call(args)` produce identical results for the same `(method, recv, args)`. | corpus (C) — holds by construction (§3.3) but assert it. |
| **3.4** | `arity` matches the dispatcher; an arity-mismatch `invokeOn`/`bound.call` raises `RuntimeError::Arity` (the surface `ArgumentError` is U-CORE-6), not a truncation/silent wrong value. | corpus (C) |

Boot additions go in `Universe::verify_invariants` (`universe.rs` L404). It today
checks the parallel rule for `Number` only (L459–464); extend it to assert the
`Function`/`Method`/`Block` rows explicitly (this is the R-INV-0.2 sweep landing
for the callable rows). Corpus additions go in `tests/invariants.rs`.

R-INV-3.2 corpus test (the subtle one): a method invoked via `invokeOn` creates
and returns an escaping block; after the `invokeOn` activation is gone, calling
that block's `return` must raise `DeadFrameError` — proving the frame-token
generation check (ADR-0013) still fences the re-entrant `run_until` that
`invoke_method_object` introduces.

---

## 5. Must-not-preclude (forward-compat check)

Applicable sections (forward-compat §5 table): **§1 concurrency** (fiber-local
frames, shared `call` protocol) and **§2 unified unwind**.

### §1 Concurrency — PASS
- **Shared `call` protocol, no third-subclass closure.** `bind` returns a value
  whose class is `Block` (not a new callable subclass), so `Function`/`Block`
  stay **open** — a future `Fiber` can still implement the same `call` protocol
  as a distinct callable (forward-compat §1c). Nothing here closes the callable
  set.
- **No single-global-stack assumption.** `invoke_method_object` references
  `self.stack`/`self.frames` symbolically and re-enters `run_until(base_frames)`
  through the **frame-token** infra (ADR-0013) — identical to the existing
  `send_dynamic`/`block_call` re-entrancy. When `Fiber` relocates
  "current stack/frames" behind a `current: PhRef<FiberObject>` pointer, this
  code relocates with it unchanged. `BoundMethod` stores a receiver `Value` + a
  method `ObjRef` and **no** frame token — a bound method is not a lexical block,
  so it has no non-local return and introduces no frame-indexing.
- **Value enum untouched.** No new `Value` arm — the `Fiber` arm hazard
  (forward-compat §1b) is not tripped. `Object::BoundMethod` is under the existing
  `Value::Obj` handle; primitives that don't care about it fall through their
  `_ =>`/`Type` arms.

### §2 Unified unwind — PASS
- `invokeOn`/`bound.call` unwind through the **same** primitive as non-local
  `return` (the re-entrant `run_until` + frame-token compare, ADR-0013), which is
  the same one `throw`/fiber `abort` will use (ADR-0008 "one unwind primitive").
  This unit does **not** fork it, and does not special-case any unwind as host
  termination. R-INV-3.2 guards that the `DeadFrameError` unwind still fires
  through the new re-entrant layer.
- Arity failures raise a plain `RuntimeError` today; when U-CORE-6 reifies
  `ArgumentError` over the unified unwind, `invoke_method_object`'s
  `RuntimeError::Arity` is exactly the native path that will be re-pointed at the
  surface class — no reshaping required.

---

## 6. Open sub-decisions and forward work

| ID | Item | Recommendation | Blocking? |
|---|---|---|---|
| **SD-3.1** | `bind` representation (native `Object::BoundMethod` vs. defer to `.ph` after variadic block literals). | **Native `BoundMethod` now** (§2.3) — coherent reflection surface, R-INV-3.3 testable this unit. | No — decided; reviewer may veto → fallback stated. |
| **SD-3.2** | `::` family reference (functions.md §3 open question): should `p::move`'s `Open`/`Pinned` `Family` **collapse into** `methodFor(…).bind(…)`, or stay a separate lightweight representation? | Recommend `::` lower to `methodFor(#move(_)).bind(p)` → a `BoundMethod`, i.e. **collapse** — one bound-callable representation. But this is **U-LEX's** call (the `::` token is U-LEX); U-CORE-3 only provides the target value. | No — `::` is out of scope; leave the ruling to U-LEX / the family unit. Do **not** build `Family` here. |
| **FW-1** | Composite `Method#signature` object. | Needs a `Signature` kernel class (not in catalog). Defer; ship `selector` + `arity` now. | — |
| **FW-2** | `Method#isPrimitive`, `callWith(_)` list-unpacking, variadic `Block#call`. | Trivial once needed; defer to U-STD reflection / the variadic-block unit. | — |

## 7. Traceability

| Claim | Source |
|---|---|
| Callable tower `Function`→`Block`/`Method` siblings; `bind` returns a Function/Block | [functions.md](../../../spec/v0.2/functions.md) §1–4; [ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md); [object-model.md](../../../spec/v0.2/object-model.md) §4 |
| `methodFor`/`invokeOn`/`bind`/`signature`/`holder` protocol | [functions.md](../../../spec/v0.2/functions.md) §3 |
| Non-local return + `DeadFrameError`, frame token | [blocks.md](../../../spec/v0.2/blocks.md) §5, §7; [ADR-0013](../../../adr/0013-closure-upvalues-and-frame-token-return.md); `vm.rs` L1068–1102 |
| §4.1 `Method < Function` re-parent + load-order; "first lands, other asserts" | [decisions.md](../../../spec/v0.2/core/decisions.md) §4.1; [catalog-delta.md](../../../spec/v0.2/core/catalog-delta.md) §4.1; `universe.rs` L132–138 |
| Callable delta / pending protocol | [catalog-delta.md](../../../spec/v0.2/core/catalog-delta.md) §2.3 |
| Pending flips (all U-LEX-gated), U10 already retired non-local-return | [pending-retirement.md](../../../spec/v0.2/core/pending-retirement.md) §3–§4 |
| R-INV-3.1…3.4 (boot vs corpus) | [invariant-requirements.md](../../../spec/v0.2/core/invariant-requirements.md) §4 |
| Must-not-preclude: fiber-local frames + shared call protocol; unified unwind | [forward-compat.md](../../../spec/v0.2/core/forward-compat.md) §1, §2, §5 |
| Floor freeze; amendment required for new primitives | [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md); [floor-census.md](../../../spec/v0.2/core/floor-census.md) §2.9/§2.10 |
| No new `Value` arm / open enum | [ADR-0010](../../../adr/0010-tagged-value-enum.md); `value.rs` L31 |
| No truthiness — `methodFor` miss returns `None`, not `nil` | [ADR-0021](../../../adr/0021-no-truthiness-enforcement.md) |

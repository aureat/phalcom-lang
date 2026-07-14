# U-CORE-1 — Kernel Reflection (implementation spec)

> **Status:** Normative work order. **New protocol + the audit substrate.**
> U-CORE-1 is the **first** U-CORE *implementation* unit, so besides shipping the
> `Object`/`Behavior` reflective surface it stands up the invariant harness
> (R-INV-0.1…0.4) every later unit extends. Post-U8 the scope is small and
> sharply bounded (§0).
>
> **Baseline:** current HEAD `de03f26`; the `docs/spec/core` census/catalog are
> re-baselined at `0f84232` (last code-affecting commit `0da64d6`; folds
> U8/U9/U-LEX/U-STD/U11 — all docs/`.ph`/compiler-only, **no new floor
> bindings**). **Pre-unit floor: 73 bindings.** **Floor delta: +7** → **80**
> (see §2.2; authorized by the now-Accepted **ADR-0023**).
>
> **Governing anchors:** [`object-model.md`](../../../spec/v0.2/object-model.md) §8 (universal
> protocol: `==`,`!=`,`class`,`isA(_)`,`hash`,`toString`,…), §4 (Behavior/Class
> role), §5 (parallel rule); [`decisions.md`](../../../spec/v0.2/core/decisions.md) **Q1** (hash is a
> floor primitive) + **§4.1** (Method re-parent); [`catalog-delta.md`](../../../spec/v0.2/core/catalog-delta.md)
> §2.1 / §4.5; [`floor-census.md`](../../../spec/v0.2/core/floor-census.md) §2.1 / §2.2 / §7;
> [`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md) R-INV-0.1…0.4,
> R-INV-1.1…1.6; [`forward-compat.md`](../../../spec/v0.2/core/forward-compat.md) §1 / §3 / §4;
> [`pending-retirement.md`](../../../spec/v0.2/core/pending-retirement.md) §4;
> [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) (frozen floor,
> amended by the Accepted [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)),
> [ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md) (`Function`
> root), [ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md) (parallel
> rule), [ADR-0015](../../../adr/0015-object-default-tostring.md) (default toString).

---

## §0. Prerequisites and scope gate

### 0.1 Prerequisites (already landed — fixed inputs)

- **Tower + parallel rule** (`universe.rs::create_core_classes`, ADR-0002/0003):
  `Object`/`Behavior`/`Class`/`Metaclass` + 8-row apex, `verify_invariants`
  (`universe.rs` L417). `Behavior#superclass` / `superclass=` already installed
  (floor-census §2.2). `class`/`superclass` are **already floor primitives** —
  this is what makes `isA(_)` derivable (§2.1).
- **U8 reflective surface**: `perform`/`respondsTo`/`doesNotUnderstand` + the
  `Message` class (floor-census §2.1/§2.14). U-CORE-1 does **not** touch these.
- **U4/U10 blocks + non-local `return`** (`Bytecode::ReturnNonLocal`): needed for
  the `.ph` `isA(_)` body (§3.3).
- **`List` floor** (ADR-0020): `Behavior#methods` returns a `List` (§3.2).
- **U-CORE-2 landed** (`0da64d6`): `Bool#ifTrue`/`ifFalse` return a well-formed
  `Option`; the sacred inliner `Some`-lifts via `WrapSome`. The `.ph` `isA(_)`
  body uses `ifTrue` in **statement (pop) position**, so its `Some`-lift is
  elided — but the body must not assume `ifTrue` returns a raw value (§3.3 note).

### 0.2 In scope

1. **`Object#hash`** (identity) + value-based `hash` overrides on the immediates
   `Number` / `String` / `Symbol` / `Bool` — **native** (decisions.md Q1). Adds an
   **ADR-0019 amendment** (§2.2).
2. **`Object#isA(_)`** — **`.ph`, derived over the existing floor** (`class`,
   `superclass`, `==`); **no** floor addition, **no** ADR (§2.1, §3.3).
3. **`Behavior#name` + `Behavior#methods`** (own method-dictionary enumeration) —
   **native** (underivable); folded into the same ADR-0019 amendment (§2.2, §3.2).
4. **Execute decisions.md §4.1**: re-parent `Method < Function` in
   `create_core_classes`, **including the load-order fix** (`Method` currently
   allocated before `Function`) (§4).

### 0.3 Explicitly OUT of scope (do not build here)

| Deferred item | Owner | Why not here |
|---|---|---|
| Per-type `toString` message overrides (incl. **class** `toString`) | U-CORE-4 | catalog-delta §4.4; U-CORE-1 fixes `Behavior#name` only, not `toString` |
| `Method#signature`/`holder`/`bind(_)`/`invokeOn(_,_)`, callable surface | U-CORE-3 | catalog-delta §2.3; U-CORE-1 only re-parents the tower row |
| Symbol/String `==` semantics, `asString`, richer value protocol | U-CORE-4 | catalog-delta §2.2 |
| `Map`/`Set` (the *consumers* of `hash`) | U-STD | catalog-delta §2.4; hash is their precondition, blocks them, is not them |
| Inherited/`allMethods` reflection, `includesSelector`, instance-var reflection | U-STD | derivable over `Behavior#methods` + `superclass` once `List` grows (U-STD) |
| `#…` selector-literal syntax, `[…]` list literals, `::` family ref | U-LEX | gates the `perform`/`functions_method_*` fixtures (§6.2), not this unit |

### 0.4 Relationship to older planning

U-CORE-1 **subsumes** the coarse "reflection" slices of the retired U-STD/U11
planning for `hash`/`isA`/`Behavior` reflection. `Bool` abstract + `True`/`False`
singletons (the old U11) remain a **separate** later unit and are untouched here.

---

## §1. What exists vs what's missing (grounded)

| Capability | State | Evidence (`file:line`) |
|---|---|---|
| `Object#class` / `class=` | ✅ floor | `universe.rs` L241–242; `primitive/object.rs::object_class` L30 |
| `Behavior#superclass` / `=` | ✅ floor | `universe.rs` L269–270; `primitive/class.rs::class_superclass` L21 |
| `Object#==` / `!=` (identity, string content) | ✅ floor | `universe.rs` L246–247; `value.rs::value_eq` L226 |
| `Object#hash` | ❌ **absent** | not in `install_primitives` (`universe.rs` L238–256) |
| `Number`/`String`/`Symbol`/`Bool` `hash` | ❌ **absent** | — |
| `Object#isA(_)` | ❌ **absent** | `metaclass/pending/metaclass_is_a` fails: `3.isA(Number)` → dNU `isA(_:)` |
| `Behavior#name` (class's **own** name) | ❌ **wrong today** | `Object#name` (`object_name`, `object.rs` L23) returns `receiver.class(vm).name` → for a class receiver `C` this is the **metaclass** name `"C class"`, not `"C"` |
| `Behavior#methods` (method-dict enum) | ❌ **absent** | no accessor to `ClassObject.methods` (`class.rs` L34) from `.ph` |
| `Method` superclass | ⚠️ **wrong** | `make_core_class(heap, "Method", object_class, …)` → `Method < Object` (`universe.rs` L147); ADR-0006 requires `Method < Function` |
| Immediate-hash substrate | ✅ reusable | `StringObject::calculate_hash`/cached `hash()` (`string.rs` L35/L54); `Symbol.0` (`interner.rs` L10); `ObjRef` is a `slotmap` key → `.data().as_ffi()` |
| Census audit (73 bindings) | ❌ **absent** | floor-census §7: "until it exists, counts are a manual checksum" |
| Parallel rule at boot | ◐ **Number only** | `verify_invariants` L472–477 checks only `Number` |
| Absence / fixed-slot at boot | ❌ **corpus only** | not in `verify_invariants` |

**Reading.** Everything `isA(_)` needs is already floor; everything `hash`/`name`/
`methods` needs sits **below** the `.ph` boundary (handle bits, `f64` value,
string bytes, interned id, the `methods` map, a class's own name). That split is
the spine of §2.

---

## §2. The native-vs-`.ph` split

> **ADR-0019 rule (non-negotiable):** the floor is frozen; the default answer to
> "add a primitive" is **no**. A capability goes native **only** if it fails the
> §1 derivability test (touches representation/identity below `.ph`). Every native
> addition below is justified against that test; every derivable one is `.ph`.

### 2.1 `isA(_)` → `.ph` (NO floor addition, NO ADR)

`isA(cls)` walks the receiver's class/superclass chain testing class identity.
Its three ingredients are **all already floor**:

| Ingredient | Floor primitive | Census |
|---|---|---|
| `self.class` | `object_class` on `Object` | §2.1 |
| class identity `c == cls` | `object_eq` → `value_eq` (identity for class objects) | §2.1 |
| `c.superclass` (class → super or `None`) | `class_superclass` on `Behavior` | §2.2 |
| loop / conditional | sacred `Block#whileTrue`, `Bool#ifTrue`; U10 non-local `return` | §2.6/§2.10 |

**Ruling — `isA(_)` is derivable and lives in `core.ph`.** It passes ADR-0019 §1
(no representation access below `.ph`) and the R-BOOT-2 self-hosting layering rule
(bootstrap-phases §5): its body sends only category-(a) native floor selectors, so
it resolves wherever it sits in `core.ph`. This is the **clean win** the brief
calls out: the reflection primitive the roadmap most expects to be native is, in
Phalcom, ordinary `.ph`. Body in §3.3.

### 2.2 `hash` + `Behavior` reflection → native (ADR-0019 AMENDMENT)

Each fails the derivability test — it reads data `.ph` cannot reach:

| New primitive | Side | Class | Native fn | Reads (below `.ph`) | Why underivable |
|---|---|---|---|---|---|
| `hash` | inst | `Object` | `object_hash` | the `ObjRef` handle | no floor primitive exposes the handle |
| `hash` | inst | `Number` | `number_hash` | the `f64` **value** | no int type / bit access in `.ph` |
| `hash` | inst | `String` | `string_hash` | the cached content hash / bytes | `String` floor is `+`/`new` only |
| `hash` | inst | `Symbol` | `symbol_hash` | the interned id `Symbol.0` | only `toString`/`new` in `.ph` |
| `hash` | inst | `Bool` | `bool_hash` | the boolean value | no `.ph` bool→number without `hash` |
| `name` | inst | `Behavior` | `behavior_name` | the class's **own** `ClassObject.name` | `Object#name` gives the *metaclass* name; no `.ph` string slicing to recover `"C"` from `"C class"` |
| `methods` | inst | `Behavior` | `behavior_methods` | the `ClassObject.methods` map | no `.ph` accessor to the method dictionary |

**7 new `(class, selector)` bindings, 7 new distinct native fns.** Floor:
**73 → 80**; distinct fns 57 → 64; floor-carrying classes unchanged at **16**
(`Behavior` already carries `superclass`). This is an **ADR-0019 amendment** — now
ratified as **ADR-0023** (Accepted;
`docs/adr/accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md`, the omnibus
floor amendment). Its draft text is preserved in §2.3. `floor-census.md` §1.1/§2 is updated in the
**same** change (the delta rows are given in §5.4).

> **Note on `Some`/`None`/`Method`/`List` hash.** They add **no** override —
> `Some`/`None`/`Method` are identity-`==` (`value_eq`), so `Object#hash`
> (identity) is already `==`-consistent for them; `None` is the shared singleton
> so its identity hash is stable. `List` is **mutable ⇒ not hashable by value**
> (decisions.md Q5); it inherits `Object#hash` (identity), and its *value*
> hashability is a U-CORE-5 contract concern, not a U-CORE-1 primitive.

### 2.3 The ADR-0019 amendment — ratified as ADR-0023 (Accepted)

> **This amendment landed as [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)**
> (Accepted — the omnibus floor amendment that also covers U-CORE-3/4/6). The
> ADR-0019 gate is **already cleared**; the implementer does **not** draft or
> ratify anything here. The draft below is retained only as the illustrative
> per-unit justification — the unit's job is to install exactly the primitives
> ADR-0023 authorizes and bump the census (§5.4).
>
> **Title:** *Amend ADR-0019 — admit `hash` and kernel reflection to the frozen
> floor.*
> **Status:** Accepted as ADR-0023 (supersedes the relevant clause of ADR-0019;
> ADR-0019 otherwise stands).
>
> **Decision.** Add to the VM-blessed floor exactly these native bindings:
> `Object#hash` (identity digest of the heap handle); per-immediate `hash`
> overrides on `Number`, `String`, `Symbol`, `Bool` (value digests);
> `Behavior#name` (a class's own name) and `Behavior#methods` (its own
> method-dictionary selectors).
>
> **Justification (ADR-0019 §1 derivability test).** Each reads representation or
> identity below the `.ph` boundary — the `ObjRef` handle, an `f64` value's
> digest, a `String`'s bytes, a `Symbol`'s interned id, a class's stored name, and
> the method-dictionary map — none of which any existing floor primitive exposes
> to Phalcom. `isA(_)`, by contrast, is derivable over `class`/`superclass`/`==`
> and stays in `core.ph`.
>
> **Constraints.** (a) `a == b ⇒ a.hash == b.hash` (R-INV-1.3): `String#hash`
> therefore digests **content**, not the handle; identity classes hash by handle.
> (b) Per forward-compat §4, `Number#hash` digests the **mathematical value**
> (class-agnostically), so a future Int/Float split (open-Q2) keeps `2` and `2.0`
> hashing equal. (c) `hash` is **stable** within a run (R-INV-1.4). (d)
> Reflection reads are **side-effect-free** (R-INV-1.6).
>
> **Consequence.** Floor count moves **73 → 80** (N = 7); `floor-census.md` is
> updated in the same change. No other floor move is authorized by this amendment.

---

## §3. Concrete Rust primitives and `.ph` bodies

All native fns follow the existing signature `fn(&mut VM, &Value, &[Value]) ->
PhResult<Value>` and are installed via `primitive!` / `primitive_static!`
(`primitive/mod.rs` L100/L114). All `hash` and reflection selectors are
`SignatureKind::Getter` (0-arg), matching `object-model.md` §8 bare-name notation
and the existing `name`/`class`/`toString` getters.

### 3.1 The `hash` family

Add one shared reducer in `primitive/mod.rs` (folds a `u64` digest to an
**exact integral** `Number` in the 2⁵³ safe-integer range, so the result is a
usable, comparable hash code):

```rust
/// Folds a 64-bit digest into an exact integral `Number` hash code.
pub(crate) fn hash_code(bits: u64) -> Value {
    // Mask to 53 bits so the cast is lossless and the value round-trips as f64.
    Value::Number((bits & 0x1F_FFFF_FFFF_FFFF) as f64)
}
```

**`Object#hash`** (`primitive/object.rs`) — identity digest of the handle. Route
through a single `Value`-level identity helper so a **future `Value` arm**
(e.g. `Fiber`, forward-compat §1) is handled in one place, not in `object_hash`:

```rust
/// Signature: `Object::hash` — a stable identity digest (heap-handle based).
pub fn object_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    use slotmap::Key;
    let bits = match receiver {
        Value::Obj(id) => id.data().as_ffi(),
        // Defensive: immediates override `hash`, but keep this total (do not
        // add a closed match that a new Value arm would silently break).
        other => { let mut h = std::collections::hash_map::DefaultHasher::new();
                   std::hash::Hash::hash(other, &mut h); std::hash::Hasher::finish(&h) }
    };
    Ok(crate::primitive::hash_code(bits))
}
```

**`Number#hash`** (`primitive/number.rs`) — digest the **mathematical value**,
class-agnostically (forward-compat §4):

```rust
/// Signature: `Number::hash` — digest of the mathematical value (not the class).
/// Integral values hash to their integer, so a future Integer 2 and Float 2.0
/// agree; -0.0 is normalized to 0.0; non-integral/inf hash by canonical bits.
pub fn number_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let n = *expect_value!(receiver, Number);
    let bits = if n == 0.0 { 0 }                                   // unify ±0.0
        else if n.is_finite() && n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
            (n as i64) as u64                                      // integral → integer
        } else { n.to_bits() };                                   // else canonical bits
    Ok(crate::primitive::hash_code(bits))
}
```

**`String#hash`** (`primitive/string.rs`) — reuse the **content** hash the string
already caches (`StringObject::hash()` / `calculate_hash`, `string.rs` L35/L54):

```rust
/// Signature: `String::hash` — djb2 content hash (equal content ⇒ equal hash).
pub fn string_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = match receiver { Value::Obj(id) => *id, _ => return Err(/* Type: String */) };
    let h = vm.heap.string(id).hash() as u64;   // cached content hash
    Ok(crate::primitive::hash_code(h))
}
```

**`Symbol#hash`** (`primitive/symbol.rs`) — digest the interned id:

```rust
/// Signature: `Symbol::hash` — digest of the interned id (equal symbols agree).
pub fn symbol_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let s = *expect_value!(receiver, Symbol);
    Ok(crate::primitive::hash_code(s.0 as u64))
}
```

**`Bool#hash`** (`primitive/boolean.rs`) — two distinct stable codes:

```rust
/// Signature: `Bool::hash` — 1 for true, 0 for false (distinct, stable).
pub fn bool_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let b = matches!(receiver, Value::Bool(true));
    Ok(crate::primitive::hash_code(if b { 1 } else { 0 }))
}
```

> `bool_hash` is **not** a sacred selector — `Universe::note_method_installed`
> ignores it, and it does not touch `BOOL_SACRED_SELECTORS`. No deopt budget.

### 3.2 `Behavior#name` and `Behavior#methods`

Both live in `primitive/class.rs` (alongside `class_superclass`, on `Behavior`);
both use the existing `expect_class` helper (`primitive/mod.rs` L141) so they
accept **classes and metaclasses** (a metaclass is a `ClassObject` too):

```rust
/// Signature: `Behavior::name` — the receiver class's OWN display name.
/// (Object#name returns the class-of-receiver's name — for a class receiver that
/// is the metaclass name "C class"; Behavior#name shadows it to return "C".)
pub fn behavior_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Behavior::methods` — a fresh List of the selectors defined
/// DIRECTLY on the receiver class (own dictionary, non-inherited), as Symbols.
/// Side-effect-free: builds a new List, reads nothing but the method map.
pub fn behavior_methods(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let selectors: Vec<Value> =
        vm.heap.class(class_id).methods.keys().map(|s| Value::Symbol(*s)).collect();
    Ok(Value::Obj(vm.heap.alloc_list(selectors)))
}
```

**Lookup correctness (why `Behavior#name` shadows `Object#name` for classes but
not for instances).** A class receiver `C` is an instance of its metaclass; an
instance send `C.name` starts at `C.class` = `C class` and walks the **metaclass**
chain `C class → … → Class → Behavior → Object`. `Behavior#name` sits *below*
`Object#name`, so it wins for class/metaclass receivers. A non-class receiver
(`3`, a user instance) has `Behavior` **nowhere** in its chain, so `3.name` still
resolves to `Object#name` → `"Number"`. `Message#name` (on `Message < Object`,
census §2.14) is likewise unaffected. **`methods` returns the interned selector
symbols** (the `_:`-encoded form, floor-census §1.2); a prettier surface and
inherited/`allMethods` walking are U-STD (§0.3).

### 3.3 `isA(_)` in `core.ph`

Add to the `Object` reopen (`core.ph` L1, currently `class Object {}`):

```phalcom
class Object {
  // Is-kind-of test: true iff `cls` is the receiver's class or an ancestor of
  // it (object-model.md §8). Derived purely over the floor — class/==/superclass
  // — so it needs no native primitive (ADR-0019). The superclass chain is a run
  // of class objects terminating in the `None` singleton (class_superclass
  // returns `None` at the root), so the walk stops on `c == None`.
  isA(cls) {
    var c = self.class
    while (c != None) {
      (c == cls).ifTrue { return true }
      c = c.superclass
    }
    return false
  }
}
```

**Layering (R-BOOT-2, bootstrap-phases §5).** Body sends `class`, `==`/`!=`,
`superclass`, `while`, `ifTrue`, `return` — all category-(a) native floor (Phase
F), plus the `None` **global** (bound Phase D, `install_core`). It depends on no
later-defined `core.ph` class, so it is acyclic at any position; place it first
(the `Object` reopen already leads the file).

**Design notes.**
- `return true` is a **non-local** return from `isA` through the (sacred-inlined
  or deopted) `ifTrue` block — landed in U10. The `ifTrue` result is in **pop
  position**, so U-CORE-2's `Some`-lift is elided; the body neither reads nor
  depends on `ifTrue`'s return shape (do not "simplify" it to consume the
  `Option`).
- **Fallback form** (semantically identical; use only if a non-local-return-
  through-sacred-inliner interaction surfaces — the acceptance fixture §6.1 pins
  the behavior either way): accumulate into a local instead of early-returning —
  `var r = false; while (c != None) { (c == cls).ifTrue { r = true }; c = c.superclass }; return r`.
  This mirrors the proven mutate-a-loop-local pattern in `List#each` (`core.ph`
  L138).
- **Immediate receivers work**: `3.class` = `Number` via `object_class`
  (`value.rs::class` handles immediates), then the walk `Number → Object → None`.
- **Class receivers work**: `Number.isA(Class)` walks the metaclass chain
  (`Number class → … → Class`) → `true`, matching Smalltalk `isKindOf:`.

---

## §4. Execute decisions.md §4.1 — re-parent `Method < Function`

**The bug.** `create_core_classes` builds `Method < Object` (`universe.rs` L147)
— a direct ADR-0006 violation (`Method`/`Block` are siblings under `Function`).
`Method` is also allocated **before** `Function` (L147 vs L148), so re-parenting
requires reordering (a `superclass` must already have its `class` link wired —
`make_core_class` reads `heap.class(superclass).class`).

**The change** (`universe.rs::create_core_classes`, §2.1 ordinary-rows block):
move `Method`'s `make_core_class` to **after** `Function` and re-parent it:

```rust
// before: … Bool, Method(<Object), Function, Block(<Function), Symbol …
// after:  … Bool, Function, Block(<Function), Method(<Function), Symbol …
let function_class = make_core_class(heap, "Function", object_class, metaclass_class);
let block_class    = make_core_class(heap, "Block",    function_class, metaclass_class);
let method_class   = make_core_class(heap, "Method",   function_class, metaclass_class);
```

Delete the old `let method_class = make_core_class(heap, "Method", object_class, …)`
at its former position. The `CoreClasses { … }` struct literal is **by-name**, so
no field reordering is needed — only the binding order and `Method`'s superclass
change. `Method`'s primitives (`method_class_new`, `universe.rs` L351) and the
call-protocol on `Function`/`Block` are unchanged; `Method` now **inherits**
`arity`/`name`/`call…`/`callWith` from `Function` (the point of ADR-0006).

**Coordination with U-CORE-3 (decisions.md §4.1).** Both units touch
`create_core_classes`. **Whichever lands first makes this change; the other
asserts it** (via R-INV-1.5 / R-INV-3.1). If U-CORE-3 has already landed it,
U-CORE-1 only **verifies** `Method.superclass == Function` and moves on. Because
both edit the same `universe.rs` block, they must **not** run in the same parallel
wave — sequence them.

---

## §5. Invariants this unit adds

U-CORE-1 both **stands up** the shared harness (R-INV-0.1…0.4) and adds its own
row (R-INV-1.1…1.6). "H" = `verify_invariants` (boot, `universe.rs`); "C" =
corpus (`tests/invariants.rs`).

### 5.1 The audit substrate (R-INV-0.x — land these first, §5.5)

| # | Assertion | Where | Mechanism |
|---|---|---|---|
| **0.1** | Installed floor = **80** bindings and the exact `(class, selector)` set matches `floor-census.md` | **C** | From a live `VM::new()`, enumerate `heap.class(id).methods` (instance side) **and** `heap.class(metaclass).methods` (static side) for every `CoreClasses` row + its metaclass; collect `(class_name, selector_str)`; assert set equality vs the census. Counts **bindings**, not macro sites (floor-census §1.1). |
| **0.2** | Parallel rule for **all** ordinary rows: `X.class.superclass == X.superclass.class` | **H** + C | Extend `verify_invariants` (currently `Number`-only, L472) to loop `Number,String,Nil,Bool,True,False,Method,Function,Block,Symbol,Module,System,Option,Some,None,List,Message`; keep the corpus sweep. |
| **0.3** | Absence non-surfacing at boot | **H** (structural) + boot check | In `verify_invariants`: `none_singleton` is an `Instance` of `none_class`, `none_class != nil_class`, and the singleton is not a class object. The **global**-resolves-to-singleton-value (not the `None` class) half needs the core module, so assert it inline in `VM::new` right after `install_core` (see §6.4 open note). |
| **0.4** | Fixed-slot layout | **H** | In `verify_invariants`: `heap.class(some_class).field_count == 1` and `heap.class(message_class).field_count == 4` (ADR-0011; fences the E→F edge, bootstrap-phases §4). |

> **U11 `True`/`False` rows (grounding note).** After U11 (ADR-0004),
> `True`/`False` are real tower rows (`universe.rs` L145–146) with their own
> metaclasses wired by the parallel rule, so R-INV-0.2's "for all ordinary
> rows" **includes** them — assert `True.class.superclass == True.superclass.class`
> (both resolve to `Bool class`) and likewise for `False`; they are in the loop
> list above. They carry **zero own floor bindings** (all six sacred `Bool`
> selectors are reached by inheritance — floor-census §2.6), so they add **0** to
> the census and the **73 → 80** delta is unchanged. R-INV-0.1 already enumerates
> them automatically (they are `CoreClasses` rows) with an empty own-method set.

### 5.2 U-CORE-1 unit invariants (R-INV-1.x)

| # | Assertion | Where |
|---|---|---|
| **1.1** | Closes R-INV-0.1…0.4 (this is the first impl unit — it erects the substrate). | H + C |
| **1.2** | `isA(_)` reflexive + superclass-closed: `x.isA(x.class)` is `true`; `x.isA(Object)` is `true` for every `x`; `x.isA(C) ⇔ C` on `x.class`'s superclass chain (test an immediate, a user instance, and a class receiver). | C |
| **1.3** | `hash`↔`==` consistency: `a == b ⇒ a.hash == b.hash`. Assert `Number` (`3==3`), `String` (two equal-content, distinct-handle strings), `Bool`, and identity objects (same handle). **Symbol caveat:** `value_eq` makes symbol pairs never `==` today (catalog-delta §2.3), so assert Symbol as **stability** (two references to the same interned symbol hash equal) rather than via surface `==`. | C |
| **1.4** | `hash` **stable** across repeated calls on the same receiver within a run (each kind). | C |
| **1.5** | `Method.superclass == Function`; parallel rule holds for `Method` (R-INV-0.2 incl. Method); `Method` responds to the `Function` call-protocol selectors (`arity`, `name`, `call…`, `callWith`). | H + C |
| **1.6** | Reflection is side-effect-free: `Behavior#name` / `Behavior#methods` return the class's data without mutating it (call twice → equal results; the class's method dict is unchanged afterward). | C |

### 5.3 Acceptance-grade golden assertions (representative)

- `3.hash == 3.hash` → `true`; `3.hash == 4.hash` → `false`; `true.hash ==
  false.hash` → `false`; `"ab".hash == "ab".hash` → `true` (content, not handle).
  *(Goldens must not pin the exact hash number — only these relations — so any
  valid digest choice passes.)*
- `3.isA(Number)` → `true`; `3.isA(String)` → `false`; `3.isA(Object)` → `true`.
- `Number.name` → `"Number"` (was `"Number class"` before the fix).

### 5.4 floor-census delta (apply in the same change)

Update `floor-census.md` §1.1 (**73 → 80**, distinct fns **57 → 64**) and add
rows: to §2.1 `Object` — `hash | instance | object_hash`; **new §2.2** rows on
`Behavior` — `name | instance | behavior_name`, `methods | instance |
behavior_methods`; to §2.4 `Number`, §2.5 `String`, §2.6 `Bool`, §2.7 `Symbol` —
`hash | instance | {number,string,bool,symbol}_hash`. Note in §2.9 that `Method`'s
`< Function` re-parent (decisions.md §4.1) is now applied.

### 5.5 Sequencing (invariant-requirements §5)

R-INV-0.1…0.4 are a **hard prerequisite** for trusting every later unit's tests:
without the census audit, a unit that accidentally adds a native primitive passes
its own tests. Land the **0.x** assertions **first** in this unit (or a standalone
slice immediately ahead of it), then the 1.x rows. The census audit (0.1) must be
green **before** the 7 new bindings are counted — implement it against 73, then
bump to 80 in lockstep with the primitive installs so the bump is deliberate.

---

## §6. Test strategy

### 6.1 Acceptance bar — a direct pending flip + new unit-local fixtures

- **Direct flip (retire):** `git mv metaclass/pending/metaclass_is_a.* metaclass/`
  — `3.isA(Number)`/`3.isA(String)` (already-`.expected`ed `true`/`false`) goes
  green on `.ph` `isA(_)` in **plain syntax**. This is the clean acceptance
  fixture (pending-retirement §4).
- **New unit-local fixtures** (already-supported syntax; the real acceptance bar,
  per pending-retirement §4's "set your own bar, not a lexer-gated one"):
  - `metaclass/metaclass_is_a_object_root.ph` → `System.print(3.isA(Object))` ⇒
    `true`.
  - `reflection/hash_stable.ph` → `System.print(3.hash == 3.hash)` ⇒ `true`.
  - `reflection/hash_distinct.ph` → `System.print(3.hash == 4.hash)` ⇒ `false`.
  - `reflection/hash_string_content.ph` → `System.print("ab".hash == "ab".hash)`
    ⇒ `true`.
  - `reflection/behavior_name.ph` → `System.print(Number.name)` ⇒ `Number`.

### 6.2 Pending flips (quote pending-retirement §4)

| Fixture | Relationship | Flips when |
|---|---|---|
| `metaclass/metaclass_is_a` | **direct** (via `isA(_)`, plain syntax) | **U-CORE-1** (this unit) |
| `functions/functions_method_bind` | **unblocks-but-gated** — U-CORE-1 supplies the `Method < Function` re-parent, but the fixture needs U-LEX `#greet(_)` selector literal **and** U-CORE-3's `methodFor(_)`/`Method#bind(_)` surface | U-LEX + **U-CORE-3** |

Do **not** claim `dispatch/dispatch_perform`, `functions_method_for_invoke_on`, or
`messages_family_reference` — they are double-gated on U-LEX `#…`/`[…]`/`::` and
U-CORE-3, not on this unit (pending-retirement §3).

### 6.3 Golden / corpus / snapshot

- **Golden corpus** (`tests/lang/`): the §6.1 fixtures, each `.ph` + `.expected`.
- **Invariant corpus** (`tests/invariants.rs`): R-INV-0.1 (census set/count),
  1.2–1.6 as above. Model the audit's class enumeration on the existing
  `core_classes_have_correct_metaclass_and_superclass` sweep (`invariants.rs`
  L264) and `metaclass_responds_to_superclass_via_behavior` (L182) — both already
  read method dicts through `heap.class(...).methods`/`get_method`.
- **Boot check** (`verify_invariants`): R-INV-0.2/0.3/0.4 + R-INV-1.5 boot half.
  `verify_invariants_holds_after_bootstrap` (`invariants.rs` L140) already guards
  that boot stays green; the new checks ride the same call.
- **No fuzz** required; hash distribution is not a correctness property. If a
  property test is added, assert only the R-INV-1.3/1.4 laws (consistency,
  stability), never a specific hash value.

### 6.4 Regression watch (run the full corpus)

- No existing `.ph` corpus/example fixture sends `.name` to a **class** receiver
  (`.name` is only ever sent to user instances / `Message`), so `Behavior#name`
  is purely additive — but the implementer must run the full corpus to confirm.
- `Behavior#name` fixes `.name` on classes **only**; `C.toString` still yields
  `"C class"` (unchanged — U-CORE-4 owns `toString`). Do not "fix" it here.

---

## §7. Must-not-preclude (forward-compat)

| §4 Int/Float split | **Cleared.** `Number#hash` digests the **mathematical value** (integral values → their integer), so a future `Integer 2` and `Float 2.0` hash equal (open-Q2). `isA(_)` walks the class chain, so `2.isA(Number)` stays `true` when `Number` becomes an abstract root over `Integer`/`Float` (they slot under it). `Object#hash` (identity) is value-representation-agnostic. |
|---|---|
| **§1 Value openness / concurrency** | **Cleared.** `isA(_)` is `.ph` (no closed `Value` match). `object_hash` routes non-`Obj` receivers through a `Value`-level helper with a catch-all, so adding a `Value::Fiber` arm needs no edit to `object_hash`; a `Fiber` class would inherit `Object#hash` and respond to `isA(_)` with no primitive change. `behavior_*` read the tower, not a global stack. |
| **§3 Modules / imports** | **Cleared.** New names land on the **core module**: `isA(_)` via `core.ph` (a core-module reopen); `hash`/`name`/`methods` as primitives on tower classes reached through method lookup — **not** an ad-hoc flat global-by-raw-string table. A future `import` (open-Q8) can re-scope them without a breaking change. |

No item in the plan trips a hazard in forward-compat §1/§3/§4.

---

## §8. Open sub-decisions and traceability

### 8.1 Sub-decisions (recommendations; none block start)

| # | Question | Recommendation |
|---|---|---|
| SD-1 | Where does the "`None` global resolves to the singleton **value**, not the `None` class" half of R-INV-0.3 live? `verify_invariants` takes only `&Heap`, so it cannot read module globals. | Assert it **inline in `VM::new`** right after `install_core` (Phase D/H boundary), where the core module exists — cheap and boot-appropriate; keep `verify_invariants` heap-structural. (Alternative: corpus-only, but that loses the boot guard invariant-requirements §1 wants.) |
| SD-2 | `Behavior#methods` return element type: selector **Symbols** vs `Method` objects. | **Symbols** (own dict, non-inherited). Minimal, needs no `Method` surface (that is U-CORE-3), and is the natural key set. `allMethods`/inherited walk deferred to U-STD, derivable over `methods` + `superclass`. |
| SD-3 | Home of the `hash_code` reducer + the 2⁵³ mask. | `primitive/mod.rs` (shared by all five `hash` fns). Keep the mask/normalization in one place so every kind produces a comparable integral `Number`. |
| SD-4 | ADR number for the amendment. | **Resolved — [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)** (Accepted). U-LEX claimed ADR-0022 for string interpolation; the floor amendment landed as the omnibus ADR-0023. Nothing to draft — install the authorized primitives and bump the census. |

### 8.2 Write-set (files this unit may modify)

`phalcom-core/src/universe.rs` (7 `primitive!` installs + Method re-parent/load
order + `verify_invariants` extension — **shared with U-CORE-3; sequence, do not
parallelize**) · `phalcom-core/src/primitive/{object,class,number,string,symbol,
boolean}.rs` (the native fns) · `phalcom-core/src/primitive/mod.rs` (`hash_code`)
· `phalcom-core/core/core.ph` (`Object#isA`) · `phalcom-core/src/vm.rs` (SD-1 boot
check only) · `phalcom-core/tests/invariants.rs` + `phalcom-core/tests/lang/**`
(new/retired fixtures) · **docs applied in lockstep by the implementer**: the census
delta in `docs/spec/core/floor-census.md` (73 → 80) — [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md) is already Accepted, so there is no ADR to draft.

### 8.3 Traceability

| Claim | Source |
|---|---|
| `hash` is a floor primitive; ADR-0019 amendment | [`decisions.md`](../../../spec/v0.2/core/decisions.md) Q1; [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) §1 |
| `isA`/`hash` on `Object`; `Behavior` adds `name`/`methods` | [`object-model.md`](../../../spec/v0.2/object-model.md) §8, §4 |
| `class`/`superclass` already floor (⇒ `isA` derivable) | [`floor-census.md`](../../../spec/v0.2/core/floor-census.md) §2.1/§2.2; `universe.rs` L241/L269 |
| `Object#name` returns metaclass name for a class receiver | `primitive/object.rs::object_name` L23; `value.rs::class` L94 |
| `Method < Function` re-parent + load order | [`decisions.md`](../../../spec/v0.2/core/decisions.md) §4.1; [ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md); `universe.rs` L147–149; [`bootstrap-phases.md`](../../../spec/v0.2/core/bootstrap-phases.md) §2.1 step 5 |
| R-INV-0.1…0.4, 1.1…1.6 | [`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md) §3–§4 |
| parallel rule (all rows) | [ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md); `verify_invariants` L472 |
| `Number#hash` by mathematical value; Value openness; module scoping | [`forward-compat.md`](../../../spec/v0.2/core/forward-compat.md) §4/§1/§3 |
| pending flips | [`pending-retirement.md`](../../../spec/v0.2/core/pending-retirement.md) §3–§4 |
| slotmap key digest; cached string hash; interned id | `heap.rs` L43–52; `string.rs` L35/L54; `interner.rs` L10 |

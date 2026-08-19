# Immediate `Option` Runtime — Detailed Implementation Specification

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this specification task-by-task, with tests at every representation boundary.

**Repository:** `aureat/phalcom-lang`  
**Repository snapshot inspected:** `main`, 2026-08-11  
**Suggested repository destination:** `docs/superpowers/plans/2026-08-11-immediate-option.md`

## Goal

Replace heap-backed `Some` and heap-singleton `None` with a bounded, immediate primitive `Option` representation that:

1. performs **zero heap allocations for Option wrapping itself**;
2. preserves the language distinctions `None`, `Some(None)`, `Some(Some(None))`, etc.;
3. supports at most **7 nested `Some` layers** in the generic VM;
4. makes `Some(x)` the canonical source form, with ordinary call lowering to `Some.call(x)`;
5. preserves ordinary class lookup, message dispatch, `Option.match`, equality, hashing, rendering, control-flow semantics, GC safety, and the private-`Nil` invariant;
6. keeps the final physical `Value` layout / NaN-boxing strategy explicitly deferred.

This is a representation rewrite, not an `Option` semantic redesign beyond the ratified bounded nesting limit and the canonical constructor surface.

---

# 1. Ratified language and runtime decisions

These decisions are normative for this implementation.

## 1.1 Class shape

```text
Object
  |
Option                 abstract + sealed + primitive root
  |
  +-- Some             final + immediate primitive variant
  |
  +-- None             final + immediate primitive variant
```

Rules:

- `Option` has no direct instances.
- `Some` and `None` are the only variants.
- `Some` and `None` cannot be subclassed by user code.
- User code cannot reopen or add methods/fields to `Option`, `Some`, or `None`.
- Core/runtime-owned methods on `Option` remain legal.
- `Some` has **zero instance fields**.
- `None` has **no singleton heap instance**.
- No surface `Some` or `None` value is ever an `Object::Instance`.

The current repository already uses the required object-model precedent for booleans: the surface values are immediate, while `Value::class()` maps them to ordinary `True` / `False` class rows and lookup proceeds through normal class metadata.

## 1.2 Construction

Canonical:

```phalcom
Some(value)
```

Conceptual lowering:

```phalcom
Some.call(value)
```

Explicit equivalent:

```phalcom
Some.call(value)
```

Compatibility form during this migration:

```phalcom
Some.new(value)
```

`Some.new(_)` remains temporarily registered as an alias of `Some.call(_)` so existing code, examples, old tests, and historical material do not break in the same representation patch. New normative docs, core-library source, and new tests must use `Some(...)`. Retiring `new(_)` is a separate compatibility decision.

There is **no parser special case** for `Some(...)`.

The existing compiler already lowers an unqualified call whose name resolves to a value by loading that receiver and sending `call(...)`. `Some` is a core global containing the `Some` class object, so installing class-side `call(_)` on `Some`'s metaclass is sufficient.

## 1.3 Bounded nesting

The generic VM supports exactly these wrapper depths:

```text
depth 0: x
depth 1: Some(x)
depth 2: Some(Some(x))
...
depth 7: Some^7(x)
```

This includes `None` as an underlying payload:

```text
None
Some(None)
Some(Some(None))
...
Some^7(None)
```

All are distinct.

Attempting to add an eighth `Some` layer raises a runtime resource/representation error.

No fallback boxing is permitted.

The error should explicitly direct programmers toward `flatMap(_)` or an explicit domain sum type because deep nested `Option` is almost always a modeling error.

Recommended diagnostic:

```text
Option nesting limit exceeded (7); use flatMap(_) or model the states explicitly
```

A future type checker may diagnose provably-overdeep `Option<...>` expressions statically, but this implementation must not depend on static typing.

## 1.4 No implicit flattening

`map` may create nesting:

```phalcom
Some(1).map |x| { Some(x + 1) }
// Some(Some(2))
```

`flatMap` deliberately avoids the extra layer:

```phalcom
Some(1).flatMap |x| { Some(x + 1) }
// Some(2)
```

The VM must not normalize nested `Some`s away.

## 1.5 Reflection and dispatch

These remain ordinary object-model operations:

```phalcom
Some(42).class == Some
None.class != None

Some(42).is(Option) == true
None.is(Option) == true
```

No heap object is created to answer them.

Method lookup remains:

```text
Value
  -> Value::class(vm)
  -> class method table
  -> superclass chain
```

For a present option:

```text
Some -> Option -> Object
```

For absence:

```text
None -> Option -> Object
```

Immediate receivers continue to execute Phalcom-defined methods through `CallContext::Immediate`.

## 1.6 Equality, identity, and hash

Surface equality remains structural:

```text
None == None                    => true
Some(a) == Some(b)              => a == b
Some(a) == Some(Some(a))        => false
```

No Option wrapper has allocation identity because no wrapper is allocated.

The existing `Option#hash` surface behavior does not need to change in this unit. It currently delegates `Some(v)` to `v.hash` and returns `0` for `None`. That can cause legal hash collisions between different wrapper depths; do not silently change public hash semantics in this representation patch.

The internal Rust `Hash for Value`, however, must include the Option variant/depth so representation-level hash tables do not treat different `Value` variants as the same key accidentally.

## 1.7 Private `Nil`

`Value::Nil` remains the private uninitialized-storage sentinel.

It is not `None`.

Read boundaries continue to transform:

```text
private Nil -> immediate None
```

`Some(private Nil)` is always an internal invariant violation.

## 1.8 GC

An Option wrapper is not a GC node.

If the payload is a heap object:

```text
Some^N(ObjRef(x))
```

the collector must trace `x`.

This applies both to:

- a wrapped value held directly in VM roots (`stack`, immediate call context, temporary roots); and
- a wrapped value stored inside another heap object (`List`, `Tuple`, module global, closed upvalue, fiber stack, class slot, etc.).

---

# 2. Deliberately deferred decision: final `Value` bit layout

This implementation does **not** ratify:

- one-word 64-bit `Value`;
- NaN boxing;
- a 128-bit `Value`;
- pointer tagging;
- niche encoding;
- specialized typed `Option<T>` layouts;
- JIT-specific representations.

Those remain future representation engineering.

The implementation must therefore introduce a correctness-first, allocation-free substrate behind helper APIs and avoid spreading assumptions about its exact physical encoding throughout the VM.

---

# 3. Landing representation for this unit

The current Rust `Value` enum cannot contain `Value::Some(Value)` directly: that would be a recursively-sized enum. `Some(Box<Value>)` would solve the Rust layout problem by allocating, which is explicitly forbidden.

For this unit, use a bounded staging representation:

```rust
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq)]
pub enum OptionPayload {
    None,
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    Obj(ObjRef),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    Nil,
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    Obj(ObjRef),

    None,
    Some1(OptionPayload),
    Some2(OptionPayload),
    Some3(OptionPayload),
    Some4(OptionPayload),
    Some5(OptionPayload),
    Some6(OptionPayload),
    Some7(OptionPayload),
}
```

This is the **landing representation**, not the final physical layout.

Why this representation is acceptable:

- every arm is still `Copy`;
- full `i64`, `f64`, `Symbol`, and `ObjRef` payloads remain intact;
- no recursive `Value`;
- no `Box`;
- no Option allocation;
- `Some(None)` is representable through `OptionPayload::None`;
- nested wrappers are flattened only in the physical representation;
- maximum nesting is structurally enforced by the available variants;
- future representation work can replace these variants behind centralized helper methods.

`OptionPayload` must not contain `Nil`.

## 3.1 Representation encapsulation rule

Outside `phalcom-core/src/value/`, do **not** pattern-match `Some1` through `Some7` directly unless the code is a low-level exhaustiveness boundary that genuinely owns representation concerns.

Prefer helpers:

```rust
pub const MAX_OPTION_NESTING: u8 = 7;

pub(crate) enum OptionCase {
    None,
    Some(Value),
    NotOption,
}

impl Value {
    pub fn is_none(self) -> bool;
    pub fn is_option(self) -> bool;
    pub fn option_depth(self) -> u8;
    pub(crate) fn option_case(self) -> OptionCase;
    pub(crate) fn wrap_some(self) -> Result<Value, RuntimeError>;
    pub(crate) fn gc_obj_ref(self) -> Option<ObjRef>;
}
```

The exact helper names may vary only if the replacement is equally representation-neutral and all call sites remain centralized.

---

# 4. Current-code map

All pointers below refer to the inspected `main` snapshot on 2026-08-11. Use symbol names as the stable pointer; comments and exact line numbers may drift after adjacent work.

| Area | Exact file | Current code pointer | Required change |
|---|---|---|---|
| `Value` representation | `phalcom-core/src/value/mod.rs` | `pub enum Value` | Add immediate `None`, `Some1`…`Some7` and payload substrate |
| GC seam | `phalcom-core/src/value/mod.rs` | `Value::as_obj` | Add Option-aware `gc_obj_ref`; preserve `as_obj` as raw-object query |
| type diagnostics | `phalcom-core/src/value/mod.rs` | `Value::type_name` | Recognize `None`/`SomeN` |
| class mapping | `phalcom-core/src/value/mod.rs` | `Value::class` | Map `None -> none_class`, `SomeN -> some_class` |
| immediate context | `phalcom-core/src/value/mod.rs` | `Value::to_context` | Route Option values through `CallContext::Immediate` |
| internal equality | `phalcom-core/src/value/mod.rs` | `Value::value_eq` | Add representation-consistent Option cases |
| `Nil` surfacing | `phalcom-core/src/value/mod.rs` | `sentinel_to_option` | Return immediate `Value::None`; remove singleton parameter |
| Rust hash | `phalcom-core/src/value/mod.rs` | `impl Hash for Value` | Hash Option variant/depth + payload |
| rendering | `phalcom-core/src/value/render.rs` | `Value::to_string`, `to_debug`, `Debug`, `Display` | Remove heap Some/None inspection; render immediate nested options |
| Option primitives | `phalcom-core/src/primitive/nil.rs` | `wrap_some`, `some_new`, `option_match` | Remove `InstanceObject` allocation and slot access |
| runtime error | `phalcom-core/src/error.rs` | `RuntimeError` | Add bounded-Option nesting error |
| Option classes | `phalcom-core/src/universe/core_classes.rs` | `create_core_classes` Option block | Stop allocating `none_singleton` |
| native representation flag | `phalcom-core/src/universe/core_classes.rs` | `native_repr_classes` | Mark Option/Some/None as non-generic-instance classes |
| core handles | `phalcom-core/src/universe/core_classes.rs` | `CoreClasses` | Remove `none_singleton` field |
| primitive registration | `phalcom-core/src/universe/primitives.rs` | `install_primitives` Option section | Install `Some class >> call(_)`; retain `new(_)` alias |
| kernel invariants | `phalcom-core/src/universe/invariants.rs` | `Universe::verify_invariants` | Replace singleton/field-count assertions with immediate-variant invariants |
| bootstrap | `phalcom-core/src/vm/bootstrap.rs` | `VM::new` | Delete `_value` field-layout stamp; update `None` global assertion |
| globals | `phalcom-core/src/vm/bootstrap.rs` | `VM::install_core` | Bind `None` global to `Value::None` |
| absence helpers | `phalcom-core/src/vm/dispatch.rs` | `VM::surface_absence`, `VM::none_value` | Remove `none_singleton`; return immediate `None` |
| VM wrapping | `phalcom-core/src/vm/dispatch.rs` | `Bytecode::WrapSome` arm | Call fallible immediate wrapper helper |
| iterator end test | `phalcom-core/src/vm/dispatch.rs` | `Bytecode::JumpIfNone` arm | Test immediate `None`, not singleton identity |
| root tracing | `phalcom-core/src/vm/gc.rs` | `collect_roots`, `push_temp_root` | Use Option-aware GC handle seam |
| heap tracing | `phalcom-core/src/heap/trace.rs` | `trace_frame`, `trace_value` | Use Option-aware GC handle seam |
| bytecode docs | `phalcom-core/src/bytecode.rs` | `Nil`, `JumpIfNone`, `WrapSome` docs | Rewrite heap-singleton/allocation claims |
| conditional inliner | `phalcom-core/src/compiler/inliner.rs` | `compile_if_true`, `compile_if_false` | No semantic rewrite; update allocation comments |
| call lowering | `phalcom-core/src/compiler/lib/expr.rs` | `Expr::UnqualifiedCall` | **No special-case change**; this already lowers `Some(x)` to `call(_)` |
| literal truthiness | `phalcom-core/src/compiler/lib/expr.rs` | `is_option_literal` | Recognize `Some(...)`, `Some.call(...)`, compatibility `Some.new(...)` |
| core library | `phalcom-core/core/core.ph` | `class Option`, `class Some` | Migrate canonical construction to `Some(...)`; update comments |
| language harness | `phalcom-core/tests/lang.rs` | `absence`, `absence_negative`, `option`, `values` | Update comments and add fixtures |
| primary semantics doc | `docs/spec/current/values-and-absence.md` | §3, especially §3.1 | Rewrite old heap layout |
| object model | `docs/spec/current/object-model.md` | value table + primitive catalog | Mark `Some`/`None` immediate, exact `.class` |
| historical ADR | `docs/adr/accepted/0007-option-as-abstract-with-some-none.md` | decision banner | Add PDR-0033 amendment notice; do not erase history |
| `Value` ADR | `docs/adr/accepted/0010-tagged-value-enum.md` | decision/consequences | Add PDR-0033 amendment notice for bounded immediate Option substrate |
| GC ADR | `docs/adr/accepted/0050-non-moving-mark-sweep-collector.md` | `Value` tracing seam | Add PDR-0033 amendment notice: wrapped Option may contain an `ObjRef` |
| new design record | `docs/pdr/0033-immediate-bounded-option.md` | new PDR | Record this ratified redesign; PDR-0032 is the current highest record and 0033 is free in the inspected tree |
| PDR tracker | `docs/pdr/STATUS.md` | status table | Add accepted PDR-0033 row; mark shipped only after implementation evidence exists |
| ADR/PDR mapping | `docs/pdr/README.md` | ADR → PDR mapping | Add ADR-0007 / ADR-0010 / ADR-0050 amendment mapping to PDR-0033 as appropriate |
| language corpus manifest | `phalcom-core/tests/lang/MANIFEST.md` | absence/Option notes | Rewrite singleton / allocation descriptions |

---

# 5. Task 1 — Add failing representation tests first

## Files

Create:

```text
phalcom-core/src/value/option.rs
```

Modify:

```text
phalcom-core/src/value/mod.rs
```

## Tests to write before implementation

Place representation-level unit tests in `value/option.rs` under `#[cfg(test)]`.

### 5.1 Basic wrapping

Test:

```rust
#[test]
fn wraps_immediate_without_object_handle() {
    let value = Value::Int(42).wrap_some().unwrap();
    assert_eq!(value.option_depth(), 1);
    assert!(value.is_option());
    assert!(!value.is_none());
    assert_eq!(value.option_case(), OptionCase::Some(Value::Int(42)));
}
```

### 5.2 `Some(None)` remains distinct

```rust
#[test]
fn some_none_is_distinct_from_none() {
    let none = Value::None;
    let some_none = none.wrap_some().unwrap();

    assert_ne!(none, some_none);
    assert!(none.is_none());
    assert_eq!(some_none.option_depth(), 1);
    assert_eq!(some_none.option_case(), OptionCase::Some(Value::None));
}
```

### 5.3 Exact peeling

Construct depth 3 and assert each `option_case()` removes exactly one wrapper:

```text
Some3(Int(7))
 -> Some2(Int(7))
 -> Some1(Int(7))
 -> Int(7)
```

### 5.4 Maximum depth

Wrap seven times successfully.

The eighth call must return:

```rust
RuntimeError::OptionNestingLimit { limit: 7 }
```

or the final exact variant name chosen in `RuntimeError`.

### 5.5 `Nil` is forbidden

Wrapping `Value::Nil` must return an internal invariant error or panic in debug-only invariant code. Prefer a typed internal error from the helper because the same helper is used by `Bytecode::WrapSome`:

```text
internal error: private Nil cannot be wrapped in Some
```

User code can never reach this path.

## Initial test command

```bash
cargo test -p phalcom-core --lib value::option
```

Expected before implementation: compile/test failure.

Commit after tests and implementation pass:

```bash
git add phalcom-core/src/value/mod.rs phalcom-core/src/value/option.rs phalcom-core/src/error.rs
git commit -m "runtime: add immediate bounded Option value substrate"
```

---

# 6. Task 2 — Implement the bounded Option substrate

## File

`phalcom-core/src/value/option.rs`

Define:

```rust
use crate::error::RuntimeError;
use crate::heap::ObjRef;
use crate::interner::Symbol;

use super::Value;

pub const MAX_OPTION_NESTING: u8 = 7;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionPayload {
    None,
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    Obj(ObjRef),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum OptionCase {
    None,
    Some(Value),
    NotOption,
}
```

The conversion logic must be exhaustive.

### 6.1 `OptionPayload::from_base`

Accept only a non-Option, non-`Nil` `Value`:

```rust
fn from_base(value: Value) -> Result<OptionPayload, RuntimeError>
```

Mapping:

```text
Value::None      -> OptionPayload::None
Value::Unit      -> Unit
Value::Bool      -> Bool
Value::Int       -> Int
Value::Float     -> Float
Value::Symbol    -> Symbol
Value::Obj       -> Obj

Value::Nil       -> internal invariant error
Value::Some1..7  -> caller bug; wrapping code handles these before conversion
```

### 6.2 `OptionPayload::into_value`

Inverse mapping:

```text
None   -> Value::None
Unit   -> Value::Unit
Bool   -> Value::Bool
Int    -> Value::Int
Float  -> Value::Float
Symbol -> Value::Symbol
Obj    -> Value::Obj
```

### 6.3 `Value::wrap_some`

Required behavior:

```rust
pub(crate) fn wrap_some(self) -> Result<Value, RuntimeError> {
    match self {
        Value::Some1(p) => Ok(Value::Some2(p)),
        Value::Some2(p) => Ok(Value::Some3(p)),
        Value::Some3(p) => Ok(Value::Some4(p)),
        Value::Some4(p) => Ok(Value::Some5(p)),
        Value::Some5(p) => Ok(Value::Some6(p)),
        Value::Some6(p) => Ok(Value::Some7(p)),
        Value::Some7(_) => Err(RuntimeError::OptionNestingLimit {
            limit: MAX_OPTION_NESTING,
        }),
        Value::Nil => Err(RuntimeError::Internal(
            "private Nil cannot be wrapped in Some".into(),
        )),
        base => Ok(Value::Some1(OptionPayload::from_base(base)?)),
    }
}
```

### 6.4 `Value::option_case`

Required behavior:

```text
Value::None       -> OptionCase::None
Value::Some1(p)   -> OptionCase::Some(p.into_value())
Value::Some2(p)   -> OptionCase::Some(Value::Some1(p))
Value::Some3(p)   -> OptionCase::Some(Value::Some2(p))
...
Value::Some7(p)   -> OptionCase::Some(Value::Some6(p))
anything else     -> OptionCase::NotOption
```

This helper is the representation-neutral primitive behind `Option.match`.

### 6.5 `option_depth`

Return `1..7` for `SomeN`, `0` otherwise.

`None` is an Option value at depth 0. A raw non-Option base value is also depth 0, so callers needing semantic classification must use `is_option()` rather than inferring from depth alone.

### 6.6 Error type

In `phalcom-core/src/error.rs`, add:

```rust
#[error("Option nesting limit exceeded ({limit}); use flatMap(_) or model the states explicitly")]
OptionNestingLimit { limit: u8 },
```

Do not reuse `DepthExceeded`: that error currently means call/native recursion and its message says the computation recurses too deeply, which is not the same resource.

---

# 7. Task 3 — Make all `Value` object-model seams Option-aware

## File

`phalcom-core/src/value/mod.rs`

At the current `pub enum Value`, add:

```rust
None,
Some1(OptionPayload),
...
Some7(OptionPayload),
```

and wire `mod option`.

## 7.1 `Value::as_obj`

Preserve the current meaning:

> “this surface value itself is a raw heap object handle.”

Therefore:

```text
Value::Obj(id) -> Some(id)
SomeN(Obj(id)) -> None
```

Do **not** silently change `as_obj()` into “find any embedded object handle.” That would make non-GC callers accidentally treat `Some(obj)` as though it were `obj`.

## 7.2 Add `Value::gc_obj_ref`

The collector needs a different seam:

```rust
pub fn gc_obj_ref(&self) -> Option<ObjRef> {
    match *self {
        Value::Obj(id) => Some(id),
        Value::Some1(OptionPayload::Obj(id))
        | Value::Some2(OptionPayload::Obj(id))
        | Value::Some3(OptionPayload::Obj(id))
        | Value::Some4(OptionPayload::Obj(id))
        | Value::Some5(OptionPayload::Obj(id))
        | Value::Some6(OptionPayload::Obj(id))
        | Value::Some7(OptionPayload::Obj(id)) => Some(id),
        _ => None,
    }
}
```

This keeps Option representation transparent to the collector without corrupting ordinary object tests.

## 7.3 `type_name`

Use:

```text
None / SomeN -> "option"
```

for the coarse diagnostic layer.

Do not return `"object"`: these are not heap objects.

Exact user-facing class identity remains available through `class()`.

## 7.4 `class`

Add before the generic `Obj` branch:

```rust
Value::None => vm.universe.classes.none_class,
Value::Some1(_)
| Value::Some2(_)
| Value::Some3(_)
| Value::Some4(_)
| Value::Some5(_)
| Value::Some6(_)
| Value::Some7(_) => vm.universe.classes.some_class,
```

This is the exact mirror of current `Value::Bool(true/false)` -> `True` / `False`.

## 7.5 `lookup_method`

No structural change.

It already does:

```rust
let class = self.class(vm);
lookup_method_in_hierarchy(&vm.heap, class, selector)
```

That is exactly the desired dispatch architecture.

## 7.6 `to_context`

Add `None` and `Some1..7` to the `CallContext::Immediate { value: *self }` branch.

Do not fabricate an instance context.

## 7.7 `value_eq`

Add direct Option cases so low-level equality remains coherent:

- `None` vs `None` => true;
- equal-depth `SomeN(a)` / `SomeN(b)` => compare the reconstructed underlying base through the same low-level `value_eq` rules;
- different depth => false;
- Option vs non-Option => false.

This does not replace the surface `Option#==` method. It keeps internal equality utilities representation-consistent.

## 7.8 `same_value_zero`, numeric helpers

Do **not** treat `Some(Int)` or `Some(Float)` as numeric.

Their class is `Some`, not `Int` / `Float`.

`is_numeric` and `is_zero` should continue to recognize only unwrapped number values.

## 7.9 `sentinel_to_option`

Change:

```rust
pub fn sentinel_to_option(value: Value, none_singleton: ObjRef) -> Value
```

to:

```rust
pub fn sentinel_to_option(value: Value) -> Value {
    match value {
        Value::Nil => Value::None,
        other => other,
    }
}
```

Remove all singleton terminology from the rustdoc.

## 7.10 Rust `Hash`

Add discriminants for `None` and every `SomeN` depth.

A safe pattern is:

```text
hash fixed variant marker
hash Option payload
```

with a different marker for every depth.

Do not derive surface language hash semantics from this Rust implementation.

## Verification

```bash
cargo test -p phalcom-core --lib value
cargo check -p phalcom-core
```

The `cargo check` step is intentional: adding variants should trigger exhaustive-match errors. Fix each one according to the classification rules in §18 rather than adding broad wildcards.

---

# 8. Task 4 — Rewrite Option primitives to immediate operations

## File

`phalcom-core/src/primitive/nil.rs`

Current implementation:

- `wrap_some` allocates `InstanceObject`;
- writes slot `0`;
- heap-allocates the object;
- `some_new` delegates to it;
- `option_match` dereferences `Object::Instance`, checks class, and reads slot `0`.

Delete that representation logic.

## 8.1 Shared constructor primitive

Prefer a representation-neutral primitive:

```rust
pub fn some_call(
    _vm: &mut VM,
    _receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    Ok(args[0].wrap_some()?)
}
```

If retaining the current exported function name minimizes churn, `some_new` may simply call `some_call`; however the underlying implementation must be one function, not duplicated.

Recommended:

```rust
pub fn some_call(...) -> PhResult<Value> { ... }

pub fn some_new(...) -> PhResult<Value> {
    some_call(vm, receiver, args)
}
```

The alias can later be removed independently.

## 8.2 `option_match`

Rewrite around `Value::option_case()`:

```rust
pub fn option_match(
    vm: &mut VM,
    receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    match receiver.option_case() {
        OptionCase::Some(value) => block_call(vm, &args[0], &[value]),
        OptionCase::None => block_call(vm, &args[1], &[]),
        OptionCase::NotOption => Err(type_error(receiver)),
    }
}
```

This must peel exactly one wrapper.

Do not inspect class IDs to determine the variant. The primitive representation already has the authoritative variant state.

## 8.3 Remove imports

`InstanceObject` and `Object` should no longer be needed in `primitive/nil.rs`.

## Tests

Add native/unit tests:

- `Some(Int)` match receives raw `Int`;
- `Some(Some(Int))` match receives one-level `Some(Int)`;
- `None` chooses none block;
- non-Option receiver produces the existing type error;
- depth-7 wrap fails before block invocation.

Commit:

```bash
git add phalcom-core/src/primitive/nil.rs phalcom-core/src/value
git commit -m "runtime: make Option construction and match immediate"
```

---

# 9. Task 5 — Install `Some.call(_)` and preserve `Some.new(_)` alias

## File

`phalcom-core/src/universe/primitives.rs`

Current Option primitive section registers static `Some.new(_)` and instance `Option.match(...)`.

Add:

```rust
primitive_static!(
    vm,
    some_cls,
    "call",
    SignatureKind::Method(1),
    some_call
);
```

Retain during migration:

```rust
primitive_static!(
    vm,
    some_cls,
    "new",
    SignatureKind::Method(1),
    some_new
);
```

Both must produce identical immediate values.

The `call` primitive is installed on `Some`'s metaclass through the existing `primitive_static!` mechanism. No new call protocol is required.

## Why no parser/compiler lowering is needed

Current pointer:

```text
phalcom-core/src/compiler/lib/expr.rs
Compiler::compile_expr_want
Expr::UnqualifiedCall
```

The existing unqualified-call path:

1. resolves the bare name;
2. loads the resolved value/global;
3. compiles the arguments;
4. encodes selector base `call`;
5. emits ordinary `Invoke`.

Therefore:

```phalcom
Some(x)
```

already compiles conceptually to:

```phalcom
Some.call(x)
```

once the metaclass implements `call(_)`.

**Do not add a `Some` parser token, AST node, bytecode, intrinsic, or compiler special case.**

That would duplicate an existing language mechanism and make `Some` unnecessarily magical.

## Constructor arity

These naturally fail through ordinary method dispatch / arity rules:

```phalcom
Some()
Some(a, b)
```

`None(x)` remains invalid because the surface `None` global is a value and has no `call(_)`.

---

# 10. Task 6 — Fix compile-time Option-literal truthiness recognition

## File

`phalcom-core/src/compiler/lib/expr.rs`

Current pointer:

```text
fn is_option_literal(expr: &Expr) -> bool
```

Current code recognizes:

- `None`;
- `Some.new(...)`.

It explicitly claims bare `Some(x)` syntax does not exist. That comment becomes false.

Recognize all canonical/compatibility literal shapes:

```text
None
Some(...)
Some.call(...)
Some.new(...)   // compatibility alias while retained
```

Pseudo-shape:

```rust
fn is_option_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Var { value, .. } => value == "None",

        Expr::UnqualifiedCall(call) => call.name == "Some",

        Expr::MethodCall(call) => {
            matches!(&call.object, Expr::Var { value, .. } if value == "Some")
                && (call.method == "call" || call.method == "new")
        }

        _ => false,
    }
}
```

Use the actual AST ownership/reference shape required by the compiler.

Add compile-error fixtures proving:

```phalcom
if (Some(1)) { ... }
```

and:

```phalcom
Some(1) and ...
```

receive the same Option-truthiness diagnostic as existing `Some.new(...)`.

Do not broaden this helper into flow analysis. Its purpose remains literal-only early diagnosis.

---

# 11. Task 7 — Remove heap `None` and `_value` bootstrap state

## File

`phalcom-core/src/universe/core_classes.rs`

### 11.1 `create_core_classes`

Keep class rows:

```text
Option
Some
None
```

Delete allocation of:

```text
none_singleton = heap.alloc(Object::Instance(...))
```

### 11.2 Native representation flags

The current `native_repr_classes` array marks classes that generic `InstanceObject::new` must not allocate.

Add at least:

```text
Option
Some
None
```

`Option` is abstract regardless; including it documents the stronger primitive-representation invariant.

### 11.3 `CoreClasses`

Remove:

```rust
pub none_singleton: ObjRef
```

and its rustdoc.

Update `Some` docs from “one field `_value`” to “final immediate Option variant; no instance layout.”

Update `None` docs from “sole singleton instance” to “final immediate Option variant.”

### 11.4 `CoreClasses::each_handle`

Because `none_singleton` disappears, remove it from any exhaustive handle enumeration/destructure.

The compiler should force this update if `CoreClasses` is exhaustively destructured.

---

# 12. Task 8 — Rewrite bootstrap bindings and kernel invariants

## File

`phalcom-core/src/vm/bootstrap.rs`

### 12.1 Remove `Some` field stamping

Delete the block in `VM::new` that currently:

```text
interns "_value"
sets Some.field_slots["_value"] = 0
sets Some.field_count = 1
```

`Some.field_count` must remain `0`.

### 12.2 Bind the `None` global

Current `install_core` registers the hidden `None` class row in `self.classes` and then binds:

```rust
Value::Obj(self.universe.classes.none_singleton)
```

Replace that binding with:

```rust
Value::None
```

Keep the separate `none_class` row in the class registry because `Value::None.class(vm)` needs it and class metadata/reflection still exists.

### 12.3 Boot assertion

Current `VM::new` asserts the `None` global is exactly `none_singleton`.

Replace with:

```rust
assert_eq!(none_value, Value::None);
assert_ne!(none_value, Value::Obj(vm.universe.classes.none_class));
```

The second assertion remains useful: the surface name `None` denotes the value, not the hidden class object.

### 12.4 Sealing

Keep the existing bootstrap sealing entries for:

```text
Option
Some
None
```

The implementation already reserves/seals all three kernel names. Do not weaken this while changing representation.

## File

`phalcom-core/src/universe/invariants.rs`

Current `verify_invariants` contains two Option-specific heap assumptions:

1. `none_singleton` is an `Instance` of `None`;
2. `Some.field_count == 1`.

Replace them.

Required invariants:

```text
None class != Nil class
Option is abstract
Some.superclass == Option
None.superclass == Option
Some.field_count == 0
None.field_count == 0
Some.native_repr == true
None.native_repr == true
```

Also retain the existing parallel-metaclass-rule checks for all three class rows.

Do not scan the entire heap during every program run merely to prove there are no accidental `Some` instances. Add a focused test instead.

---

# 13. Task 9 — Update VM absence helpers and bytecodes

## File

`phalcom-core/src/vm/dispatch.rs`

### 13.1 `surface_absence`

Change:

```rust
crate::value::sentinel_to_option(value, self.universe.classes.none_singleton)
```

to:

```rust
crate::value::sentinel_to_option(value)
```

### 13.2 `none_value`

Return:

```rust
Value::None
```

No heap access.

### 13.3 `Bytecode::WrapSome`

Current code:

```rust
let wrapped = crate::primitive::nil::wrap_some(self, value);
self.stack.push(wrapped);
```

Change to the fallible immediate helper:

```rust
let wrapped = value.wrap_some()?;
self.stack.push(wrapped);
```

or a thin `primitive::nil::wrap_some(value)?` helper if representation logic remains centralized there.

The opcode must no longer take `&mut VM` merely to allocate.

### 13.4 `Bytecode::JumpIfNone`

Replace singleton/ObjRef identity logic with:

```rust
if value.is_none() {
    self.apply_jump_offset(offset);
}
```

Only exact `None` terminates the iteration protocol.

`Some(None)` is **not** end-of-iteration.

This test is critical.

### 13.5 Read boundaries and return boundaries

Existing calls to:

```text
surface_absence
none_value
```

remain architecturally correct and should become cheaper automatically.

Audit:

- `Bytecode::Nil`;
- `GetGlobal`;
- `GetLocal`;
- `GetUpvalue`;
- `GetField`;
- ordinary `Return`;
- non-local `Return`;
- top-level frame fall-through.

No raw `Value::Nil` may escape.

---

# 14. Task 10 — Update bytecode contracts and the conditional inliner

## File

`phalcom-core/src/bytecode.rs`

Rewrite rustdocs:

### `Bytecode::Nil`

Old concept:

```text
push shared None singleton
```

New:

```text
push immediate None
```

### `Bytecode::JumpIfNone`

Old concept:

```text
identity-test shared None ObjRef
```

New:

```text
direct immediate-None test
```

Explicitly state `Some(None)` does not branch.

### `Bytecode::WrapSome`

Old concept:

```text
allocate a fresh Some instance
```

New:

```text
add exactly one immediate Some layer; fail at nesting depth 7
```

## File

`phalcom-core/src/compiler/inliner.rs`

Pointers:

```text
Compiler::compile_if_true
Compiler::compile_if_false
```

Keep `Bytecode::WrapSome`.

Do not redesign one-armed conditional lowering.

Update comments that currently describe `WrapSome` as allocation and the discarded-result optimization as “skip allocation.”

After this patch:

- taken, value-used branch: immediate wrapper operation;
- untaken branch: immediate `None`;
- discarded-result path may still omit `WrapSome`, but it is now a micro-optimization rather than a GC/allocation optimization.

Required semantic regression:

```phalcom
true.ifTrue || { None }
```

must produce:

```text
Some(None)
```

not `None`.

---

# 15. Task 11 — Make GC tracing Option-aware

This is a correctness-critical task.

A missed wrapped `ObjRef` becomes a use-after-free after a later safepoint.

## File

`phalcom-core/src/vm/gc.rs`

Current root enumeration uses:

```rust
value.as_obj()
```

for stack values and temporary roots.

Change GC-only sites to:

```rust
value.gc_obj_ref()
```

At minimum audit:

```text
VM::collect_roots
VM::push_temp_root
```

This means:

```text
Obj(x)               -> root x
Some1(Obj(x))        -> root x
...
Some7(Obj(x))        -> root x
```

No Option wrapper is rooted because no wrapper object exists.

## File

`phalcom-core/src/heap/trace.rs`

Current module-level invariant says all `Value` children go through `Value::as_obj`.

Change the documented seam to `Value::gc_obj_ref`.

Update:

```text
trace_frame
trace_value
```

so a wrapped object in any heap-owned `Value` slot remains live.

This automatically covers:

- instance slots;
- class static slots;
- method attributes/contracts;
- module globals;
- closure constants;
- bound receivers;
- closed upvalues;
- list entries;
- fiber stacks/results;
- map/set entries;
- tuple values;
- record values;
- range endpoints;
- family receivers;
- pack builders.

## GC regression tests

Two independent tests are required.

### 15.1 VM-root path

A heap object is reachable only through a `Some` stored in a live VM value/root. Force GC, unwrap, and verify the payload still works.

### 15.2 Heap-edge path

A heap object is reachable through:

```text
outer heap object
  -> Value::SomeN(ObjRef(inner))
  -> inner heap object
```

Force GC and verify `inner` survives.

This catches the difference between `vm/gc.rs` root enumeration and `heap/trace.rs` outgoing-edge enumeration.

Use existing test hooks where appropriate:

```text
VM::force_gc
VM::push_root_for_test
VM::pop_root_for_test
```

Do not consider the GC work complete if only immediate/int payload tests pass.

Commit:

```bash
git add phalcom-core/src/vm/gc.rs phalcom-core/src/heap/trace.rs
git commit -m "gc: trace heap payloads through immediate Option values"
```

---

# 16. Task 12 — Rewrite raw rendering

## File

`phalcom-core/src/value/render.rs`

Current raw rendering recognizes `Some`/`None` by inspecting heap `Object::Instance` class IDs and reads `inst.slots[0]`.

Delete those cases.

Render direct variants.

Required output:

```text
None                         -> "None"
Some1(Int(1))                -> "Some(1)"
Some2(Int(1))                -> "Some(Some(1))"
Some3(None)                  -> "Some(Some(Some(None)))"
...
```

Implement through a helper rather than seven hand-written format expressions.

Suggested representation-neutral helper:

```rust
fn render_option(value: Value, vm: &VM) -> Option<String>
```

or iterate:

1. get `option_depth`;
2. reconstruct/render the base payload;
3. wrap the string with `Some(` / `)` `depth` times.

Preserve the rule that raw rendering and `Option#toString` agree.

Update:

```text
Value::to_string
Value::to_debug
impl Debug for Value
impl Display for Value
```

Do not accidentally call ordinary message dispatch from `Debug`/`Display`.

---

# 17. Task 13 — Migrate core-library construction to `Some(...)`

## File

`phalcom-core/core/core.ph`

Current Option methods are pure `.ph` over native `match`; keep that architecture.

Change canonical construction:

```phalcom
Some.new(f.call(v))
```

to:

```phalcom
Some(f.call(v))
```

At minimum update:

```text
Option#map(_)
Result#ok()
```

and every other live core-library `Some.new(...)` occurrence.

Update Option comments:

- remove “shared singleton” wording;
- remove `_value` field wording;
- state `Some` / `None` are immediate primitive variants;
- state `Some(...)` is an ordinary call to the `Some` class object's `call(_)`.

Keep:

```phalcom
class Some {}
```

if it is still required as the core stub-completion mechanism. It must remain empty.

There is still no `class None {}` surface body because `None` is a value global rather than the class global.

## Repository-wide source migration

Run:

```bash
rg -n 'Some\.new\(' \
  phalcom-core \
  examples \
  docs/spec \
  docs/learn \
  docs/theory
```

Migrate live/current examples and normative docs to `Some(...)`.

Do **not** mechanically rewrite historical ADR/as-built documents whose purpose is to record the old implementation. Add amendment notes instead where their claims would otherwise be mistaken for current normative behavior.

Keep a small number of explicit `Some.new(...)` compatibility tests.

---

# 18. Task 14 — Exhaustive `Value` audit

After adding the new arms:

```bash
cargo check -p phalcom-core
```

Fix every non-exhaustive `match` intentionally.

Do not “solve” compiler errors by adding `_ => ...` to core representation boundaries.

Use this classification:

| Code asks... | `SomeN` behavior |
|---|---|
| “what class is this?” | `Some` |
| “is this numeric?” | no |
| “is this a raw heap object?” | no |
| “does this contain a GC edge?” | yes iff payload is `ObjRef` |
| “is this a Symbol?” | no |
| “is this exactly None?” | no |
| “what is its display?” | nested `Some(...)` |
| “can it be field-accessed as an instance?” | no |
| “can it be a method receiver?” | yes, immediate context |
| “is it an Option?” | yes |
| “does JumpIfNone take it?” | no |
| “does match peel it?” | yes, exactly one layer |

Known files that necessarily require review:

```text
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs
phalcom-core/src/primitive/nil.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/gc.rs
phalcom-core/src/heap/trace.rs
```

Also audit compiler errors reported elsewhere, especially diagnostic/value-conversion helpers. Any code treating every non-`Obj` value as handle-free must be checked against `SomeN(ObjRef)`.

---

# 19. Task 15 — Language conformance fixtures

The harness is:

```text
phalcom-core/tests/lang.rs
phalcom-core/tests/lang/<label>/*.ph
phalcom-core/tests/lang/<label>/*.expected
```

Add the following.

## 19.1 `absence/absence_some_call_sugar.ph`

Prove canonical and explicit forms:

```phalcom
System.print(Some(42))
System.print(Some.call(42))
System.print(Some.new(42))
System.print(Some(42) == Some.call(42))
System.print(Some(42).class == Some)
System.print(Some(42).is(Option))
```

Expected:

```text
Some(42)
Some(42)
Some(42)
true
true
true
```

Adjust `System.print` spelling only to the exact existing test style.

## 19.2 `absence/absence_nested_some_distinct.ph`

Exercise:

```text
None
Some(None)
Some(Some(None))
Some(Some(Some(None)))
```

Verify:

- exact rendering;
- pairwise inequality across depth;
- `.class == Some` for every depth > 0;
- one `match` peels exactly one level.

## 19.3 `absence/absence_some_depth_seven.ph`

Construct seven nested wrappers.

Verify successful render and repeated match/unwrapping.

## 19.4 `absence/negative/absence_some_depth_eight.ph`

Construct an eighth wrapper.

Pin substring:

```text
Option nesting limit exceeded (7)
```

Also pin `flatMap(_)` guidance if negative fixture style permits full-message matching.

## 19.5 `option/option_flatmap_prevents_nesting.ph`

Demonstrate:

```phalcom
const nested = Some(1).map |x| { Some(x + 1) }
const flat = Some(1).flatMap |x| { Some(x + 1) }
```

Expected render:

```text
Some(Some(2))
Some(2)
```

This is the semantic reason developers rarely approach the nesting limit.

## 19.6 `compile-errors/compile_error_option_truthiness_some_call.ph`

Use `Some(1)` in a condition and pin the existing `OptionTruthiness` diagnostic.

## 19.7 `values/value_nested_option_tostring.ph`

Verify raw/display rendering agrees for nested options and payload `toString` dispatch remains correct.

## 19.8 GC fixtures / Rust tests

Add the two GC paths from §15. Prefer in-crate Rust tests if precise forced-GC control is easier; add a language fixture too if `System.gc` already provides a stable forcing surface.

---

# 20. Task 16 — VM/internal tests

Create or extend in-crate tests for representation-sensitive properties that the language surface cannot observe directly.

## 20.1 No heap representation

After constructing `Some` through the primitive, assert the returned `Value` is a `SomeN` immediate, never `Value::Obj`.

Do this for:

```text
Int payload
Float payload
Bool payload
None payload
ObjRef payload
```

## 20.2 No `None` heap object

After `VM::new`:

- core global `None` equals `Value::None`;
- `None.class(vm) == none_class`;
- no `none_singleton` field exists in `CoreClasses`.

## 20.3 Dispatch

Send these to immediate values:

```text
isSome
isNone
toString
unwrapOr
match
```

Prove `CallContext::Immediate` supports the ordinary `.ph` Option methods.

## 20.4 `Some.call`

Resolve the `call(_)` method from:

```text
Value::Obj(some_class)
```

through the class object's metaclass and invoke it.

Prove it returns an immediate Some.

## 20.5 `JumpIfNone`

A direct VM/compiler test must distinguish:

```text
None        -> jump
Some(None)  -> fall through
```

This protects iterator termination semantics.

## 20.6 `WrapSome`

Test opcode behavior at depth:

```text
0 -> 1
6 -> 7
7 -> runtime error
```

No heap allocation is part of the expected path.

---

# 21. Task 17 — Update public specifications

## `docs/spec/current/values-and-absence.md`

Rewrite §3.1.

Required normative shape:

```markdown
| Class | Kind | State |
|---|---|---|
| `Option` | abstract, sealed primitive root | no direct values |
| `Some` | final immediate variant | one immediate payload, no fields |
| `None` | final immediate variant | immediate absence value |
```

State:

- `Some(v)` is canonical;
- it is ordinary call syntax and behaves as `Some.call(v)`;
- `Some.new(v)` is compatibility-only during migration;
- Option wrapper construction never allocates;
- nested Options are distinct;
- generic runtime maximum nesting is 7;
- eighth nesting raises;
- prefer `flatMap` when callback returns `Option`;
- no wrapper identity;
- class/reflection works through immediate class mapping.

Rewrite §3.6 equality wording from “None by identity” to value semantics consistent with immediate representation.

## `docs/spec/current/object-model.md`

Update value representation table:

```text
None     -> class None
Some(v)  -> class Some
```

not generic `Option`.

Update primitive catalog:

```text
Option  A
Some    I
None    I
```

Explain that `Option` mirrors the `Bool` tower structurally while its variants carry bounded immediate sum-state.

## `phalcom-core/tests/lang/MANIFEST.md`

Remove statements saying:

- `None` is the shared singleton object;
- `Some.new` is the construction surface;
- `Some` allocates.

Replace with current semantics.

---

# 22. Task 18 — Record the ruling as PDR-0033 and amend frozen ADRs

The repository has moved new decisions to `docs/pdr/`. `docs/adr/` is frozen at ADR-0064. The inspected tree's highest PDR is PDR-0032 and no PDR-0033 exists, so this ruling should be recorded as:

```text
docs/pdr/0033-immediate-bounded-option.md
```

## Create PDR-0033

Required header:

```markdown
# PDR-0033 — Make Option an immediate bounded sum value

- Status: Accepted
- Date: 2026-08-11
- Amends: ADR-0007 (runtime representation and construction surface);
  ADR-0010 (Value representation admits immediate bounded Option state);
  ADR-0050 (a non-Obj Value may carry one traceable ObjRef through Some)
- Related: docs/spec/current/values-and-absence.md;
  docs/spec/current/object-model.md;
  phalcom-core/src/value/mod.rs;
  phalcom-core/src/primitive/nil.rs
```

The PDR must ratify:

- no Option heap wrappers;
- `None` immediate;
- `Some` immediate;
- maximum nesting depth 7;
- no fallback boxing;
- canonical `Some(x)` with ordinary lowering to `Some.call(x)`;
- temporary `Some.new(x)` compatibility alias;
- `Some` has no `_value` field;
- class/dispatch behavior;
- private `Nil` remains separate;
- GC traces an embedded ObjRef through immediate Option;
- final bit layout / NaN boxing remains explicitly deferred.

Its Consequences section must state the cost plainly:

> The generic VM now has a finite Option nesting limit of seven, and every future
> `Value` representation must reserve enough state to distinguish the eight
> wrapper levels 0–7 without allocating Option wrappers.

It must also state what the ruling precludes:

- unlimited generic nested Option values;
- a fallback heap representation for overdeep Option;
- treating `Value::Obj` as the only possible `Value` carrying a GC edge.

## Update `docs/pdr/STATUS.md`

Add an Accepted PDR-0033 row.

Before the code lands, `Shipped` is `❌` or `?` according to the tracker's convention. In the implementation commit that verifies the behavior, change it to `✅` and cite concrete files/tests.

## Amend ADR-0007

File:

```text
docs/adr/accepted/0007-option-as-abstract-with-some-none.md
```

Do not rewrite the historical decision body.

Add a dated amendment callout near the top:

```markdown
> **Amended 2026-08-11 by PDR-0033.** The `Option` / `Some` / `None`
> semantic hierarchy remains, but `Some` and `None` are now immediate primitive
> values rather than heap instances. `Some(v)` is canonical and lowers through
> `Some.call(v)`. Generic nesting is bounded to seven.
```

Update `docs/adr/STATUS.md` with the same amendment relationship without falsely marking ADR-0007 retired; the semantic hierarchy remains authoritative where PDR-0033 does not replace it.

## Amend ADR-0010

File:

```text
docs/adr/accepted/0010-tagged-value-enum.md
```

Add a PDR-0033 callout stating that the tagged `Value` API now admits immediate bounded Option state and the final physical encoding / NaN boxing remains deferred.

Do not claim `Some1`…`Some7` is the permanent bit-level design; it is the correctness-first landing substrate.

## Amend ADR-0050

File:

```text
docs/adr/accepted/0050-non-moving-mark-sweep-collector.md
```

Add a PDR-0033 callout clarifying:

```text
A Value may contain one GC edge even when it is not Value::Obj,
because immediate SomeN may carry an ObjRef payload.
```

The collector's normative seam becomes `Value::gc_obj_ref()`.

## Update `docs/pdr/README.md`

Add the ADR → PDR amendment relationships for the revisited ADRs, following the existing mapping-table conventions. PDR-0033 amends rather than wholesale-supersedes ADR-0007/0010/0050.

# 23. Task 19 — Performance and allocation regression

The representation change is motivated by eliminating Option allocation, so performance evidence is part of completion.

## Required benchmark shapes

### 23.1 Construct/extract loop

Repeatedly execute:

```phalcom
Some(i).unwrapOr(0)
```

Do not benchmark repeated nesting because it intentionally trips the depth bound.

### 23.2 `map`

Repeated:

```phalcom
Some(i).map(f).unwrapOr(0)
```

The successful `map` path must perform zero Option-wrapper heap allocations.

### 23.3 one-armed conditional

Repeated:

```phalcom
condition.ifTrue || { value }
```

taken branch must no longer allocate a `Some`.

### 23.4 `None`

Absence remains zero-allocation but now also avoids the singleton ObjRef load/dereference model.

## Allocation invariant

At least one Rust-level regression test must prove that constructing an Option wrapper changes no heap object count.

If the current heap does not expose a stable test-only count, add a narrow `#[cfg(test)]` heap inspection helper rather than exposing allocation counters in the public VM API.

The stronger representation assertion must also remain:

```text
Some(...) result is never Value::Obj
```

## Do not over-optimize combinators yet

Leave `map`, `flatMap`, `filter`, `unwrapOr`, etc. in `.ph` initially.

Representation elimination is the first optimization.

Only intrinsicize specific combinators later if profiling shows dispatch/block-call overhead dominates.

---

# 24. Task 20 — Compatibility migration policy

For this unit:

```text
Some(x)       canonical
Some.call(x)  explicit equivalent
Some.new(x)   compatibility alias
```

Rules:

1. new core code uses `Some(...)`;
2. new tests use `Some(...)` except compatibility-specific tests;
3. current normative docs use `Some(...)`;
4. examples should migrate;
5. historical documents may retain `Some.new(...)` when describing old behavior;
6. no deprecation warning is required unless Phalcom already has a general deprecation mechanism;
7. removing `new(_)` is a later source-compatibility decision.

This prevents the representation migration from becoming unnecessarily coupled to a mass breaking syntax migration.

---

# 25. Exact implementation sequence

Follow this order. It minimizes states where the VM can compile but GC or bootstrap is unsound.

## Phase A — representation

- [ ] Add failing `value::option` tests.
- [ ] Add `RuntimeError::OptionNestingLimit`.
- [ ] Add `OptionPayload`, `OptionCase`, `MAX_OPTION_NESTING`.
- [ ] Add `Value::None`, `Some1`…`Some7`.
- [ ] Implement wrapping/peeling/classification helpers.
- [ ] Update `Value` exhaustive methods.
- [ ] `cargo check -p phalcom-core`.

## Phase B — primitive behavior

- [ ] Rewrite `primitive/nil.rs`.
- [ ] Install `Some.call(_)`.
- [ ] Keep `Some.new(_)` alias.
- [ ] Add primitive unit tests.

## Phase C — bootstrap

- [ ] Remove `none_singleton`.
- [ ] Remove `_value` layout stamp.
- [ ] Mark Option classes native representation.
- [ ] Bind `None` global to immediate value.
- [ ] Replace kernel invariant assertions.

## Phase D — VM

- [ ] Rewrite `surface_absence`.
- [ ] Rewrite `none_value`.
- [ ] Rewrite `WrapSome`.
- [ ] Rewrite `JumpIfNone`.
- [ ] Update bytecode docs.

## Phase E — GC

- [ ] Add `gc_obj_ref`.
- [ ] Migrate `vm/gc.rs`.
- [ ] Migrate `heap/trace.rs`.
- [ ] Add VM-root and heap-edge forced-GC tests.

**Do not proceed to broad language migration before the GC tests are green.**

## Phase F — compiler/surface

- [ ] Update `is_option_literal`.
- [ ] Add `Some(...)` truthiness negative.
- [ ] Migrate core library to `Some(...)`.
- [ ] Update inliner comments.

## Phase G — rendering and corpus

- [ ] Rewrite raw rendering.
- [ ] Add nested rendering tests.
- [ ] Add depth-7 and depth-8 fixtures.
- [ ] Add `Some.call`/`Some.new` compatibility fixture.
- [ ] Add `flatMap` nesting fixture.

## Phase H — documentation

- [ ] Update current specs.
- [ ] Add ADR amendment/new decision record.
- [ ] Update test manifest.
- [ ] Run repository-wide stale-wording grep.

## Phase I — verification

- [ ] Run focused tests.
- [ ] Run full crate tests.
- [ ] Run full workspace tests if the repository's normal gate does so.
- [ ] Run formatter/lints.
- [ ] Run performance/allocation regressions.

---

# 26. Required command gate

Focused:

```bash
cargo test -p phalcom-core --lib value::option
cargo test -p phalcom-core --test lang absence
cargo test -p phalcom-core --test lang absence_negative
cargo test -p phalcom-core --test lang option
cargo test -p phalcom-core --test lang compile_errors
cargo test -p phalcom-core --test lang values
```

GC / VM targeted tests:

```bash
cargo test -p phalcom-core gc
cargo test -p phalcom-core option
```

Compile exhaustiveness:

```bash
cargo check -p phalcom-core
```

Full crate:

```bash
cargo test -p phalcom-core
```

Formatting/lints:

```bash
cargo fmt --all -- --check
cargo clippy -p phalcom-core --all-targets -- -D warnings
```

If the repository's established CI gate runs broader workspace commands, run those too before completion.

---

# 27. Stale-assumption audit

Before declaring completion:

```bash
rg -n \
  'none_singleton|shared `None`|shared None|singleton.*None|Some\.new\(|_value.*Some|Some.*field|fresh `Some`|fresh Some|Some.*allocat' \
  phalcom-core docs examples
```

Classify every hit:

- **current code/spec/test comment:** update;
- **historical ADR/as-built:** preserve history, add amendment context if needed;
- **compatibility test:** keep intentionally;
- **unrelated Result/Ok/Err `_value`:** do not change.

Also run:

```bash
rg -n 'Value::Obj.*none|none_class.*Instance|Some.*InstanceObject' phalcom-core/src
```

There must be no live Option representation path left.

---

# 28. Non-goals

Do not include these in this patch:

- final NaN-boxing design;
- JIT layout specialization;
- `Option<T>` niche optimization;
- making `Result` immediate;
- changing `Result` / `Ok` / `Err` representation;
- intrinsicizing the whole Option combinator suite;
- automatic Option flattening;
- removing `Some.new(_)` immediately;
- changing public Option hash semantics;
- generalized sum-type runtime representation.

Those are separate design/performance units.

---

# 29. Completion invariants

The unit is complete only when all of the following are true.

## Representation

- [ ] `Some(v)` never creates an `Object::Instance`.
- [ ] `None` is never represented by `ObjRef`.
- [ ] `CoreClasses` has no `none_singleton`.
- [ ] `Some.field_count == 0`.
- [ ] `None.field_count == 0`.
- [ ] Option wrappers are `Copy` immediate `Value`s.

## Semantics

- [ ] `None != Some(None)`.
- [ ] every depth 1–7 is preserved.
- [ ] depth 8 raises the pinned error.
- [ ] `match` removes exactly one layer.
- [ ] `flatMap` avoids unnecessary extra nesting.
- [ ] `Some(x)` and `Some.call(x)` are equivalent.
- [ ] compatibility `Some.new(x)` is equivalent during migration.

## Object model

- [ ] `Some(x).class == Some`.
- [ ] `None.class != None`.
- [ ] both are `isA(Option)`.
- [ ] ordinary `.ph` methods execute with `CallContext::Immediate`.
- [ ] no object materialization path exists.

## Control flow

- [ ] one-armed conditional taken path returns immediate `Some`.
- [ ] untaken path returns immediate `None`.
- [ ] `ifTrue { None }` returns `Some(None)`.
- [ ] `JumpIfNone` does not treat `Some(None)` as absence.

## GC

- [ ] wrapped heap payload survives when Option is a VM root.
- [ ] wrapped heap payload survives when Option is stored inside another heap object.
- [ ] wrapper itself contributes no GC node.

## Documentation

- [ ] current specs contain no heap-singleton / `_value` model.
- [ ] current examples use canonical `Some(...)`.
- [ ] historical decisions are amended rather than rewritten.

---

# 30. Final architecture after this unit

```text
                    heap class metadata
                         Object
                            |
                         Option
                       /        \
                    Some        None
                     ^            ^
                     |            |
            immediate class   immediate class
                mapping          mapping
                     |            |
            +--------+------------+
            |
       generic Value
            |
     +------+------+-------------------------+
     |             |                         |
  raw value      None                 Some^1 ... Some^7
                                     |
                                  payload
                                     |
                   Int / Float / Bool / Unit /
                      Symbol / ObjRef / None
```

Creation:

```text
Some(x)
 -> ordinary global lookup of Some class
 -> ordinary call(_) send
 -> Some metaclass primitive
 -> Value::wrap_some
 -> immediate Value
```

Elimination:

```text
option.match(...)
 -> ordinary Option method lookup
 -> native match primitive
 -> OptionCase
 -> peel one layer
 -> invoke selected block
```

GC:

```text
Some^N(ObjRef(x))
 -> Value::gc_obj_ref()
 -> trace x
```

There is no Option heap object anywhere in those paths.

---

# 31. Implementation guidance

The most important engineering rule in this patch is:

> **Make Option representation a property of `Value`, not a special case scattered through the VM.**

The second is:

> **Treat GC support as part of the representation landing, not as cleanup after the semantic tests pass.**

The third is:

> **Do not special-case `Some(...)` in the parser/compiler. Its elegance comes from using Phalcom's existing callable-object protocol: `Some(x)` is simply `Some.call(x)`.**

With those constraints, the migration is conceptually straightforward: remove one heap object family, replace it with bounded immediate state, and preserve the existing object model above it.

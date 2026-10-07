---
kind: implementation-plan
title: "universe.math Corrective Completion — Luna Patch-Grade Implementation Plan"
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
repository: aureat/phalcom-lang
branch: stdlib-random-math
prepared_against_revision: 9742d085580a98fee8822e1c66026d179ddd772e
main_revision_observed: 615f5887f262f8adcb666f4cb4cb431e1a828e11
prepared: 2026-10-07
scope:
  - universe.math
  - exported static-constant runtime seam required by universe.math
excludes:
  - universe.random
  - Family privacy/receiver opacity redesign
  - general numeric-tower redesign
source_audit: stdlib-math-aggregate-audit-and-design.md
---

# `universe.math` Corrective Completion
## Guided Luna Patch-Grade Implementation Plan

This plan converts the finalized `universe.math` aggregate audit and architecture ruling into an executable patch sequence for the current `stdlib-random-math` branch.

The branch was re-checked before this plan was prepared:

```text
repository: aureat/phalcom-lang
branch:     stdlib-random-math
HEAD:       9742d085580a98fee8822e1c66026d179ddd772e
main:       615f5887f262f8adcb666f4cb4cb431e1a828e11
ahead:      4
behind:     0
```

No branch drift was observed from the aggregate audit.

The implementation is now **complete for the math scope**. The exported-static-constant seam,
hidden-child topology, public math surface, and T9/T10 acceptance are now
implemented and focused-tested. T7 and T8 numeric matrices are now covered by
focused core audit probes. T11 is recorded. T12 focused gates pass; workspace
formatting drift and three unrelated `phalcom-core` library failures remain
recorded baseline exceptions, so verification is `BASELINE_BLOCKED` rather than
release certification. The final evidence and successor handoff are recorded in
the paired walkthrough/handoff.

> **Central implementation ruling:** preserve the public exact-Family API, keep exported-constant semantics at the general module boundary, and keep implementation owners in hidden `universe.math` children. Do not turn constants into runtime `let` bindings or weaken Family semantics to hide kernels.

---

# 0. Executor contract

The implementer must follow these rules throughout the patch.

## 0.1 Repository and branch discipline

Work only on the intended math worktree/branch corresponding to:

```text
remote branch: stdlib-random-math
expected HEAD: 9742d085580a98fee8822e1c66026d179ddd772e
```

Before editing:

```sh
git status --short
git rev-parse HEAD
git merge-base HEAD origin/main
git diff --check
```

If the branch has advanced, been rebased, or contains new unrelated changes, re-read the touched live files before applying this plan. Mechanical line drift is acceptable. Architectural drift is a stop condition.

Preserve unrelated work. Do not stage or rewrite unrelated files simply to make the worktree appear clean.

## 0.2 Scope

This plan owns:

```text
universe.math
+
the general exported-static-constant runtime seam that math requires
```

This plan does **not** implement:

```text
universe.random
Generator / Xoshiro
SecureRandom
Distribution
```

Those begin only after the math gate closes.

## 0.3 Accepted decisions that are not to be reopened

The following decisions are settled for this slice:

1. `universe.math` is the public namespace; there is no public `Math` class.
2. Public mathematical callables are exact captured `Family` values.
3. Float transcendental APIs take `Float`, not `Number`.
4. No implicit `Int -> Float` conversion is added for math convenience.
5. Rust `f64` methods are the approved binary64 backend for this implementation.
6. `gcd`, `lcm`, and `isqrt` remain source-authored Telos algorithms.
7. Exported math constants remain `const`; they are not downgraded to runtime `let`.
8. `FloatMathKernel` and integer implementation owners should live under hidden child modules, not directly in `package.ph`.
9. Hidden implementation modules are registered in the Universe graph but are **not exposed** by `math/package.ph`.
10. `Family.receiver` may reveal the stored kernel class. This is accepted technical debt for this slice.
11. Do not alter Family reflection, Family authorization, or callable-reference semantics to hide the kernel.

## 0.4 General architectural prohibitions

Do not introduce any of the following to make this slice easier:

```text
new VM opcodes for math
new Value tags for math
new heap variants for math
magic compiler recognition of universe.math
synthetic runtime global slots for const
closure wrappers replacing exact Family captures
public Math namespace class
implicit Int -> Float conversion
duplicate static-constant semantic authority
a second module-linking identity system
a math-specific GC root side channel
```

The patch must strengthen existing general abstractions rather than bypass them.

## 0.5 Testing strategy

Use discriminating tests during BUILD.

Do not run the entire workspace after every patch.

The intended cadence is:

```text
one exact reproducer
    ↓
one affected unit/integration cluster
    ↓
math corpus
    ↓
broader module/native/math suites
    ↓
final workspace-adjacent verification
```

Every named command must be reported truthfully as PASS / FAIL / NOT RUN.

---

# 1. Goal

Deliver a merge-ready first complete `universe.math` package with:

```text
canonical source ownership
exact Family exports
binary64 elementary math
exact arbitrary-precision integer math
compile-time constants
hidden implementation modules
correct cross-module const behavior
accurate reflection/provenance
GC-safe static materialization
honest backend precision documentation
full focused verification
```

The final user experience should be:

```telos
import universe.math as math

let circumference = math.tau * 5.0
let diagonal = math.hypot(3.0, 4.0)
let root = math.sqrt(2.0)

let logarithm = math.log(8.0, base: 2.0)

let angle = math.sin(math.radians(45.0))

let divisor = math.gcd(84, 30)
let common = math.lcm(21, 6)
let exactRoot = math.isqrt(10 ** 100)
```

The target implementation topology is:

```text
universe.math
│
├── package.ph
│   ├── documentation
│   ├── canonical public const values
│   ├── imports of hidden implementation owners
│   ├── exact Family captures
│   ├── tiny source helpers such as degrees/radians
│   └── public export list
│
├── float_kernel.ph          hidden from public traversal
│   └── native class FloatMathKernel
│
└── integer.ph               hidden from public traversal
    └── class IntegerMath
```

No:

```telos
expose .float_kernel
expose .integer
```

may appear in the final facade.

---

# 2. Current repository truth

## 2.1 Current `math/package.ph`

The branch currently contains all Float kernel declarations in the package facade and publicly exposes `.integer`:

```telos
expose .integer

from universe.math.integer import IntegerMath

native class FloatMathKernel is Object {
  @class native sqrt(_ value: Float) -> Float
  ...
}

const pi: Float = ...
...
let sqrt = &FloatMathKernel.sqrt(_)
...
```

This is the correct API idea but the wrong package decomposition.

## 2.2 Current integer implementation

`phalcom-core/core/universe/src/math/integer.ph` currently owns:

```text
IntegerMath.gcd
IntegerMath.lcm
IntegerMath.isqrt
```

The algorithms are sound enough to preserve and harden rather than replace.

## 2.3 Current Float native implementation

`phalcom-core/src/primitive/math.rs` contains:

```text
21 unary f64 kernels
2 binary f64 kernels
```

and uses source-owned primitive descriptors:

```rust
#[phalcom_native_macros::primitive(
    FloatMathKernel,
    "sqrt(_)",
    ...
)]
```

Registration is descriptor-driven. Do not hand-install these in `Universe::install_primitives`.

## 2.4 Current exported-static-constant patch

The branch added:

```rust
RuntimeExportRef::StaticConstant(Value)
```

and:

```rust
RuntimeLinkedRead::StaticConstant {
    module: ObjRef,
    value: Value,
}
```

plus primitive materialization in:

```text
phalcom-core/src/modules/static_constant.rs
```

The current exact failing reproducer is:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  exported_static_constant_materializes_for_selected_and_module_reads \
  -- --nocapture
```

Observed failure:

```text
3.141592653589793 does not understand 'call'
```

on module-qualified:

```telos
constants.pi
```

The immediate cause is that the environment-aware module export send resolves the constant to a Float, then continues through callable activation rather than returning the value for Getter dispatch.

## 2.5 Deeper constant problem

The current runtime representation stores the `Value` directly inside export/read descriptors and loses canonical source identity after re-export boundaries.

The semantic owner already exists:

```rust
StaticConstantId(SymbolId)
```

The runtime representation should be an isomorphic VM-local projection of that identity, not only a copied `Value`.

---

# 3. Final invariants

These invariants are mandatory acceptance conditions.

## 3.1 API and module identity

```text
MATH-P-INV-01  universe.math is the sole public math namespace.
MATH-P-INV-02  There is no public Math class.
MATH-P-INV-03  Public Float operations are exact bound Family captures.
MATH-P-INV-04  Public captures use exact selector shapes, not whole-family captures.
MATH-P-INV-05  FloatMathKernel is declared in universe.math.float_kernel.
MATH-P-INV-06  IntegerMath is declared in universe.math.integer.
MATH-P-INV-07  float_kernel and integer are registered Universe child nodes.
MATH-P-INV-08  float_kernel and integer are not exposed by universe.math.
MATH-P-INV-09  External traversal to hidden implementation children fails.
MATH-P-INV-10  Family.receiver kernel reachability is accepted and not “fixed” here.
```

## 3.2 Numeric semantics

```text
MATH-P-INV-11  Float elementary/transcendental functions accept Float only.
MATH-P-INV-12  No math call silently converts Int to Float.
MATH-P-INV-13  gcd/lcm/isqrt accept Int and remain exact.
MATH-P-INV-14  gcd(a,b) is non-negative and sign-insensitive.
MATH-P-INV-15  lcm(a,b) is non-negative and zero if either operand is zero.
MATH-P-INV-16  isqrt(n) satisfies r² <= n < (r+1)².
MATH-P-INV-17  isqrt(negative) raises ArgumentError.
MATH-P-INV-18  Existing Float receiver operations are not duplicated under math.
MATH-P-INV-19  Trigonometric values use radians.
MATH-P-INV-20  degrees/radians are explicit Float-only helpers.
```

## 3.3 Static constants

```text
MATH-P-INV-21  math constants remain compile-time const.
MATH-P-INV-22  const does not acquire a synthetic runtime global slot.
MATH-P-INV-23  exported const identity remains the canonical source StaticConstantId.
MATH-P-INV-24  re-export aliases do not become new constant identities.
MATH-P-INV-25  selected imports retain canonical source provenance.
MATH-P-INV-26  module-qualified Getter access returns the constant value directly.
MATH-P-INV-27  one canonical VM-local materialization is reused per static constant.
MATH-P-INV-28  heap-backed materialized constants are traced by the owning module.
MATH-P-INV-29  unsupported/deferred static materialization fails closed.
MATH-P-INV-30  no fallible constant path panics via expect/unwrap.
```

## 3.4 Native/runtime

```text
MATH-P-INV-31  FloatMathKernel native metadata points to math.float_kernel.
MATH-P-INV-32  native descriptors and source declarations have one canonical identity.
MATH-P-INV-33  math does not add VM opcodes/value kinds/heap variants.
MATH-P-INV-34  native Float methods are pure and do not allocate.
MATH-P-INV-35  native surface generation remains deterministic.
MATH-P-INV-36  unrelated generated-surface baseline drift is explained or isolated.
```

## 3.5 Precision/documentation

```text
MATH-P-INV-37  Float is IEEE-754 binary64.
MATH-P-INV-38  sqrt may inherit Rust's stronger correctly-rounded guarantee.
MATH-P-INV-39  unspecified-precision Rust transcendental methods are not advertised as <=1 ULP.
MATH-P-INV-40  unspecified-precision functions are not promised bit-identical across platforms/toolchain versions.
MATH-P-INV-41  NaN payload bits are not public API state.
```

---

# 4. Required implementation architecture for exported static constants

This is the most important corrective step in the slice.

Do not merely patch the Getter early return and leave `RuntimeExportRef::StaticConstant(Value)` as the final architecture.

The preferred patch-grade runtime design is module-owned canonical materialization.

## 4.1 Add `RuntimeStaticConstantRef`

Add a compact, `Copy`, VM-local reference, preferably in `phalcom-core/src/modules/linkage.rs` or another module-runtime identity file:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeStaticConstantRef {
    /// Canonical source module that owns the const declaration.
    pub module: ObjRef,

    /// Canonical source name inside that module.
    pub name: Symbol,
}
```

This pair is the VM-local projection of:

```text
StaticConstantId(SymbolId {
    module,
    name,
})
```

It intentionally stores the canonical source name, not a facade/re-export alias.

Why this design:

- remains `Copy`;
- avoids propagating semantic-layer owned `ModuleId`/`String` objects through hot runtime enums;
- retains canonical provenance;
- allows re-export aliases to point to the same source constant;
- allows one module-local materialization;
- avoids storing duplicate heap-backed Values in every import/export descriptor.

## 4.2 Change runtime enum shapes

Target:

```rust
pub enum RuntimeLinkedRead {
    Module(ObjRef),
    Binding(BindingRef),
    StaticConstant(RuntimeStaticConstantRef),
}

pub enum RuntimeExportRef {
    Binding(BindingRef),
    StaticConstant(RuntimeStaticConstantRef),
    Module(ObjRef),
}
```

Do not keep:

```rust
StaticConstant(Value)
```

as the public runtime export identity.

## 4.3 Give `ModuleObject` owned static-constant storage

Add:

```rust
pub static_constants: HashMap<Symbol, Value>,
```

to `ModuleObject`.

Initialize it empty in `ModuleObject::new`.

The owner module then physically owns the one VM-local materialized representation of each materialized primitive constant.

Conceptual invariant:

```text
RuntimeStaticConstantRef
    -> canonical owner ModuleObject
    -> canonical source-name Symbol
    -> owner.static_constants[name]
    -> Value
```

## 4.4 Add one materialization authority

Refactor `phalcom-core/src/modules/static_constant.rs`.

Current:

```rust
materialize_static_constant(&StaticConstantLoweringSpec) -> Value
```

should become an internal low-level constructor plus a canonical ensuring operation.

Recommended shape:

```rust
impl VM {
    fn materialize_static_constant_value(
        &mut self,
        spec: &StaticConstantLoweringSpec,
    ) -> PhResult<Value>;

    pub(crate) fn ensure_static_constant_materialized(
        &mut self,
        spec: &StaticConstantLoweringSpec,
    ) -> PhResult<RuntimeStaticConstantRef>;

    pub(crate) fn resolve_static_constant(
        &self,
        reference: RuntimeStaticConstantRef,
    ) -> PhResult<Value>;

    pub(crate) fn runtime_static_constant_symbol(
        &self,
        reference: RuntimeStaticConstantRef,
    ) -> PhResult<SymbolId>;
}
```

Names may vary. Ownership may not.

### `ensure_static_constant_materialized`

Algorithm:

```text
read spec.id
    ↓
locate canonical owner module in registry
    ↓
intern spec.id.name()
    ↓
if owner.static_constants already contains name
    return RuntimeStaticConstantRef(owner,name)
    ↓
materialize descriptor once
    ↓
insert value into owner.static_constants
    ↓
return RuntimeStaticConstantRef(owner,name)
```

Do not allocate another String/LargeInt when the constant is already present.

### `resolve_static_constant`

Algorithm:

```text
reference.module
    ↓
ModuleObject.static_constants
    ↓
reference.name
    ↓
return stored Value
```

If absent, return a structured `RuntimeError::Internal` with:

```text
module id
canonical constant name
```

Do not return `None`, `Nil`, or an empty runtime slot.

## 4.5 Re-export identity law

For:

```text
A:
    const x = ...

B:
    re-export A.x as y

C:
    import B.y
```

all runtime references must eventually be:

```text
RuntimeStaticConstantRef {
    module: A,
    name: #x,
}
```

not:

```text
B/#y
```

The alias affects the public export-table key only.

It does not change source identity.

---

# 5. Execution DAG

Implement in this order:

```text
T0 Baseline
 │
 ▼
T1 RuntimeStaticConstantRef + owner storage
 │
 ▼
T2 Static materialization/export/import/send/reflection repair
 │
 ├───────────────┐
 ▼               │
T3 Static const  │
focused proof    │
 │               │
 └──────┬────────┘
        ▼
T4 Hidden math child topology
        │
        ▼
T5 Move FloatMathKernel + native owner metadata
        │
        ▼
T6 Complete public math facade/API
        │
        ├───────────────┐
        ▼               ▼
T7 Float backend      T8 Exact integer
verification          verification
        │               │
        └───────┬───────┘
                ▼
T9 Family/type/topology acceptance
                │
                ▼
T10 Generated surface hygiene
                │
                ▼
T11 Documentation/lifecycle
                │
                ▼
T12 Final verification and delivery
```

`T4+` must not proceed while `T3` is still failing. Package restructuring on top of a broken const runtime will make root-cause discrimination harder.

---

# 6. T0 — Baseline and discriminator freeze

## Purpose

Establish the exact failure state before structural edits.

## Files

No production edits.

Potential test inspection:

```text
phalcom-core/tests/core/modules/parity.rs
phalcom-core/tests/core/modules/linking.rs
phalcom-core/tests/core/language/corpus.rs
phalcom-core/tests/fixtures/language/math/math_foundation.ph
```

## Procedure

Run:

```sh
git status --short
git diff --check

RUSTFLAGS='' cargo check -p phalcom-core

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  exported_static_constant_materializes_for_selected_and_module_reads \
  -- --nocapture
```

Confirm that the failure remains:

```text
module-qualified constants.pi
-> Float value
-> attempted call
-> does not understand 'call'
```

Then run the existing math lane:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus math -- --nocapture
```

Record whether it selects tests and its exact failure/pass state.

## Exit criterion

A written execution note records:

```text
HEAD
working-tree state
cargo check result
focused const discriminator result
math corpus result
```

Do not edit production code until the current failure is reproduced or explained.

## Stop condition

If the current failure is no longer reproducible without branch changes, re-audit the branch before continuing.

---

# 7. T1 — Introduce canonical runtime static-constant references

## Purpose

Correct the runtime identity model before fixing call sites.

## Primary files

```text
phalcom-core/src/modules/linkage.rs
phalcom-core/src/heap/module.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/heap/trace.rs
```

Possible exhaustiveness consumers:

```text
phalcom-core/src/modules/materialize.rs
phalcom-core/src/modules/context.rs
phalcom-core/src/primitive/reflection.rs
phalcom-core/src/modules/reflection_cache.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/bootstrap.rs
tests that match RuntimeLinkedRead / RuntimeExportRef
```

## T1.1 Add `RuntimeStaticConstantRef`

In `modules/linkage.rs`, define:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeStaticConstantRef {
    pub module: ObjRef,
    pub name: Symbol,
}
```

Import `crate::interner::Symbol`.

Change:

```rust
RuntimeLinkedRead::StaticConstant {
    module: ObjRef,
    value: Value,
}
```

to:

```rust
RuntimeLinkedRead::StaticConstant(RuntimeStaticConstantRef)
```

Re-export as needed from `modules/mod.rs`.

## T1.2 Update `RuntimeExportRef`

In `heap/module.rs`:

```rust
RuntimeExportRef::StaticConstant(RuntimeStaticConstantRef)
```

instead of:

```rust
RuntimeExportRef::StaticConstant(Value)
```

Update imports/re-exports in `heap/mod.rs`.

## T1.3 Add module-owned value cache

Add to `ModuleObject`:

```rust
pub static_constants: HashMap<Symbol, Value>,
```

Initialize in `ModuleObject::new`.

Add narrow helpers if they improve invariants:

```rust
pub fn static_constant(&self, name: Symbol) -> Option<Value>
pub fn set_static_constant(&mut self, name: Symbol, value: Value)
```

Do not expose this as a language-level binding table.

It is not a runtime global namespace.

## T1.4 Rewrite static materialization helper

In `modules/static_constant.rs`, separate:

```text
descriptor -> Value
```

from:

```text
canonical owner -> materialize/cache/reference
```

Required behaviors:

### Primitive descriptors

Continue supporting:

```text
Unit
Bool
small Int
large Int
Float
String
Symbol
```

### Unsupported descriptors

Continue failing closed for:

```text
DeferredStructural
Unsupported
```

Use a structured `RuntimeError::Internal` carrying `spec.id`.

### Canonical caching

Before allocating `LargeInt` or String:

```text
look up owner.static_constants[name]
```

If present, return the existing ref.

If absent:

1. materialize;
2. insert into owner storage;
3. return `RuntimeStaticConstantRef`.

## T1.5 Remove panic paths

Replace:

```rust
.expect("static constant owner registered")
```

with:

```rust
.ok_or_else(|| RuntimeError::Internal(...))?
```

Every invariant error should include the `StaticConstantId` or owner module/name.

## T1.6 GC tracing

In `heap/trace.rs`, the module arm must:

1. trace every `module.static_constants.values()`;
2. for `RuntimeExportRef::StaticConstant(reference)`, trace/push `reference.module`;
3. for `RuntimeLinkedRead::StaticConstant(reference)`, trace/push `reference.module`.

Do **not** trace copied Values through import/export descriptors because those descriptors no longer own Values.

Canonical heap-backed constant Values are rooted by their owner module's `static_constants`.

This creates one clear ownership graph:

```text
module registry root
    ↓
owner ModuleObject
    ↓
static_constants
    ↓
String / LargeInt object
```

## T1 tests

Add focused Rust tests where natural.

At minimum verify:

```text
same spec materialized twice -> same runtime ref
String materialization is reused
LargeInt materialization is reused
unsupported descriptor -> error
missing owner -> error, not panic
```

If object-handle equality is available in tests, prove no second String/LargeInt allocation is required for a repeated ensure.

## T1 exit criterion

```text
cargo check -p phalcom-core
```

passes and all enum match sites compile with the new reference type.

Do not yet claim behavioral correctness.

---

# 8. T2 — Route exports, imports, getters, and reflection through the canonical constant ref

## Purpose

Make all runtime consumers use the new source-owned constant reference.

## Primary files

```text
phalcom-core/src/modules/materialize.rs
phalcom-core/src/modules/context.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/primitive/reflection.rs
phalcom-core/src/modules/reflection_cache.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/vm/bootstrap.rs
```

## T2.1 `materialize_linked_exports_for_module`

Current code:

```rust
if let Some(spec) = constant {
    exports.insert(
        public_sym,
        RuntimeExportRef::StaticConstant(
            self.materialize_static_constant(&spec)?
        )
    );
    continue;
}
```

Replace with:

```text
lookup canonical source `symbol`
    ↓
static_constant_spec(symbol.module, symbol.name)
    ↓
ensure_static_constant_materialized(spec)
    ↓
RuntimeExportRef::StaticConstant(reference)
```

The export table key remains `public_sym`.

The `reference` remains source canonical.

This is what makes renamed re-exports correct automatically.

## T2.2 `materialize_linked_reads_for_module`

For:

```rust
LinkedReadSpec::Binding(symbol_id)
```

if that canonical binding is a static constant:

```text
ensure owner materialization
    ↓
RuntimeLinkedRead::StaticConstant(reference)
```

Do not store copied `Value`.

Do not allocate a global slot.

Do not use `continue` in a way that makes logic structurally different from other cases unless needed; keep one clear push path if borrow structure allows.

## T2.3 Runtime linked-read evaluation

Inventory every consumer of:

```rust
RuntimeLinkedRead
```

including bytecode linked read handling in `vm/dispatch.rs`.

Static read behavior must be:

```text
RuntimeLinkedRead::StaticConstant(reference)
    ↓
resolve owner.static_constants[reference.name]
    ↓
push Value
```

No global slot.

No re-materialization at read time.

## T2.4 Module export sends

There are two module export-send paths in `vm/send.rs`.

Unify behavior conceptually even if helpers remain separate.

For:

```rust
RuntimeExportRef::StaticConstant(reference)
```

resolve the value through the canonical helper.

Then:

### Getter-shaped send

Return the value immediately:

```rust
if matches!(kind, SignatureKind::Getter) {
    ...
    return Ok(...)
}
```

This fixes the recorded `Float.call` failure.

### Non-Getter send

If a user explicitly performs a Method-shaped operation with the export base, existing semantics may route to `call` on the exported value just as for ordinary binding exports.

Do not special-case arbitrary method calls merely because the value came from `const`.

The fix is Getter correctness, not a new dispatch category.

## T2.5 `ModuleExecutionContext` selective imports

Current static path reconstructs:

```text
resolved facade module
+
public item alias/name
```

and loses the canonical source target.

Replace it.

For:

```rust
RuntimeExportRef::StaticConstant(reference)
```

derive canonical source identity from the reference:

```text
reference.module -> ModuleObject.id
reference.name   -> canonical source name
```

Construct:

```rust
SymbolId {
    module: canonical_owner_id,
    name: canonical_name,
}
```

Then:

```rust
RuntimeLinkedRead::StaticConstant(reference)
LinkedReadSpec::Binding(canonical_symbol.clone())
symbol: Some(canonical_symbol)
```

This makes static selected imports match ordinary linked-import provenance behavior.

## T2.6 Reflection `Module.dependencies`

For static linked reads, dependency module must be the canonical source owner:

```text
reference.module
```

not the facade from which an alias happened to be imported.

Update the exhaustiveness match.

## T2.7 Reflection `Export.value`

Inventory and update:

```text
Export.value
Module.exports
ExportTable construction
```

`Export.value` for a static constant must resolve the same canonical value through `RuntimeStaticConstantRef`.

`Export.kind` should remain `#binding` unless the public reflection spec defines a new static-constant kind. Do not add a new public reflection enum merely for this slice.

## T2.8 Reflection caches

`ReflectionCache::get_or_create_export_table` may continue classifying:

```text
Binding | StaticConstant -> #binding
Module                  -> #module
```

but any code expecting `StaticConstant(Value)` must be updated.

No reflection descriptor should duplicate constant storage.

## T2.9 Bootstrap

`prepare_universe_static_constants` currently attaches lowering data.

Do not make it eagerly allocate every Universe constant unless required by normal materialization order.

Preferred behavior:

```text
attach canonical lowering specs
    ↓
materialize constants on actual linked export/read demand
```

If bootstrap order makes eager preparation necessary, use the same canonical `ensure_static_constant_materialized` authority.

Do not create a second bootstrap-only materializer.

Consider renaming:

```text
prepare_universe_static_constants
```

to a name accurately describing what it does if the edit is low risk, e.g.:

```text
attach_universe_lowering_semantics
prepare_universe_static_constant_specs
```

This rename is optional if it would create noisy unrelated churn. At minimum correct the doc comment.

## T2 tests

Before any package restructure, make the existing reproducer pass:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  exported_static_constant_materializes_for_selected_and_module_reads \
  -- --nocapture
```

Extend it or add adjacent focused cases for:

```text
selected import
module-qualified getter
selected alias
direct re-export
renamed re-export
two-hop re-export
```

Use both Float and heap-backed primitive constants.

## T2 exit criterion

The known getter failure is gone and canonical source identity is preserved through re-export.

---

# 9. T3 — Prove the static-constant runtime seam before touching math structure

## Purpose

Close the generic runtime blocker with tests that would fail under the old representation.

## Primary test files

Likely:

```text
phalcom-core/tests/core/modules/parity.rs
phalcom-core/tests/core/modules/linking.rs
phalcom-core/tests/core/modules/reflection.rs
phalcom-core/tests/core/modules_universe.rs
```

Use existing suite organization rather than creating a new top-level harness without need.

## T3.1 Required regression matrix

Create discriminating cases for:

### A. direct selected import

```telos
// constants.ph
const pi = 3.141592653589793
export pi

// main.ph
from .constants import pi
System.print(pi)
```

Assert:

```text
value correct
no runtime pi global slot required
```

### B. module-qualified read

```telos
import .constants as constants
System.print(constants.pi)
```

### C. selected alias

```telos
from .constants import pi as circleConstant
```

Canonical identity must still be `constants::pi`.

### D. direct re-export

```text
source.pi
    ↓
facade re-exports pi
    ↓
consumer
```

### E. renamed re-export

```text
source.pi
    ↓
facade exports pi as circle
    ↓
consumer.circle
```

Runtime ref must point to source `pi`.

### F. two-hop renamed re-export

Use two facades.

### G. reflection

Verify:

```text
Module.exports
Export.value
Module.dependencies
```

observe correct value/owner.

### H. heap-backed String

```telos
const message = "persistent"
```

Read through export before and after forced GC.

### I. heap-backed LargeInt

Use a compile-time constant beyond inline `Int`.

Read before and after forced GC.

### J. unsupported/deferred static value

If a structural static constant currently cannot materialize at runtime, exporting/reading it must produce explicit failure, never `None` or an uninitialized slot.

## T3.2 GC proof

The test must force:

```rust
vm.force_gc()
```

after the constant is materialized and before its next read.

The only required root should be the canonical owner module path through `ModuleObject.static_constants`.

Do not root test Values manually in a way that masks the module-edge requirement.

## T3.3 Slot absence

For `const pi`, assert the source module has no ordinary global slot for `pi`, consistent with:

```text
const = compile-time binding
```

The static constant map is not the globals table.

## T3 commands

Focused:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  exported_static_constant_materializes_for_selected_and_module_reads \
  -- --nocapture
```

Then the focused module cluster containing the new cases.

Do not advance if any provenance/re-export test fails.

## T3 exit criterion

The exported-const runtime seam is considered stable enough for math only when:

```text
selected PASS
qualified PASS
alias PASS
re-export PASS
renamed re-export PASS
two-hop PASS
reflection PASS
String GC PASS
LargeInt GC PASS
unsupported materialization FAILS CLOSED
no runtime const slot PASS
```

---

# 10. T4 — Reorganize `universe.math` into a facade with hidden children

## Purpose

Correct package topology before extending the public API.

## Files

```text
phalcom-core/core/universe/src/math/package.ph
phalcom-core/core/universe/src/math/float_kernel.ph   NEW
phalcom-core/core/universe/src/math/integer.ph
phalcom-modules/src/builtin.rs
```

## T4.1 Create `float_kernel.ph`

Move the complete `FloatMathKernel` source declaration out of `package.ph`.

Initial target:

```telos
/// Internal implementation owner for binary64 mathematical operations.
///
/// Public users consume exact Families re-exported by `universe.math`.
native class FloatMathKernel is Object {
  @class native sqrt(_ value: Float) -> Float
  ...
}

export FloatMathKernel
```

Do not mark the methods `internal`; Family activation would enforce caller authority and make the public captures unusable.

The child module export is required only so the parent package can import the class.

The child path remains private because the package does not expose it.

## T4.2 Keep `IntegerMath` in `integer.ph`

Do not move the exact algorithms back into `package.ph`.

Keep:

```telos
export IntegerMath
```

inside the child so the parent can import it.

## T4.3 Rewrite `package.ph` imports

Target:

```telos
from .float_kernel import FloatMathKernel
from .integer import IntegerMath
```

Use canonical relative-import syntax supported by the current resolver.

Remove:

```telos
expose .integer
```

Do **not** add:

```telos
expose .float_kernel
```

The package's implementation children must not become external traversal paths.

## T4.4 Register hidden child nodes

`phalcom-modules/src/builtin.rs::UNIVERSE_NODES` must know physical logical children even if they are not public exposure edges.

Change math node topology from:

```text
math children: integer
```

to:

```text
math children:
    float_kernel
    integer
```

Add:

```text
["math", "float_kernel"] -> Module
["math", "integer"]      -> Module
```

Do not confuse provider topology with package `expose` semantics.

## T4.5 Add source-text route

In `UniverseSourceProvider::source_text` add:

```rust
[c, m] if c == "math" && m == "float_kernel" =>
    include_str!(".../math/float_kernel.ph")
```

Keep the `integer.ph` mapping.

## T4.6 Public exposure negative tests

Add a module-resolution test that external code cannot traverse:

```text
universe.math.float_kernel
universe.math.integer
```

The exact expected diagnostic should use current module exposure semantics, likely `ModulePathNotExposed`.

Also prove relative imports from `universe.math` to those children succeed.

## T4 exit criterion

```text
math package resolves
parent can import children
external paths are not exposed
package.ph contains no native class body
```

---

# 11. T5 — Move native canonical ownership with `FloatMathKernel`

## Purpose

Keep source declaration identity and native metadata aligned after T4.

## Files

```text
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs
phalcom-core/src/primitive/math.rs
phalcom-core/src/universe/core_classes.rs
native/source installation code if path assertions require updates
native-surface tests / source-anchor tests
```

## T5.1 Change canonical path

In:

```rust
UniverseKey::source_path()
```

change:

```rust
Self::FloatMathKernel => &["math"],
```

to:

```rust
Self::FloatMathKernel => &["math", "float_kernel"],
```

The class name remains:

```text
FloatMathKernel
```

## T5.2 Keep it source-owned

Do not turn `FloatMathKernel` into a primordial runtime class with an independent native declaration.

Its class object should still be created from the source-owned Universe declaration and have native primitive descriptors attached afterward.

## T5.3 Update native module documentation

In:

```text
phalcom-core/src/primitive/math.rs
```

change the module doc comment from:

```text
declared in universe.math
```

to:

```text
declared in universe.math.float_kernel
```

## T5.4 Native descriptor identity

All `#[primitive(FloatMathKernel, ...)]` rows remain associated with the same `UniverseKey`, but the generated source anchor must now point to the new module path.

Regenerate only after the source move and API additions are final enough to avoid repeated large generated diffs.

## T5.5 Core-class ownership audit

Re-read:

```text
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/native/source.rs
phalcom-core/src/native/install.rs
```

No new primordial slot should appear for this source-owned class.

Any path-based source association must use `UniverseKey::source_path()` rather than a hard-coded `["math"]`.

## T5 tests

Run source-owned native association tests and descriptor parity tests.

Add/adjust a test that the declaration resolves as:

```text
universe.math.float_kernel::FloatMathKernel
```

not:

```text
universe.math::FloatMathKernel
```

## T5 exit criterion

There is one and only one canonical source declaration for `FloatMathKernel`, and all native metadata points to it.

---

# 12. T6 — Complete the public `math` facade

## Purpose

Finish the v1 elementary math API after the architecture is stable.

## Files

```text
phalcom-core/core/universe/src/math/package.ph
phalcom-core/core/universe/src/math/float_kernel.ph
phalcom-core/src/primitive/math.rs
```

## T6.1 Preserve current constants

Keep in `package.ph`:

```telos
const pi: Float = 3.141592653589793
const tau: Float = 6.283185307179586
const e: Float = 2.718281828459045
const ln2: Float = 0.6931471805599453
const ln10: Float = 2.302585092994046
const sqrt2: Float = 1.4142135623730951
```

They must remain canonically owned as:

```text
universe.math::pi
...
```

Do not move them to a child module and re-export them.

## T6.2 Preserve current Float families

Retain:

```text
sqrt
cbrt
exp
exp2
expm1
ln
ln1p
log2
log10
sin
cos
tan
asin
acos
atan
atan2
sinh
cosh
tanh
asinh
acosh
atanh
hypot
```

## T6.3 Add arbitrary-base `log`

Add a kernel declaration whose exact selector is:

```text
log(_,base)
```

Use the current Telos labeled-parameter declaration syntax; verify it from live grammar before editing.

The public capture must be exact:

```telos
let log = &FloatMathKernel.log(_, base)
```

not a whole-family capture.

Native implementation:

```rust
value.log(base)
```

using Rust `f64::log`.

Do not implement:

```text
ln(value) / ln(base)
```

in source when the selected backend provides a dedicated operation.

Native descriptor should reflect selector:

```text
log(_,base)
```

and parameter types:

```text
(Float, Float) -> Float
```

## T6.4 Add angle conversion helpers

Add:

```text
degrees(Float) -> Float
radians(Float) -> Float
```

Semantics:

```text
degrees(x) = x * (180.0 / pi)
radians(x) = x * (pi / 180.0)
```

Preferred implementation: ordinary source method families in `package.ph` if top-level method declarations are supported cleanly by the live parser/checker.

If direct top-level source methods are not a supported package declaration form:

1. STOP before inventing closures.
2. Use a source-owned helper class in a hidden child only if needed.
3. Capture exact Families from that owner.

Do not make these native merely because package-level source methods are inconvenient.

## T6.5 Preserve integer captures

Keep:

```telos
let gcd = &IntegerMath.gcd(_, _)
let lcm = &IntegerMath.lcm(_, _)
let isqrt = &IntegerMath.isqrt(_)
```

## T6.6 Final export list

Target public API:

```text
constants:
    pi
    tau
    e
    ln2
    ln10
    sqrt2

elementary:
    sqrt
    cbrt
    exp
    exp2
    expm1
    ln
    ln1p
    log
    log2
    log10
    hypot

trigonometric:
    sin
    cos
    tan
    asin
    acos
    atan
    atan2
    degrees
    radians

hyperbolic:
    sinh
    cosh
    tanh
    asinh
    acosh
    atanh

exact integer:
    gcd
    lcm
    isqrt
```

## T6.7 Explicit non-additions

Do not add:

```text
abs
floor
ceil
round
trunc
pow
min
max
clamp
factorial
comb
perm
gamma
erf
complex math
statistics
```

in this slice.

---

# 13. T7 — Float backend correctness and contract verification

## Purpose

Verify the native seam without promising accuracy beyond Rust's actual contract.

## Production files

Likely only:

```text
phalcom-core/src/primitive/math.rs
```

for the new `log` method.

## Test locations

Prefer a combination of:

```text
Rust primitive/integration tests
language corpus fixtures
semantic/type tests
```

## T7.1 Backend mapping table

Each public kernel must have an explicit test mapping:

| Telos | Rust backend |
|---|---|
| `sqrt` | `f64::sqrt` |
| `cbrt` | `f64::cbrt` |
| `exp` | `f64::exp` |
| `exp2` | `f64::exp2` |
| `expm1` | `f64::exp_m1` |
| `ln` | `f64::ln` |
| `ln1p` | `f64::ln_1p` |
| `log` | `f64::log` |
| `log2` | `f64::log2` |
| `log10` | `f64::log10` |
| `sin` | `f64::sin` |
| `cos` | `f64::cos` |
| `tan` | `f64::tan` |
| `asin` | `f64::asin` |
| `acos` | `f64::acos` |
| `atan` | `f64::atan` |
| `atan2` | `f64::atan2` |
| `sinh` | `f64::sinh` |
| `cosh` | `f64::cosh` |
| `tanh` | `f64::tanh` |
| `asinh` | `f64::asinh` |
| `acosh` | `f64::acosh` |
| `atanh` | `f64::atanh` |
| `hypot` | `f64::hypot` |

For ordinary finite inputs, test Telos output against the exact same host operation.

This verifies:

```text
native registration
argument extraction
argument order
result transport
Family routing
```

without inventing a stronger numerical guarantee.

## T7.2 Exact bit checks for constants

Freeze:

```text
pi    0x400921fb54442d18
tau   0x401921fb54442d18
e     0x4005bf0a8b145769
ln2   0x3fe62e42fefa39ef
ln10  0x40026bb1bbb55516
sqrt2 0x3ff6a09e667f3bcd
```

Do not rely on formatted decimal output.

## T7.3 Special-value matrix

Create focused tests for:

```text
+0.0
-0.0
+Infinity
-Infinity
NaN
representative subnormal
```

and function boundaries.

### `sqrt`

Verify:

```text
sqrt(-1.0).isNaN
sqrt(-0.0) preserves negative-zero bits
sqrt(+0.0) preserves positive-zero bits
sqrt(+Infinity) == +Infinity
```

### logarithms

Verify:

```text
ln(-1.0) is NaN
ln(+0.0) == -Infinity
ln(-0.0) == -Infinity
ln(1.0) == +0.0
ln(+Infinity) == +Infinity
```

Apply analogous targeted cases to:

```text
ln1p
log
log2
log10
```

according to the backend behavior.

### inverse trigonometry

Test:

```text
-1
+1
just outside domain
Infinity
NaN
```

### `atan2`

This is especially important because argument order is easy to reverse.

Verify signed-zero quadrants against Rust:

```text
atan2(+0,+0)
atan2(+0,-0)
atan2(-0,+0)
atan2(-0,-0)
```

plus finite quadrants and infinities.

### hyperbolic

Test boundary/domain cases for:

```text
acosh
atanh
```

## T7.4 Precision tiers

Do not write:

```text
all math <= 1 ULP
```

as a test invariant.

Use:

### Tier A — specified exact backend behavior

For `sqrt`, broad bit-exact testing is appropriate.

### Tier B — same-platform backend equivalence

For the other Rust methods, Telos must equal the selected Rust operation for the same inputs on the same test run.

### Tier C — optional high-precision sanity

If a high-precision oracle already exists in the repository or can be added without dependency churn, use it only to detect catastrophic numerical regressions.

Such thresholds are test-engineering limits, not public API guarantees.

Do not add a new heavy numerical dependency merely for this slice without approval.

## T7.5 Purity

The native descriptor already declares:

```text
effects = pure
```

Verify no kernel:

```text
mutates VM state
allocates heap objects
uses system time
uses random state
```

---

# 14. T8 — Exact integer algorithm hardening

## Purpose

Prove the source-authored algorithms over both inline and arbitrary-precision `Int`.

## Production file

```text
phalcom-core/core/universe/src/math/integer.ph
```

Change code only if tests reveal a defect or resource-path problem.

## T8.1 `gcd` tests

Required examples:

```text
gcd(0,0) == 0
gcd(0,1) == 1
gcd(1,0) == 1
gcd(84,30) == 6
gcd(-84,30) == 6
gcd(84,-30) == 6
gcd(-84,-30) == 6
```

Property-style checks over a deterministic matrix:

```text
g = gcd(a,b)

g >= 0

if g != 0:
    a % g == 0
    b % g == 0

gcd(a,b) == gcd(b,a)
```

Include LargeInt values.

## T8.2 `lcm` tests

Examples:

```text
lcm(0,n) == 0
lcm(n,0) == 0
lcm(21,6) == 42
lcm(-21,6) == 42
lcm(21,-6) == 42
lcm(-21,-6) == 42
```

For nonzero pairs:

```text
gcd(a,b) * lcm(a,b) == abs(a*b)
```

Use existing expressions for absolute-value normalization rather than adding `math.abs`.

## T8.3 `isqrt` tests

Base cases:

```text
isqrt(0) == 0
isqrt(1) == 1
isqrt(2) == 1
isqrt(3) == 1
isqrt(4) == 2
```

For many `k`:

```text
isqrt(k*k - 1) == k - 1
isqrt(k*k)     == k
isqrt(k*k + 1) == k
```

Cover:

```text
small values
near inline Int representation boundary
LargeInt values
10^100
10^1000
large powers of two
large perfect squares
```

Universal result invariant:

```text
r*r <= n
n < (r+1)*(r+1)
```

## T8.4 Negative validation

Verify:

```telos
isqrt(-1)
```

raises `ArgumentError` with the intended message/kind.

Do not convert negative exact input to NaN because the return type is exact `Int`.

## T8.5 Resource limits

Use the existing numeric policy.

Test that exact integer operations do not bypass:

```text
#numericLimit
```

through library-specific loops or intermediate values.

No host panic/OOM is acceptable.

## T8.6 Performance sanity

`gcd` and `isqrt` should not degrade to linear scan algorithms.

The existing Euclidean and Newton algorithms are correct complexity choices.

Do not rewrite them in Rust merely for micro-performance without profiling evidence.

---

# 15. T9 — Public Family, type-boundary, and topology acceptance

## Purpose

Prove the Telos-specific API architecture, not only numerical output.

## T9.1 Family identity

Representative exports:

```text
math.sqrt
math.atan2
math.log
math.gcd
```

must be `Family` values.

They must not be:

```text
Closure
BoundMethod
Method reflection object
```

Use existing Family reflection/protocol tests.

Verify exact selectors:

```text
sqrt(_)
atan2(_,_)
log(_,base)
gcd(_,_)
```

## T9.2 Exact capture behavior

The public package must use exact references:

```telos
&FloatMathKernel.sqrt(_)
```

not:

```telos
&FloatMathKernel.sqrt...
```

This prevents future internal overloads from silently joining the public surface.

Add a semantic/source inspection test if the repository has an appropriate API-surface harness.

## T9.3 Accepted receiver exposure

A test may document:

```text
math.sqrt.receiver
```

can reveal the kernel receiver.

This is **not** a failure.

Do not attempt to make the test assert opacity.

The reason for recording it is to prevent a future maintainer from “fixing” the package by replacing Families with closures.

## T9.4 Positive type cases

Compile and run:

```telos
math.sqrt(4.0)
math.log(8.0, base: 2.0)
math.gcd(12, 8)
math.isqrt(81)
math.radians(180.0)
```

## T9.5 Negative type cases

Semantic checking must reject:

```telos
math.sqrt(4)
math.ln(1)
math.log(8, base: 2.0)
math.gcd(12.0, 8.0)
math.isqrt(81.0)
```

These should fail before runtime.

Do not add runtime coercion to make them pass.

## T9.6 Hidden child topology

External source must fail to import:

```telos
import universe.math.float_kernel
import universe.math.integer
```

while the facade's relative imports compile.

Also verify:

```text
Package.children / reflection
```

follows the public exposure contract. If reflection currently enumerates loaded physical children rather than exposed children, do not conflate that pre-existing behavior with import-path exposure; test the canonical external resolver rule.

---

# 16. T10 — Generated native surface hygiene

## Purpose

Regenerate once, review deliberately, and keep unrelated baseline synchronization understandable.

## Files

```text
phalcom-native-surface/src/generated.rs
```

and possibly generator tests.

## T10.1 Regenerate

After all native declaration changes:

```sh
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root .
```

Then:

```sh
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

## T10.2 Expected math changes

Expected delta should include:

```text
FloatMathKernel source owner moved to math.float_kernel
new log(_,base) native row
existing 23 rows retained
```

## T10.3 Existing stale Record rows

The previous regeneration pulled unrelated Record descriptors into the generated artifact because `main`'s generated surface was stale.

Do one of:

### Preferred

Split baseline synchronization into a separate commit:

```text
chore: synchronize generated native surface baseline
```

then keep math-generated delta isolated.

### Acceptable

If history cannot be cleanly split, explicitly record the unrelated generated rows in the handoff/walkthrough and verify that they correspond to existing source-authored Record primitives.

Do not silently edit generated output by hand to remove them.

## T10.4 Descriptor count

Do not hard-code an expected count without recalculating after `log`.

The generator is the authority.

Verify no primitive silently disappeared.

---

# 17. T11 — Documentation and lifecycle records

## Purpose

Make implementation records agree with actual behavior and tested guarantees.

## Files

At minimum:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/stdlib-random-math.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/stdlib-random-math-implementation-handoff.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/stdlib-random-math-walkthrough.md
```

Do not claim this work completes unrelated `LANG007.C4.P4` FieldKey acceptance.

## T11.1 Correct obsolete package-layout guidance

Update any text saying:

```text
FloatMathKernel lives in math/package.ph
expose .integer is required
```

to the hidden-child architecture.

## T11.2 Document static constant repair

Record the final runtime model:

```text
StaticConstantId
    ↓
RuntimeStaticConstantRef(owner module, canonical name)
    ↓
ModuleObject.static_constants
    ↓
materialized Value
```

Include re-export identity law.

## T11.3 Correct Float accuracy claims

Remove blanket claims of:

```text
<= 1 ULP for every function
cross-platform last-bit reproducibility
```

when using Rust unspecified-precision methods.

Document:

```text
sqrt -> stronger Rust/IEEE guarantee
other transcendentals -> supported f64 backend, no last-bit portability guarantee
```

## T11.4 Public package docs

Ensure user-facing documentation states:

```text
Float-only transcendental APIs
no implicit Int conversion
radians for trig
degrees/radians helpers
NaN/infinity domain behavior
exact Int gcd/lcm/isqrt
```

## T11.5 Record accepted debt

Explicitly state:

```text
Family.receiver can expose FloatMathKernel receiver
accepted for this slice
future general callable privacy problem
```

---

# 18. T12 — Final verification sequence

Run in this order.

## Gate A — hygiene

```sh
git status --short
git diff --check
cargo fmt --all -- --check
```

If formatting the whole workspace would alter unrelated files, format only touched Rust files or inspect/revert unrelated formatter drift.

## Gate B — compile

```sh
RUSTFLAGS='' cargo check -p phalcom-core
RUSTFLAGS='' cargo check -p phalcom-modules
RUSTFLAGS='' cargo check -p phalcom-native-meta
RUSTFLAGS='' cargo check -p phalcom-native-surface
```

## Gate C — exported static constants

Run the exact original discriminator:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  exported_static_constant_materializes_for_selected_and_module_reads \
  -- --nocapture
```

Then the new re-export/GC/reflection focused tests.

## Gate D — math corpus

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus math -- --nocapture
```

Confirm nonzero selected tests.

## Gate E — module/runtime affected suites

Run focused modules suites containing:

```text
linking
parity
reflection
Universe import/exposure
GC constant roots
```

Use actual harness names discovered from the live repository.

## Gate F — native surface

```sh
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

Then native descriptor/source-owned primitive tests.

## Gate G — semantic/type tests

Run the focused tests that prove:

```text
Float-only math typing
Family exact-reference typing
hidden child import rejection
```

## Gate H — broader core verification

After all focused gates pass:

```sh
RUSTFLAGS='' cargo test -p phalcom-core
```

If this is prohibitively expensive, use the repository's accepted directly-affected broad suite and record exactly what was run.

Do not claim workspace-wide verification if it was not executed.

---

# 19. Detailed patch sequence and recommended commits

Use small, reviewable commits that preserve a useful bisection story.

## Commit 1 — canonical runtime static constants

Suggested message:

```text
fix: preserve canonical runtime static constants
```

Scope:

```text
RuntimeStaticConstantRef
ModuleObject.static_constants
single materialization authority
GC tracing
enum exhaustiveness
no math package reorganization yet
```

Focused tests:

```text
materialization cache
missing owner error
heap-backed root
```

## Commit 2 — static const import/export parity

Suggested:

```text
fix: complete exported const module semantics
```

Scope:

```text
materialize exports/reads
module Getter dispatch
ModuleExecutionContext provenance
reflection
re-export tests
original failing reproducer
```

Do not mix math API expansion here.

## Commit 3 — hide math implementation children

Suggested:

```text
refactor: isolate universe math implementation modules
```

Scope:

```text
float_kernel.ph
package relative imports
remove expose .integer
register hidden children
move UniverseKey source path
topology tests
```

## Commit 4 — complete elementary math surface

Suggested:

```text
feat: complete universe math elementary surface
```

Scope:

```text
log(_,base)
degrees
radians
facade export list
native kernel addition
```

## Commit 5 — math correctness and API verification

Suggested:

```text
test: harden universe math contracts
```

Scope:

```text
constant bits
Float backend mapping
IEEE edges
Family identity
type failures
integer properties
resource checks
```

## Commit 6 — generated surface/docs

Suggested:

```text
docs: finalize universe math implementation records
```

Potentially precede with:

```text
chore: synchronize generated native surface baseline
```

if the stale Record rows are separated.

Do not squash away meaningful runtime architecture boundaries before review unless project workflow requires squashing.

---

# 20. Exact file/symbol checklist

This is the patch auditor's checklist.

## `phalcom-core/core/universe/src/math/package.ph`

Final responsibilities:

```text
documentation
hidden child imports
6 consts
exact captures
degrees/radians small source helpers
export list
```

Must not contain:

```text
native class FloatMathKernel body
expose .integer
expose .float_kernel
```

## `phalcom-core/core/universe/src/math/float_kernel.ph`

New.

Owns:

```text
FloatMathKernel
24 native methods after adding log
```

Exports class to parent module only through hidden child module.

## `phalcom-core/core/universe/src/math/integer.ph`

Owns:

```text
IntegerMath
gcd
lcm
isqrt
```

No public child exposure.

## `phalcom-core/src/primitive/math.rs`

Owns Rust binary64 implementation.

Add:

```text
math_log
```

Keep concrete primitive functions rather than macro-wrapped attribute declarations.

## `phalcom-native-meta/src/universe.rs`

Update:

```rust
UniverseKey::FloatMathKernel.source_path()
```

to:

```text
["math", "float_kernel"]
```

## `phalcom-modules/src/builtin.rs`

Register:

```text
math
math.float_kernel
math.integer
```

and source-text routes.

Remember:

```text
registered child != exposed child
```

## `phalcom-core/src/modules/linkage.rs`

Add:

```text
RuntimeStaticConstantRef
```

and use it in `RuntimeLinkedRead`.

## `phalcom-core/src/heap/module.rs`

Use ref in `RuntimeExportRef`.

Add module-owned:

```text
static_constants
```

## `phalcom-core/src/modules/static_constant.rs`

Become the single runtime primitive static-constant materialization authority.

## `phalcom-core/src/modules/materialize.rs`

Route linked reads/exports through canonical static refs.

Remove panic path.

## `phalcom-core/src/modules/context.rs`

Preserve canonical source symbol through selected imports and re-export aliases.

## `phalcom-core/src/vm/send.rs`

Fix both module export-send paths consistently.

Getter of a static export returns the value.

## `phalcom-core/src/vm/dispatch.rs`

Resolve static linked reads through owner module cache.

## `phalcom-core/src/heap/trace.rs`

Trace module-owned static constant values.

Static ref descriptors only need their owner module traced.

## `phalcom-core/src/primitive/reflection.rs`

Update:

```text
Module.dependencies
Export.value
```

for canonical static refs.

## `phalcom-core/src/modules/reflection_cache.rs`

Update static export exhaustiveness/classification.

## `phalcom-core/src/vm/bootstrap.rs`

Attach/use lowering metadata through the same materialization authority.

Do not add a second Universe-only const materializer.

## `phalcom-native-surface/src/generated.rs`

Regenerate, never hand-edit.

## Tests

Update/add around:

```text
phalcom-core/tests/core/modules/parity.rs
phalcom-core/tests/core/modules/linking.rs
phalcom-core/tests/core/modules/*
phalcom-core/tests/core/language/corpus.rs
phalcom-core/tests/fixtures/language/math/*
semantic/type tests for rejection
native metadata/source-owner tests
```

---

# 21. Test fixture design

Avoid one giant fixture whose output makes failures ambiguous.

Use several targeted fixtures.

## `math_foundation.ph`

Keep basic happy path:

```telos
from universe.math import (
  pi,
  sqrt,
  hypot,
  gcd,
  lcm,
  isqrt
)

System.print(pi)
System.print(sqrt(9.0))
System.print(hypot(3.0, 4.0))
System.print(gcd(-84, 30))
System.print(lcm(-21, 6))
System.print(isqrt(10_000))
```

## `math_elementary.ph`

Exercise every public Float family at representative finite values.

This is a routing/backend smoke test, not a portable last-bit oracle for unspecified-precision functions.

## `math_special_values.ph`

Exercise classification/zero signs/infinities.

If source syntax cannot construct all desired raw Float bit patterns, use Rust integration tests for exact bit-level cases rather than bloating source APIs.

## `math_integer_large.ph`

Exercise BigInt-backed exact functions.

## Compile-fail cases

Use the repository's compile-fail/semantic diagnostic harness for:

```text
sqrt(Int)
ln(Int)
gcd(Float,Float)
isqrt(Float)
external import of hidden children
```

Do not encode expected runtime TypeErrors for cases that should be statically rejected.

---

# 22. Resource and performance requirements

## 22.1 Float kernels

Expected complexity:

```text
one argument extraction per operand
one Rust f64 call
one immediate Float Value return
```

No allocation.

## 22.2 `gcd`

Expected Euclidean complexity.

No Float conversion.

## 22.3 `lcm`

Use divide-before-multiply.

Do not regress to:

```text
a * b / gcd
```

## 22.4 `isqrt`

Keep Newton iteration.

Do not implement linear trial search.

## 22.5 Static constant cache

The new module-owned map must not turn every read into re-materialization.

Lookups should be:

```text
owner module handle
+
interned source-name symbol
+
HashMap lookup
```

If repeated read performance becomes relevant later, it can be optimized without altering semantic identity.

---

# 23. Diagnostics requirements

## 23.1 Const materialization failures

Error text should name:

```text
StaticConstantId / owner module
constant name
unsupported descriptor kind or missing runtime owner
```

Never silently surface `None`.

## 23.2 Hidden child imports

Use existing module-resolution diagnostics.

Do not invent a math-specific "private module" error.

## 23.3 Type errors

Use normal checker diagnostics showing expected:

```text
Float
Int
```

Do not insert math-specific dynamic checks to compensate for missing checker enforcement.

## 23.4 `isqrt`

Use existing `ArgumentError`.

No new `MathDomainError`.

---

# 24. Stop-and-consult conditions

Stop implementation and request architectural review if any of these become necessary:

1. exported const correctness would require changing semantic `StaticConstantId`;
2. re-export linked products do not retain the canonical source `SymbolId`;
3. hidden Universe child modules cannot be relatively imported without public `expose`;
4. moving `FloatMathKernel` would require duplicate source/native class identities;
5. the package-level `degrees/radians` source helpers cannot be expressed without closure-valued exports and no clean hidden source owner exists;
6. the existing Family semantics cannot call public kernel methods after the source move without changing authorization rules;
7. static constant caching would require observable object identity for immutable consts;
8. supporting primitive exported consts requires synthesizing runtime global slots;
9. Rust backend behavior differs materially from the documented numeric contract;
10. adding `log(_,base)` requires changing selector grammar rather than using existing labeled slots.

Do not improvise around these architecture breaks.

---

# 25. Explicitly rejected shortcuts

The following patches may make a test green but are not acceptable:

## Shortcut A

```telos
let pi = 3.14159
```

instead of `const`.

Rejected because it abandons compile-time const semantics.

## Shortcut B

```rust
RuntimeExportRef::StaticConstant(Value)
```

plus only the missing Getter early return.

Rejected as the final architecture because re-export provenance and duplicate materialization remain unresolved.

## Shortcut C

```telos
let sqrt = |x| FloatMathKernel.sqrt(x)
```

Rejected because it destroys the exact Family API.

## Shortcut D

Make kernel methods `internal` and special-case public Family activation.

Rejected because it changes general Family authorization for one stdlib package.

## Shortcut E

Keep:

```telos
expose .integer
```

because the facade needs the child.

Rejected because relative internal import is the correct topology model.

## Shortcut F

Move constants to a hidden child and re-export them.

Rejected because canonical identities should remain:

```text
universe.math::pi
```

and because it needlessly exercises re-export identity for core public constants.

## Shortcut G

Promise `<=1 ULP` because tests happen to pass on one machine.

Rejected because the selected Rust methods do not provide that universal contract.

---

# 26. Merge-readiness gate

The math slice is complete only when every row below is true.

| Gate | Required evidence |
|---|---|
| Static const direct selected import | PASS |
| Static const module Getter | PASS |
| Static const alias | PASS |
| Static const renamed re-export | PASS |
| Static const two-hop re-export | PASS |
| Static const reflection | PASS |
| Static String survives GC | PASS |
| Static LargeInt survives GC | PASS |
| No ordinary global slot for const | PASS |
| Unsupported static materialization fails closed | PASS |
| `package.ph` contains no Float kernel class | PASS |
| `float_kernel` hidden external path | PASS |
| `integer` hidden external path | PASS |
| Parent package relative imports children | PASS |
| Native source owner is `math.float_kernel` | PASS |
| All public math operations are exact Families | PASS |
| Float-only type rejection | PASS |
| All Float native kernels routed/tested | PASS |
| Constants exact bit patterns | PASS |
| IEEE/special-value matrix | PASS |
| `atan2` argument order/signed zero | PASS |
| `gcd` laws / LargeInt | PASS |
| `lcm` laws / LargeInt | PASS |
| `isqrt` invariant / LargeInt | PASS |
| `isqrt(-1)` ArgumentError | PASS |
| Numeric resource limits preserved | PASS |
| Native surface generator `--check` | PASS |
| Focused math corpus | PASS |
| Focused module/runtime suites | PASS |
| Broader `phalcom-core` suite | PASS or explicitly recorded exception |

---

# 27. Final expected source shape

## `math/package.ph`

Conceptually:

```telos
@!documentation(
  "Elementary IEEE binary64 mathematics and exact integer algorithms. " +
  "Float operations do not implicitly convert Int values."
)

from .float_kernel import FloatMathKernel
from .integer import IntegerMath

const pi: Float = 3.141592653589793
const tau: Float = 6.283185307179586
const e: Float = 2.718281828459045
const ln2: Float = 0.6931471805599453
const ln10: Float = 2.302585092994046
const sqrt2: Float = 1.4142135623730951

let sqrt = &FloatMathKernel.sqrt(_)
let cbrt = &FloatMathKernel.cbrt(_)

let exp = &FloatMathKernel.exp(_)
let exp2 = &FloatMathKernel.exp2(_)
let expm1 = &FloatMathKernel.expm1(_)

let ln = &FloatMathKernel.ln(_)
let ln1p = &FloatMathKernel.ln1p(_)
let log = &FloatMathKernel.log(_, base)
let log2 = &FloatMathKernel.log2(_)
let log10 = &FloatMathKernel.log10(_)

let sin = &FloatMathKernel.sin(_)
let cos = &FloatMathKernel.cos(_)
let tan = &FloatMathKernel.tan(_)

let asin = &FloatMathKernel.asin(_)
let acos = &FloatMathKernel.acos(_)
let atan = &FloatMathKernel.atan(_)
let atan2 = &FloatMathKernel.atan2(_, _)

let sinh = &FloatMathKernel.sinh(_)
let cosh = &FloatMathKernel.cosh(_)
let tanh = &FloatMathKernel.tanh(_)

let asinh = &FloatMathKernel.asinh(_)
let acosh = &FloatMathKernel.acosh(_)
let atanh = &FloatMathKernel.atanh(_)

let hypot = &FloatMathKernel.hypot(_, _)

let gcd = &IntegerMath.gcd(_, _)
let lcm = &IntegerMath.lcm(_, _)
let isqrt = &IntegerMath.isqrt(_)

// Use exact live top-level declaration syntax.
// Do not turn these into closure-valued exports.
degrees(_ radians: Float) -> Float {
  radians * (180.0 / pi)
}

radians(_ degrees: Float) -> Float {
  degrees * (pi / 180.0)
}

export (
  pi,
  tau,
  e,
  ln2,
  ln10,
  sqrt2,

  sqrt,
  cbrt,

  exp,
  exp2,
  expm1,

  ln,
  ln1p,
  log,
  log2,
  log10,

  sin,
  cos,
  tan,

  asin,
  acos,
  atan,
  atan2,

  sinh,
  cosh,
  tanh,

  asinh,
  acosh,
  atanh,

  hypot,

  degrees,
  radians,

  gcd,
  lcm,
  isqrt
)
```

The exact declaration spelling for labeled `log` and top-level source methods must be confirmed from the current grammar before committing. The semantic target is fixed:

```text
selector: log(_,base)
public value: exact Family
```

## `math/float_kernel.ph`

Conceptually:

```telos
/// Internal implementation owner for `universe.math` binary64 operations.
///
/// Public users consume the exact Families exported by `universe.math`.
native class FloatMathKernel is Object {
  @class native sqrt(_ value: Float) -> Float
  @class native cbrt(_ value: Float) -> Float

  @class native exp(_ value: Float) -> Float
  @class native exp2(_ value: Float) -> Float
  @class native expm1(_ value: Float) -> Float

  @class native ln(_ value: Float) -> Float
  @class native ln1p(_ value: Float) -> Float
  @class native log(_ value: Float, base valueBase: Float) -> Float
  @class native log2(_ value: Float) -> Float
  @class native log10(_ value: Float) -> Float

  @class native sin(_ value: Float) -> Float
  @class native cos(_ value: Float) -> Float
  @class native tan(_ value: Float) -> Float

  @class native asin(_ value: Float) -> Float
  @class native acos(_ value: Float) -> Float
  @class native atan(_ value: Float) -> Float
  @class native atan2(_ y: Float, _ x: Float) -> Float

  @class native sinh(_ value: Float) -> Float
  @class native cosh(_ value: Float) -> Float
  @class native tanh(_ value: Float) -> Float

  @class native asinh(_ value: Float) -> Float
  @class native acosh(_ value: Float) -> Float
  @class native atanh(_ value: Float) -> Float

  @class native hypot(_ x: Float, _ y: Float) -> Float
}

export FloatMathKernel
```

Again, confirm the exact labeled-parameter spelling from live syntax; preserve selector `log(_,base)`.

---

# 28. Implementation intuition for the agent

This slice is intentionally a stress test of Telos's own architecture.

The implementation should look like ordinary language composition:

```text
source declaration
    ↓
canonical semantic identity
    ↓
native attachment / source algorithm
    ↓
first-class Family capture
    ↓
module export
```

and:

```text
const source declaration
    ↓
StaticConstantId
    ↓
StaticValue
    ↓
MaterializationDescriptor
    ↓
one owner-module runtime materialization
    ↓
RuntimeStaticConstantRef
    ↓
imports / re-exports / reflection / module Getter
```

The key quality criterion is that **the runtime must not invent a second model of the source program**.

The math package should therefore become a high-quality example of:

```text
canonical Universe source ownership
module topology
static evaluation
first-class Families
native source attachment
arbitrary-precision Int
binary64 Float
reflection
GC rooting
```

working together coherently.

---

# 29. Final delivery checklist

Before handing the slice back:

```text
[x] branch/head recorded
[x] scoped math worktree changes reviewed; unrelated untracked work preserved
[x] git diff --check PASS
[ ] formatter check PASS — workspace check reports existing formatting drift outside this continuation's Rust files; touched Rust files pass targeted rustfmt check
[x] original exported-const reproducer PASS
[x] const alias/re-export/GC tests PASS
[x] hidden module topology tests PASS
[x] math corpus PASS
[x] Float kernel tests — per-function host-operation mapping, six-constant bit oracle, IEEE/domain/signed-zero/subnormal matrix, and purity/resource probes
[x] integer property/boundary tests — gcd/lcm laws, negative/sign boundaries, inline/BigInt isqrt boundaries, large powers, and numeric-limit enforcement
[x] type rejection tests PASS
[x] Family identity tests PASS
[x] native surface --check PASS
[x] directly affected broader tests — core library 160/3 and modules cluster 54/6; non-math failures explicitly recorded as baseline exceptions; focused math/runtime tests pass
[x] generated diff reviewed (generator produced no changes)
[x] implementation handoff updated
[x] walkthrough updated
[x] status remains truthful
[x] no random implementation mixed into this patch
```

If all gates pass, the math phase may be marked complete and the next plan may begin `universe.random`.

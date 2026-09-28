# LANG007.C3.P4 — Integrated Architect Ruling

## T4/T5 Semantic Completion, Phantom Generic Construction, Runtime Specialization, Constructor Surface, Selector Syntax, and Remaining Execution Guidance

**Status:** ARCHITECT RULING — ADOPTED  
**Applies to:** `LANG007.C3.P4`  
**Repository state reviewed:** current `main`, including `145deded` (`feat(core): preserve runtime RecordRow application identity`) and the current `LANG007.C3-CHECKPOINT.md` / P4 plan  
**Primary affected tasks:** T4, T5, with bounded syntax/constructor work required before T5 acceptance; then T6–T13 continue in plan order.

---

# 1. Executive ruling

The current T4 semantic architecture and the T5 heterogeneous runtime-applied-type architecture are retained.

The following additional rulings are now adopted:

1. **Phantom generic parameters are valid and semantically significant for `data`, including nullary data.**
2. **Data-constructor generic specialization may be selected from expected/result context even when a generic parameter does not participate in the physical constructor shape.**
3. The previously reported failing `PhantomRow` discriminator is therefore **not deferred**. The earlier provisional recommendation to defer it is superseded by this ruling.
4. The `PhantomRow` failure must be fixed in **semantic constructor/result-context specialization**, not in the VM. Runtime open-row rejection remains correct and mandatory.
5. Runtime `Applied` identity continues to represent heterogeneous arguments, including `RecordFields` arguments whose canonical semantic representation is `RecordRow` / `RecordRowId`.
6. Product `data` receives one canonical associated constructor family named:

   ```phalcom
   ::Construct
   ```

7. Direct construction remains ergonomic:

   ```phalcom
   Tagged(42)
   Tagged<Int>(42)

   Person {
       name: "Ada",
       payload: 42,
   }

   Person<Int> {
       name: "Ada",
       payload: 42,
   }
   ```

8. The explicit associated constructor forms are:

   ```phalcom
   Tagged<Int>::Construct(42)

   Person<Int>::Construct(
       name: "Ada",
       payload: 42,
   )
   ```

9. Canonical constructor references are:

   ```phalcom
   &Tagged<Int>::Construct(_)

   &Person<Int>::Construct(
       name: _,
       payload: _,
   )
   ```

10. Ergonomic constructor-reference aliases are also supported:

    ```phalcom
    &Tagged<Int>(_)

    &Person<Int>(
        name: _,
        payload: _,
    )
    ```

11. No record-shaped callable-reference syntax using braces is introduced. Once a constructor is referenced, it is an ordinary callable and is invoked using ordinary call syntax.
12. For record constructors, direct `{ ... }` construction remains label-oriented and may retain existing flexible field ordering. The `Construct(...)` callable shape and constructor references should initially use **source declaration order**. Do not add permutation-independent selector normalization merely for this feature.
13. Selector references/specifications must begin accepting the clearer labeled-slot spelling:

    ```phalcom
    callable(_, name: _)
    ```

    while preserving the existing spelling:

    ```phalcom
    callable(_, name)
    ```

    Both normalize to the same existing selector identity:

    ```rust
    SelectorSlot::Label("name")
    ```

14. Canonical/runtime selector encoding remains unchanged for compatibility. Do **not** migrate selector transport from `callable(_,name)` to `callable(_,name:_)`.
15. A statically resolved data-family application such as:

    ```phalcom
    Tagged(42)
    ```

    always denotes primary data construction, even if the declaration has a class-side `call(_)`.
16. Explicit ordinary behavior remains available through:

    ```phalcom
    Tagged.call(42)
    ```

17. Once the type/class value is stored in an ordinary runtime variable, ordinary callable semantics apply:

    ```phalcom
    let factory = Tagged
    factory(42) // ordinary factory.call(42)
    ```

    By contrast:

    ```phalcom
    let factory = &Tagged(_)
    factory(42)
    ```

    captures the primary constructor capability.

These changes must converge on the existing canonical semantic identities. Do not introduce a parallel constructor system, runtime row solver, per-specialization class objects, or a second exact-type registry.

---

# 2. Existing architecture that remains authoritative

## 2.1 `RecordFields` versus `RecordRow`

Do not reopen this naming/domain decision.

The language-facing structural domain is:

```phalcom
RecordFields
```

For example:

```phalcom
data MyRecord<**Fs: RecordFields> {
    **Fs
}
```

The compiler-internal canonical representation remains the row calculus:

```text
source/public domain:
    RecordFields

internal kind/domain machinery:
    KindId::RECORD_ROW
    RecordRow
    RecordRowId
    RecordRowVarId
    RecordRowSolver

heterogeneous semantic generic argument:
    StaticArgument::RecordRow(RecordRowId)

runtime applied argument:
    RuntimeStaticArgument::RecordRow(...)
```

A concrete source argument such as:

```phalcom
**{ name: String, age: Int }
```

is a value of the public `RecordFields` domain and is canonically represented internally as a row.

Do not replace `RecordRowId` with `TypeId`.  
Do not encode a row as a proper structural `Record` type.  
Do not introduce row-solving authority in the VM.

When writing user-facing specifications and diagnostics, prefer **RecordFields**.

When discussing the implementation calculus, `RecordRow`, `RecordRowId`, and related names remain appropriate.

---

# 3. T4 ruling — semantic product-shape specialization remains accepted

T4 established the semantic side of generalized product-shaped `data`.

Its accepted architecture should not be redesigned while implementing the changes in this ruling.

The important separation remains:

```text
declaration identity
    |
    +-- generic signature
    |
    +-- product-shape template
    |
    +-- primary DataConstructorId
             |
             v
      exact specialization
             |
             +-- exact product shape
             +-- exact constructor result type
             +-- argument/component mapping
             +-- inference/evidence
```

## 3.1 Product expansion remains semantic structure

For shape-generic data such as:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)

data MyRecord<**Fs: RecordFields> {
    **Fs
}
```

the declaration owns a product-shape template.

A concrete specialization materializes that template.

Do not manufacture declaration components for expansion-derived fields.

The established distinction remains conceptually:

```rust
enum DataComponentResolution {
    Declared {
        component: DataComponentId,
        logical_index: u32,
        value_type: TypeId,
    },

    Expanded {
        logical_index: u32,
        label: Box<str>,
        value_type: TypeId,
        expansion_source: SemanticSourceSpan,
    },
}
```

The exact Rust spelling may differ, but this invariant must hold:

> Expansion-derived product members do not receive fake `DataComponentId`s.

---

# 4. T4 constructor inference/evidence ruling

T4's existing constructor adapter remains the shared semantic application path.

It must continue to distinguish:

```text
CompleteTypePack inference:
    ordinary heterogeneous inference terms

RecordFields inference:
    row equations / RecordRow inference

ordinary generic parameters:
    canonical generic inference variables
```

Do not collapse those domains merely to make constructor application easier.

## 4.1 Existing source-derived specialization

Construction such as:

```phalcom
data MyRecord<**Fs: RecordFields> {
    **Fs
}

MyRecord {
    payload: 42,
}
```

contains source evidence for:

```text
Fs = { payload: Int }
```

That is different from a phantom parameter.

Likewise:

```phalcom
data PackBox<***Ps: CompleteTypePack>(***Ps)

PackBox()
```

can establish an exact empty pack because `Ps` is the constructor/product shape itself.

This remains valid.

## 4.2 Phantom parameters are different

Consider:

```phalcom
data Signal<State>()
```

or:

```phalcom
data PhantomRow<**Fs: RecordFields>()
```

The parameter does not participate in the product shape.

Therefore:

```phalcom
Signal()
PhantomRow()
```

with no explicit generic arguments and no expected type are underconstrained.

They must **not** silently mean:

```text
Signal<some arbitrary type>
PhantomRow<{}>
```

The absence of constructor arguments is not evidence for an empty phantom generic argument.

This distinction must remain explicit:

```text
parameter contributes to product/constructor shape
    zero contributions may establish an empty value

parameter is phantom
    zero constructor arguments provide no constraint
```

---

# 5. T4 proof/evidence architecture remains authoritative

The existing row proof bridge remains query/session-local.

Do not attach inference proof strength to canonical row identity.

In particular:

```text
RecordRowId
```

must remain proof-neutral.

The current accepted design is:

```text
GenericApplicationSession
    owns row inference/proof state

RecordRowVarId
    keys query-local proof evidence

RecordRowSolver
    solves row equations
    but does not own durable evidence semantics
```

The established proof-strength rule remains:

> Semantic derivation may not increase proof strength beyond its weakest required premise.

Relevant cases remain:

```text
source-established exact row
    => Established

context-only selected row
    => Assumed

compatible expected context + independently Established source
    => remains Established

Unknown source dependency
    => Unknown propagates when return-relevant

Dynamic source dependency
    => Dynamic propagates when return-relevant

weak row unrelated to the return
    => must not weaken an otherwise independent result
```

This becomes directly relevant to `PhantomRow`, because contextual selection must not be mislabeled as source-established evidence.

---

# 6. Superseding PhantomRow ruling

## 6.1 Previous provisional deferral is withdrawn

The earlier provisional suggestion to defer the nullary `PhantomRow` case beyond T5 is superseded.

Repository inspection establishes that the language already accepts all of the underlying principles.

The accepted `data` design already permits phantom type parameters:

```phalcom
data Id<Kind>(_ raw: Int)
```

where:

```text
Id<User>
Id<Order>
```

are distinct exact semantic types despite identical physical representation.

The accepted `data` specification also explicitly permits:

```phalcom
data Signal<State>()
```

and defines one semantic singleton per exact applied nullary data type.

The existing runtime test suite already contains phantom generic data such as:

```phalcom
data Tagged<T>(_ id: Int)

let t1 = Tagged<Int>::Tagged(42)
let t2 = Tagged<String>::Tagged(42)
```

and proves that specialization identity survives even when `T` contributes no payload bits.

The source specification also already says, in substance, that primary construction participates in ordinary generic inference and may infer omitted generic arguments from **constructor arguments and surrounding type context**.

There are also existing class/generic precedents:

```phalcom
class Box<T> {
    @constructor
    new() {}
}

let stringBox: Box<String> = Box.new()
let intBox: Box<Int> = Box.new()
```

and:

```phalcom
class Probe {
    @class
    make<T>() -> T {
        42
    }

    @class
    run() {
        let result: Int = Probe.make()
    }
}
```

The latter is already tested with context-selected result evidence marked `Assumed`.

Therefore the failure:

```phalcom
data PhantomRow<**Fs: RecordFields>()

class Factory {
    @class
    make<T>(_ value: T) -> PhantomRow<**{ payload: T }> {
        PhantomRow()
    }
}
```

is a consistency gap in the data-constructor semantic path.

It is not a reason to weaken runtime exact-type rules.

---

# 7. Required PhantomRow semantic behavior

For:

```phalcom
data PhantomRow<**Fs: RecordFields>()

class Factory {
    @class
    make<T>(_ value: T) -> PhantomRow<**{ payload: T }> {
        PhantomRow()
    }
}
```

semantic analysis must establish the constructor expression as:

```text
PhantomRow<{ payload: T }>
```

from the expected result.

It must not leave the result as:

```text
PhantomRow<**Fs>
```

with an unresolved stable row tail.

The intended flow is:

```text
declared return:
    PhantomRow<{ payload: T }>

constructor expression:
    PhantomRow()

fresh constructor specialization:
    Fs = ?row

expected-result constraint:
    PhantomRow<?row>
        =
    PhantomRow<{ payload: T }>

solve:
    ?row = { payload: T }

published constructor result:
    PhantomRow<{ payload: T }>
```

The resulting exact semantic type contains a **closed row** whose field type may still contain the enclosing ordinary generic parameter `T`.

That is important because the runtime already knows how to substitute:

```text
T = Int
```

inside a closed runtime row.

So the final runtime recipe becomes conceptually:

```text
PhantomRow<{ payload: T }>
```

and invocation with:

```text
T = Int
```

reifies:

```text
PhantomRow<{ payload: Int }>
```

No row-valued runtime generic environment binding is required.

---

# 8. Do not implement a RecordFields-only contextual hack

The semantic rule is general:

> An otherwise-underconstrained generic parameter of a data primary constructor may be selected from a compatible expected exact result type using the ordinary generic inference system.

That applies, where the existing kind/application machinery supports it, to declaration-owned generic parameters such as:

```text
T: Type
*Ts: TypePack
**Ls: LabeledTypePack
**Fs: RecordFields
***Es: CompleteTypePack
other existing canonical static domains
```

Do not special-case only:

```text
RecordRowVarId
```

or only the identifier `Fs`.

The implementation should reuse the same generic application frame/constraint machinery used by ordinary generic calls.

For P4 acceptance, the minimum required discriminators are:

```phalcom
data Phantom<T>()

let x: Phantom<Int> = Phantom()
```

and:

```phalcom
data PhantomRow<**Fs: RecordFields>()

make<T>() -> PhantomRow<**{ payload: T }> {
    PhantomRow()
}
```

A CompleteTypePack phantom contextual test is desirable if it falls naturally out of the shared implementation, but do not expand the patch into an unrelated generic-inference redesign merely to manufacture more examples.

---

# 9. Contextual specialization evidence

Context can determine an exact type without providing value-derived proof.

Therefore:

```phalcom
let x: Phantom<Int> = Phantom()
```

may have exact type:

```text
Phantom<Int>
```

while the specialization evidence attributable only to the expected context remains:

```text
Assumed
```

Similarly:

```phalcom
PhantomRow()
```

inside:

```phalcom
-> PhantomRow<{ payload: T }>
```

should publish that exact applied type with context-selected evidence at no stronger than `Assumed`, unless independent constructor/value evidence establishes the same generic argument.

Do not downgrade independent stronger evidence merely because a compatible expected type exists.

Existing T4 tests for:

```text
expected_result_can_select_row
source_established_row_stays_established_under_compatible_expected_result
```

remain the model.

---

# 10. Required negative cases for phantom contextual specialization

Add explicit tests for the following.

### No context

```phalcom
data Phantom<T>()

let x = Phantom()
```

Expected:

```text
underconstrained
```

No arbitrary `T` is invented.

### No context — row phantom

```phalcom
data PhantomRow<**Fs: RecordFields>()

let x = PhantomRow()
```

Expected:

```text
underconstrained
```

Do not infer `{}` merely because construction is nullary.

### Explicit specialization

```phalcom
Phantom<Int>()
PhantomRow<**{ payload: Int }>()
```

Expected exact specialized results.

### Context selection

```phalcom
let x: Phantom<Int> = Phantom()
```

Expected:

```text
Phantom<Int>
```

with context-derived evidence.

### Generic contextual row

```phalcom
make<T>() -> PhantomRow<**{ payload: T }> {
    PhantomRow()
}
```

Expected exact symbolic result:

```text
PhantomRow<{ payload: T }>
```

before runtime substitution.

### Conflict

If constructor/value evidence and expected-result evidence contradict, emit the ordinary generic inference conflict.

Do not silently prefer context.

---

# 11. T5 runtime architecture remains adopted

The runtime representation decision already implemented by `145deded` is correct.

`RuntimeOverlayTypeNode::Applied` must preserve heterogeneous static arguments.

Conceptually:

```rust
enum RuntimeStaticArgument {
    Term(RuntimeTypeRef),
    RecordRow(RuntimeRecordRow),
}
```

with:

```rust
struct RuntimeRecordRow {
    fields: Box<[RuntimeRecordRowField]>,
    tail: RuntimeRecordRowTail,
}

struct RuntimeRecordRowField {
    name: Box<str>,
    ty: RuntimeTypeRef,
}
```

The exact spelling may follow current code, but the semantic distinction is mandatory.

A `RecordFields` generic argument is not a proper `Record` type.

Do not encode:

```text
StaticArgument::RecordRow(...)
```

as:

```text
RuntimeOverlayTypeNode::Record(...)
```

Those inhabit different semantic domains.

---

# 12. Runtime row substitution rule

Runtime instantiation may substitute ordinary runtime type parameters **inside an already structurally known row**.

For example:

```text
MyRecord<{ payload: T }>
```

under:

```text
T = Int
```

may become:

```text
MyRecord<{ payload: Int }>
```

This is ordinary substitution, not row solving.

The VM/runtime may:

```text
project
substitute
intern
look up descriptor
```

It may not:

```text
infer a missing row
solve RecordFields constraints
invent row bindings
close unresolved tails
```

---

# 13. Open runtime row tails must continue to fail exact instantiation

The runtime behavior exposed by the original failing `PhantomRow` test was correct:

```text
PhantomRow<**Fs>
```

with unresolved `Fs` is not an exact reifiable runtime type.

Therefore exact runtime instantiation must continue to fail.

Do not repair `PhantomRow` by:

```text
treating Fs as {}
```

or:

```text
erasing Fs from exact identity
```

or:

```text
having the VM inspect the enclosing callable return type
```

or:

```text
adding runtime row solving
```

The semantic layer must resolve:

```text
Fs = { payload: T }
```

before lowering.

---

# 14. Do not broaden RuntimeTypeEnvironment unnecessarily

The current bounded T5 design remains:

```rust
RuntimeTypeEnvironment {
    bindings: Box<[(StableTypeParameterRef, RuntimeTypeRef)]>,
}
```

Do not generalize this immediately into:

```rust
StableTypeParameterRef -> RuntimeStaticArgument
```

merely to fix `PhantomRow`.

Once semantic contextual specialization produces:

```text
PhantomRow<{ payload: T }>
```

the row itself is structurally present and runtime substitution only needs:

```text
T -> Int
```

inside its field.

A future feature that genuinely requires a runtime row-valued binding:

```text
Fs -> { payload: Int }
```

would justify a separate architecture decision.

It is not required for the current P4 case.

---

# 15. Runtime exact specialization identities

The existing runtime interner remains the sole canonical authority for runtime overlay type identity.

For example:

```text
PhantomRow<{ payload: Int }>
PhantomRow<{ payload: String }>
```

must have:

```text
different exact RuntimeTypeRef
different exact data descriptor
same declaration behavior class
possibly identical physical layout
```

For the nullary phantom case, the physical layouts are trivially identical.

This is intentional.

The invariant is:

```text
exact type identity
    !=
physical layout identity
    !=
behavior class identity
```

---

# 16. Phantom specialization does not imply monomorphization

Do not interpret reified phantom generic types as requiring duplicated executable code.

The intended cost model is:

```text
one declaration
    -> one behavior class

many exact specializations
    -> interned exact runtime type identities/descriptors

physical layouts
    -> shared where structurally equal

generic executable code
    -> shared by default
```

For example:

```text
Tagged<User>
Tagged<Order>
Tagged<Session>
```

may all share the same:

```text
ProductLayout(Int)
```

while preserving three exact semantic types.

No per-value generic argument array should be introduced.

---

# 17. Runtime reification remains demand-driven

Phalcom does not need to enumerate every possible generic specialization.

The relevant cost is:

```text
O(distinct exact specializations actually reified)
```

not:

```text
O(all theoretically possible generic applications)
```

and not:

```text
O(number of constructed values)
```

For nullary data, the runtime already has immediate singleton representation via exact descriptor identity.

Repeated:

```phalcom
Signal<Int>()
Signal<Int>()
Signal<Int>()
```

does not require repeated heap allocation or repeated independent generic metadata.

Do not reject phantom generics for speculative specialization-count concerns.

If future runtime metaprogramming makes arbitrary unbounded type generation possible, runtime descriptor lifetime/GC can be revisited separately.

---

# 18. T5 descriptor and behavior-class invariants

Continue enforcing:

```text
one data declaration
    -> one behavior class

one exact specialization
    -> one exact runtime type identity

one exact specialization
    -> one exact runtime descriptor

different specializations
    may share one physical ProductLayout
```

The existing `RuntimeDataRegistry` remains authoritative.

Do not add another specialization registry.

Registration of an already-existing exact specialization should validate consistency rather than silently accepting mismatched:

```text
behavior class
layout
component mapping
exact type
```

---

# 19. Required T5 identity matrix

The following remain mandatory.

### Closed row survives runtime projection

```text
MyRecord<{ payload: Int }>
```

must retain the row argument.

### Type substitution inside row

```text
MyRecord<{ payload: T }>
T = Int
```

must instantiate to:

```text
MyRecord<{ payload: Int }>
```

### Distinct exact specializations

```text
MyRecord<{ payload: Int }>
!=
MyRecord<{ payload: String }>
```

as exact runtime types.

### Repeated exact specialization interns

Two occurrences of:

```text
MyRecord<{ payload: Int }>
```

must return the same canonical runtime type identity.

### Same layout does not merge type identity

```text
PhantomRow<{ payload: Int }>
PhantomRow<{ payload: String }>
```

have the same empty physical layout but distinct exact types/descriptors.

### Behavior class remains shared

Both still report the same declaration behavior class.

### Row/type boundary

A row generic argument must not become a proper `Record` runtime type merely to fit the overlay.

### Open-tail rejection

An unresolved runtime row tail still fails exact instantiation.

### Type-only applied types

Existing ordinary:

```text
Box<Int>
List<String>
```

runtime application must remain unchanged.

---

# 20. T5 reflection/API consideration

Audit consumers that currently assume every `Applied` argument is a `RuntimeTypeRef`.

In particular, an API conceptually equivalent to:

```rust
arguments() -> Option<Vec<RuntimeTypeRef>>
```

cannot silently omit row arguments.

Preferred bounded direction:

```rust
static_arguments() -> Option<Vec<RuntimeStaticArgument>>
```

while retaining the existing type-only API, if necessary, as a compatibility projection that succeeds only when every argument is `Term`.

Do not fabricate `RuntimeTypeRef`s for rows.

Audit at least:

```text
instantiate_type_ref
Applied display/formatting
remaining_parameter_count
reflection/inspection argument APIs
runtime type application primitives
exact equality/interning
metadata -> runtime projection
data descriptor specialization
```

---

# 21. Canonical product constructor associated surface

Product-shaped runtime `data` owns one primary constructor.

The canonical associated name is now:

```phalcom
Construct
```

Examples:

```phalcom
Tagged<Int>::Construct(42)

Person<Int>::Construct(
    name: "Ada",
    payload: 42,
)
```

The semantic identity remains:

```rust
DataConstructorId {
    owner: DeclarationId,
}
```

Do not introduce a new constructor ID whose identity contains the word `Construct`.

`Construct` is the associated namespace spelling for the existing declaration-owned primary constructor.

---

# 22. Supersession of the current `Type::Type` constructor family spelling

Current code constructs the product-data associated family under:

```rust
SelectorBase::Named(owner.name.to_string())
```

and `DataInfo::constructor_selector()` likewise uses the owner name.

That is why existing tests use:

```phalcom
&Point::Point(_, _)
```

This should migrate to:

```phalcom
&Point::Construct(_, _)
```

and:

```phalcom
Point::Construct(1, 2)
```

The earlier provisional allowance of:

```phalcom
Tagged<Int>::Tagged(42)
```

as a canonical long-term spelling is superseded.

The canonical associated constructor is:

```phalcom
Tagged<Int>::Construct(42)
```

Direct construction remains:

```phalcom
Tagged<Int>(42)
Tagged(42)
```

If compatibility with already-landed tests/source requires temporarily accepting:

```phalcom
Tagged<Int>::Tagged(42)
```

it may be treated as a migration alias, but:

- do not publish a second constructor semantic identity;
- do not publish a second independently reflected associated family;
- new tests/specifications should use `Construct`;
- the final P4 source surface should converge on `Construct`.

Prefer eliminating the legacy spelling during this P4 migration if doing so is mechanically safe.

---

# 23. Associated namespace construction rules

For:

```phalcom
data Tagged<T>(_ value: Int)
```

these are equivalent constructor invocations:

```phalcom
Tagged(42)

Tagged<Int>(42)

Tagged<Int>::Construct(42)
```

subject to ordinary generic inference/specialization.

For:

```phalcom
data Person<T> {
    name: String
    payload: T
}
```

these are equivalent:

```phalcom
Person {
    name: "Ada",
    payload: 42,
}

Person<Int> {
    name: "Ada",
    payload: 42,
}

Person<Int>::Construct(
    name: "Ada",
    payload: 42,
)
```

All must converge on:

```text
same DataConstructorId
same exact semantic specialization
same argument/component semantics
same lowering path
```

---

# 24. Product-data `::Construct` namespace ownership

Within the associated namespace of a product `data`, `Construct` belongs to the compiler-defined primary-constructor family.

A conflicting user-authored **associated** declaration named `Construct` must obey the existing associated-surface collision policy and must not create an ambiguous parallel meaning.

Do not introduce another associated-family precedence system just for constructors.

Ordinary dot behavior remains a separate semantic plane unless existing namespace collision rules explicitly forbid a particular declaration.

In particular:

```phalcom
Tagged.Construct(...)
```

and:

```phalcom
Tagged::Construct(...)
```

are not required to denote the same thing.

The latter is the compiler-defined primary-constructor associated identity.

---

# 25. Tuple-shaped constructor references

Canonical:

```phalcom
&Tagged<Int>::Construct(_)
```

Ergonomic alias:

```phalcom
&Tagged<Int>(_)
```

Both produce the same first-class constructor capability.

Once captured:

```phalcom
let factory = &Tagged<Int>(_)

let x = factory(42)
```

the invocation is an ordinary callable invocation.

The constructor thunk already exists in the compiler path through the current data-constructor callable-reference lowering machinery.

Do not introduce a special runtime constructor-call protocol.

---

# 26. Record-shaped constructor references

Do not invent brace-shaped first-class callable syntax.

Do **not** add:

```phalcom
&Person<Int> {
    name: _,
    payload: _,
}
```

or:

```phalcom
&Person<Int>::Construct {
    name: _,
    payload: _,
}
```

The constructor is a callable and should use ordinary callable selector syntax:

```phalcom
&Person<Int>::Construct(
    name: _,
    payload: _,
)
```

with ergonomic alias:

```phalcom
&Person<Int>(
    name: _,
    payload: _,
)
```

Invocation is ordinary:

```phalcom
let factory = &Person<Int>(
    name: _,
    payload: _,
)

let ada = factory(
    name: "Ada",
    payload: 42,
)
```

This deliberately separates:

```text
record literal / direct product construction
```

from:

```text
first-class callable invocation
```

---

# 27. Record constructor ordering ruling

For direct braced record construction:

```phalcom
Person {
    payload: 42,
    name: "Ada",
}
```

retain the existing label-oriented semantics and source evaluation guarantees.

Do not unnecessarily remove flexible record field ordering.

For the callable constructor shape:

```phalcom
Person::Construct(
    name: ...,
    payload: ...,
)
```

and exact constructor references:

```phalcom
&Person::Construct(
    name: _,
    payload: _,
)
```

use **source declaration order** for P4.

Given:

```phalcom
data Person {
    name: String
    payload: Int
}
```

the canonical callable selector shape is:

```text
Construct(name,payload)
```

not every permutation of those labels.

Therefore this should be canonical:

```phalcom
&Person::Construct(
    name: _,
    payload: _,
)
```

Do not add, as part of this work, selector-equivalence machinery merely so that:

```phalcom
&Person::Construct(
    payload: _,
    name: _,
)
```

resolves identically.

If ordinary labeled-selector semantics are later redesigned to be order-insensitive, data constructors may inherit that rule.

Until then, declaration order keeps implementation and reflection identity simple.

---

# 28. Preserve source evaluation order

Even when constructor arguments map into declaration-order logical slots, source expression evaluation order remains observable and must not be reordered.

Keep the distinction:

```text
source evaluation order
logical component order
physical layout order
```

No constructor-reference or `Construct` migration may weaken this invariant.

---

# 29. Direct constructor shorthand applies only to statically resolved data-type heads

The privileged form:

```phalcom
Tagged(42)
```

or:

```phalcom
Tagged<Int>(42)
```

is recognized because the head statically resolves to a product `data` declaration/type application.

It is not ordinary `call` dispatch.

Similarly:

```phalcom
Person<Int> {
    ...
}
```

is privileged direct product construction because the head statically resolves as a data type form.

Do not make arbitrary runtime values eligible merely because dataflow can prove they currently contain the same class/type object.

Alias handling should follow the existing canonical type-form resolution rules. Do not broaden alias semantics merely for this task.

The critical boundary is:

```text
statically resolved data type form
    => constructor syntax

ordinary runtime expression
    => ordinary callable protocol
```

---

# 30. `call(_)` collision ruling

Suppose a type has user-authored class-side behavior:

```phalcom
impl Tagged {
    @class
    call(_ value: Int) {
        ...
    }
}
```

Then:

```phalcom
Tagged(42)
```

still means:

```text
primary DataConstructorId(Tagged)
```

and:

```phalcom
Tagged<Int>(42)
```

still means exact specialized primary construction.

To invoke the user-authored behavior:

```phalcom
Tagged.call(42)
Tagged<Int>.call(42)
```

Use an explicit callable reference for that behavior:

```phalcom
&Tagged.call(_)
&Tagged<Int>.call(_)
```

The constructor references remain:

```phalcom
&Tagged(_)
&Tagged<Int>(_)

&Tagged::Construct(_)
&Tagged<Int>::Construct(_)
```

No ambiguity should remain.

---

# 31. Runtime-variable call behavior

The constructor shorthand is not contagious through runtime values.

Example:

```phalcom
let factory = Tagged
factory(42)
```

Here `factory` is an ordinary runtime value.

Therefore:

```phalcom
factory(42)
```

uses the ordinary callable protocol:

```phalcom
factory.call(42)
```

if such behavior exists.

By contrast:

```phalcom
let factory = &Tagged(_)
factory(42)
```

captures a constructor callable, so its ordinary `call` implementation invokes the primary data constructor.

This is intentional.

Do not add compiler dataflow rules that convert arbitrary runtime class values back into privileged data-construction syntax.

---

# 32. Selector source syntax ruling

The current selector model structurally stores:

```rust
pub enum SelectorSlot {
    Positional,
    Label(String),
}
```

and canonical selector transport currently emits forms such as:

```text
callable(_,name)
```

This representation remains authoritative.

However, source selector/reference syntax must additionally accept:

```phalcom
callable(_, name: _)
```

as a clearer spelling of the same labeled selector slot.

Therefore:

```phalcom
&object.callable(_, name)

&object.callable(_, name: _)
```

must normalize identically.

Likewise:

```phalcom
&Person::Construct(name, payload)

&Person::Construct(name: _, payload: _)
```

must identify the same selector.

The new spelling is preferred for newly authored examples and documentation.

---

# 33. Do not change canonical selector encoding

The parser sugar:

```phalcom
name: _
```

must normalize to:

```rust
SelectorSlot::Label("name")
```

It must not become a new runtime selector-slot type.

Do not change:

```rust
Selector::encode()
```

merely to print the new syntax.

Canonical selector transport may continue to use:

```text
callable(_,name)
```

This avoids compatibility churn across:

```text
runtime interned selector symbols
metadata
hashing/equality
method lookup
reflection
existing source compatibility
incremental fingerprints
tests
```

If a nicer source-oriented formatter is desirable for diagnostics/tooling, add or reuse a presentation formatter rather than changing semantic encoding.

---

# 34. Parser implementation for `name: _`

The existing parser helper:

```rust
parse_selector_spec_slots(...)
```

currently recognizes approximately:

```text
_
identifier
...
```

Extend the labeled branch so that:

```text
identifier
```

and:

```text
identifier : _
```

both produce:

```rust
SelectorSlot::Label(identifier)
```

The syntax range for the latter should include the full:

```text
name: _
```

for source indexing/diagnostics where useful.

Apply the same accepted spelling consistently to selector-spec surfaces that share this parser machinery:

```phalcom
&object.foo(_, label: _)

&Type::Construct(label: _)

#foo(_, label: _)

#foo(_, ..., tail: _)
```

where the corresponding selector/pattern form is otherwise legal.

Do not create a second independent selector parser solely for constructor references.

---

# 35. Constructor-reference parser implementation guidance

The canonical explicit form:

```phalcom
&Tagged<Int>::Construct(_)
```

can continue to use the existing associated callable-reference path.

The ergonomic shorthand:

```phalcom
&Tagged<Int>(_)
```

needs a direct source representation.

Implement it with the smallest AST/parser extension that:

- preserves source ranges;
- lets semantic analysis resolve the head as a type form;
- normalizes semantically to the same `DataConstructorId`;
- does not invent fake source text or a fake `Construct` token;
- does not create another runtime callable category.

If the existing AST can represent the sugar without falsifying source ranges or source-index identities, reuse it.

Otherwise introduce a narrow syntactic constructor-reference form and normalize it during semantic analysis.

Do **not** implement the shorthand by fabricating a synthetic `AssociatedLookupExpr` whose source claims the user wrote `::Construct`.

Tooling/source-index correctness matters.

---

# 36. Existing associated constructor machinery should be reused

Current repository infrastructure already has:

```text
AssociatedFamilyKind::DataConstructor
AssociatedMemberId::DataConstructor
InvocationTargetId::DataConstructor
DataConstructorId
CallableReferenceLoweringSpec::MakeDataConstructorThunk
AssociatedLoweringSpec::MakeDataConstructorThunk
```

and compiler support for constructing a callable thunk that executes the exact data constructor.

Reuse these.

The feature is primarily about:

```text
associated-family naming
source normalization
generic specialization
selector syntax
```

not a new lowering/runtime abstraction.

---

# 37. Update `build_data_associated_surface`

Current implementation conceptually does:

```rust
let base = SelectorBase::Named(owner.name.to_string());
```

Change the canonical associated family base to:

```rust
SelectorBase::Named("Construct".to_string())
```

or the repository's preferred interned/static equivalent.

The family still contains:

```rust
AssociatedMemberId::DataConstructor(
    DataConstructorId::new(owner.clone())
)
```

No constructor identity change occurs.

---

# 38. Update `DataInfo::constructor_selector`

Current implementation uses:

```rust
SelectorBase::Named(self.owner.name.to_string())
```

for the primary data constructor selector.

For the associated callable family, migrate this to the canonical base:

```text
Construct
```

while preserving the constructor operation slots.

Tuple-shaped:

```text
Construct(_, _)
```

Record-shaped:

```text
Construct(name,payload)
```

The operation shape remains derived from the product declaration in source order.

---

# 39. Visibility remains attached to the constructor capability

All constructor spellings must enforce the same primary-constructor visibility rules.

If:

```phalcom
data Secret private(_ value: Int)
```

then all unauthorized forms must fail consistently:

```phalcom
Secret(42)
Secret::Construct(42)
&Secret(_)
&Secret::Construct(_)
```

Legally acquired constructor capabilities remain stable after escape, as already established.

Do not rerun lexical access checks every time an already-authorized first-class constructor is invoked.

---

# 40. T5 acceptance should now include PhantomRow

Once contextual constructor specialization is repaired, the original empty-layout discriminator becomes a strong T5 test:

```phalcom
data PhantomRow<**Fs: RecordFields>()

class Factory {
    @class
    make<T>(_ value: T) -> PhantomRow<**{ payload: T }> {
        PhantomRow()
    }
}
```

Construct at:

```text
T = Int
T = String
```

and prove:

```text
exact types differ
exact descriptors differ
physical layouts may be identical/empty
behavior class is shared
construction succeeds
runtime row substitution is ordinary substitution only
```

This is an excellent discriminator precisely because layout cannot accidentally carry specialization identity.

The test should no longer remain an expected failing supplemental test after the semantic fix lands.

---

# 41. Keep a separate low-level open-tail rejection test

Do not delete the test proving that an actually unresolved runtime row fails exact instantiation.

Both facts must coexist:

```text
semantic PhantomRow constructor with sufficient expected context
    => must be specialized before lowering
    => succeeds

genuinely unresolved runtime RecordRow tail
    => runtime cannot make it exact
    => fails closed
```

These are complementary invariants, not contradictions.

---

# 42. T5 acceptance checklist after this ruling

T5 is accepted only when focused evidence covers all of the following:

- shared behavior class exists independently of specialization layout;
- exact specialized runtime type identity survives heterogeneous arguments;
- exact descriptors are keyed by exact runtime type;
- closed `RecordFields` arguments survive runtime projection;
- ordinary `T` substitution works inside row fields;
- same specialization interns identically;
- distinct specializations remain distinct;
- equal/empty physical layouts do not merge exact identities;
- expanded specialization-only components resolve through exact logical slots;
- no fake declaration component IDs exist for expansion-derived members;
- fixed-only data behavior is unchanged;
- MyTuple specialization behavior passes;
- MyRecord specialization behavior passes;
- fixed-plus-expanded Envelope behavior passes;
- generic callable MyRecord behavior passes;
- `PhantomRow` generic callable behavior passes;
- runtime open-tail rejection remains active and passing;
- ordinary type-only runtime `Applied` behavior remains unchanged;
- `.class` remains declaration behavior identity;
- `.type` remains exact specialization identity;
- no per-specialization behavior class is introduced;
- no runtime row solver is introduced.

---

# 43. Required constructor-surface acceptance tests

Add focused tests covering at least the following equivalences.

## Tuple-shaped direct construction

```phalcom
data Tagged<T>(_ value: Int)

let a = Tagged<Int>(42)
let b = Tagged<Int>::Construct(42)
```

Prove same exact type/value semantics.

Also preserve inferred:

```phalcom
Tagged(42)
```

where inference is sufficient.

## Tuple constructor references

```phalcom
let a = &Tagged<Int>::Construct(_)
let b = &Tagged<Int>(_)

let x = a(42)
let y = b(42)
```

Prove equivalent constructor capability/results.

## Record direct construction

```phalcom
data Person<T> {
    name: String
    payload: T
}

let a = Person<Int> {
    name: "Ada",
    payload: 42,
}

let b = Person<Int>::Construct(
    name: "Ada",
    payload: 42,
)
```

Prove equivalent exact type/product values.

## Record constructor references

```phalcom
let a = &Person<Int>::Construct(
    name: _,
    payload: _,
)

let b = &Person<Int>(
    name: _,
    payload: _,
)
```

Invoke both through:

```phalcom
a(name: "Ada", payload: 42)
b(name: "Ada", payload: 42)
```

and prove equivalent construction.

## Constructor visibility

Test all shorthand and canonical forms against public/private/module/package/internal constructor rules as appropriate to the existing matrix.

---

# 44. Required `call` collision tests

Given a product `data` with ordinary class-side `call` behavior, prove:

```phalcom
Tagged(42)
```

resolves to `DataConstructorId`.

Prove:

```phalcom
Tagged.call(42)
```

resolves to ordinary behavioral dispatch.

Prove:

```phalcom
&Tagged(_)
```

references the constructor.

Prove:

```phalcom
&Tagged.call(_)
```

references the behavior.

If supported by the current object model, also prove:

```phalcom
let factory = Tagged
factory(42)
```

takes the ordinary callable-value path rather than being retrospectively rewritten as primary data construction.

Do not special-case runtime variables based on constant propagation.

---

# 45. Required selector-syntax tests

These must normalize identically:

```phalcom
&object.callable(_, name)
&object.callable(_, name: _)
```

Likewise:

```phalcom
#callable(_, name)
#callable(_, name: _)
```

where selector-symbol syntax supports the form.

Prove that both result in:

```rust
SelectorSlot::Positional
SelectorSlot::Label("name")
```

and the same canonical selector encoding.

Add pattern coverage where currently supported:

```phalcom
&object.callable(_, ..., tail)
&object.callable(_, ..., tail: _)
```

Do not change positional-before-labeled validation.

The following should remain invalid if it violates existing selector ordering:

```phalcom
callable(name: _, _)
```

unless a separately accepted selector-order redesign says otherwise.

---

# 46. Do not broaden record callable permutation semantics

For this P4 work, do not add equivalence between:

```phalcom
&Person::Construct(
    name: _,
    payload: _,
)
```

and:

```phalcom
&Person::Construct(
    payload: _,
    name: _,
)
```

if doing so requires modifying selector identity or associated-family matching.

Direct record construction may continue accepting arbitrary field ordering:

```phalcom
Person {
    payload: 42,
    name: "Ada",
}
```

because that is product-label construction semantics, not selector identity.

This distinction is acceptable and intentional.

---

# 47. Do not add record-callable brace syntax

Explicitly out of scope:

```phalcom
&Person {
    name: _,
    payload: _,
}

&Person::Construct {
    name: _,
    payload: _,
}
```

There is no need for a second syntax for first-class callable values.

Use:

```phalcom
&Person(
    name: _,
    payload: _,
)
```

or:

```phalcom
&Person::Construct(
    name: _,
    payload: _,
)
```

and ordinary:

```phalcom
factory(...)
```

after capture.

---

# 48. Implementation sequence from the current repository state

Proceed in this order.

## Step 1 — Update checkpoint state with this ruling

The current checkpoint still records an architect scope decision as the blocker.

Replace that state with the adopted ruling:

```text
PhantomRow is in scope.

Semantic expected-result specialization of otherwise-underconstrained
data-owner generic parameters is required.

Runtime open-row rejection remains mandatory.

T5 may proceed after the semantic constructor/result-context gap is repaired.
```

Record this as a consultation/amendment entry.

Do not claim T5 complete merely because the runtime representation slice is focused-tested.

---

## Step 2 — Repair semantic contextual specialization for primary data constructors

Start from the existing data constructor application path and generic application session.

Do not create a new inference engine.

The fix should make an expected exact result constrain unresolved declaration-owner generic parameters.

For example:

```text
constructor result template:
    PhantomRow<Fs>

expected:
    PhantomRow<{payload:T}>

constraint:
    Fs = {payload:T}
```

The final semantic constructor expression must publish:

```text
PhantomRow<{payload:T}>
```

not a result retaining unresolved `PhantomRow.Fs`.

Preserve context-only proof strength as `Assumed`.

---

## Step 3 — Verify ordinary phantom generic contextual selection

Before touching runtime behavior further, prove at the semantic level:

```phalcom
data Phantom<T>()

let x: Phantom<Int> = Phantom()
```

and the no-context negative case.

This prevents accidentally implementing only a RecordFields-specific path.

---

## Step 4 — Restore and pass the PhantomRow end-to-end discriminator

Run the existing failing:

```text
p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable
```

or its renamed final equivalent.

It must now reach runtime with a closed structural row:

```text
{payload:T}
```

rather than an unresolved `Fs` tail.

The runtime should then substitute ordinary `T` and succeed without any additional row-solving machinery.

---

## Step 5 — Complete T5 runtime consumer audit

Audit every use of heterogeneous `Applied` runtime arguments.

At minimum:

```text
typing/environment.rs
typing/overlay.rs
typing/inspect.rs
runtime type display
runtime type reflection
remaining parameter count
metadata runtime projection
data descriptor binding
runtime type application primitives
```

Do not silently drop `RecordRow` arguments.

---

## Step 6 — Implement the canonical `::Construct` associated surface

Change product data's compiler-defined associated family base from owner-name repetition to:

```text
Construct
```

Update:

```text
build_data_associated_surface
DataInfo::constructor_selector
associated-resolution tests
callable-reference tests
visibility tests
source-index/tooling expectations
```

Do not change `DataConstructorId`.

---

## Step 7 — Implement direct explicit specialization syntax

Ensure tuple-shaped:

```phalcom
Tagged<Int>(42)
```

directly resolves to the exact primary data constructor.

Ensure record-shaped:

```phalcom
Person<Int> {
    ...
}
```

does likewise.

They should not dispatch through user-authored `call`.

---

## Step 8 — Implement ergonomic constructor-reference sugar

Support:

```phalcom
&Tagged<Int>(_)

&Person<Int>(
    name: _,
    payload: _,
)
```

Normalize them semantically to the same constructor capability as:

```phalcom
&Tagged<Int>::Construct(_)

&Person<Int>::Construct(
    name: _,
    payload: _,
)
```

Preserve source ranges.

Do not fake a written `::Construct` token in the AST/source index.

---

## Step 9 — Add `label: _` selector-spec syntax

Extend the canonical selector-spec parser.

Both:

```phalcom
foo(_, label)
foo(_, label: _)
```

must normalize to the same selector.

Update new examples/tests to prefer the clearer form.

Do not modify runtime selector encoding.

---

## Step 10 — Add call-collision regression tests

Prove the distinction among:

```phalcom
Tagged(...)
Tagged::Construct(...)
Tagged.call(...)
&Tagged(...)
&Tagged::Construct(...)
&Tagged.call(...)
```

and ordinary runtime-variable invocation.

---

## Step 11 — Re-run T4/T5 focused matrix

Do not move to T6 until the semantic and runtime product-specialization boundary is coherent.

At minimum include:

```text
T4 constructor inference
T4 row proof matrix
expected-result row selection
source-established row + compatible context
phantom Type contextual selection
phantom RecordFields contextual selection
MyTuple
MyRecord
Envelope
fixed-only control
generic-callable MyRecord
PhantomRow
runtime closed-row substitution
runtime open-tail rejection
same-layout exact descriptor identity
shared behavior class
constructor associated/reference aliases
selector alias syntax
```

---

# 49. T4 completion disposition

T4's previously recorded focused implementation remains architecturally accepted.

The new contextual phantom constructor work is best treated as a **semantic completion discovered by T5**, not as a reason to reopen the entire T4 design.

After the contextual specialization fix passes:

```text
T4 semantic architecture:
    accepted

T4 focused evidence:
    retained

additional P4 semantic completion:
    expected-result specialization for phantom owner parameters
```

Do not rewrite T4's row calculus, product template model, or proof bridge.

---

# 50. T5 completion disposition

After the changes above, T5 may be marked implemented/focused-tested only if:

```text
semantic exact specialization reaches lowering
+
runtime heterogeneous exact identity survives
+
same-layout phantom specialization remains distinct
+
constructor surface aliases converge
+
no runtime solver/erasure is introduced
```

Do not mark T5 complete while the active `PhantomRow` discriminator still fails.

---

# 51. Remaining P4 execution order after T5

Once T5 is accepted, resume the original plan rather than inventing a new P4 lifecycle.

The remaining tasks are:

```text
T6 — Complete declaration type-family/runtime projections and
     consolidate intrinsic application

T7 — Complete source-visible structural domains, intrinsic associated
     types, and specialization-aware Tuple/Record APIs

T8 — Migrate Record/Map/Set syntax, row-tail spelling, and Record
     remainder patterns

T9 — Separate Unit from EmptyRecord at runtime and make zero-product
     construction obey the new family laws

T10 — Replace Unit runtime-class ownership with intrinsic aliases and
      direct Tuple/Record runtime owners

T11 — Migrate the Universe/core source and repository corpus to the
      completed structural model

T12 — Persist the new semantic surface through metadata, incremental
      fingerprints, source indexing, LSP, and specifications

T13 — Stabilize, certify C3.P4, re-close LANG007.C3, and produce
      execution records
```

Do not skip directly from T5 to broad certification.

---

# 52. T6 guidance

T6 should consume the exact specialization model established by T4/T5.

Do not reintroduce intrinsic-only type-application shortcuts that bypass canonical generic application.

Preserve:

```text
semantic type-family projection
runtime owner projection
```

as distinct concerns.

The runtime class/owner may be shared even when exact types differ.

Ensure no T6 projection path accidentally reconstructs exact type identity from behavior class identity.

---

# 53. T7 guidance

T7 should complete the public intrinsic structural surface without weakening the product/generic laws.

Tuple and Record associated/type-member APIs must use canonical specialization.

No public intrinsic member should depend on source-less parallel semantic identity if an authored/intrinsic declaration product is already authoritative.

Preserve non-injective projection rules already established for:

```text
Tuple::Element
pack projections
associated members
```

Do not reverse-infer subjects from non-injective result equality.

---

# 54. T8 guidance

Continue the planned migration toward the public spelling:

```phalcom
RecordFields
```

while preserving internal row machinery.

Record literal, Map literal, Set literal, row-tail syntax, and record remainder patterns should converge on their intended distinct syntactic categories.

Do not expose compiler-internal `RecordRow` terminology in final public source examples unless discussing implementation internals.

---

# 55. T9/T10 zero-product guidance

Preserve the planned laws:

```text
Unit = Tuple<>
EmptyRecord = Record<**{}>
```

They are different structural values/types despite both being zero-product shapes.

Runtime representation may optimize both aggressively, but must preserve:

```text
().class == Tuple
{}.class == Record
() != {}
```

according to the plan's final semantics.

Do not resurrect a standalone semantic/runtime `Unit` class merely for convenience.

---

# 56. T11 source migration guidance

After semantic/runtime behavior is complete, migrate Universe/core source and corpus.

The migration must use the final source vocabulary, including:

```text
RecordFields
::Construct
preferred labeled selector spelling `name: _`
completed Tuple/Record generic surfaces
```

Do not leave implementation-era examples such as:

```phalcom
Point::Point(...)
```

as canonical documentation if the final associated constructor spelling is `::Construct`.

A repository census should identify stale constructor references and old public row terminology.

---

# 57. T12 persistence/tooling guidance

Any newly durable constructor-associated identity must persist using the existing stable identity:

```text
DataConstructorId(owner)
AssociatedFamilyKind::DataConstructor
```

Do not serialize the source sugar as identity.

The following source forms must resolve back to the same canonical target in source indexing/LSP:

```phalcom
Tagged(...)
Tagged<T>(...)
Tagged<T>::Construct(...)

&Tagged<T>(...)
&Tagged<T>::Construct(...)
```

The same applies to record-shaped product construction/reference forms.

Metadata must preserve exact heterogeneous applied arguments, including row-domain arguments, without lowering them into proper types.

Cold/incremental parity is required.

---

# 58. Verification gates remain unchanged in structure

After focused task completion, follow the P4 plan's gates.

## G1

Structural kind, binder, declaration-shape correctness.

## G2

Specialization layout and declaration projection.

This gate must now include the final T5 phantom/equal-layout runtime discriminator.

## G3

Source surface, syntax, zero-product behavior.

Include:

```text
::Construct
constructor shorthand/reference aliases
name: _ selector syntax
```

in affected focused groups.

## G4

Persistence/tooling.

Ensure all constructor aliases point to one stable semantic identity.

## G5

Migrated source/corpus.

Search for stale:

```text
::OwnerName primary constructor spelling
public RecordRow terminology
old selector examples where migration expects name: _
```

while distinguishing intentional internal implementation names.

## G6

Broad C3.P4 re-closure.

Run once the focused gates are coherent.

---

# 59. Broad certification remains last

Do not use broad workspace failures to drive architecture before focused T4/T5 acceptance is green.

The planned final commands remain:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Classify failures according to the plan/checkpoint conventions.

Do not silently claim release-wide verification from focused tests.

---

# 60. Explicit forbidden approaches

The following remain prohibited.

### Runtime identity shortcuts

Do not:

```text
erase RecordFields arguments
encode RecordFields as proper Record RuntimeTypeRef
merge specializations because layouts match
create per-specialization behavior classes
add a second descriptor registry
store generic argument arrays on each data instance
```

### Runtime inference shortcuts

Do not:

```text
solve rows in the VM
infer Fs from enclosing runtime return types
silently close unresolved row tails
broaden RuntimeTypeEnvironment without an actual required case
```

### Semantic shortcuts

Do not:

```text
special-case only PhantomRow by name
special-case only RecordFields contextual selection
attach proof strength to canonical RecordRowId
make context override conflicting source evidence
infer empty phantom parameters merely from nullary construction
```

### Constructor shortcuts

Do not:

```text
model the primary data constructor as an ordinary user method
create distinct constructor identities for shorthand and ::Construct
introduce record-brace syntax for first-class callable references
make arbitrary runtime class values use privileged constructor syntax
```

### Selector shortcuts

Do not:

```text
change canonical runtime selector encoding merely for name: _ source sugar
create a second labeled-selector identity for name: _
make record-constructor selector permutations equivalent unless ordinary selector semantics are redesigned
```

---

# 61. Required checkpoint amendment

Add a consultation/ruling entry substantially equivalent to:

```text
INC-C3-<next> — Phantom generic contextual construction and canonical
product-constructor associated surface

Plan/task:
    LANG007.C3.P4 / T4-T5 semantic/runtime boundary.

Trigger:
    The empty-layout PhantomRow discriminator reached runtime with an unresolved
    declaration-owned RecordFields tail even though the enclosing expected result
    was PhantomRow<{payload:T}>. Repository inspection also showed that phantom
    generic data, contextual generic construction, and expected-result generic
    selection are already accepted language behavior.

Decision:
    The prior provisional deferral is superseded.

    Data primary constructors participate in ordinary expected-result generic
    inference even when a declaration-owned generic parameter is phantom with
    respect to product construction.

    Semantic analysis must resolve PhantomRow() under expected
    PhantomRow<{payload:T}> to that exact symbolic specialization before lowering.

    Context-only selection contributes Assumed evidence and must not overwrite
    contradictory source evidence.

    Runtime open-row rejection remains mandatory. The VM does not solve rows.

    Product data exposes its canonical primary constructor through the associated
    family `::Construct`. Direct Type(...) / Type<T>(...) and record Type{...} /
    Type<T>{...} forms are ergonomic constructor syntax over the same
    DataConstructorId.

    Exact constructor references use `&Type<T>::Construct(...)`; ergonomic
    `&Type<T>(...)` is also accepted and resolves to the same capability.

    Record constructor references use ordinary labeled callable syntax, not braces,
    and declaration order is the required selector order for this checkpoint.

    Selector specifications additionally accept `label: _` as source sugar for the
    existing `SelectorSlot::Label(label)`; canonical selector identity/encoding does
    not change.

Architecture impact:
    No new constructor identity, runtime solver, runtime row-valued environment,
    specialization registry, or per-specialization behavior class.

Required verification:
    Phantom<T> contextual construction;
    PhantomRow contextual construction;
    no-context phantom underconstraint;
    context/source conflict;
    closed-row runtime substitution;
    runtime open-tail rejection;
    same-layout distinct exact descriptors;
    shared behavior class;
    ::Construct invocation/reference;
    direct constructor aliases;
    record constructor callable;
    call(_) collision;
    old/new labeled selector syntax normalization.

Status:
    ADOPTED — semantic PhantomRow repair is in scope; T5 may complete after
    focused acceptance.
```

---

# 62. Final architecture summary

The intended end state is:

```text
                         SOURCE

data Tagged<T>(_ value: Int)

Tagged(42)
Tagged<Int>(42)
Tagged<Int>::Construct(42)

&Tagged<Int>(_)
&Tagged<Int>::Construct(_)


data Person<T> {
    name: String
    payload: T
}

Person<Int> {
    name: "Ada",
    payload: 42,
}

Person<Int>::Construct(
    name: "Ada",
    payload: 42,
)

&Person<Int>(
    name: _,
    payload: _,
)

&Person<Int>::Construct(
    name: _,
    payload: _,
)


                         SEMANTICS

                 one DataConstructorId
                          |
                          v
             ordinary generic application
                          |
       +------------------+------------------+
       |                                     |
constructor/source evidence           expected-result evidence
       |                                     |
 Established when proven              Assumed when contextual
       |                                     |
       +------------------+------------------+
                          |
                          v
                exact semantic type
                          |
                          v
              semantic lowering recipe
                          |
                          v


                         RUNTIME

           RuntimeStaticArgument::Term(...)
           RuntimeStaticArgument::RecordRow(...)
                          |
                          v
              exact RuntimeTypeRef interning
                          |
                          v
            exact RuntimeDataDescriptorId
                          |
          +---------------+---------------+
          |                               |
shared behavior class            shared/distinct layout
          |                               |
          +---------------+---------------+
                          |
                          v
                     data value
```

For phantom data:

```text
generic argument participates in exact type identity
but contributes zero payload bits
```

For contextual construction:

```text
expected type may select an otherwise-underconstrained generic specialization
but does not magically constitute stronger source evidence
```

For runtime specialization:

```text
only actually reified exact specializations need runtime descriptors
```

For constructor syntax:

```text
static data type form + construction syntax
    => primary DataConstructorId

ordinary runtime value + (...)
    => ordinary call(...) protocol
```

For selector syntax:

```text
foo(_, name)
foo(_, name: _)

    => same SelectorSlot::Label("name")
    => same canonical selector identity
```

---

# 63. Immediate instruction to the implementing agent

Do not continue directly to T6 yet.

The next bounded implementation sequence is:

1. update the checkpoint with this adopted ruling;
2. repair semantic expected-result specialization for phantom data-owner parameters;
3. make the existing `PhantomRow` discriminator pass without touching runtime row-solving authority;
4. retain and pass the low-level runtime open-tail rejection test;
5. finish the heterogeneous-runtime-argument consumer audit;
6. migrate the product-data associated constructor family to `::Construct`;
7. implement direct explicit `Type<T>(...)` / `Type<T> { ... }` construction where missing;
8. implement `&Type<T>(...)` as constructor-reference sugar over the same `DataConstructorId`;
9. support record constructor references/invocations through ordinary labeled callable syntax in declaration order;
10. add `label: _` selector-spec syntax while preserving canonical selector encoding;
11. add the `call(_)` collision/ordinary-runtime-value regressions;
12. rerun the complete focused T4/T5 acceptance matrix;
13. only then mark T5 accepted and resume T6 → T13 in the existing P4 plan.

If implementation pressure appears to require any of the following, stop and report the exact failing case before proceeding:

```text
RecordRow -> TypeId collapse
runtime row solving
row-valued RuntimeTypeEnvironment bindings
per-specialization behavior classes
parallel exact-type registry
fake DataComponentId for expanded fields
new selector identity for `name: _`
permutation-independent selector identity
a second primary constructor identity
```

Those are architecture boundaries, not acceptable local workarounds.

---

# 64. Acceptance statement for this ruling

This amendment is complete when the repository can demonstrate all of the following simultaneously:

```text
Phantom generic data is a first-class exact semantic type mechanism.

Phantom generic parameters need not participate in storage.

Primary data construction can obtain otherwise-unconstrained generic arguments
from ordinary expected-result context.

Context-only generic selection preserves Assumed evidence.

Runtime receives exact semantic specializations rather than unresolved phantom
row tails.

Runtime substitutes ordinary type parameters inside structurally known closed
RecordFields arguments.

Runtime rejects genuinely unresolved row tails.

Distinct exact specializations remain distinct despite identical physical layout.

One declaration behavior class is shared across specializations.

Product primary construction has one stable DataConstructorId.

`::Construct` is the canonical explicit associated constructor family.

Direct construction and constructor-reference sugar normalize to that same
constructor identity.

Record constructors use ordinary labeled callable shapes for explicit invocation
and first-class references.

`label: _` and legacy bare-label selector syntax normalize to one existing
SelectorSlot::Label identity.

Static data-family construction is not confused with user-authored `call(...)`.

Ordinary runtime values continue to use the normal callable protocol.

No runtime static-domain or row solver has been introduced.
```

Once these conditions are focused-tested, proceed with the existing P4 remainder and final certification sequence.
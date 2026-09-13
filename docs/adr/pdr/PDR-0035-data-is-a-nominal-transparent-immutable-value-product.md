# PDR-0035 — `data` is a nominal transparent immutable value product

- Status: Accepted
- Date: 2026-09-12
- Supersedes: none
- Amends: none
- Related: LANG005.C1; PDR-0033 (immediate `Option` representation); ADR-0010 (`Value` representation); ADR-0050 (precise tracing); SEMA005 runtime typing metadata and reflection; legacy `@data` decorator specification
- Repository evidence revision: `aureat/phalcom-lang@40910f65fed02edc03dbb1ada4a789da98451f1f` (`main`, remote repository state inspected 2026-09-12)

## Context

Phalcom already has three nearby but semantically different product/object mechanisms:

1. ordinary class instances, represented by `phalcom-core/src/heap/instance.rs::InstanceObject`, carry a `ClassId` plus a fixed `Box<[Value]>` and initialize all instance slots to `Value::nil()` before mutation;
2. anonymous tuples and records are immutable native heap products, represented by `TupleObject` and `RecordObject`, but both retain universal `Value` storage and carry labels in each materialized value (`TupleObject` for the labeled lane, `RecordObject` for every field);
3. enum constructor cases are immutable products represented by `AdtCaseObject { variant: RuntimeVariantId, payload: Box<[Value]> }`, while nullary variants may use the immediate `AdtSingleton` `Value` tag.

Those mechanisms are useful precedents but none is the semantic or physical definition of a named immutable value product.

The compiler type system already has the correct high-level distinction. `phalcom-semantic/src/types/store.rs::TypeData` distinguishes `Nominal`, `Applied`, `ExactCase`, structural `Tuple`, and structural `Record` forms. A named data declaration therefore does not require a new structural type constructor: it is a nominal declaration whose applications use the existing `Nominal`/`Applied` calculus.

The runtime typing architecture also already separates nominal declaration identity from applied type descriptors. `phalcom-core/src/typing/reify.rs::reify_type_form` resolves a nominal form to the existing runtime class/behavior object while synthetic forms, including applications, are represented by weakly cached typing descriptors. The metadata specification explicitly rejects per-instance generic-token arrays and specialized runtime classes for every generic application.

The runtime ADT implementation provides another important precedent: `VariantId`, `RuntimeVariantId`, and `CaseDiscriminant` are intentionally different identities. Semantic identity, VM-local descriptor identity, and physical layout identity are not interchangeable.

LANG005 introduces first-class `data` declarations and later introduces `impl` blocks, traits, associated types, and reflection over those features. The representation decision must therefore be made now: if `data` is initially implemented as a class expansion or as a permanently boxed `Value` array, later compiler optimization would have to undo an observable object model rather than optimize a value model.

## Decision

### 1. `data` is a distinct nominal declaration category

Phalcom has the following semantic categories:

```text
anonymous (...) / #{...}   structural transparent immutable products
data                        nominal transparent immutable products
enum                        nominal closed sums of products
class                       nominal opaque object abstractions
trait                       behavioral/conformance abstractions
```

These distinctions are semantic and survive even when the compiler/runtime shares lower-level product machinery.

A data declaration is not a class declaration, not an enum with one case, and not an alias for an anonymous structural product.

Examples:

```phalcom
data Pair<A, B>(
  _ first: A,
  _ second: B,
)

data Point<T>(
  x: T,
  y: T,
)

data Response<T>(
  _ body: T,
  status: Status,
  headers: Headers,
)

data Person<Id> {
  id: Id
  name: String
  age: Int
}

data Signal<State>()
```

### 2. Data has tuple-shaped and record-shaped declaration forms

The parenthesized form declares a named tuple product. It may contain the same positional and labeled lanes used by Phalcom's tuple/call selector model:

```phalcom
data Pair<A, B>(
  _ first: A,
  _ second: B,
)

data Point<T>(
  x: T,
  y: T,
)
```

The braced form declares a named record product:

```phalcom
data Person {
  name: String
  age: Int
}
```

Construction follows the declared product shape:

```phalcom
Pair(42, "hello")
Point(x: 10, y: 20)
Person { name: "Altun", age: 24 }
```

The canonical nullary spelling is the parenthesized form:

```phalcom
data Signal<State>()
```

Constructor visibility syntax is not ruled by this record. Until a later visibility ruling, an otherwise visible data declaration has a publicly usable primary product constructor.

### 3. Declared components are the complete semantic instance state

Every declared component is simultaneously:

- part of the product's logical shape;
- an immutable value component;
- an exposed immutable property;
- a first-class semantic component with stable declaration-order identity.

A data component is not modeled as a hidden mutable field plus a compiler-generated getter.

Conceptually, component identity is owner plus declaration index:

```text
DataComponentId {
    owner: DeclarationId,
    index: u32,
}
```

Names, labels, type annotations, source spans, and visibility metadata are attributes of that identity rather than components of the identity itself.

An `impl` block introduced later may add behavior. It may never add data storage or hidden semantic instance state.

### 4. Data is shallowly immutable and non-subclassable

Once a data value has been constructed, no component can be rebound.

This is shallow immutability. If a component refers to a mutable object, mutation through that referenced object remains governed by the referenced object's own semantics.

A data declaration does not participate in class inheritance and cannot be subclassed. This is required so that the complete state and cardinality of a data value are determined by the declaration and its type arguments rather than by an open-world subclass.

The absence of subclassing does not preclude later trait conformance or inherent behavior through `impl`.

### 5. The primary data constructor has semantic identity of its own

The primary product constructor is a first-class semantic constructor associated with the data declaration.

Conceptually:

```text
DataConstructorId {
    owner: DeclarationId,
}
```

It is not automatically an ordinary user-defined `new` method and must not be implemented by synthesizing such a method.

Tuple-shaped construction may reuse ordinary call syntax at the parser surface, but semantic resolution must identify a data constructor rather than fabricate a normal behavioral callable.

### 6. Generic data applications retain exact nominal type identity

Data uses Phalcom's existing generic binder, kind, constraint, substitution, and inference machinery.

No parallel data-specific generic calculus is introduced.

Applied types remain distinct according to the canonical type system even when they have the same physical layout:

```phalcom
data Id<Kind>(_ raw: Int)

Id<User>   // distinct semantic applied type
Id<Order>  // distinct semantic applied type
```

The implementation may share the same physical `{ i64 }` layout for both.

Phantom type parameters therefore remain part of exact semantic and reflective type identity even when they contribute zero payload bits.

### 7. A nullary data application is one singleton value per exact applied type

A nullary data declaration has product cardinality one for each exact applied type:

```phalcom
data Signal<State>()

Signal<Connected>()
Signal<Disconnected>()
```

`Signal<Connected>()` and `Signal<Disconnected>()` are distinct singleton values because their exact applied nominal types differ.

Repeated construction of the same exact nullary application denotes the same semantic singleton:

```phalcom
Signal<Connected>() === Signal<Connected>()
```

No heap allocation is required for an ordinary materialized nullary data value. The physical encoding may be an immediate VM-local exact-type/descriptor identity.

This ruling applies to `data`. It does not silently change the existing erased/special-case semantics of `Option::None` or other enum variants; enum specialization behavior remains governed by the enum/GADT model and later explicit rulings.

### 8. Data has value semantics and no observable allocation identity

A data value is a semantic value, not an allocation.

The implementation may, without observable change:

- keep all components only in compiler temporaries;
- scalar-replace the aggregate;
- copy native-width components;
- share an immutable backing allocation;
- stack-allocate a temporary representation;
- inline the value in a future collection representation;
- flatten nested fixed products;
- materialize a compact heap box only when required.

The following must not become part of ordinary data semantics:

- stable backing addresses;
- backing `ObjRef` identity;
- weak references to the backing box;
- per-allocation finalizers;
- mutable dynamic instance attributes;
- class reassignment;
- reflection of current boxing/flattening/offset decisions.

A physical box is an implementation artifact.

### 9. Exact type reification is required; per-value generic argument arrays are forbidden

At every language-defined reflection/type-observation boundary, the runtime must be able to recover the data value's exact semantic type, including applied type arguments and phantom parameters.

This requirement does **not** require every optimized temporary to carry a full runtime type descriptor.

The implementation may recover exact type information from the narrowest valid source:

```text
compile-time exact type knowledge
generic invocation/type environment
immediate singleton type key
materialized data descriptor
```

When a value must cross into a universal/dynamic representation, its conservative representation must preserve or reference enough interned descriptor information to recover the exact type.

Per-instance arrays such as:

```text
generic_arguments = [T1, T2, ...]
```

are forbidden as the default representation.

Likewise, applying a generic data declaration does not create a separate runtime `ClassObject`/metaclass tower for every application.

### 10. Semantic type identity, behavior identity, and physical layout identity are distinct

For a declaration such as:

```phalcom
data Point<T>(x: T, y: T)
```

the implementation must distinguish:

```text
Point declaration / behavior identity
Point<Int> exact semantic type identity
Point<Int> physical product layout
```

The declaration-level behavior identity may be shared by all applications.

Different exact applied types may share a physical layout.

One exact applied type may eventually have more than one temporary physical representation as optimization requires.

No one of these identities may be substituted for the others merely because a particular implementation revision happens to map them one-to-one.

### 11. The shared physical substrate is a product-layout abstraction

The lower-level runtime/compiler representation is not data-specific.

LANG005.C1 establishes a reusable product layout/storage abstraction capable of representing:

- named data payloads;
- enum variant payloads;
- and, in a later LANG005.C1 plan, statically closed anonymous tuple/record products.

The semantic categories do not collapse when they share a physical layout.

A product layout is responsible for physical facts such as:

- component physical representations;
- component locations;
- payload size and alignment or equivalent storage units;
- exact GC/reference tracing information;
- immediate eligibility;
- materialized storage construction/projection;
- recursive-layout indirection boundaries;
- a physical-layout fingerprint/key.

Logical names and labels live in shared declaration/shape metadata and must not be duplicated in each named data value.

### 12. P1 may use a conservative universal slot when representation proof is unavailable

Failure to prove a specialized representation is not a semantic error.

A component whose runtime representation is not safely known may use the universal `Value` representation.

Known components remain eligible for native-width representation even when another component is generic or dynamic.

For example, the architecture must admit a representation morally equivalent to:

```text
data Event {
    sequence: Int
    payload: Dynamic
    valid: Bool
}

physical slots:
    i64
    Value
    bool
```

rather than forcing every component to become `Value`.

More aggressive scalar replacement, interprocedural unboxing, nested flattening, and runtime-generic specialization are optimization stages built on this representation and do not redefine `data`.

### 13. Enum payloads may share the product substrate without sharing semantics

LANG005.C1 will migrate general enum constructor payload storage from an unconditional `Box<[Value]>` to the same product-layout/storage substrate used by data.

The following existing enum distinctions remain authoritative:

```text
VariantId            semantic case identity
RuntimeVariantId     VM-local case descriptor identity
CaseDiscriminant     physical sum discriminant
```

Enum roots, exact-case types, GADT result specialization, variant-local generics, pattern refinement, case behavior classes, per-variant behavior, and closed case contracts must not be weakened by the representation migration.

Nullary enum variants retain zero-allocation representation where already valid.

This PDR does not change enum source syntax; the variants-only enum syntax and `impl` migration belong to LANG005.C2.

### 14. `===`, `==`, and `hash` must not reveal backing representation

The current runtime's low-level `Value::same_as` compares representation bits/heap handles. That helper may remain useful for runtime-internal representation identity, but it is not sufficient as the language-level exact relation for data.

For data values, language-level `===` is representation-independent:

```text
same exact reified data type
AND
corresponding components are recursively ===
```

For nullary data, exact applied type identity is sufficient.

Ordinary `==` and `hash` must likewise not change merely because one execution boxes a value and another scalar-replaces or shares it. LANG005.C1 must provide representation-independent baseline value behavior sufficient to preserve this law.

The later derived-behavior program may decide which methods are synthesized, customizable, or trait-governed. That later policy may not reintroduce backing allocation identity.

Opaque class instances retain their existing identity/value behavior according to the class object model.

### 15. `data` is not the legacy `@data` decorator

The existing `@data class` facility is a compatibility mechanism implemented in `phalcom-core/src/compiler/attributes.rs` by `DataExpander`/`derive_data`. It derives an ordinary class constructor, getters, equality, hash, string conversion, and update helpers over class fields.

First-class `data` must not parse or lower through that expander.

The legacy mechanism remains until the LANG005 migration checkpoint explicitly removes or retires it.

### 16. Physical layout is not a stable FFI ABI

Ordinary data layout is compiler/runtime-controlled and may change across compiler versions, optimization modes, or applications.

This decision intentionally precludes treating ordinary `data` as a stable C-compatible structure layout.

A future explicit representation/ABI annotation may define such a contract, but it must do so as a separate ruling.

## Consequences

The immediate benefit is that Phalcom can give named immutable products a strong semantic identity without forcing them into the object representation used by classes.

It also makes the optimization story compositional:

```text
semantic data value
        ↓
product layout
        ↓
virtual / immediate / compact boxed representation
```

while exact semantic type information remains available through compiler/runtime metadata.

Phantom types become physically free in the normal case. Nullary generic data can be zero-allocation while retaining distinct specialized singleton identity. Primitive-heavy products can use native-width storage instead of one 16-byte `Value` per component. Later compiler passes can eliminate even that materialized storage.

The design also gives enum payloads and future optimized anonymous products one physical layout language without conflating their semantic type identities.

**The cost, named plainly:** this decision requires a real layout/storage subsystem, precise GC metadata for non-`Value` payloads, a descriptor/registry seam for exact data type identity, representation-independent exact/equality/hash behavior, additional bytecode/lowering integration, and careful reification across generic/dynamic boundaries. It is substantially more engineering than desugaring `data` into an immutable class.

The decision also closes several future shortcuts:

- `impl` cannot add fields to `data`;
- data cannot later become subclassable without revisiting this PDR;
- ordinary reflection cannot promise stable offsets or backing identity;
- the runtime cannot make generic specialization identity depend solely on a per-declaration class object;
- a compiler cannot permanently erase phantom/applied type information when the value crosses a reflection boundary;
- ordinary data layout cannot be treated as stable FFI ABI by accident.

## Compatibility and migration

Existing class declarations are unchanged.

Existing anonymous tuple and record semantics are unchanged by the first LANG005.C1 implementation plan; their representation convergence is a later C1 plan.

Existing enum source syntax and behavior are unchanged by the first C1 plan even when their payload storage migrates.

Existing `@data class` programs continue through the legacy attribute expander until LANG005's explicit migration checkpoint.

`TypeData::Nominal`/`Applied` remain the canonical semantic representation of named data types. No competing `TypeData::Data` node is introduced.

## Required implementation obligations

An implementation conforming to this record must, before claiming LANG005.C1 data support shipped:

1. introduce a real `Data` declaration kind through AST, module binding, semantic identity, query publication, source indexing, compiler lowering, and runtime declaration identity;
2. give components stable owner/index semantic identities and publish their declared type facts;
3. reuse existing generic signatures, constraints, kinding, substitution, and inference;
4. prohibit component mutation and class-style allocation/subclassing;
5. introduce one reusable product layout/storage authority rather than separate data and enum storage systems;
6. avoid per-instance labels and generic-argument arrays for named data;
7. provide zero-allocation ordinary materialization of exact nullary data applications;
8. preserve exact applied data type identity for reification boundaries;
9. make data `===`, `==`, and `hash` independent of backing allocation identity;
10. keep `Value`'s hot representation compact and avoid inflating every heap arena slot with the largest product;
11. give product storage precise GC tracing;
12. migrate general enum payload storage to the shared product subsystem without changing enum/GADT semantic authority;
13. keep the legacy `@data` expander out of the new declaration path;
14. provide focused semantic/runtime/GC/incremental tests and performance/allocation evidence.

## Repository evidence for the ruling

The following current files were inspected at the revision named in the header:

- `phalcom-core/src/value/repr.rs` — 16-byte `Value`, `AdtSingleton`, reserved metadata bits, representation-level equality.
- `phalcom-core/src/value/mod.rs` — `Value::class` and representation-level `same_as`.
- `phalcom-core/src/heap/instance.rs` — class instance `ClassId + Box<[Value]>`, Nil initialization.
- `phalcom-core/src/heap/tuple.rs` — immutable `Box<[Value]>` with per-tuple label suffix.
- `phalcom-core/src/heap/record.rs` — immutable labeled `Box<[Value]>`.
- `phalcom-core/src/heap/adt.rs` — `AdtCaseObject` payload as `Box<[Value]>`.
- `phalcom-core/src/heap/object.rs` — boxed large heap variants and current ADT/typing arms.
- `phalcom-core/src/heap/trace.rs` — exhaustive precise tracing discipline.
- `phalcom-core/src/product.rs` — existing internal anonymous-product construction boundary.
- `phalcom-core/src/adt.rs` — separate semantic/runtime/discriminant enum identities.
- `phalcom-core/src/typing/reify.rs` — nominal class-object reuse and lazy synthetic type descriptors.
- `phalcom-semantic/src/types/store.rs` — `Nominal`, `Applied`, `ExactCase`, `Tuple`, and `Record` canonical forms.
- `phalcom-semantic/src/identity.rs` — stable declaration/callable/variant/field identities.
- `phalcom-semantic/src/enum_semantics.rs` — enum/variant/field/constructor semantic products.
- `phalcom-core/src/compiler/attributes.rs` — legacy `DataExpander`/`derive_data`.
- `phalcom-ast/src/ast.rs` — no first-class `Data` declaration currently exists.

## Alternatives rejected

### Desugar `data` to `@data class`

Rejected because it would make ordinary class storage, allocation identity, field layout, constructor lifecycle, inheritance assumptions, and class-side allocation semantics the initial authority for a feature whose defining semantics are value-oriented.

It would also make later unboxing/scalar replacement an attempt to prove away observable object semantics rather than an ordinary representation choice.

### Represent every materialized data value as `ClassId + Box<[Value]>`

Rejected because it duplicates class identity per value, pays 16 bytes per component regardless of proven representation, requires class-style field machinery, and encourages Nil-initialize-then-write construction.

### Reuse `AdtCaseObject { variant, Box<[Value]> }` unchanged

Rejected because data has no sum discriminant and because unconditional universal `Value` payload storage defeats the representation goal. The reusable abstraction is the product layout/storage layer below both data and enum cases.

### Give immutable data ordinary object identity but promise not to use it

Rejected because a promise is insufficient once `===`, hashing, weak references, reflection, or future APIs can observe the backing object. Allocation identity must be absent from the semantic contract.

### Create a runtime class/metaclass for every applied data type

Rejected because Phalcom's metadata architecture already separates generic applications from nominal class rows. It would duplicate method tables, class-side state, hierarchy data, and reflection structures while preventing physically identical phantom specializations from sharing layouts.

### Erase phantom generic arguments from runtime identity

Rejected because generic specialization is part of the semantic type. `Id<User>` and `Id<Order>` must remain distinguishable even when the parameter does not occur in the payload.

### Expose the chosen product layout through ordinary reflection

Rejected because layout observability would forbid future scalar replacement, flattening, packing, inline collection storage, and architecture-dependent representation choices.

### Defer the representation ruling until after adding syntax

Rejected because the first implementation would necessarily pick an accidental authority—class instances, ADT boxes, or another permanent heap shape. Once value identity and reflection are built on that authority, later optimization becomes a semantic migration rather than an implementation improvement.

## Landing maintenance

This artifact is prepared as the next available PDR number at the inspected revision: `PDR-0034` exists and no `PDR-0035` was found.

When this record is added to the repository, `docs/pdr/STATUS.md` must be updated in the **same commit**, per `docs/pdr/README.md`.

Suggested tracker row:

```markdown
| [0035](0035-data-is-a-nominal-transparent-immutable-value-product.md) | `data` is a nominal transparent immutable value product | Accepted (ratified 2026-09-12) | | | ❌ ratified; LANG005.C1 unimplemented |
```

The shipped column must remain unshipped until implementation evidence actually exists.

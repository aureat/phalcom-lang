# LANG005.C1.P3 — Anonymous Product Convergence, Value-Semantic Optimization, and Runtime Reification Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete LANG005.C1 by giving anonymous Tuple and Record values the same identity-free representation freedom and scalar-replacement eligibility as first-class `data`, converging all immutable product materialization on the shared `ProductLayout`/`ProductStorage` substrate, and closing the Dynamic/runtime-generic/reification paths so every required type-observation boundary remains exact without per-value generic argument arrays or per-specialization classes.

**Architecture:** P1 remains the canonical semantic/materialized baseline for `data` and enum payloads; P2 remains the compiler-local virtualization/scalar-replacement framework. P3 extends the shared product substrate to Tuple/Record, makes language-level exactness for Tuple/Record representation-independent, generalizes the P2 virtual-product model to all identity-free products, and introduces the minimum runtime generic-type environment needed to instantiate exact runtime type recipes at materialization/reification boundaries. Semantic type identity, product presentation shape, physical layout, runtime behavior class, and backing allocation remain separate identities throughout.

**Tech Stack:** Rust 2024 workspace; `phalcom-ast`; `phalcom-modules`; `phalcom-semantic`; `phalcom-type-meta`; `phalcom-core` compiler/bytecode/VM/GC; `phalcom-lsp`; P1 `ProductLayout`/`ProductStorage`/data descriptors; P2 `ProductFunctionPlan`/virtual leaf slots; existing runtime typing metadata, overlay, weak reification cache, source semantic index, and incremental semantic products.

**Spec:** PDR-0035 (`data` is a nominal transparent immutable value product), the ratified Tuple/Record specifications under `docs/spec/collections-next/` and `docs/spec/typing/`, the user-ratified P3 ruling that Tuple and Record have the same optimization freedom as `data`, and the new structural-product value-semantics decision landed by Task 1 of this plan.

---

# 1. Repository grounding

Prepared against the current remote repository state inspected for this plan:

```text
repository: aureat/phalcom-lang
branch: main
remote HEAD: 7b5046e5ab5db195128788cf938d7f10ed3d4792
HEAD message: test: promote collection and module coverage
```

The single commit after the P2 planning baseline (`b38adeb1... -> 7b5046e5...`) promotes collection/module test coverage and does not change the product representation architecture. Re-read HEAD before execution because this repository moves quickly.

Current baseline facts relevant to P3:

1. `TupleObject` is a positive-arity native immutable object with per-instance `values: Box<[Value]>` and `labels: Box<[Symbol]>`.
2. `RecordObject` is a positive-arity native immutable object with per-instance `labels: Box<[Symbol]>` and `values: Box<[Value]>`.
3. `finish_tuple` and `finish_record` are the canonical dynamic product finalizers and normalize zero arity to `Unit`.
4. Static literals lower through `Bytecode::BuildTuple { positional, labeled }` and `Bytecode::BuildRecord { fields }`.
5. Tuple and Record already have structural `==`/`hash` behavior in the core library; Tuple equality is ordered/label-sensitive and Record equality/hash are order-insensitive.
6. `Value::same_as` is currently representation identity: immediate bits or heap handle identity. `Bytecode::Same` and `Object#===` use it directly.
7. Structural semantic types are already first-class: `TypeNode::Tuple`, `TypeNode::Record`, runtime overlay Tuple/Record forms, canonical record rows, and runtime synthetic type descriptors already exist.
8. Record semantic type identity is canonical/order-independent, while record encounter order remains observable for iteration/presentation/reflection.
9. `CallFrame` is `Copy` and currently carries no runtime generic substitution environment.
10. `BlockObject` already captures activation-specific metadata (closure + home-frame token), providing a natural place to retain a compact generic-environment identity for escaping lexical blocks if needed.
11. Runtime typing metadata explicitly rejects per-instance generic tokens and specialized runtime classes/metaclasses.
12. P1 is planned to add `ProductLayout`, `ProductStorage`, `ProductLayoutRegistry`, `RuntimeDataDescriptor`, `DataObject`, `DataSingleton`, and shared enum payload storage.
13. P2 is planned to add `ProductOptimizationMode`, `ProductFunctionPlan`, virtual leaf slots, nested-data virtualization, exact enum-match virtualization, and Enabled/Disabled differential testing.

P3 is therefore a convergence/hardening checkpoint, not a second product system and not a general IR rewrite.

---

# 2. Global constraints

Every task in this plan inherits these requirements.

1. **Tuple, Record, and data are identity-free transparent values.** Their backing `ObjRef`, allocation address, allocation count, or chosen physical representation is not language-visible identity.
2. **Semantic categories remain distinct.** Structural Tuple, structural Record, nominal `data`, enum payload, and opaque class instance never collapse semantically merely because they share storage machinery.
3. **Language-level `===` is representation-independent for transparent products.** `Value::same_as` remains an internal representation-identity helper and must not become the language-level authority for Tuple/Record/data.
4. **Tuple exactness:** two positive Tuple values are `===` iff their tuple shape is identical—same arity, positional/labeled lane boundary, ordered labels—and corresponding components are recursively `===`.
5. **Record exactness:** two positive Record values are `===` iff they have the same key set and the corresponding values are recursively `===`; encounter/presentation order is ignored by `===` just as it is ignored by Record semantic type identity and `==`.
6. **Unit remains the canonical zero product.** `()` and `#{}` normalize to the single `Unit` value; no heap-allocated empty Tuple or Record may exist through any path.
7. **`==` and `hash` keep their existing public semantic distinction from `===`.** Tuple/Record `==` continues to recurse through ordinary `==`; hash continues to agree with `==`. P3 does not silently replace those APIs with exact comparison.
8. **Record presentation order remains observable.** Construction/iteration/printing/reflection/expansion order must survive packing, scalar replacement, rematerialization, and layout sharing.
9. **Record presentation shape is not record type identity.** Two records with the same semantic field set but different encounter order may share one exact structural type and one physical layout while retaining different presentation-shape descriptors.
10. **Labels never live in `ProductLayout`.** Layout contains only physical storage/tracing facts. Tuple/Record labels and presentation mappings live in shared shape metadata, never per-instance arrays after P3.
11. **No per-value generic argument arrays.** Runtime generic evidence lives in compile-time facts, compact invocation/type environments, runtime descriptor IDs, or immediate singleton IDs.
12. **No applied-type ClassObjects.** `.class` remains declaration/builtin behavior identity (`Point`, `Tuple`, `Record`); exact applied/structural type forms remain runtime typing descriptors.
13. **Source evaluation order is sacred.** Logical component order, semantic record-field canonical order, presentation order, and physical storage order may differ; none may reorder source effects.
14. **Canonical `BindingId` owns optimizer binding identity.** P3 must remove compiler-local `(name, range)` binding identity if P2 landed it.
15. **Optimization failure means canonical materialization, never weaker semantics.** Unknown, blocked, Dynamic, computed-label, spread, alias, capture, or unsupported use falls back.
16. **Explicit `Dynamic` is not “compiler could not prove.”** `Dynamic` remains an epistemic/runtime boundary already represented by semantic analysis.
17. **The disabled optimizer remains the executable oracle.** Enabled and Disabled compilation must have identical source diagnostics and observable behavior.
18. **Ordinary dispatch semantics remain authoritative.** Identity-free products may be scalar-replaced, but P3 must not bypass an ordinary method/index/getter send whose dynamic method-table semantics are not already protected by existing compiler guards.
19. **No new optimizer-owned semantic DB.** P3 may project existing semantic identities/facts into compiler lowering products; it must not create a competing type/member/binding solver.
20. **No unchecked raw representation tricks.** Continue P1's checked word packing and `Value` raw-word encapsulation; no unchecked transmute/pointer alias representation.
21. **GC remains precise.** Every materialized product traces exactly the slots whose physical representations can contain GC edges.
22. **Runtime type recipes must fail closed.** Missing runtime generic evidence must never silently become `Dynamic`, raw declaration identity, or an erased generic specialization.
23. **P3 may optimize observation without materialization.** `.class`, exact type reification, and `===` may be answered directly from virtual-product metadata where the result is statically/runtime-provably identical to materializing first.
24. **No stable FFI ABI is created.** Product packing, flattening, descriptor identity, and layout sharing remain implementation details.

---

# 3. The semantic ruling P3 implements

P3 ratifies the anonymous structural counterpart of PDR-0035:

```text
Tuple and Record are transparent immutable structural value products.

They have semantic value identity, not allocation identity.

An implementation may:
    scalar-replace them
    flatten nested products
    keep components in ordinary VM locals
    stack/register allocate them
    pack primitive-width components
    share immutable materialized boxes
    rematerialize on demand
    canonicalize representation metadata

provided that:
    exact/value relations are preserved
    Tuple lane/label semantics are preserved
    Record key semantics are preserved
    Record presentation order is preserved where observable
    runtime type/reflection boundaries remain truthful
```

The language-level exact relation becomes:

```text
Unit:
    the sole Unit value is exact to itself

Tuple:
    same tuple structural shape
    AND recursive component ===

Record:
    same key set, independent of presentation order
    AND per-key recursive value ===

data:
    same exact reified nominal data type
    AND recursive component ===
```

This is the semantic fact that unlocks the P2-style optimizer for anonymous products. A rematerialized `(1, 2)` must not become observably “different” merely because it has a new heap handle.

---

# 4. Identity and representation model

The implementation must keep the following identities separate:

```text
Tuple / Record semantic TypeId / RuntimeTypeRef
    structural type identity

ProductShapeId
    runtime product coordinate/presentation shape

ProductLayoutId
    physical word layout and GC map

RuntimeAnonymousProductDescriptorId
    VM-local materialized Tuple/Record descriptor tying shape + layout
    + optional exact retained RuntimeTypeRef

ClassId
    Tuple or Record behavior class for ordinary dispatch/.class

ObjRef
    backing allocation handle only; not semantic identity
```

For records specifically:

```text
source/presentation order
        |
        v
RecordProductShape.presentation_labels
        |
        +--> presentation_to_logical[]
        |
        v
canonical logical field coordinates
        |
        v
ProductLayout logical indexes
```

A statically exact record should normally store physical components in canonical semantic field order so values with different source order can share a layout. The shape descriptor preserves encounter order and maps it to those logical coordinates.

---

# 5. Required runtime/compiler data model

Names may be adjusted mechanically to repository conventions after drift inspection, but the information boundaries below are required.

## 5.1 Shared anonymous-product shape metadata

Add shared VM-owned shape metadata; do not put it in the GC heap and do not duplicate it per value.

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductShapeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AnonymousProductKind {
    Tuple,
    Record,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TupleProductShape {
    pub positional_len: u32,
    pub labels: Box<[Symbol]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordProductShape {
    /// Observable encounter/presentation order.
    pub presentation_labels: Box<[Symbol]>,
    /// Canonical logical/storage coordinate labels.
    pub logical_labels: Box<[Symbol]>,
    /// presentation index -> logical/storage index.
    pub presentation_to_logical: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ProductShape {
    Tuple(TupleProductShape),
    Record(RecordProductShape),
}
```

The runtime registry may build a non-key lookup cache beside an interned `RecordProductShape` for O(1)/O(log n) label-to-logical-index lookup. Such a cache is shared metadata, not semantic identity and not part of deterministic hashing.

## 5.2 Anonymous materialization descriptor

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeAnonymousProductDescriptorId(pub u32);

pub struct RuntimeAnonymousProductDescriptor {
    pub runtime_id: RuntimeAnonymousProductDescriptorId,
    pub kind: AnonymousProductKind,
    pub shape: ProductShapeId,
    pub layout: ProductLayoutId,
    /// Present only when semantic metadata establishes a retained exact
    /// structural type; never fabricated from runtime payload values.
    pub exact_type: Option<RuntimeTypeRef>,
}
```

The descriptor registry is VM-owned metadata. Multiple descriptors may share one `ProductLayoutId`. Record descriptors with different presentation order may share the same `exact_type` and `ProductLayoutId` while using different `ProductShapeId`s.

## 5.3 Tuple/Record heap payloads after convergence

Retain distinct `Object::Tuple` and `Object::Record` variants to preserve clear runtime category boundaries and reduce migration fanout.

```rust
pub struct TupleObject {
    pub descriptor: RuntimeAnonymousProductDescriptorId,
    pub storage: ProductStorage,
}

pub struct RecordObject {
    pub descriptor: RuntimeAnonymousProductDescriptorId,
    pub storage: ProductStorage,
}
```

After P3 these structs must not contain `Box<[Symbol]>` or `Box<[Value]>`.

## 5.4 Compiler lowering spec for statically closed products

```rust
pub struct AnonymousProductConstructionLoweringSpec {
    pub kind: AnonymousProductKind,
    pub shape: AnonymousProductShapeSpec,
    pub type_recipe: Option<RuntimeTypeRecipe>,
    pub layout: ProductLayoutSpec,
    /// source evaluation ordinal -> logical/storage component index
    pub source_to_logical: Box<[u32]>,
}
```

Static Tuple/Record literals with fixed labels and known component semantic facts use this path. Computed labels, dynamic spreads, or otherwise unresolved shapes use the dynamic finalizer and a universal-`Value` layout.

## 5.5 Runtime type recipe/environment

P1's closed `exact_type: RuntimeTypeRef` field is insufficient for shared generic bytecode such as `Point<T>` when `T` is only known from the current invocation. P3 therefore introduces a reusable type recipe and compact environment.

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeTypeRecipe {
    Closed(RuntimeTypeRef),
    Template(RuntimeTypeRef),
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeTypeEnvironmentId(pub u32);

impl RuntimeTypeEnvironmentId {
    pub const EMPTY: Self = Self(0);
}

pub struct RuntimeTypeEnvironment {
    pub bindings: Box<[(StableTypeParameterRef, RuntimeTypeRef)]>,
}
```

`Template(handle)` means the retained metadata type graph may contain stable type-parameter references. Runtime instantiation substitutes through the current environment and creates/reuses overlay forms only on demand. It does not mutate compiler metadata.

`CallFrame` carries one compact `RuntimeTypeEnvironmentId`; lexical `BlockObject`s that may outlive their creating activation capture the current environment identity. No product value contains a generic-argument vector.

## 5.6 P2 optimizer model after P3

P3 replaces range/name binding identity and data-only nesting with canonical/generic forms:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProductBindingKey {
    pub callable: CallableId,
    pub binding: BindingId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VirtualProductKind {
    Data {
        constructor_site: LoweringSite,
    },
    Tuple {
        construction_site: LoweringSite,
    },
    Record {
        construction_site: LoweringSite,
    },
    Variant {
        variant: VariantId,
        constructor_site: LoweringSite,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VirtualComponentPlan {
    Scalar {
        logical_component: u32,
        leaf_offset: u16,
    },
    NestedProduct {
        logical_component: u32,
        leaf_offset: u16,
        shape: Box<VirtualShapePlan>,
    },
}
```

`VirtualShapePlan` must also carry enough product-kind-specific metadata to rematerialize Tuple/Record shape/presentation exactly; do not attempt to recover record presentation order from canonical `TypeId`.

---

# 6. Materialization boundary policy after P3

The post-P3 floor is:

| Use of a virtual transparent product | `data` | Tuple | Record | exact enum case |
|---|---|---|---|---|
| direct semantic component/pattern projection | virtual | virtual when structurally proven | virtual when structurally proven | payload projection may be virtual |
| nested transparent product construction | recursively virtual | recursively virtual | recursively virtual | enum remains boundary except existing exact-case rule |
| `.class` | return known behavior class without materializing | return Tuple class | return Record class | existing enum policy |
| exact retained type observation | reify type recipe without materializing | reify retained structural recipe when present | same | existing enum policy |
| `===` between two compatible virtual transparent values | compare virtual shape/type + leaves directly | compare shape + leaves | compare key set + leaves | keep P2 enum identity controls |
| ordinary `==` / `hash` / `toString` send | materialize unless an existing guarded optimizer proves safe | same | same | materialize |
| ordinary method/index/getter send | materialize unless existing dispatch guard proves equivalent | same | same | materialize |
| call/send argument | materialize at boundary | materialize | materialize | canonical materialized enum |
| return/non-local return | materialize at boundary | materialize | materialize | canonical materialized enum |
| store to global/field/index/collection | materialize | materialize | materialize | canonical materialized enum |
| closure capture of product value | P3 still falls back unless existing P2 capture support exists | same | same | same |
| mutable/reassigned local | fallback | fallback | fallback | fallback |
| computed label / opaque expansion | fallback | fallback | fallback | fallback |
| Dynamic/universal boundary | materialize exact truthful descriptor | materialize truthful anonymous descriptor | same | existing enum path |

Identity no longer forces Tuple/Record materialization. Dynamic dispatch and unsupported compiler proofs still may.

---

# 7. Checkpoint map

```text
C0  Semantic ruling + C1 takeover repairs
 |
 |-- T1 structural product PDR/spec exactness ruling
 |-- T2 repair P1 evaluation-order + non-subclassability gaps
 |-- T3 migrate P2 binding identity to BindingId
 `-- T4 close zero-product bypasses and establish baseline

C1  Anonymous materialized product convergence
 |
 |-- T5 ProductShape/anonymous descriptor registries
 |-- T6 structural product lowering specs
 |-- T7 TupleObject/RecordObject -> ProductStorage
 |-- T8 dynamic/static finalizer convergence
 `-- T9 static product bytecode/lowering integration

C2  Runtime semantics and consumer migration
 |
 |-- T10 representation-independent language ===
 |-- T11 representation-neutral Tuple/Record view API + primitives/render
 |-- T12 pack/expansion/rest/block consumer migration
 |-- T13 pattern/destructuring migration
 `-- T14 Record order/equality/hash/reflection/map-interoperability proof

C3  P2 optimizer generalization to anonymous products
 |
 |-- T15 generalize optimizer kind/shape/use model
 |-- T16 identify/plan Tuple/Record candidates and safe projections
 |-- T17 virtual construction/projection/rematerialization
 |-- T18 recursive mixed transparent-product flattening
 |-- T19 virtual .class/type/=== observations
 `-- T20 Enabled/Disabled/GC/fiber/performance proof

C4  Runtime generic/Dynamic/reification closure
 |
 |-- T21 runtime type recipe/environment registry
 |-- T22 invocation/frame/block environment propagation
 |-- T23 instantiate exact product descriptors from runtime type recipes
 |-- T24 Dynamic + phantom/nullary + anonymous structural reification matrix
 `-- T25 closure/fiber/reflection lifetime and fail-closed proof

C5  LANG005.C1 delivery closure
 |
 |-- T26 incremental/artifact/LSP/tooling audit
 |-- T27 representation/performance qualification
 |-- T28 negative/deletion gates and broad repository verification
 `-- T29 implementation-state/PDR delivery closure
```

Each checkpoint is an evidence boundary. Do not mark a checkpoint complete merely because its code exists.

---

# Checkpoint C0 — Semantic ruling and C1 takeover repairs

## Entry conditions

- P1 and P2 implementation state exists, or the implementer can identify their final landed interfaces.
- P1 Disabled/canonical product execution is green.
- P2 Enabled/Disabled data differential suite is green.
- Re-read current HEAD and record drift from `7b5046e5...`.

## Completion claim

Before representation convergence begins, C1 has one coherent language ruling, source effects are ordered correctly, data cannot enter class inheritance, optimizer bindings use canonical semantic identity, and zero products have no bypass allocation path.

### Task 1 — Land anonymous structural products as identity-free transparent values

**Files:**
- Create: `docs/pdr/0036-tuple-and-record-are-transparent-structural-value-products.md`
- Modify: `docs/spec/collections-next/tuple-record-and-symbols-spec.md`
- Modify: `docs/spec/typing/phalcom-tuples-records-sets-maps-spec.md`
- Modify: `docs/implementation/LANG003-language-semantics/C1-language-semantics/language-semantics-spec.md`
- Modify: P1/P2/P3 implementation-state documentation as appropriate.

**Consumes:** PDR-0035's distinction between semantic value identity and allocation identity.

**Produces:** the normative reason Tuple/Record scalar replacement/rematerialization is legal.

**Required decisions to record:**

```text
Tuple/Record backing allocation identity is unobservable.
Tuple === uses shape + recursive ===.
Record === uses key set + per-key recursive ===; presentation order ignored.
Unit remains the sole zero product.
Tuple/Record == and hash keep their existing structural contracts.
Record presentation order remains observable for traversal/presentation.
.class remains Tuple/Record, never a structural specialized ClassObject.
Exact type descriptors remain semantic metadata, not physical layout.
Weak references/finalizers/address APIs may not expose a product backing box.
```

**Edit operations:**
1. [ ] Write PDR-0036 with Context, Decision, Consequences, Required implementation obligations, Alternatives rejected, and compatibility notes.
2. [ ] Amend Tuple spec exact-relation language; remove/replace any wording that makes positive Tuple `===` depend on heap allocation identity.
3. [ ] Amend Record spec exact-relation language with explicit order independence.
4. [ ] Preserve existing `==`/hash rules verbatim except where documentation must distinguish them from `===`.
5. [ ] Amend LANG003's general “`===` object identity” rule with the transparent-product exception rather than redefining opaque class identity.
6. [ ] Add examples proving independently materialized equal transparent products may be `===` even with different backing allocations.
7. [ ] Add counterexamples proving Tuple vs Record, named data vs structural product, and different data phantom specializations remain non-exact.

**Evidence gate:** documentation consistency search for `===` + Tuple/Record/data across current specs; no contradictory normative statement remains.

---

### Task 2 — Repair P1 constructor evaluation order and data non-subclassability

**Files:**
- Modify: final P1 data compiler/lowering module (planned `phalcom-core/src/compiler/lib/data_decl.rs` or landed successor).
- Modify: `phalcom-core/src/modules/semantic_lowering.rs`.
- Modify: `phalcom-semantic/src/session.rs` hierarchy formation/validation path.
- Modify: declaration-kind/hierarchy diagnostics as required.
- Test: P1 data semantic/compiler integration suites.

**Consumes:** P1 `DataConstructionLoweringSpec.argument_to_component`; `DeclarationKind::Data`.

**Produces:** source-order-safe constructor lowering and hard “data is non-subclassable” hierarchy validation.

**Required lowering law:**

```text
source argument evaluation ordinal
    -> evaluate exactly once in source order
    -> map to logical DataComponentId/index
    -> store/rematerialize in logical/physical order as needed
```

Never compile arguments in declaration order merely because storage is declaration-ordered.

**Required hierarchy law:**

```text
class superclass target = Data
    -> semantic diagnostic
    -> no HierarchyEdge published
```

**Edit operations:**
1. [ ] Add/rebase a regression `Pair(second: mark(2), first: mark(1))` proving side effects remain source ordered.
2. [ ] Add a throwing labeled-argument regression proving failure timing matches ordinary call evaluation.
3. [ ] Ensure the data construction emitter walks source arguments and consults `argument_to_component` only for destination placement.
4. [ ] Add an incoming hierarchy-target admission check for `DeclarationKind::Data`.
5. [ ] Add a negative semantic fixture where a class names data as superclass.
6. [ ] Prove rejected hierarchy publication is identical in cold and incremental analysis.

**Evidence gate:** focused P1 constructor and hierarchy tests; Enabled/Disabled P2 output remains behaviorally identical.

---

### Task 3 — Replace P2 range/name binding identity with canonical `BindingId`

**Files:**
- Modify: `phalcom-core/src/compiler/lib/product_opt.rs`.
- Modify: `phalcom-core/src/modules/semantic_lowering.rs` or the final compiler-facing semantic projection carrying source occurrences.
- Modify only if needed: `phalcom-semantic/src/source_index/{mod.rs,builder.rs,occurrence.rs}`.
- Test: P2 planner unit/integration tests.

**Consumes:** compiler-owned `SourceSemanticIndex`, `BindingId`, `CallableId`, exact source occurrence attachments.

**Produces:** `ProductBindingKey { callable, binding }` and occurrence-to-binding facts; no lexical name re-resolution in the optimizer.

**Required rule:** the AST visitor classifies *use kind*, but semantic source products decide *which binding an occurrence denotes*.

**Edit operations:**
1. [ ] Replace `ProductBindingSite { range, name }` with `ProductBindingKey` using canonical binding identity.
2. [ ] Project exact binding identity into compiler lowering once; do not make codegen query names/scopes independently.
3. [ ] Delete the P2 lexical environment whose purpose is resolving same-name references to declarations.
4. [ ] Retain source-order AST traversal solely for use classification, capture detection, and expression-shape inspection.
5. [ ] Add hostile tests for same-name shadowing, parameters, loop/pattern bindings, nested blocks, and rebased incremental ranges.
6. [ ] Add a negative test proving optimizer identity remains stable if source spelling/range presentation changes without changing canonical binding identity.

**Evidence gate:** all prior P2 planner tests plus new binding-identity tests; emitted chunks remain unchanged for the same optimization decisions.

---

### Task 4 — Close every zero-product allocation bypass

**Files:**
- Modify: `phalcom-core/src/primitive/typing.rs` (`empty_tuple` current bypass).
- Audit/modify: `phalcom-core/src/product.rs`.
- Audit: all `TupleObject::positional(Vec::new())`, direct empty `RecordObject`, and raw heap allocation call sites.
- Test: `phalcom-core/tests/core/collections/contract.rs` and typing/reflection tests.

**Consumes:** canonical `Value::unit()` and `finish_tuple`/`finish_record` zero normalization.

**Produces:** one invariant: no positive product heap variant can represent arity zero.

**Edit operations:**
1. [ ] Change typing/reflection `empty_tuple` to return `Value::unit()` or the canonical product finalizer result.
2. [ ] Search the entire repository for direct empty Tuple/Record allocations.
3. [ ] Replace every executable bypass with the canonical zero-product path.
4. [ ] Preserve AST/source distinction between written `()` and `#{}` only in syntax/semantic provenance; runtime remains Unit.
5. [ ] Add heap-live-count tests for every dynamic/reflection empty-product construction route.
6. [ ] Add a debug/assertion guard preventing `TupleObject`/`RecordObject` constructors from accepting zero arity after P3 migration.

**C0 checkpoint evidence:**

```sh
cargo test -p phalcom-semantic --test semantic data_
cargo test -p phalcom-core product
cargo test -p phalcom-core empty_tuple
cargo test -p phalcom-core data_
```

Also run P2 focused planner tests. Record exact commands/results in the LANG005 C1 state file.

---

# Checkpoint C1 — Anonymous materialized product convergence

## Completion claim

Every positive Tuple/Record materialization uses P1 `ProductStorage`; labels/presentation metadata are interned once per shared shape rather than stored per value; static exact products can use compact primitive-width layouts; dynamic products use the same storage machinery with conservative `Value` slots.

### Task 5 — Add ProductShape and anonymous-product descriptor registries

**Files:**
- Create: `phalcom-core/src/product/shape.rs`.
- Create: `phalcom-core/src/product/anonymous.rs`.
- Modify: `phalcom-core/src/product.rs` exports.
- Modify: `phalcom-core/src/vm/mod.rs` registry ownership/init.
- Test: product registry unit tests.

**Consumes:** P1 `ProductLayoutId`, `ProductLayoutRegistry`; VM Symbol interner; runtime typing handles.

**Produces:** `ProductShapeId`, `ProductShapeRegistry`, `RuntimeAnonymousProductDescriptorId`, descriptor registry.

**Edit operations:**
1. [ ] Implement deterministic Tuple shape validation: `labels.len() <= total_len`, labels unique, positional prefix count explicit.
2. [ ] Implement deterministic Record shape validation: presentation labels unique; logical labels unique/canonical; `presentation_to_logical` a complete permutation.
3. [ ] Intern shapes by semantic shape data only; exclude lookup caches from equality/hash identity.
4. [ ] Build shared label lookup caches after interning.
5. [ ] Intern anonymous descriptors by `(kind, shape, layout, exact_type)`; `exact_type` is optional and never inferred from runtime payloads.
6. [ ] Add tests proving two equal-layout Tuples with different labels have different shape IDs but may share layout.
7. [ ] Add tests proving two Records with the same field set/layout and different presentation order have different shape IDs but may share the same retained exact type.
8. [ ] Add tests proving descriptor identity never substitutes for structural semantic Type equality.

---

### Task 6 — Project static Tuple/Record literals into executable semantic lowering specs

**Files:**
- Modify: `phalcom-core/src/modules/semantic_lowering.rs`.
- Modify: compiler executable semantic-pool/chunk types used by P1.
- Read/possibly modify: `phalcom-semantic/src/checker/expression.rs` only to expose already-established expression/type facts, not to add optimizer semantics.
- Test: semantic-lowering projection tests.

**Consumes:** `Expr::TupleLiteral`, `Expr::RecordLiteral`; published expression `TypeKnowledge`; canonical `TypeData::Tuple`/record row; runtime metadata exporter; P1 `ProductLayoutSpec`.

**Produces:** `AnonymousProductConstructionLoweringSpec` at statically closed literal sites.

**Static eligibility:**

Tuple static spec requires:
- fixed positional/labeled entries;
- no computed label;
- no spread/pack expansion affecting shape;
- component semantic knowledge sufficient to choose each physical slot or conservative `Value`;
- retained Tuple structural type recipe when exportable.

Record static spec requires:
- fixed direct labels;
- no computed key/spread affecting shape;
- canonical record semantic row available when exact/closed;
- source presentation labels preserved separately from canonical logical field coordinates.

**Edit operations:**
1. [ ] Add Tuple/Record construction lowering spec variants to `ModuleLoweringSemantics`/executable semantic pool.
2. [ ] Derive `ProductSlotRepr` from semantic component knowledge using exactly P1's Int/Float/Bool/Symbol/Value policy.
3. [ ] For Tuple, preserve source order as logical order and encode the positional/labeled boundary in shape spec.
4. [ ] For Record, derive canonical logical field order from the semantic closed record type/row, not source order.
5. [ ] Build `source_to_logical` for Record construction so source effects and canonical storage can differ safely.
6. [ ] Preserve source presentation labels in shape spec.
7. [ ] Export a `RuntimeTypeRecipe` when the structural type form is retained/exportable; do not fabricate one for broad/dynamic record knowledge.
8. [ ] Route computed-label/spread/broad cases to the dynamic builder path rather than guessing a static shape.
9. [ ] Add deterministic tests for primitive-packed Tuple, labeled Tuple, reordered Record, Dynamic component fallback, generic structural component recipe, and broad-record fallback.

---

### Task 7 — Migrate TupleObject and RecordObject to ProductStorage

**Files:**
- Modify: `phalcom-core/src/heap/tuple.rs`.
- Modify: `phalcom-core/src/heap/record.rs`.
- Modify: `phalcom-core/src/heap/{object.rs,mod.rs,accessors.rs,trace.rs}`.
- Modify: memory-management representation documentation if repository policy requires.
- Test: heap/product unit tests.

**Consumes:** anonymous descriptor registry + P1 `ProductStorage`.

**Produces:** compact Tuple/Record heap payloads with no per-instance labels or `Value` arrays.

**Edit operations:**
1. [ ] Replace Tuple fields with `{ descriptor, storage }`.
2. [ ] Replace Record fields with `{ descriptor, storage }`.
3. [ ] Keep `Object::Tuple` and `Object::Record` distinct variants.
4. [ ] Remove object-local `get`, `get_label`, `labels`, `entries` helpers that assume labels/values live inside the object; replace with registry-aware view helpers in Task 11.
5. [ ] Ensure `storage.layout` agrees with descriptor layout at construction and assert this invariant.
6. [ ] Update heap constructors/accessors.
7. [ ] Update exhaustive GC tracing to use `ProductStorage.layout` through the Heap-owned layout registry exactly as Data/enum storage does.
8. [ ] Add GC test with a heap child reachable only through a Tuple `Value` slot and separately through a Record `Value` slot.
9. [ ] Add deterministic size/word-count tests for `(Int, Int)`, labeled Tuple, and fixed Record.

---

### Task 8 — Converge static and dynamic product finalizers

**Files:**
- Modify: `phalcom-core/src/product.rs`.
- Modify: `product/anonymous.rs`.
- Modify dynamic builders if they currently bypass `finish_tuple`/`finish_record`.
- Test: product construction tests.

**Consumes:** static construction spec or runtime labels/values.

**Produces:** two safe routes into one materialized representation:

```text
static closed product
    -> precomputed shape/layout/type recipe
    -> ProductStorage

dynamic/computed product
    -> runtime interned shape
    -> universal Value-slot ProductLayout
    -> ProductStorage
```

**Required APIs (mechanical names may match repository conventions):**

```rust
fn finish_tuple_from_spec(
    vm: &mut VM,
    spec: &AnonymousProductConstructionLoweringSpec,
    source_values: Vec<Value>,
) -> PhResult<Value>;

fn finish_record_from_spec(
    vm: &mut VM,
    spec: &AnonymousProductConstructionLoweringSpec,
    source_values: Vec<Value>,
) -> PhResult<Value>;
```

Keep `finish_tuple`/`finish_record` as dynamic/general construction boundaries used by rest capture, pack assembly, reflection, and runtime-computed labels.

**Edit operations:**
1. [ ] Preserve zero normalization before descriptor/heap allocation.
2. [ ] Dynamic Tuple: intern Tuple shape from runtime label suffix; use one universal `Value` slot per component.
3. [ ] Dynamic Record: reject direct duplicate labels, preserve encounter order, use logical order equal to presentation order unless a closed semantic spec is present.
4. [ ] Static Record: evaluate source values in source order, place them using `source_to_logical`, and retain presentation mapping in shape.
5. [ ] Static Tuple: do not re-materialize label Values merely to recover known labels.
6. [ ] Resolve optional exact type recipe only through Task 21+ environment machinery when needed; until C4, closed recipes work and template recipes remain conservative/fail-closed.
7. [ ] Add tests showing static and dynamic materializations of the same Tuple/Record have identical language behavior despite different layout specialization.

---

### Task 9 — Add static product-build bytecodes without breaking dynamic pack construction

**Files:**
- Modify: `phalcom-core/src/bytecode.rs`.
- Modify: `phalcom-core/src/compiler/lib/expr.rs`.
- Modify: `phalcom-core/src/vm/dispatch.rs`.
- Modify: disassembly/opcode-stat snapshot tests.

**Consumes:** Task 6 static specs; Task 8 finalizers.

**Produces:** efficient static literal path with no runtime label payload duplication.

**Target bytecode direction:** append semantically explicit static forms rather than overloading dynamic forms ambiguously:

```rust
BuildStaticTuple { spec: u16 }
BuildStaticRecord { spec: u16 }
```

Existing `BuildTuple { positional, labeled }` and `BuildRecord { fields }` remain the dynamic/general shape path until all pack/reflection callers are naturally migrated.

**Edit operations:**
1. [ ] Append static Tuple/Record opcodes following current stable opcode-number discipline.
2. [ ] Static literal compiler emits component values only, in source evaluation order, then static build opcode referencing semantic spec.
3. [ ] Do not push known label Symbols as runtime stack payload for the static path.
4. [ ] VM resolves spec, applies source-to-logical mapping, interns/reuses descriptor, and builds `ProductStorage`.
5. [ ] Empty static Tuple/Record continues to compile directly to Unit.
6. [ ] Dynamic/computed-label/spread literals retain existing general product/pack construction until proven static.
7. [ ] Add disassembly tests proving fixed Record/Tuple literals use static build opcodes and dynamic label cases do not.

**C1 checkpoint evidence:**
- materialized Tuple/Record no longer contain per-instance label/value boxes;
- static two-Int Tuple/Record payload consumes two scalar words, not two 16-byte `Value`s;
- dynamic Tuple/Record still work through universal `Value` layout;
- GC tests green;
- old core collection semantics green.

---

# Checkpoint C2 — Runtime semantics and consumer migration

## Completion claim

No runtime consumer depends on the legacy Tuple/Record array layout, and language-level exactness/collection behavior is independent of which product layout was used.

### Task 10 — Extend VM-aware language exactness to Tuple and Record

**Files:**
- Modify: P1 VM-aware exact comparison helper in `phalcom-core/src/value/mod.rs` or its landed module.
- Modify: `phalcom-core/src/vm/dispatch.rs::Bytecode::Same`.
- Modify: `phalcom-core/src/primitive/object.rs::object_same`.
- Test: value relation tests.

**Consumes:** P1 data exact relation; ProductShape/descriptor registries.

**Produces:** one VM-aware language exactness authority for Data/Tuple/Record while internal `Value::same_as` remains representation identity.

**Required rules:**

Tuple:
1. require both values be Tuple/Unit-compatible according to zero normalization;
2. compare interned shape semantics (not merely descriptor ID if equivalent descriptors can differ by exact-type metadata);
3. recursively VM-language-`===` each logical component in tuple order.

Record:
1. require both Record values;
2. compare key sets independent of presentation order;
3. lookup corresponding logical values by label;
4. recursively VM-language-`===` values.

Data:
- preserve P1 exact semantic type + component rule.

**Edit operations:**
1. [ ] Rename/refactor the P1 helper if necessary into a product-capable VM-aware exact relation.
2. [ ] Add Tuple exact comparison across independently allocated static/dynamic boxes.
3. [ ] Add Record exact comparison with reversed presentation order.
4. [ ] Add nested mixed product exactness: data containing Tuple, Tuple containing Record, Record containing data.
5. [ ] Add negative cross-kind tests.
6. [ ] Search internal `same_as` users; keep sentinel/unsupported/runtime-handle checks on representation identity unless they are language `===`.
7. [ ] Prove `Bytecode::Same` and `Object#===` call the same language helper.

---

### Task 11 — Introduce registry-aware Tuple/Record views and migrate primitives/rendering

**Files:**
- Create or extend: `phalcom-core/src/product/view.rs`.
- Modify: `phalcom-core/src/primitive/tuple.rs`.
- Modify: `phalcom-core/src/primitive/record.rs`.
- Modify: `phalcom-core/src/value/render.rs`.
- Modify: heap accessors only for raw object borrow, not semantic decoding.

**Consumes:** descriptor -> shape/layout -> storage.

**Produces:** one representation-neutral product read API used by all runtime consumers.

**Required view operations:**

```text
Tuple:
    len
    positional_len
    labeled_len
    value_at(logical/index)
    label_at(labeled_index)
    lookup_label
    positionals iterator
    labeled entries in encounter/order semantics

Record:
    len
    label_at(presentation_index)
    value_at_presentation_index
    value_at_logical_index
    lookup_label
    entries in presentation order
```

**Edit operations:**
1. [ ] Implement read-only Tuple/Record views borrowing VM registries + heap object storage.
2. [ ] Centralize logical-index -> ProductStorage decode through P1 checked load helpers.
3. [ ] Migrate all Tuple raw primitives to views.
4. [ ] Migrate all Record raw primitives to views.
5. [ ] Migrate `Value::to_string`/debug rendering.
6. [ ] Preserve Tuple label fallback/public behavior and Record safe-get semantics exactly.
7. [ ] Add tests proving no primitive reads object-local label/value arrays because those arrays no longer exist.

---

### Task 12 — Migrate packs, expansion, rest capture, and block/function invocation

**Files:**
- Modify: `phalcom-core/src/vm/dispatch.rs`.
- Modify: `phalcom-core/src/vm/send.rs`.
- Modify: `phalcom-core/src/primitive/block.rs`.
- Modify: outgoing pack/argument assembly modules.
- Modify: any `Object::Tuple` direct storage matches found by repository search.
- Test: COLL005 expansion/pack suites + callables/rest suites.

**Consumes:** representation-neutral views + dynamic `finish_tuple`.

**Produces:** argument-pack behavior independent of physical Tuple storage.

**Edit operations:**
1. [ ] Replace direct `TupleObject.values()/positionals()/labels()` access in VM call routing with Tuple views.
2. [ ] Keep rest capture finalization through `finish_tuple` so empty rest is Unit and positive rest uses new ProductStorage.
3. [ ] Migrate `*`, `**`, `***` pack extraction paths.
4. [ ] Preserve left-to-right evaluation and duplicate-label behavior.
5. [ ] Preserve unlimited Tuple arity versus send-arity restrictions.
6. [ ] Run static/dynamic expansion fixtures with packed primitive layouts and universal dynamic layouts.
7. [ ] Add a regression where a packed Tuple survives fiber suspension and resumes with correct labels/values.

---

### Task 13 — Migrate Tuple/Record patterns and destructuring

**Files:**
- Modify: `phalcom-core/src/compiler/lib/patterns.rs` only where direct native assumptions exist.
- Modify runtime helper/primitive paths used by pattern lowering.
- Test: destructuring/pattern language suites, including nested Tuple and Record patterns.

**Consumes:** product views; exact semantic pattern rules.

**Produces:** patterns that remain correct for packed, dynamic, and later virtual products.

**Edit operations:**
1. [ ] Verify current pattern compiler does not depend on heap handle identity.
2. [ ] Replace native-array assumptions in arity/component reads with view/helper calls.
3. [ ] Preserve exact Tuple arity requirements and Record field-label requirements.
4. [ ] Preserve nested destructuring scratch-slot discipline.
5. [ ] Add nested packed-product destructuring tests.
6. [ ] Add pattern failure timing tests against static and dynamic materialized products.

---

### Task 14 — Prove Record order/equality/hash/reflection/map interoperability

**Files:**
- Modify only as needed: `phalcom-core/core/universe/src/collections/{tuple.ph,record.ph}`.
- Modify: reflection adapters that directly construct/read Tuple/Record.
- Modify: Map/Record conversion/runtime helpers.
- Test: collection contract, map interop, typing reflection suites.

**Consumes:** ProductShape presentation metadata.

**Produces:** proof that compact/canonical storage does not collapse Record presentation semantics or structural value behavior.

**Required hostile cases:**

```phalcom
const a = #{ name: "A", age: 24 }
const b = #{ age: 24, name: "A" }

// required
(a == b)   == true
(a === b)  == true
(a.hash == b.hash) == true

// required presentation distinction
iteration(a) -> name, age
iteration(b) -> age, name
```

Also prove Tuple label order remains semantic and order-sensitive.

**Edit operations:**
1. [ ] Keep Record `==` order-independent after storage canonicalization.
2. [ ] Keep Record hash order-independent.
3. [ ] Keep Record traversal/printing/reflection presentation ordered.
4. [ ] Keep Tuple equality/hash/labels ordered.
5. [ ] Migrate reflection cache Tuple construction to canonical finalizer or static equivalent; never allocate raw empty Tuple.
6. [ ] Migrate Record/Map conversions to product views.
7. [ ] Add shape-sharing tests showing presentation order metadata is shared, not copied per instance.

**C2 checkpoint evidence:** run collection, map, argument expansion, pattern/destructuring, reflection, GC, and language `===` focused suites. No direct consumer may depend on legacy Tuple/Record fields.

---

# Checkpoint C3 — Generalize P2 scalar replacement to Tuple and Record

## Completion claim

Tuple and Record have the same representation freedom as data in optimized bytecode: eligible literal aggregates can disappear into ordinary VM leaf locals, nested transparent products can flatten recursively, and materialization happens only at a proven observation/ABI boundary.

### Task 15 — Generalize the optimizer data model to all transparent products

**Files:**
- Modify: `phalcom-core/src/compiler/lib/product_opt.rs`.
- Modify: `compiler/lib/state.rs` if active virtual-product metadata changes.
- Test: product optimizer planner tests.

**Consumes:** Task 3 `ProductBindingKey`; static anonymous construction specs; P2 data/enum planner.

**Produces:** Tuple/Record variants and generic nested transparent-product shapes.

**Edit operations:**
1. [ ] Add `VirtualProductKind::Tuple` and `::Record`.
2. [ ] Replace `NestedData` with `NestedProduct` and update data recursion unchanged.
3. [ ] Extend `MaterializationReason` with explicit structural-product bailout reasons where diagnostics/bench evidence benefit (computed label, spread shape, unguarded structural send).
4. [ ] Keep enum payload recursion boundary unchanged unless the existing exact enum virtualization rule explicitly owns it.
5. [ ] Preserve `MAX_VIRTUAL_PRODUCT_LEAVES` as one centralized budget across data/Tuple/Record mixed shapes; do not silently increase frame pressure.
6. [ ] Carry Tuple shape and Record presentation/logical mapping in `VirtualShapePlan` without consuming leaf slots.
7. [ ] Add planner unit tests for pure Tuple, pure Record, mixed nested products, nullary Unit, and every bailout.

---

### Task 16 — Identify Tuple/Record candidates and only semantically safe direct uses

**Files:**
- Modify: `compiler/lib/product_opt.rs` AST/use visitor.
- Modify: `modules/semantic_lowering.rs` projection facts if new compiler-facing structural projection evidence is required.
- Read: semantic expression/source occurrence products; do not re-resolve types/names in compiler.

**Consumes:** exact `BindingId`, literal construction lowering spec, semantic Type/dispatch facts.

**Produces:** conservative use classification for anonymous product bindings.

**Eligible initializers:**
- positive static Tuple literal with static shape;
- positive static Record literal with static shape;
- later nested exact transparent-product literal accepted recursively.

**Safe direct uses in P3:**
- Tuple/Record destructuring/pattern component reads whose semantics are structural and compiler-owned;
- structural projection already published by semantic lowering as a direct product coordinate;
- `.class` observation;
- exact retained type observation;
- `===` against another provably virtual compatible transparent product;
- whole-value boundary with one sinkable materialization.

**Conservative boundaries:**
- ordinary `at`, `get`, index getter, arbitrary getter fallback, `==`, `hash`, `toString`, or user send unless existing compiler guard/inliner proves that bypassing dispatch is valid;
- computed labels;
- spread/pack-produced unknown shape;
- captures/reassignment under existing P2 policy.

**Edit operations:**
1. [ ] Recognize candidate construction solely through Task 6 lowering attachments.
2. [ ] Use semantic exact targets/projection facts for direct structural coordinates.
3. [ ] Never infer “record.name means field #name” in optimizer merely from AST spelling; getter precedence must remain correct.
4. [ ] Classify tuple literal constant-index operations only if semantic/dispatch evidence proves the builtin structural operation and existing world/sacred guards remain valid; otherwise materialize.
5. [ ] Add negative tests where a potentially overriding getter/index send forces materialization.
6. [ ] Add same-name/shadowing tests using canonical BindingId.

---

### Task 17 — Virtualize Tuple/Record construction, projection, and rematerialization

**Files:**
- Modify: `compiler/lib/product_opt.rs`.
- Modify: `compiler/lib/expr.rs`.
- Modify: static product construction emitter from Task 9.

**Consumes:** `VirtualShapePlan`, static anonymous lowering spec, contiguous leaf-slot substrate from P2.

**Produces:** zero-allocation eligible anonymous products.

**Construction law:** reserve leaf slots first; evaluate source expressions exactly once in source order; map each result to logical leaf range; shape labels/order are metadata, not runtime values.

**Rematerialization law:** reconstruct from logical leaves through the original static product construction spec; Record presentation order comes from shape spec, not canonical type order.

**Edit operations:**
1. [ ] Add `compile_virtual_tuple_constructor_into` and `compile_virtual_record_constructor_into` equivalents.
2. [ ] Add direct leaf/subshape projection helpers.
3. [ ] Add Tuple rematerialization through static product spec.
4. [ ] Add Record rematerialization with canonical logical storage + original presentation shape.
5. [ ] Sink one required whole-value materialization using P2's existing policy.
6. [ ] Add disassembly/allocation test: `(mark(1), mark(2)).<safe structural projection>` executes effects but allocates no Tuple.
7. [ ] Add Record test where source order differs from canonical field order and scalar replacement preserves effect/presentation semantics.
8. [ ] Add GC test where the only live reference is a virtual product leaf.

---

### Task 18 — Flatten recursively across data, Tuple, and Record

**Files:**
- Modify: `compiler/lib/product_opt.rs` recursive shape builder/materializer.
- Test: nested optimizer suites.

**Consumes:** P2 nested data flattening; P3 generic `NestedProduct`.

**Produces:** mixed transparent aggregates represented as one leaf region when all nested constructor expressions are exact and within budget.

**Examples:**

```phalcom
data Point(x: Int, y: Int)

data Envelope(
  point: Point,
  metadata: #{ code: Int, ok: Bool },
)

const x = (
  Envelope(
    point: Point(x: 1, y: 2),
    metadata: #{ ok: true, code: 9 },
  ),
  label: #{ value: Point(x: 3, y: 4) },
)
```

Eligible leaf plan contains only scalar/object `Value` leaves; no intermediate Point/Record/Envelope/Tuple heap allocation is required until a boundary.

**Edit operations:**
1. [ ] Generalize recursive shape creation to Data/Tuple/Record constructor expressions.
2. [ ] Preserve each nested node's own materialization recipe and Record presentation shape.
3. [ ] Preserve source evaluation tree/order independently from flattened leaf order.
4. [ ] Keep enum/class/dynamic/unproven existing values as scalar leaves unless separately virtualized by existing exact-enum logic.
5. [ ] Preserve the global leaf budget with checked arithmetic.
6. [ ] Add 2-level/3-level mixed nesting tests and bailout-at-budget tests.
7. [ ] Add selective subshape materialization test: requesting nested Record materializes only that Record, not the outer Data/Tuple.

---

### Task 19 — Answer `.class`, exact type observation, and `===` without materialization where proven

**Files:**
- Modify: `compiler/lib/product_opt.rs` use classification/codegen.
- Modify: expression emitter around `.class`/typing reification if current lowering has dedicated forms.
- Modify: no runtime semantics beyond Task 10 helper.

**Consumes:** virtual shape, Data exact type recipe, Tuple/Record behavior class, Task 21 runtime type recipe API when template environments are needed.

**Produces:** observation-aware scalar replacement rather than conservative boxing for observations that do not require a backing object.

**Required fast paths:**

```text
virtual data .class   -> declaration behavior class
virtual Tuple .class  -> Tuple class
virtual Record .class -> Record class

virtual exact type observation
    -> reify static/runtime type recipe directly

virtual transparent lhs === virtual transparent rhs
    -> compare semantic product kind/shape/type requirement
    -> recursively compare scalar leaves / nested virtual products
```

Record `===` must ignore presentation order and pair fields by key/logical label.

**Edit operations:**
1. [ ] Add use kinds for class observation, exact type observation, and virtual exact relation.
2. [ ] Emit known class object without aggregate materialization.
3. [ ] Emit/reify exact type recipe without materializing where recipe is available.
4. [ ] Emit componentwise exact comparison for two virtual compatible products.
5. [ ] For virtual-vs-materialized or unsupported mixed relation, materialize virtual side and use canonical runtime helper.
6. [ ] Keep ordinary `==` and `hash` as materialization boundaries in P3 unless an existing guarded send optimizer proves them.
7. [ ] Add allocation assertions proving `.class` and virtual `===` can remain zero-allocation.

---

### Task 20 — Differentially prove anonymous-product optimization and suspension safety

**Files:**
- Extend: P2 optimizer differential test harness.
- Extend: allocation/disassembly tests.
- Extend: GC/fiber tests.

**Required differential matrix:**

```text
Tuple:
  literal -> structural projection
  local -> destructuring
  nested Tuple/Data/Record
  one whole use
  .class
  === virtual/virtual
  ordinary send bailout
  computed label bailout
  expansion bailout

Record:
  literal reordered fields -> projection/destructure
  presentation order after rematerialization
  same-key different-order ===
  nested Record/Data/Tuple
  one whole use
  .class
  ordinary getter/index bailout
  spread/computed key bailout

Cross-cutting:
  side effects
  throwing component expressions
  closure capture fallback
  mutable/reassigned fallback
  GC-only leaf
  fiber yield/await with virtual leaves live
  optimizer Disabled oracle
```

**Edit operations:**
1. [ ] Compare stdout/result/error/stack-trace behavior Enabled vs Disabled.
2. [ ] Assert allocation counts, not just bytecode text.
3. [ ] Assert static materialization bytecode disappears for successful virtual candidates.
4. [ ] Assert canonical build bytecode remains for every bailout row.
5. [ ] Exercise scratch relocation/open upvalues while unrelated virtual leaves are live.
6. [ ] Exercise fiber suspension/resumption with heap objects reachable only through leaf slots.

**C3 checkpoint evidence:** all P2 data/enum optimization evidence still green plus new Tuple/Record matrix; no diagnostic drift; no unsupported direct dispatch bypass.

---

# Checkpoint C4 — Runtime generic, Dynamic, and reification closure

## Completion claim

A transparent product may be unboxed/virtual for most of its lifetime yet still materialize or reify with truthful exact semantic type information at a Dynamic/reflection boundary, including generic and phantom specializations. Generic evidence is attached to activation lifetime, not copied into every value.

### Task 21 — Add runtime type recipes and an interned runtime type-environment registry

**Files:**
- Create: `phalcom-core/src/typing/environment.rs`.
- Modify: `phalcom-core/src/typing/mod.rs`.
- Modify: `phalcom-core/src/typing/overlay.rs` and/or reification helpers only for lazy substitution support.
- Modify: P1/P3 lowering semantic spec types from closed `RuntimeTypeRef` to `RuntimeTypeRecipe` where necessary.
- Test: runtime typing unit tests.

**Consumes:** immutable loaded `phalcom-type-meta` graph; stable type-parameter refs; runtime overlay.

**Produces:** compact environment IDs and lazy template instantiation.

**Instantiation rule:**

```text
RuntimeTypeRecipe::Closed(handle)
    -> handle

RuntimeTypeRecipe::Template(handle containing parameter refs)
    + RuntimeTypeEnvironment
    -> recursively/lazily substitute parameters
    -> intern/reuse runtime overlay Applied/Tuple/Record/etc. nodes
    -> closed RuntimeTypeRef
```

Do not eagerly clone the whole metadata graph. Memoize within the typing context/environment as appropriate.

**Edit operations:**
1. [ ] Add `RuntimeTypeEnvironmentId::EMPTY` and VM-owned intern registry.
2. [ ] Store sorted/stable parameter->runtime-type bindings; environment equality is structural, not allocation-order dependent.
3. [ ] Add lazy `instantiate_type_recipe` with cycle/budget protection consistent with existing metadata safety rules.
4. [ ] Support at minimum Nominal, Applied, Tuple, Record, Parameter, Union, Callable, SelfType forms already needed by retained metadata; preserve unsupported statuses explicitly rather than erasing.
5. [ ] Reuse runtime overlay interning for synthesized applied/structural forms.
6. [ ] Add tests for `Point<T> -> Point<Int>`, `(T,T) -> (Int,Int)`, `{value:T} -> {value:Int}`, and phantom `Id<K>`.
7. [ ] Add cache tests proving repeated instantiation reuses semantic runtime handles without rooting reflection descriptor objects.

---

### Task 22 — Propagate runtime type environments through invocation, frames, blocks, and fibers

**Files:**
- Modify: `phalcom-core/src/frame.rs::CallFrame`.
- Modify: `phalcom-core/src/vm/{dispatch.rs,send.rs}` call-frame creation routes.
- Modify: `phalcom-core/src/heap/block.rs::BlockObject`.
- Modify: compiler/chunk semantic side metadata for generic call-site environment recipes.
- Audit: bound-method/callable invocation, constructor invocation, associated family invocation, reflective invocation.
- Test: generic call/closure/fiber integration tests.

**Consumes:** semantic generic application results already known by compiler; Task 21 environment registry.

**Produces:** every generic bytecode activation that can require runtime type reification has a truthful compact environment ID.

**Target frame change:**

```rust
pub struct CallFrame {
    // existing fields...
    pub type_environment: RuntimeTypeEnvironmentId,
}
```

Keep `CallFrame: Copy`.

**Call-site carriage:** prefer compiler executable side metadata keyed by invocation site/IP over duplicating type arguments in ordinary runtime values. Each semantically resolved generic call site publishes a callee-environment recipe from canonical inference results. VM resolves that recipe against the caller frame environment and interns the callee environment before frame push.

**Escaping lexical block rule:** a `BlockObject` created inside a generic activation captures the current `RuntimeTypeEnvironmentId`, because the block may execute after the home frame has returned and may contain `Point<T>`/Tuple/Record materialization or reification.

**Edit operations:**
1. [ ] Add frame environment ID with EMPTY default for nongeneric activations.
2. [ ] Project generic call-site substitutions into executable semantic metadata; never reconstruct inference in VM.
3. [ ] Thread optional environment recipes through every bytecode invocation path that can activate generic bytecode.
4. [ ] Ensure first-class callable/bound-method calls receive the call-site environment when semantic generic application established one.
5. [ ] Capture current environment ID in lexical `BlockObject` and restore/use it on block activation.
6. [ ] Verify fibers preserve frame/block environment IDs by ordinary copied frame/object lifetime.
7. [ ] Audit reflective/dynamic invocation entrypoints: if they can activate a generic callable without an explicit/retained type environment, use the existing underconstrained/invalid path or fail closed; do not invent types from runtime argument classes.
8. [ ] Add generic closure escape test where `T` survives after defining activation ends.
9. [ ] Add generic fiber suspension test.

---

### Task 23 — Instantiate exact materialized data/anonymous descriptors from current runtime type recipe

**Files:**
- Modify: P1 data descriptor registry/materialization path.
- Modify: `product/anonymous.rs` descriptor registry.
- Modify: static build bytecode handlers.
- Modify: virtual rematerialization helpers.

**Consumes:** Task 21 recipes + current frame/block environment.

**Produces:** closed exact runtime type handles on actual materialization without per-value generic arrays.

**Required cases:**

```phalcom
fn wrap<T>(_ x: T) -> Dynamic {
  Point<T>(x: x, y: x)
}

wrap(42)
    -> materialized descriptor exact type Point<Int>
```

```phalcom
fn pair<T>(_ x: T) -> Dynamic {
  (x, x)
}

pair(42)
    -> anonymous descriptor exact type (Int, Int) when the semantic type is retained/exportable
```

```phalcom
fn record<T>(_ x: T) -> Dynamic {
  #{ value: x }
}

record(42)
    -> exact closed Record type when semantic analysis exported one
```

**Edit operations:**
1. [ ] Replace P1 construction spec's closed exact-type-only field with `RuntimeTypeRecipe` while preserving closed fast path.
2. [ ] At Data materialization/nullary singleton load, instantiate recipe using current environment before interning `RuntimeDataDescriptorId`.
3. [ ] At static Tuple/Record materialization, instantiate optional structural type recipe and store closed handle in anonymous descriptor.
4. [ ] Ensure phantom generic data specializations with identical physical layouts obtain distinct exact data descriptor IDs.
5. [ ] Ensure layout/shape registries remain independent from exact type recipe/environment IDs.
6. [ ] Add assertion/fail-closed error for a template recipe that reaches materialization without all required bindings.

---

### Task 24 — Qualify Dynamic boundaries, phantom/nullary data, and structural reification

**Files:**
- Extend compiler/runtime integration tests.
- Modify materialization boundary classification only where evidence shows a missing boundary.
- Modify typing/reflection tests; do not invent new public reflection API.

**Consumes:** semantic `DynamicBoundary` status, P2/P3 materialization, Task 23 descriptor closure.

**Produces:** exhaustive boundary evidence.

**Required test matrix:**

```text
Data:
  static Point<Int> -> Dynamic -> exact descriptor Point<Int>
  generic Point<T> with T=Int -> Dynamic -> Point<Int>
  Id<User> and Id<Order> same layout -> Dynamic -> remain distinct exact types
  Signal<Connected>() -> Dynamic -> same exact singleton specialization
  Signal<Disconnected>() -> Dynamic -> distinct from Connected
  Dynamic component inside otherwise packed data does not poison outer Int/Bool slots

Tuple:
  virtual (Int, Int) -> Dynamic -> Tuple class + truthful retained structural descriptor
  labeled Tuple retains label order
  generic (T,T) with T=Int closes through environment

Record:
  exact closed record retains structural type independent of presentation order
  different presentation orders retain their own shape metadata
  broad/computed-key record does NOT fabricate a closed exact Record type
  generic closed record recipe substitutes T when such a type was semantically established

Observation:
  .class does not require materialization for virtual product
  exact retained type observation does not require materialization when recipe can be reified directly
  weak typing-descriptor cache semantics remain unchanged
```

**Edit operations:**
1. [ ] Add low-level runtime descriptor assertions if the public reflection surface does not expose an exact value-type selector yet.
2. [ ] Use current reflection APIs where they exist; do not create P3-only user syntax.
3. [ ] Test static -> Dynamic -> observation and generic -> Dynamic -> observation.
4. [ ] Test Dynamic storage inside List/Map/Record and return across function boundaries.
5. [ ] Test re-narrow/type-test paths already supported by current semantics without extending syntax.
6. [ ] Assert no path turns missing proof into `Dynamic` solely to make a test pass.

---

### Task 25 — Prove environment lifetime, closure capture, fiber suspension, and fail-closed behavior

**Files:**
- Test-heavy changes around `frame.rs`, `block.rs`, fibers, typing environment registry.
- Modify implementation only for defects exposed by hostile tests.

**Required hostile tests:**
1. generic function creates escaping block that later constructs `data<T>`;
2. escaping block later returns Tuple/Record containing `T` and crosses Dynamic boundary;
3. generic activation yields/suspends before materialization;
4. fiber switches while virtual product leaves and type environment are live;
5. recursive/nested type recipe uses bounded instantiation and cannot loop indefinitely;
6. deliberately missing environment binding fails closed with internal/semantic invariant, never erased type;
7. descriptor weak-cache collection does not invalidate exact semantic environment/recipe identity;
8. environment registry does not retain heap values or create GC cycles.

**C4 checkpoint evidence:** exact runtime specialization is correct at every tested observation boundary; no product stores per-instance generic args; generic closures/fibers preserve required evidence.

---

# Checkpoint C5 — LANG005.C1 delivery closure

## Completion claim

C1 has one reusable immutable-product representation and optimization architecture across data, enum payloads, Tuple, and Record; all semantic distinctions remain intact; Dynamic/reification boundaries are truthful; deterministic representation/performance evidence proves the intended wins.

### Task 26 — Audit incrementality, artifacts, source index, and LSP invisibility

**Files:**
- `phalcom-semantic` incremental tests.
- `phalcom-core/src/modules/{semantic_lowering.rs,artifact.rs,compile.rs}` as landed.
- `phalcom-lsp` only if source/presentation consumers regress.
- LANG005 state file.

**Consumes:** existing semantic source/index/type metadata authority.

**Produces:** proof P3 representation changes do not become a second semantic truth or cause broad invalidation.

**Edit operations:**
1. [ ] Verify anonymous product layout/shape compiler specs are regenerated from existing semantic expression products; they are not new semantic DB query authority.
2. [ ] Verify body-only edits that do not change product semantic type do not unnecessarily invalidate unrelated declaration products.
3. [ ] Verify structural type changes regenerate affected lowering/artifact metadata.
4. [ ] Verify optimizer Enabled/Disabled does not alter source-index identities, hover types, definitions, references, or diagnostics.
5. [ ] Verify RuntimeTypeRecipe metadata is deterministic across cold/incremental builds.
6. [ ] Record incremental work stats and ensure no workspace-wide scan was introduced for one product expression.

---

### Task 27 — Deterministic representation and performance qualification

**Files:**
- Extend P1/P2 allocation/performance harnesses.
- Add focused product representation tests/benchmarks.

**Required deterministic assertions:**

```text
size_of::<Value>() == 16

materialized data(Int,Int):
    one DataObject
    two scalar payload words
    no component labels

materialized static Tuple(Int,Int):
    one TupleObject
    two scalar payload words
    no per-instance labels

materialized static Record{name:Int,age:Int}:
    one RecordObject
    two scalar payload words
    no per-instance labels
    shared ProductShape metadata

same exact Record type, two presentation orders:
    may share ProductLayout
    retain different ProductShape/presentation order

virtual eligible data/Tuple/Record literal -> projection/destructure:
    zero product heap allocations

nested mixed transparent product within leaf budget:
    zero intermediate product heap allocations

Dynamic boundary:
    exactly required materialization, no duplicate rematerialization

phantom specializations:
    distinct exact descriptors
    same ProductLayout when physical shape equal
```

**Measured workloads:** constructor throughput, component read/destructure, nested product creation, GC pressure, generic Dynamic boundary, Record lookup, Tuple rest/pack, Enabled/Disabled optimizer comparison. Timing is supporting evidence; deterministic allocation/word-count gates are release-critical.

---

### Task 28 — Run negative/deletion gates and broad verification

**Negative searches after migration:**

```text
TupleObject fields containing Box<[Value]>
TupleObject fields containing Box<[Symbol]>
RecordObject fields containing Box<[Value]>
RecordObject fields containing Box<[Symbol]>
direct empty Tuple allocation
direct empty Record allocation
optimizer binding identity by source name/range
per-value generic_arguments arrays on Data/Tuple/Record
ClassObject per applied data/structural type
layout ID used as semantic type identity
ProductLayout containing labels
runtime payload inspection used to fabricate exact structural type
```

Do not require deletion of dynamic product builders, `Value::same_as`, or existing runtime typing weak caches; they remain valid mechanisms with narrower responsibilities.

**Broad verification order:** smallest-first after focused checkpoints are green.

```sh
cargo fmt --all -- --check
cargo test -p phalcom-ast
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-type-meta
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo clippy --workspace --all-targets -- -D warnings
./scripts/verify.sh --full
```

If current repository scripts/CI changed, record the new canonical commands in state before running them.

---

### Task 29 — Close implementation state, PDR maintenance, and C1 handoff

**Files:**
- LANG005 C1 implementation state.
- PDR-0035 implementation notes only where shipped representation facts changed.
- PDR-0036 status/evidence.
- P1/P2 plans/state exclusions amended to show P3 completion.

**Edit operations:**
1. [ ] Record exact repository revision used for final qualification.
2. [ ] Record final type/shape/layout/descriptor/runtime-environment names and locations.
3. [ ] Record all C0–C5 checkpoint evidence commands/results.
4. [ ] Record Enabled/Disabled differential matrix.
5. [ ] Record allocation/word-count matrix and measured performance data.
6. [ ] Record negative-search results.
7. [ ] Record any deliberately deferred optimizer opportunities separately from unproven C1 requirements.
8. [ ] Mark C1.P3 COMPLETE only when no C1-owned evidence remains deferred and no active incident exists.
9. [ ] Set next LANG005 resume action to C2 inherent `impl` / enum behavioral separation.

---

# 8. Detailed semantic edge cases the implementation must cover

## 8.1 Record canonical storage versus presentation order

This is the highest-risk anonymous-product correctness edge.

For:

```phalcom
const a = #{ name: "A", age: 24 }
const b = #{ age: 24, name: "A" }
```

semantic type identity is order-independent. P3 may therefore choose canonical logical storage such as:

```text
logical 0 = age
logical 1 = name
```

for both values. But their shape descriptors must retain:

```text
a presentation: name -> logical 1, age -> logical 0
b presentation: age -> logical 0, name -> logical 1
```

Iteration/render/reflection use presentation order. Equality/hash/`===` use field identity rather than presentation index. Compiler virtualization must retain the same mapping so rematerialization cannot accidentally canonicalize visible order.

## 8.2 Tuple labels are semantic ordered coordinates

Tuple does not use Record's order-insensitive rule. `(x: 1, y: 2)` and `(y: 2, x: 1)` have different Tuple shape/type identity and are not `===` merely because label/value pairs can be permuted.

## 8.3 Nullary structural products stay Unit, not anonymous descriptors

No `RuntimeAnonymousProductDescriptorId` should be allocated merely for `()` or `#{}`. Their runtime value is Unit. Source/reflection provenance may still know which syntax was written where that information belongs in compiler/source products.

## 8.4 Broad/dynamic Record must not fabricate a closed exact type

A runtime key set is not automatically a compiler-semantic exact record type. If semantic analysis established broad `Record<K,V>`, open row/intersection, or Dynamic knowledge, materialization may intern the runtime presentation shape for operational access but `exact_type` remains absent or the actual retained broad type recipe according to existing metadata semantics. Never synthesize “closed `{actual runtime keys...}`” solely from payload observation.

## 8.5 Value semantics does not authorize dispatch bypass

Even though backing identity is gone, ordinary sends remain language behavior. Example:

```phalcom
record.name
```

has ordinary getter dispatch precedence before Record field fallback. P3 cannot replace it with a leaf load merely because the virtual Record has a `#name` key unless semantic lowering/guard machinery proves the field fallback is the actual operation under the current world/dispatch rules.

Likewise `tuple[0]`, `record[#x]`, `==`, `hash`, and arbitrary methods stay materialization/send boundaries until proven safe by existing guard architecture.

## 8.6 `===` may be optimized because its transparent-product semantics are now intrinsic

Unlike ordinary overridable `==`, P3's language exact relation is defined independently of backing representation. The optimizer may compare two virtual values directly using their semantic kind/shape and leaves, provided it produces exactly the same result as the VM-aware exact helper.

## 8.7 Generic runtime evidence belongs to activation lifetime

For shared generic bytecode:

```phalcom
fn make<T>(_ x: T) -> Dynamic {
  Point<T>(x: x, y: x)
}
```

`T = Int` is an invocation fact. It should normally live in the frame environment. Only if the resulting value escapes into universal storage does the materialized value retain a compact exact descriptor ID. This is the narrowest-lifetime rule:

```text
compile-time closed fact
    > runtime activation environment
        > materialized exact descriptor
```

Never jump directly to per-value `[T1,T2,...]` storage.

## 8.8 Lexical block escape must retain type environment

A block created inside a generic activation may use `T` after that activation returns. Its ordinary value captures may already be closed over, but runtime type evidence is separate from value upvalues. Capture the compact environment ID with the block so reification/materialization remains exact later.

## 8.9 Fiber suspension is not a reification boundary by itself

Suspension should not force materialization of virtual products or duplicate generic descriptors. VM leaf slots and frame environment IDs survive with the fiber; only actual ABI/observation boundaries materialize.

---

# 9. Required test families

## 9.1 Exact relation

- independently allocated equal Tuple is `===`;
- different Tuple label/order is not `===`;
- independently allocated equal Record is `===`;
- Record different presentation order but same key/value set is `===`;
- differing nested component exactness makes outer product non-exact;
- data phantom specialization remains non-exact across different type args;
- Tuple/Record/data cross-kind exactness is false;
- Unit exactness remains canonical.

## 9.2 Representation convergence

- no labels per Tuple instance;
- no labels per Record instance;
- primitive slots pack to one word;
- Dynamic/object leaves remain safe `Value` slots;
- static and dynamic product paths interoperate;
- GC sees all and only traceable slots.

## 9.3 Source order

- reversed labeled data construction;
- Record source order differing from canonical field order;
- component expression throws after prior side effects;
- nested mixed product side effects preserve source tree order;
- computed-label fallback preserves label expression order.

## 9.4 Optimizer

- ephemeral Tuple -> structural projection/destructure zero allocation;
- local Tuple projection/destructure scalar replacement;
- same for Record;
- nested mixed products;
- one whole use sinks materialization;
- ordinary send bailout;
- capture/mutation/global bailout;
- leaf-budget bailout;
- `.class` no materialization;
- virtual `===` no materialization;
- Enabled/Disabled differential.

## 9.5 Runtime generic/reification

- closed generic data;
- phantom generic data;
- nullary generic data singleton;
- generic Tuple type recipe;
- generic closed Record type recipe;
- broad Record does not fabricate exact closed type;
- escaping generic block;
- generic fiber suspension;
- weak reflection descriptor cache;
- missing environment binding fails closed.

## 9.6 Incrementality/tooling

- product-expression type edit invalidates lowering spec;
- body-only unrelated edit does not regenerate unrelated product metadata;
- optimizer mode does not alter semantic/source identities;
- structural record canonical type identity remains deterministic across presentation order edits where semantic type is unchanged.

---

# 10. Failure protocol

Stop the current checkpoint and record an active incident when any of these occurs:

1. Tuple/Record optimization changes observable `===`, `==`, hash, presentation order, error timing, or source evaluation order.
2. A runtime product descriptor begins using `ProductLayoutId` as semantic type identity.
3. A static Record layout requires per-instance labels to preserve order.
4. Runtime generic materialization cannot recover a required parameter binding and an implementation attempts to erase it instead of failing closed.
5. `CallFrame`/Block environment carriage breaks `Copy`/arena-size/runtime invariants materially; investigate a compact side-table alternative before widening hot objects blindly.
6. Optimizer needs to resolve a name/type/member independently from semantic products.
7. GC requires object-specific duplicated trace maps rather than the shared layout registry.
8. A Dynamic/computed Record path invents a closed semantic type from runtime keys.
9. An optimization requires bypassing an unguarded ordinary method-table lookup.
10. Enabled and Disabled compilation disagree on source diagnostics.

For an incident, record:

```text
checkpoint/task
repository revision
minimal reproducer
expected invariant
actual result
suspected owning layer
whether canonical Disabled/P1 execution is correct
smallest command reproducing failure
next investigation action
```

Do not continue layering optimizations over an unresolved semantic incident.

---

# 11. Drift protocol

Before every checkpoint:

1. fetch current `main` revision;
2. compare against the last recorded LANG005 state revision;
3. inspect changes touching:
   - product/heap/value;
   - compiler expression/pattern/call lowering;
   - semantic expression/binding/source index;
   - runtime typing metadata/reification;
   - call frames/blocks/fibers;
   - collections/argument expansion;
4. update file/symbol names mechanically if ownership moved;
5. do not silently change semantic boundaries because a helper moved.

If P1/P2 landed with names differing from their plans, preserve the P3 information model and adapt to final repository names rather than recreating planned files literally.

---

# 12. Staged commit groups

Recommended reviewable grouping:

```text
C0
  docs(lang): ratify structural product value identity
  fix(data): preserve source constructor order and close data inheritance
  refactor(optimizer): use canonical binding identity
  fix(product): enforce zero-product normalization globally

C1
  feat(product): intern anonymous product shapes and descriptors
  feat(product): pack tuple and record materialization
  feat(bytecode): add static structural product builds

C2
  fix(runtime): make transparent product exactness representation-independent
  refactor(runtime): migrate tuple record consumers to product views
  test(product): preserve packs patterns record order and gc

C3
  feat(optimizer): scalar replace tuple and record products
  feat(optimizer): flatten mixed transparent products
  feat(optimizer): avoid materialization for class type and exact observations
  test(optimizer): differential anonymous product semantics

C4
  feat(typing): add runtime generic type environments
  feat(runtime): instantiate exact product descriptors from type recipes
  test(typing): close dynamic generic reification boundaries

C5
  test(lang005): qualify product representation and performance
  docs(lang005): close C1 implementation state
```

Combine adjacent commits if the actual patch is cleaner, but do not mix runtime type-environment introduction into the initial Tuple/Record storage migration; those are independently diagnosable semantic risks.

---

# 13. Known scope exclusions after P3

P3 deliberately does **not** implement:

- interprocedural escape analysis;
- captured multi-leaf virtual-product upvalue ABI (ordinary capture remains a bailout unless independently implemented safely);
- mutable virtual product locals;
- global/static/field virtual products;
- multi-value function ABI;
- untagged machine-register VM locals;
- generic method/function monomorphization;
- whole-program specialization;
- inline `List<Data>`/`List<Tuple>`/array-of-struct or struct-of-array element storage;
- JIT/deoptimization;
- stable FFI product layout;
- arbitrary optimization of overridable `==`, `hash`, `toString`, index/getter sends;
- new public value-type reflection syntax if the current reflection surface does not already expose one;
- enum variants-only syntax or behavior migration to `impl` (LANG005.C2);
- traits/associated types/conformance machinery;
- final data-derived API policy.

These are later opportunities, not missing C1 evidence.

---

# 14. Release-complete criteria

LANG005.C1.P3 is COMPLETE only when all of the following are true:

1. PDR-0036/value-semantics docs are landed and no normative contradiction remains.
2. P1 source evaluation order and data non-subclassability gaps are closed.
3. P2 optimizer binding identity uses canonical semantic `BindingId`.
4. No executable path allocates empty Tuple/Record; Unit is universal zero product.
5. Positive materialized Tuple/Record use `ProductStorage` and shared shape metadata.
6. Tuple/Record objects contain no per-instance labels or universal `Box<[Value]>` payload arrays.
7. Static closed Tuple/Record can use primitive-width packed layouts; dynamic products use safe universal `Value` slots through the same substrate.
8. Tuple/Record language `===` is representation-independent and agrees between materialized, rematerialized, and virtual execution.
9. Record `===`/`==`/hash ignore presentation order while iteration/render/reflection preserve it.
10. All runtime consumers—packs, rest, blocks, patterns, primitives, reflection, rendering, Map interop—operate through representation-neutral views.
11. P2 scalar replacement supports eligible Tuple/Record and mixed Data/Tuple/Record nesting.
12. `.class`, exact retained type observation, and compatible virtual `===` can avoid materialization where proven.
13. Ordinary unguarded dispatch remains a conservative materialization boundary.
14. Runtime generic type recipes close through an activation environment; generic/phantom/nullary data remains exact at Dynamic/reification boundaries.
15. Escaping lexical blocks and fibers preserve required runtime type environment identity.
16. No product stores per-value generic argument arrays and no applied-type ClassObjects are created.
17. Broad/dynamic Records never fabricate a closed exact semantic type from runtime keys.
18. GC hostile tests prove precise tracing for packed materialized and virtual leaf values.
19. Enabled/Disabled differential suites are green for data, Tuple, Record, and exact-enum P2 cases.
20. Deterministic allocation/word-count gates prove the intended representation wins.
21. Incremental/source-index/LSP products are unchanged by optimizer mode and remain semantically authoritative.
22. All negative/deletion searches are green.
23. Affected crates, Clippy, formatting, and `./scripts/verify.sh --full` are green.
24. LANG005 C1 state has no active incident and no deferred C1-owned evidence.
25. The next resume action is LANG005.C2 inherent `impl` and enum behavioral separation.


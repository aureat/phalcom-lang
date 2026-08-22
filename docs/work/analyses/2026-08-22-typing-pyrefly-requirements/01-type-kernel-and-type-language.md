# Specification 01: Stratified Type Kernel, Relations, and Source Type Language

**Status:** Draft implementation specification<br>
**Depends on:** ratification of the semantic-level/kinding direction in [`ontology.md`](../../../spec/typing/ontology.md); the existing core lattice and generic-signature design decisions<br>
**Enables:** typed module interfaces, generic applications, protocol and member specialization, formal flow checking, and all later proving work<br>
**Primary owners:** `phalcom-semantic`, `phalcom-ast`, parser diagnostics, core native-type metadata<br>
**Non-goal:** changing runtime message dispatch, object representation, or reflection semantics in this phase

## 1. Problem statement

Phalcom’s first checker has a useful storage seam but its current representation is more expressive than its executable invariants. `TypeData` can encode nominal, applied, union, tuple, record, callable, parameter, and inference nodes ([`store.rs`](../../../../phalcom-semantic/src/types/store.rs#L35-L56)); `KindData` can encode `Type` and arrows ([`kind.rs`](../../../../phalcom-semantic/src/types/kind.rs)). Construction nevertheless assigns `Kind::Type` to every nominal or applied type, makes `kind_of` default to `Type`, and performs no constructor-kind or arity validation ([`store.rs`](../../../../phalcom-semantic/src/types/store.rs#L77-L100), [`kind_of`](../../../../phalcom-semantic/src/types/store.rs#L126-L161)). The code can therefore represent a future HKT vision without yet preventing ill-kinded applications.

The same mismatch appears in relationships. `Applied` types are compared covariantly in every argument ([`relation.rs`](../../../../phalcom-semantic/src/types/relation.rs#L110-L126)). This is unsound for mutable containers: if `List` exposes both write and read operations, deriving `List<Int> <: List<Any>` permits a write of a non-`Int` value through the widened alias. `TypeParameterId(u32)` and `InferVarId(u32)` are session-local integers ([`id.rs`](../../../../phalcom-semantic/src/types/id.rs)); they cannot safely identify a declaration parameter across a project snapshot, and a fresh local solver starts variable numbering at zero. The current constraint solver is appropriately labeled local but lacks occurs checks, substitution normalization, worklists, attribution, cycle handling, or result states ([`constraint.rs`](../../../../phalcom-semantic/src/types/constraint.rs)).

Finally, the source type grammar has not caught up with the AST. Application, tuple, and callable annotations exist in the AST but are deferred in lowering and not parsed. A user-visible type language cannot be layered on top of such partial semantics without locking in accidental ambiguity, unsound defaults, or diagnostic churn.

This specification makes the kernel a trustworthy substrate before it becomes broadly visible in source syntax.

## 2. Authority and compatibility boundary

The following inputs guide this work with different authority:

- `docs/spec/typing/01-core-type-lattice-and-unit.md` defines the intended lattice direction, including `Never <: T <: Any`, and the distinction between gradual `Dynamic` and static lattice membership. Treat it as a design commitment only to the degree it has been ratified by project governance; current code does not yet represent `Any` as a canonical static type.
- `docs/spec/typing/STATUS.md` requires owner-qualified parameter identity, invariant-by-default type parameters, `None` rather than implicit dynamic for absent annotations, and separate relation concepts. These are the immediate compatibility constraints for this implementation.
- `docs/spec/typing/02-type-expression-foundation.md` and `03-generic-declaration-signatures.md` describe source forms and signature construction. They are forward design, not proof that parser/runtime support exists.
- The user-provided [`ontology.md`](../../../spec/typing/ontology.md) is currently untracked. It supplies a coherent target: values, types, and kinds occupy distinct semantic levels; type constructors have arrow kinds; reflection reifies without collapsing those levels; higher-kinded constructors are expected. This implementation must be compatible with that model, but adoption of the document itself requires an explicit decision.

Nothing in this specification requires `Constraint` kinds, type-level computation, type lambdas, or HKT source syntax in the first release. It requires that the representation not paint Phalcom into a `Type :: Type` corner or conflate a type constructor with an ordinary runtime class object.

## 3. Goals and non-goals

### 3.1 Goals

1. Define canonical, session-owned static type terms and kinded constructors with explicit construction invariants.
2. Define separate, terminating algorithms for type equivalence, subtyping, assignment acceptance, gradual consistency, protocol conformance, and member lookup.
3. Make variance, substitution, binder identity, record mutability, recursion, and normalization rules explicit rather than inferred from Rust match arms.
4. Parse and lower a deliberately selected annotation language with source ranges, recovery, and stable unsupported-feature diagnostics.
5. Retain `TypeKnowledge` as an evidence envelope rather than using unknown/dynamic values as ordinary canonical type terms.
6. Supply deterministic unit, property, corpus, and adversarial tests sufficient to make later solver/module work depend on this kernel.

### 3.2 Non-goals

- Full type inference, flow-sensitive narrowing, overload search, protocol solving, or generic body checking. They consume this kernel in Specification 03.
- Altering how a selector is named, how `perform` works, or how a class object is represented at runtime.
- Implicitly choosing `Any`, `Dynamic`, or missing annotation behavior beyond decisions already ratified. The implementation must leave explicit policy points where design remains open.
- Treating `Type`, `Kind`, or reflected type objects as user-value hierarchy nodes. That would contradict the ontology’s stratification.

## 4. Formal model and required distinctions

### 4.1 Semantic levels

Phalcom must retain these different judgments:

```text
e ⇓ v                         evaluation of expression e to runtime value v
Δ; Γ ⊢ τ :: κ                type term τ has kind κ
Δ; Γ ⊢ τ ≡ σ                 τ and σ are equivalent canonical types
Δ; Γ ⊢ τ <: σ                τ is a subtype of σ
Δ; Γ ⊢ value : τ             optional runtime observation/classification
```

`v.class`, `T`, and the class of a class are different concepts. A static nominal type may name values constructed by a runtime class, while the type term itself is not automatically an instance of the runtime class hierarchy. Reflection may expose a value representing a type only through an explicit bridge with a stated kind and capability. The implementation must therefore not implement kind checking by inspecting runtime class values or make subtyping depend on a reflected object’s identity.

### 4.2 Canonical categories

The kernel shall distinguish at least the following categories in its API. They may share an arena internally, but public wrappers and query contracts must prevent accidental interchange.

| Category | Role | May occur in a fully published type? |
|---|---|---|
| `CanonicalTypeId` | Interned, store-owned static type term. | Yes. |
| `KindId` | Interned `Type` or arrow kind; later explicitly extensible. | Yes. |
| `TypeConstructorId` | Declaration-backed constructor with kind, arity, parameter descriptors, and revision. | Yes. |
| `TypeParameterId` | Persistent owner-qualified declaration binder, not a bare local integer. | Yes, only under an environment which binds it. |
| `InferVarId` | Ephemeral solver variable owned by an inference session/query. | No; must be solved, generalized, or reported as underconstrained before publication. |
| `TypeKnowledge` | Evidence envelope around a type fact, absence, or dynamic boundary. | Yes, as fact metadata; never a substitute for `CanonicalTypeId`. |
| `RelationOutcome` | `Yes`, `No(counterexample)`, or `Blocked(reason)` for a named relation. | Yes, as query result. |

`Unknown` means no sufficiently authoritative static fact currently exists. `Dynamic` means an explicit or trusted dynamic escape crosses a boundary. `Any`, if ratified, is a static top type with lattice laws; it is neither `Unknown` nor `Dynamic`. `Never` is static bottom and represents unreachable computation, not a missing result. These distinctions are compulsory because they control rejection, diagnostic suppression, caching, and later proof validity.

### 4.3 Canonical type grammar

The minimum internal grammar is:

```text
κ ::= Type | κ1 -> κ2

τ ::= Never | Unit | Any                       // Any only after ratification
    | Nominal(DeclarationId)
    | Apply(TypeConstructorId, [τ])
    | Union(CanonicalSet<τ>)
    | Tuple([TupleElement])
    | Record(CanonicalMap<FieldName, RecordField>)
    | Callable(CallableSignature)
    | Parameter(TypeParameterId)
    | Recursive(RecursiveBinderId, τ)          // introduced only when recursion is implemented
```

`Dynamic`, `Unknown`, unresolved names, parser error placeholders, and inference variables do not belong in this grammar’s published form. During analysis, inference variables live in a separate inference-session arena and are referenced through `InferenceType`; they cannot escape into `SemanticTypeSnapshot` without a deliberate publication result.

The first representation may retain the current `TypeData` storage and `TypeId` internals, but it must add a store/session identity at every cross-store boundary. A raw integer ID has meaning only inside one immutable `TypeStore` generation. Rust newtypes should encode this distinction rather than relying on callers to remember it.

### 4.4 Kinding and application

Every declaration exporting a type constructor shall expose a descriptor:

```rust
struct TypeConstructorDescriptor {
    id: TypeConstructorId,
    declaration: DeclarationId,
    kind: KindId,
    parameters: Arc<[TypeParameterDescriptor]>,
    result_kind: KindId,
    revision: InterfaceRevision,
}

struct TypeParameterDescriptor {
    id: TypeParameterId,
    owner: DeclarationId,
    index: u16,
    name: Symbol,
    variance: Variance,
    bound: Option<CanonicalTypeId>,
}
```

Construction of `Apply(C, args)` shall check that `C :: κ1 -> … -> κn -> Type`, that `args.len() == n`, and that every argument has the expected kind. Partial application is not silently represented as a proper type. If HKT use is deferred, constructors whose result is not `Type` may exist only in descriptors/tests and cannot be written by the parser; this preserves the architecture without promising syntax. A later `Constraint` kind is a new explicit kind constructor, not an overloaded `Type`.

The descriptor owns parameter identity. `TypeParameterId` must be based on `(owner declaration identity, zero-based declaration position)`, with display name kept for diagnostics only. Two textual `T`s never unify by spelling. Alpha-equivalence compares binder positions under a renaming environment; it does not compare `u32` allocation order.

## 5. Relation contracts

The present `is_subtype`/`is_assignable` pair must become named query contracts. A caller must not decide whether it wants gradual acceptance by calling raw subtyping and then special-casing `Dynamic`.

| Relation | Judgment | Purpose | Dynamic/unknown policy |
|---|---|---|---|
| Equivalence | `τ ≡ σ` | Canonical equality, cache keys, duplicate-union removal. | Neither side is knowledge state. |
| Subtyping | `τ <: σ` | Static semantic relation with proof-like laws. | No implicit dynamic success. Unresolved dependency yields `Blocked`. |
| Acceptance/assignability | `τ ⇐ σ` | Whether value of source type `σ` may initialize/flow to target `τ` under active typing mode. | Explicitly delegates to consistency only where mode permits. |
| Consistency | `τ ~ σ` | Gradual boundary compatibility, if `Dynamic` design is ratified. | Dynamic can be consistent without being a subtype. |
| Conformance | `τ ⊧ P` | Nominal/structural protocol obligation. | Deferred until protocol design; never inferred from member lookup. |
| Member lookup | `lookup(τ, selector, side)` | Resolves declared surface under inheritance/substitution. | Returns a reasoned missing/ambiguous/dynamic result. |

All relations return a total, diagnostic-ready result:

```rust
enum RelationOutcome {
    Yes(RelationEvidence),
    No(RelationFailure),
    Blocked(RelationBlocker),
}
```

`Blocked` covers cancelled work, unresolved interface, dynamic boundary where a strict proof is requested, recursive relation currently in progress, and configured budget exhaustion. It must not be folded into `Yes` or `No`. The enclosing checker decides whether a blocked relation becomes a suppressed diagnostic, warning, strict-mode error, or deferred query.

### 5.1 Required static laws

After the lattice decision is ratified, test these laws as executable properties:

- Reflexivity: `τ <: τ`.
- Transitivity for canonical, fully resolved types: if `τ <: σ` and `σ <: ρ`, then `τ <: ρ`.
- Bottom: `Never <: τ`; `Never` disappears from normalized unions.
- Top, if `Any` is adopted: `τ <: Any`; it does not make `Dynamic` a subtype.
- Union: `Union[Aᵢ] <: B` iff every `Aᵢ <: B`; `A <: Union[Bⱼ]` iff at least one `A <: Bⱼ`.
- Callable: parameter positions contravariant, return position covariant, labels/rest compatibility checked by callable contract.
- Declared variance only: covariant (`out`) arguments use `<:`, contravariant (`in`) reverse it, invariant arguments require equivalence. No default covariance.
- Mutable record fields and mutable generic surfaces are invariant unless a read-only capability makes covariance sound.
- Equivalence is deterministic across repeated construction in one store and does not depend on source traversal order for observable output.

### 5.2 Recursive types, canonicalization, and termination

Canonicalization must normalize only laws actually promised by the language. Flattening, sorting, deduplicating, and removing `Never` from a union are sound operations. Sorting by transient `TypeId` is not a cross-snapshot semantic ordering; it may remain an arena-local optimization only if rendering/caching uses stable structural or declaration keys. Do not distribute unions through all constructors, erase tuple labels, reorder records without preserving field identity, or normalize recursive types by unbounded expansion.

Introduce a `RelationContext` carrying memo entries keyed by `(relation kind, left, right, substitution environment)`, a recursion stack, node budget, and cancellation token. Coinductive acceptance of recursive nominal relationships must be specified before it is implemented. Until then, encountering a cycle returns `Blocked(RecursiveRelation)` rather than recursing forever or accepting by accident. Equality and display both need size/depth limits; a pretty-printer is not allowed to cause a non-terminating diagnostic.

## 6. Annotation syntax, parsing, and lowering

### 6.1 Committed first syntax surface

Subject to parser syntax compatibility, the first source release shall support this intentionally bounded set:

```text
TypeAnnotation ::= TypeUnion
TypeUnion       ::= TypePrimary ('|' TypePrimary)*
TypePrimary     ::= QualifiedName TypeArguments?
                  | 'Never' | 'Unit' | 'Any' | 'Dynamic'
                  | '(' TypeTupleOrGrouped ')'
                  | CallableType
TypeArguments   ::= '[' TypeAnnotation (',' TypeAnnotation)* ']'
TypeTupleOrGrouped ::= TypeAnnotation (',' TypeAnnotation)* ','?
CallableType    ::= '(' CallableParameters? ')' '->' TypeAnnotation
```

`Any` enters the grammar only when its semantic decision is ratified. `Dynamic` must be explicit source syntax; absence of annotation remains absence of declared evidence. Type lambdas, higher-kinded parameter syntax, intersections, associated types, refinement types, and type-level functions are excluded from this release, even though the kernel preserves a path for them.

The grammar must resolve precedence before parser implementation: application binds tighter than union; callable return annotation extends to the right; parentheses distinguish grouping from a one-element tuple; tuple labels and callable labels have separate token productions. The parser must retain enough error nodes/ranges to issue one primary annotation diagnostic instead of cascades after a missing `]`, `)`, or `->`.

### 6.2 Lowering contract

Lowering is a query from parsed annotation plus module-header environment to:

```rust
enum LoweredAnnotation {
    Resolved { ty: CanonicalTypeId, evidence: Evidence },
    Error { diagnostic: SemanticDiagnostic, placeholder: InvalidAnnotationId },
    Blocked { reason: AnnotationBlocker },
}
```

It resolves names through imports, aliases once implemented, local generic binders, builtins, and declaration headers. It checks kind and application arity at the application range, records every resolved declaration/interface dependency, and associates source provenance with the resulting canonical type. Lowering never returns a fabricated `Dynamic` to recover from an unresolved name. Error recovery may use an internal invalid placeholder to keep body analysis running, but that placeholder cannot be interned as a normal canonical type or used to prove an assignment.

`SimpleTypeResolver` is acceptable as a bootstrap test harness. It is not the production resolver because it is string-keyed, has no import/module identity, cannot represent shadowing/aliases/generic binders, and cannot produce dependency edges.

## 7. Native type surface requirements

Native metadata must converge on the same descriptors, type-expression lowering, and relation engine used by source declarations. The present normalizer preserves a small subset and makes other native forms opaque/unknown ([`native.rs`](../../../../phalcom-semantic/src/types/native.rs)). That is safe as a temporary dynamic boundary but prevents native APIs from being authoritative generic surfaces.

The implementation shall:

1. Version native surface descriptors by ABI/library revision and give every type/callable/member a stable declaration identity.
2. Lower native type expressions into the same canonical grammar with parameter environments; unsupported metadata produces `TrustedNative`/opaque evidence and an explicit capability boundary.
3. Forbid hardcoded special cases from assigning semantics that native metadata cannot describe. Transitional shims, such as `List.add`, must be isolated behind a feature/compatibility test and removed when metadata is authoritative.
4. Test that core source declarations and native descriptors agree on selector labels, receiver side, generic arity, mutability/capability, and declared result types.

## 8. Implementation sequence

### Phase 1 — make illegal kernel states unrepresentable

- Split persistent canonical IDs from inference-session IDs; add store/session identity to cross-query handles.
- Add `TypeConstructorDescriptor`, owner-qualified `TypeParameterId`, `Variance`, and a declaration/header environment interface.
- Make application constructors fallible or return an explicit construction diagnostic when kind/arity fails. Remove `kind_of` fallback-to-`Type` behavior from production paths.
- Preserve the existing `TypeStore` interning and union normalization tests, then add deterministic rendering/ordering tests.

**Exit criterion:** no production code can construct a published `Apply` without a validated descriptor, and no generic parameter is identified solely by an integer or name.

### Phase 2 — separate and harden relations

- Replace boolean-like helpers with query-specific `RelationOutcome` APIs and provenance/failure payloads.
- Implement declared variance; mark all existing type parameters invariant until their descriptors say otherwise. Audit mutable native collections before assigning covariance.
- Add substitution maps keyed by owner-qualified parameter IDs and capture-safe binder traversal.
- Add relation memoization, recursion/budget/cancellation policy, and stable diagnostics for blocked outcomes.

**Exit criterion:** a test can demonstrate that `List[Int]` is not assignable to `List[Any]` under an invariant/mutable `List`, while a read-only explicitly covariant abstraction is accepted.

### Phase 3 — complete selected annotation pipeline

- Add grammar productions and recovery tests for applications, unions, tuples, and callable annotations.
- Replace string-only annotation resolution with a header-environment query interface. This interface is implemented fully in Specification 02; until then it may have a one-program adapter.
- Implement kind, arity, missing-name, and unsupported-feature diagnostics with stable codes and ranges.
- Keep unstable HKT/type-lambda syntax behind an explicit parser feature, or omit it entirely.

**Exit criterion:** source annotations round-trip through parse/lower/render with correct binding, no panic, and no fallback to `Dynamic` for an unresolved/ill-kinded source type.

## 9. Tests and acceptance evidence

| Test family | Required cases |
|---|---|
| Canonical store | Interning, unions, records/callables labels, stable display, store-identity misuse rejection. |
| Kinding | Correct `F[A]`, arity under/over-application, argument kind mismatch, unavailable partial application, future HKT descriptor regression. |
| Relations | Lattice laws, variance counterexamples, callable labels/rest, record mutability, unions, recursive/budget/cancelled blockers. |
| Substitution | Shadowed binders, same names/different owners, alpha-equivalence, nested applications, no capture. |
| Parser/lowering | Precedence, recovery, imports/local binders, range precision, invalid annotations remain invalid rather than dynamic. |
| Native/source parity | Core generic surfaces, setter/write invariance, side/selector labels, opaque-native boundary. |
| Adversarial | Deep unions, recursive aliases/descriptors, repeated app construction, huge error types, cancellation during relation/display. |

Add property tests only after a small, hand-checked oracle corpus exists. A property such as transitivity must generate only well-kinded fully resolved types; testing an undefined relation on unresolved/inference terms does not establish soundness.

## 10. Pyrefly transfer: direct, adapted, rejected

**Take directly:** canonical type storage; separation of canonical descriptors from temporary inference state; explicit equality/normalization limits; stable identity discipline; result-rich query states; regression/performance corpus around pathological types. The transfer specifically calls for deterministic bounded normalization and semantic equality rather than raw allocator identity ([executive report](../pyrefly-transfer/executive-report.md)).

**Adapt:** Pyrefly’s type-store and query ideas must accept Phalcom type constructors/kinds, class-side versus instance-side member surfaces, selector labels, native metadata, and future HKT descriptors. Use a safe Rust arena/table implementation and immutable snapshot ownership rather than copying a foreign runtime or import model.

**Reject:** Python `Any` semantics, Python protocols/overloads/descriptors, semantic behavior based on Python attribute lookup, and any performance shortcut that leaks unbounded inference terms or unsafe shared answer slots into published type facts. A Phalcom `Dynamic` boundary remains an explicit language choice, not a renamed Python `Any`.

## 11. What this must not preclude

- Higher-kinded constructors and a later `Constraint` kind.
- Nominal class types, protocol types, class-object/metaclass types, and reflected type values being representable without identifying them.
- Explicit variance, bounds, F-bounds, type aliases, intersections, associated types, ADTs, and existential/opaque forms in later documents.
- Incremental, parallel-safe query evaluation in Specification 02.
- A future proof layer needing canonical, provenance-bearing type facts rather than ad hoc printed strings.

## 12. Risks and open decisions

The highest risk is exposing generic syntax before parameter identity, variance, and substitution are correct. The second is assuming all existing generic/native collections are covariant because the current relation implementation does so. The third is canonically interning error, dynamic, or inference placeholders and then allowing them to escape into cache keys. Each creates later compatibility debt that is much harder to remove than finishing this kernel first.

Before merge, record decisions for `Any`, first generic syntax, mutable collection variance, recursive type policy, and whether type aliases are transparent/equivalent or nominal. If a decision remains open, retain a named `Unsupported` or `Blocked` result and a diagnostic; do not use `Dynamic` as a temporary answer.

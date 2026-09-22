---
id: LANG007.C2.P3
category: LANG
program: LANG007
checkpoint: LANG007.C2
plan: LANG007.C2.P3
kind: requirements-analysis
status: PROPOSED
prepared: 2026-09-22
repository: aureat/phalcom-lang
branch: main
repository_baseline_revision: 7da5b04378dbd68fc20df6cda9a890cce0e0afa0
repository_baseline_subject: "feat(lang007): implement variadic generic binding and structural type expansion (LANG007.C2.P2)"
previous_plan: LANG007.C2.P2
previous_plan_status: COMPLETE
next_plan: LANG007.C2.P3
---

# LANG007.C2.P3 — Requirements Analysis

## Whole-Pack Inference, Durable Persistence, Incremental Certification, and C2 Closure

## 0. Executive determination

`LANG007.C2.P3` is the checkpoint-closing plan for `LANG007.C2`, but the live post-P2 repository shows that it is **not merely a broad test-and-bookkeeping plan**.

P1 and P2 have established the canonical pack algebra and the complete source-facing syntax/elaboration vertical. However, a fresh audit of post-P2 `main` identifies **two concrete implementation gaps that P3 must close before certification can be truthful**:

1. **The inference term calculus does not recursively represent the new C2 compound forms.**  
   `InferenceSession::type_id_to_inference` and `type_id_to_inference_with_rows` recursively lift ordinary `Applied`, `Tuple`, `Callable`, `Record`, `Family`, and associated-projection structure, but C2 pack terms currently fall through to `InferenceTerm::Canonical`. As a result, a generic parameter nested inside `TypePackConcat`, `LabeledTypePackConcat`, `CompleteTypePack`, `TupleFromPacks`, `CallableFromPacks`, or a symbolic lane projection is not necessarily replaced by its query-local `InferVarId`. Whole-pack inference through those structures therefore cannot yet be claimed checkpoint-complete.

2. **Durable metadata cannot export the scoped pack algebra created by P2, and type-lambda metadata does not preserve binder modes.**  
   `phalcom-semantic/src/metadata/export.rs::export_scoped_type` explicitly rejects all scoped C2 pack forms with `MetadataExportError::ScopedPackForm`. In addition, the live stable `TypeLambdaRef` and nested `ScopedTypeNode::Lambda` retain only parameter kinds and body; they do not retain `GenericParameterMode`. Therefore a variadic transparent alias/type lambda cannot yet be faithfully serialized as a durable public semantic product.

Everything else in P3 should be organized around closing and certifying the architecture that already exists. P3 must **not** introduce a second pack model, a sequence-decomposition solver, a persistent `PackVarId`, new pack syntax, or any runtime pack ABI.

The correct P3 objective is:

> **Complete the inference and persistence integration of P1/P2's canonical pack model, then prove checkpoint-wide cold/incremental parity, hostile symbolic behavior, fixed-generic compatibility, bounded resource behavior, source identity, and the compile-time/runtime boundary before closing LANG007.C2.**

The intended dependency order is:

```text
P1 canonical pack algebra
        ↓
P2 source elaboration / aliases / Tuple / callable domains
        ↓
P3.1 inference-term closure
        ↓
P3.2 durable scoped metadata closure
        ↓
P3.3 cross-module / incremental certification
        ↓
P3.4 hostile + density + compatibility certification
        ↓
P3.5 runtime-negative proof + lifecycle closure
```

---

# 1. Live repository baseline

The requirements analysis is grounded in remote `main` at:

```text
7da5b04378dbd68fc20df6cda9a890cce0e0afa0
feat(lang007): implement variadic generic binding and structural type expansion (LANG007.C2.P2)
```

P2 is recorded as:

```text
COMPLETE
IMPLEMENTED
FOCUSED_TESTED
```

and the C2 checkpoint currently names:

```text
next_plan: LANG007.C2.P3
```

P2's handoff defines P3 as:

```text
Whole-Pack Inference, Persistence/Incremental Certification,
and Checkpoint Closure
```

with five takeover obligations:

1. whole-pack inference validation/materialization across all generic paths;
2. cross-module metadata round-trip and bundle validation;
3. checkpoint-wide incremental/cold parity and invalidation;
4. density/hostile/fixed-generic compatibility verification;
5. C2 closure and handoff to the next checkpoint.

That decomposition remains correct, but the live audit refines the first two items from "certify existing behavior" into bounded implementation work.

---

# 2. Frozen P1/P2 takeover

P3 must consume the following as established architecture.

## 2.1 Canonical semantic domains

```text
TypePack
LabeledTypePack
CompleteTypePack
```

are distinct kinded `TypeId` domains.

`CompleteTypePack` is not a proper Tuple type and is not a runtime pack.

Canonical terms include:

```rust
TypeData::TypePack(...)
TypeData::LabeledTypePack(...)
TypeData::CompleteTypePack {
    positionals,
    labeled,
}

TypeData::TypePackConcat(...)
TypeData::LabeledTypePackConcat(...)

TypeData::TupleFromPacks {
    positionals,
    labeled,
}

TypeData::CallableFromPacks {
    positionals,
    labeled,
    return_type,
}
```

P3 must not replace them.

## 2.2 Lane operators

The frozen language law remains:

```text
*    = positional lane
**   = labeled lane
***  = complete two-lane pack
```

Grammar context changes whether the operator captures, expands, or unpacks. It does not change lane meaning.

`...` remains independent callable-rest syntax and is **not** C2 pack syntax.

## 2.3 Binder semantics

Canonical binder mode is:

```rust
GenericParameterMode::{
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}
```

and is distinct from parameter kind.

In particular:

```text
Single(TypePack) != PositionalPack(TypePack)
```

P3 must preserve mode in all durable identities, inference contracts, metadata, fingerprints, and incremental dependency decisions.

## 2.4 Binder-aligned application

Source arity does not survive into canonical applied-type identity.

For:

```phalcom
class C<T, *Ts> {}
```

source:

```phalcom
C<Int, String, Bool>
```

canonically means:

```text
Applied(
    C,
    [
        Int,
        TypePack[String, Bool]
    ]
)
```

There is no `VariadicAppliedType`.

## 2.5 Staged generic declaration publication

P2 established:

```text
GenericBinderHeader
```

as the early declaration-shell application contract, followed by later full `GenericSignature` realization.

This separation exists to let aliases consume binder kinds/modes without forcing alias-dependent generic constraints to be lowered too early.

P3 must certify this architecture; it must not collapse it back into a fake partial `GenericSignature`.

## 2.6 Scoped type-lambda algebra

P2 added scoped counterparts for the C2 algebra:

```rust
ScopedTypeData::TypePack
ScopedTypeData::LabeledTypePack
ScopedTypeData::CompleteTypePack
ScopedTypeData::TypePackConcat
ScopedTypeData::LabeledTypePackConcat
ScopedTypeData::TupleFromPacks
ScopedTypeData::CallableFromPacks
ScopedTypeData::PackProjection
```

and `TypeLambdaData` now contains:

```rust
parameter_kinds
parameter_modes
body
result_kind
```

Residual partial beta reduction preserves both kinds and modes.

P3 owns persistence and inference certification of this model, not a replacement representation.

## 2.7 Checked rewriting/materialization

P1/P2 already established:

```text
TypeSubstitution::try_apply
materialize_type
```

as fallible C2-aware canonical rewrite paths.

The live `materialize_type_inner` already recursively handles:

```text
TypePack
LabeledTypePack
TupleFromPacks
CallableFromPacks
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TypeMemberProjection
```

and re-enters the canonical constructors.

That is an existing strength. P3 should test it through real inference and cross-module paths rather than reimplement it.

---

# 3. P3 is both closure work and bounded missing implementation

| Area | Live post-P2 state | P3 role |
|---|---|---|
| parser / AST | implemented | regression only |
| generic binder legality | implemented | regression only |
| binder-aligned generic application | implemented | regression/incremental certification |
| Tuple expansion | implemented | inference/persistence certification |
| callable pack expansion | implemented | inference/persistence certification |
| scoped pack algebra | implemented in semantic arena | durable metadata completion |
| canonical substitution/materialization | implemented | integrated inference/hostile certification |
| basic pack `InferVarId(kind)` allocation | implemented | certify across all C2 structures |
| recursive inference lifting of C2 forms | incomplete | **implementation required** |
| scoped pack metadata | explicitly rejected | **implementation required** |
| type-lambda binder-mode metadata | absent | **implementation required** |
| cross-module source forwarding | focused-tested | broader persistence/parity certification |
| incremental foundations | partially/focused-tested | checkpoint-wide matrix |
| runtime boundary | appears preserved | static + integration certification |
| lifecycle/roadmap | C2 still open | closure |

---

# 4. Proven implementation gap A — inference-term closure

## 4.1 Current inference model

The live query-local inference calculus contains:

```rust
InferenceTerm::Canonical(TypeId)
InferenceTerm::Var(InferVarId)
InferenceTerm::Rigid(...)
InferenceTerm::Applied { ... }
InferenceTerm::ExactCase { ... }
InferenceTerm::Union(...)
InferenceTerm::Tuple(...)
InferenceTerm::Callable(...)
InferenceTerm::Record(...)
InferenceTerm::Family(...)
InferenceTerm::AssociatedProjection { ... }
```

Generic parameters are instantiated using their declared kind:

```rust
let var = self.fresh_variable_in_frame(
    frame,
    store.type_parameter(param).kind,
);
```

This is the correct C2 foundation.

Therefore these are already valid query-local domains:

```text
InferVarId(kind TypePack)
InferVarId(kind LabeledTypePack)
InferVarId(kind CompleteTypePack)
```

No new metavariable class is required.

## 4.2 Current lifting gap

`type_id_to_inference` recursively replaces generic parameters only inside the compound forms it explicitly understands.

The C2 forms are currently reached by the wildcard arm and become:

```rust
InferenceTerm::Canonical(ty)
```

as one opaque term.

That means a signature form conceptually equivalent to:

```text
TupleFromPacks(P, [])
```

where:

```text
P : TypePack
```

may retain canonical parameter `P` inside an opaque `TypeId` rather than becoming a solver-local structure containing:

```text
Var(InferVarId(kind TypePack))
```

The same issue applies to:

```text
CompleteTypePack(P, L)
TypePackConcat(P, Q)
LabeledTypePackConcat(L1, L2)
CallableFromPacks(P, L, R)
Positionals(Q)
Labeled(Q)
```

when they contain generic parameters instantiated for one inference session.

This is the principal P3 inference implementation requirement.

---

# 5. Whole-pack inference semantics

## 5.1 Meaning of "whole-pack inference"

C2 permits inference of a pack as **one semantic term**.

For example, if a generic parameter is used as one canonical argument:

```text
Applied(C, [P])
```

and the actual type is:

```text
Applied(C, [TypePack[Int, String]])
```

then the solver may infer:

```text
P = TypePack[Int, String]
```

because the entire `TypePack` value is matched to one unknown of kind `TypePack`.

Likewise:

```text
L : LabeledTypePack
Q : CompleteTypePack
```

may each be solved to one canonical value of the corresponding exact kind.

This is not pack decomposition.

## 5.2 Allowed structural projection for whole-lane inference

Where an accepted C2 type form semantically exposes an entire lane, P3 may infer the whole lane without sequence partitioning.

For example, conceptually:

```text
TupleFromPacks(P, [])
    == (Int, String)
```

may infer:

```text
P = TypePack[Int, String]
```

if the implementation compares the **entire positional lane** against the single unknown `P`.

Similarly:

```text
CallableFromPacks(P, L, R)
```

may constrain whole positional/labeled lane unknowns against a concrete callable domain when the correspondence is lane-exact.

This is lane projection, not prefix/suffix search.

## 5.3 Forbidden decomposition

P3 must not solve:

```text
TypePackConcat(P, [String])
    == [Int, Bool, String]
```

by deducing:

```text
P = [Int, Bool]
```

That is pack-sequence subtraction / decomposition and remains outside C2.

Likewise, P3 must not solve unknown partitions such as:

```text
Concat(P, Q) = ConcretePack
```

by trying arbitrary split points.

No backtracking over pack lengths is permitted.

## 5.4 No runtime-call variadic inference expansion by implication

P3's whole-pack inference requirement does **not** automatically authorize a new runtime-call algorithm that infers a variadic type pack from an arbitrary count of value arguments.

That is a distinct future feature unless the existing callable checker already produces one canonical pack-valued evidence term without sequence partitioning.

The safe rule is:

> P3 may infer one pack variable from one semantically whole pack/lane constraint. It must not invent call-argument sequence collection or pack splitting merely to infer a variadic generic binder.

---

# 6. Required inference representation behavior

P3 must make every source-reachable C2 compound form capable of exposing query-local variables recursively.

The exact Rust representation is flexible, but the semantic requirement is not.

An acceptable design would extend `InferenceTerm` with equivalents of:

```rust
TypePack(Box<[InferenceTerm]>)

LabeledTypePack(Box<[InferenceLabeledPackElement]>)

CompleteTypePack {
    positionals: Box<InferenceTerm>,
    labeled: Box<InferenceTerm>,
}

TypePackConcat(Box<[InferenceTerm]>)

LabeledTypePackConcat(Box<[InferenceTerm]>)

TupleFromPacks {
    positionals: Box<InferenceTerm>,
    labeled: Box<InferenceTerm>,
}

CallableFromPacks {
    positionals: Box<InferenceTerm>,
    labeled: Box<InferenceTerm>,
    return_type: Box<InferenceTerm>,
}

PackProjection {
    subject: Box<InferenceTerm>,
    lane: ...
}
```

or a smaller equivalent term family that preserves exactly the same information.

P3 does **not** have to mirror every `TypeData` variant one-for-one if a more compact solver-local form is already architecturally preferred.

However, it must satisfy all of these laws:

```text
generic parameter under C2 node -> solver variable remains reachable
kind is preserved exactly
materialization re-enters TypeStore canonical constructors
concrete canonicalization still occurs
symbolic terms stay symbolic when not reducible
invalid labeled overlap remains fallible
no query-local variable enters TypeStore
```

---

# 7. Inference materialization requirements

The solver-local materializer must reconstruct every new inference C2 term through canonical store APIs:

```text
TypePack                -> TypeStore::type_pack
LabeledTypePack         -> TypeStore::labeled_type_pack
CompleteTypePack        -> TypeStore::complete_type_pack
TypePackConcat          -> TypeStore::type_pack_concat
LabeledTypePackConcat   -> TypeStore::labeled_type_pack_concat
TupleFromPacks          -> TypeStore::tuple_from_packs
CallableFromPacks       -> TypeStore::callable_from_packs
lane projection         -> TypeStore projection authority / normalizer
```

It must not directly intern malformed raw nodes.

The materializer must preserve normalization:

```text
all-concrete TypePackConcat
    -> concrete TypePack

all-concrete valid LabeledTypePackConcat
    -> concrete LabeledTypePack

concrete duplicate labeled contributions
    -> error

concrete TupleFromPacks
    -> Tuple / Unit

concrete CallableFromPacks
    -> Callable
```

---

# 8. Inference projection requirements

Opaque complete packs introduced canonical residual lane projections:

```text
Positionals(Q)
Labeled(Q)
```

where:

```text
Q : CompleteTypePack
```

The inference layer must not freeze:

```text
Positionals(P_generic)
```

as an opaque canonical `TypeId` containing the original declaration parameter while the rest of that declaration is instantiated with query-local variables.

P3 must either represent pack projections directly in `InferenceTerm` or defer them in an equivalent solver-local form whose subject can be the corresponding `InferVarId`.

Once the subject is solved, projection normalization must reduce through the existing TypeStore projection authority.

P3 must not add a second projection semantics specific to inference.

---

# 9. Inference unification boundary

## 9.1 Required equality behavior

Whole-pack variables must bind by exact kind:

```text
Var(TypePack)         ↔ TypePack term
Var(LabeledTypePack)  ↔ LabeledTypePack term
Var(CompleteTypePack) ↔ CompleteTypePack term
```

A kind mismatch remains a hard inference failure.

## 9.2 Structural comparison

C2 compound terms may be compared recursively only where their structure establishes an unambiguous correspondence.

For example:

```text
CompleteTypePack(P1,L1)
vs
CompleteTypePack(P2,L2)
```

can compare corresponding lanes.

A Tuple or callable with a known complete lane decomposition may constrain a whole pack variable through exact lane projection.

## 9.3 No sequence algebra solver

The solver must not gain general rules such as:

```text
Concat(X,A) == B  => solve X
Concat(A,X) == B  => solve X
length(X) = ...
slice(B, ...) = ...
```

Those are explicitly out of scope.

---

# 10. Generic instantiation remains the publication authority

After inference solves whole packs, durable substitution should still flow through existing `GenericInstantiation`.

No new binding table is needed.

Conceptually:

```text
InferVarId(TypePack) solved to TypePack[Int,String]
        ↓
projected generic solution
        ↓
GenericInstantiation::bind_type(parameter, solved_pack)
        ↓
materialize_type(...)
        ↓
canonical P1/P2 term
```

P3 must not introduce:

```text
PackInstantiation
PackBindingMap
PackSolutionId
```

as a parallel authority.

---

# 11. Proven implementation gap B — scoped metadata persistence

## 11.1 Current stable canonical nodes

The live stable metadata graph already contains canonical nodes for:

```rust
TypeNode::TypePack
TypeNode::LabeledTypePack
TypeNode::CompleteTypePack
TypeNode::TypePackConcat
TypeNode::LabeledTypePackConcat
TypeNode::TupleFromPacks
TypeNode::CallableFromPacks
TypeNode::TypeMemberProjection
```

Canonical C2 type terms are therefore serializable.

## 11.2 Current scoped metadata gap

The live stable `ScopedTypeNode` currently supports:

```text
Bound
Free
Applied
Union
Tuple
Record
Callable
Lambda
OpenRecord
```

but does not define stable forms corresponding to P2's scoped pack algebra.

The exporter explicitly rejects:

```rust
ScopedTypeData::TypePack(_)
ScopedTypeData::LabeledTypePack(_)
ScopedTypeData::CompleteTypePack { .. }
ScopedTypeData::TypePackConcat(_)
ScopedTypeData::LabeledTypePackConcat(_)
ScopedTypeData::TupleFromPacks { .. }
ScopedTypeData::CallableFromPacks { .. }
ScopedTypeData::PackProjection { .. }
```

through:

```text
MetadataExportError::ScopedPackForm
```

Therefore:

```phalcom
type AddHead<*Ts> = (Int, *Ts)
```

can exist in the semantic arena but its lambda body cannot currently be faithfully exported as stable scoped metadata.

P3 must close this gap.

---

# 12. Stable scoped C2 node requirements

The durable scoped graph needs structural equivalents for all P2 scoped forms that can occur in public type lambdas/transparent aliases.

Conceptually:

```rust
ScopedTypeNode::TypePack(Box<[ScopedTypeNodeId]>)

ScopedTypeNode::LabeledTypePack(
    Box<[ScopedLabeledTypePackElementRef]>
)

ScopedTypeNode::CompleteTypePack {
    positionals: ScopedTypeNodeId,
    labeled: ScopedTypeNodeId,
}

ScopedTypeNode::TypePackConcat(Box<[ScopedTypeNodeId]>)
ScopedTypeNode::LabeledTypePackConcat(Box<[ScopedTypeNodeId]>)

ScopedTypeNode::TupleFromPacks {
    positionals: ScopedTypeNodeId,
    labeled: ScopedTypeNodeId,
}

ScopedTypeNode::CallableFromPacks {
    positionals: ScopedTypeNodeId,
    labeled: ScopedTypeNodeId,
    return_type: ScopedTypeNodeId,
}

ScopedTypeNode::PackProjection {
    subject: ScopedTypeNodeId,
    lane: ScopedPackProjectionLaneRef,
}
```

Exact names are flexible.

Ordered children, labels, projection lane, and result kind must participate in stable structural identity.

---

# 13. Type-lambda binder modes must be stable metadata

P2 correctly made `parameter_modes` part of in-memory `TypeLambdaData`.

The stable metadata currently does not retain it.

Live structures conceptually remain:

```rust
TypeLambdaRef {
    parameter_kinds,
    body,
}

ScopedTypeNode::Lambda {
    parameter_kinds,
    body,
}
```

This is insufficient because:

```text
Single(TypePack)
```

and:

```text
PositionalPack(TypePack)
```

have the same parameter kind but different application semantics.

P3 must persist parameter modes for:

1. top-level `TypeLambdaRef`;
2. nested `ScopedTypeNode::Lambda`;
3. structural fingerprints of those lambda forms.

Mode count must equal parameter-kind count.

---

# 14. Metadata schema/version requirement

The live metadata schema is currently v3.

P3 should treat durable scoped pack forms and type-lambda binder modes as a **new stable semantic capability**, not silently reinterpret schema v3.

A consumer that claims to understand a schema version must not silently discard a variadic type-lambda binder mode or reject a public scoped pack body that another producer considers valid under that same version.

Accordingly, P3 should introduce a deliberate schema revision, expected to be:

```text
TYPE_METADATA_SCHEMA_VERSION = 4
```

unless repository-wide metadata policy discovered during implementation provides an equally explicit compatibility mechanism.

Recommended compatibility:

```text
schema v1/v2
    historical pre-C2 forms only

schema v3
    canonical C2 pack nodes and stable generic parameter modes
    but no scoped C2 lambda forms / variadic lambda mode contract

schema v4
    scoped C2 forms
    TypeLambda parameter modes
    nested lambda parameter modes
```

A new reader should continue to decode valid historical v1-v3 bundles.

A v3 bundle must not claim a variadic lambda mode it cannot represent.

---

# 15. Stable metadata validation laws

The validator must check, at minimum:

```text
Scoped TypePack elements       : proper Type
Scoped LabeledTypePack payload : proper Type
Scoped CompleteTypePack.pos    : TypePack
Scoped CompleteTypePack.lab    : LabeledTypePack

Scoped TypePackConcat segments
    : TypePack

Scoped LabeledTypePackConcat segments
    : LabeledTypePack

Scoped TupleFromPacks.pos
    : TypePack

Scoped TupleFromPacks.lab
    : LabeledTypePack

Scoped CallableFromPacks.pos
    : TypePack

Scoped CallableFromPacks.lab
    : LabeledTypePack

Scoped CallableFromPacks.return
    : Type
```

Pack projection result kinds must match the requested lane.

The validator must preserve topological ordering and lambda-scope correctness.

For lambda modes:

```text
mode count == parameter kind count
PositionalPack -> TypePack
LabeledPack    -> LabeledTypePack
CompletePack   -> CompleteTypePack
```

---

# 16. Metadata structural fingerprints

All new durable forms must receive unique structural fingerprint tags.

Do not reuse or renumber existing tags.

For ordered structures, fingerprint ordered children.

For example:

```text
ScopedTypePackConcat:
    tag
    count
    child fingerprints in order

ScopedLabeledTypePack:
    tag
    count
    label + child fingerprint in order

ScopedPackProjection:
    tag
    lane discriminator
    subject fingerprint

TypeLambdaRef:
    ordered parameter-kind fingerprints
    ordered parameter-mode discriminators
    body fingerprint
```

Mode-only lambda changes must change the structural/public fingerprint.

---

# 17. Metadata round-trip meaning

`phalcom-type-meta` currently owns:

```text
SemanticMetadataBundle
JSON encode
JSON decode
bundle validation
runtime/public metadata consumption
```

There is not currently a general "rebuild a semantic TypeStore from metadata" importer.

P3 does **not** need to invent a semantic-store deserializer merely to satisfy "round-trip."

Required proof is:

```text
semantic export
    ↓
SemanticMetadataBundle
    ↓
encode_metadata_json
    ↓
decode_metadata_json + validate_metadata_bundle
    ↓
stable node/mode/fingerprint equality
```

Where an existing cross-module/runtime metadata consumer exists, prove that it accepts the resulting bundle without interpreting pack nodes as runtime pack values.

---

# 18. Cross-module persistence requirements

Representative provider:

```phalcom
class Target<*Ts> {}

type AddHead<*Ts> = (Int, *Ts)

type FunctionOf<***P> = (***P,) -> Result
```

Public semantic metadata must retain:

```text
Target's PositionalPack mode

AddHead's TypeLambda:
    parameter kind TypePack
    parameter mode PositionalPack
    scoped TupleFromPacks body

FunctionOf's TypeLambda:
    parameter kind CompleteTypePack
    parameter mode CompletePack
    scoped pack projection / CallableFromPacks body
```

Encoding and decoding must not turn these into fixed generic constructors.

---

# 19. Query-local inference must never cross the metadata boundary

P3 must retain:

```text
InferVarId != stable semantic identity
```

No solver-local identifier may enter:

```text
TypeNode
ScopedTypeNode
TypeParameterRecord
GenericSignatureRecord
TypeLambdaRef
SemanticMetadataBundle
incremental snapshot fingerprint
```

Before durable publication, inference must materialize to canonical `TypeId` or block/reject publication according to existing product policy.

---

# 20. Incremental certification is a P3 acceptance requirement

Cold and incremental analysis must converge on:

```text
canonical type structure
diagnostics
generic binder modes
metadata fingerprints
source-index targets
alias forms
Tuple/callable pack forms
```

for the same final source.

---

# 21. Required incremental edit matrix

## 21.1 Binder mode edits

Certify legal transitions around:

```text
T
*Ts
**Ls
***P
```

Required effects:

```text
declaration interface fingerprint changes
semantic parameter/version changes where required
dependent applications invalidate
dependent aliases invalidate
unrelated declarations remain reusable
cold/incremental final state matches
```

## 21.2 Single(TypePack) versus PositionalPack(TypePack)

Edit:

```phalcom
class C<P: TypePack> {}
```

to:

```phalcom
class C<*P> {}
```

must be observed as a public semantic change even though the parameter kind remains `TypePack`.

## 21.3 Alias forwarding edits

Provider:

```phalcom
class Target<*Ts> {}
```

consumer:

```phalcom
type Forward<*Ts> = Target<*Ts>
```

Binder contract changes in `Target` must invalidate `Forward` through the P2 alias-contract dependency index.

Alias-cycle dependency tracking remains separate.

## 21.4 Pack composition edits

Changes inside positional/labeled concat structure must produce the same final canonical state as a cold build.

## 21.5 Tuple and callable edits

Pack expansion changes in Tuple and callable-domain types must invalidate their semantic and metadata products correctly.

## 21.6 Duplicate-label transitions

Incremental transition:

```text
unknown overlap -> known disjoint
```

or:

```text
unknown overlap -> known duplicate
```

must match cold formation exactly.

---

# 22. Incremental dependency precision

P3 should prove both correctness and boundedness.

A relevant binder-mode edit should invalidate:

```text
the declaration header/product
applications that consume the contract
variadic aliases that depend on the contract
metadata/public fingerprint products
affected source/advisory products
```

It should not force indiscriminate workspace invalidation.

The normal path must remain dependency-directed.

---

# 23. Source-index parity requirements

For:

```phalcom
class C<*Ts> {}
type F<*Us> = C<*Us>
```

source targets must remain canonical:

```text
binder name Ts -> TypeParameterId
binder name Us -> TypeParameterId
reference Us inside *Us -> same TypeParameterId
```

The `*` token is syntax, not a second semantic symbol.

Equivalent requirements apply to `**` and `***`, Tuple expansion operands, callable-domain expansion operands, and generic application expansion operands.

---

# 24. Hostile labeled-pack certification

## Immediate duplicate

```text
[name: String] ++ [name: Int]
```

must fail.

## Deferred duplicate

```text
L1 ++ L2
```

with unknown keys remains residual.

If substitution later yields duplicate labels, checked rewriting must become invalid.

It must not remain symbolic, pick a winner, convert to `RecordRow`, or drop a label.

## Deferred disjointness

If substitution proves disjoint packs, concat should canonicalize to one concrete ordered `LabeledTypePack`.

---

# 25. Density and resource behavior

Required complexity expectations:

```text
positional concat
    O(segments + concrete elements)

labeled concat
    O(segments + concrete entries)

complete formation
    O(lane sizes)

binder-aligned application
    O(binders + contributions)

metadata export/validation
    O(reachable nodes)
```

Forbidden:

```text
exponential partition search
recursive binary concat pathology
permutation search over labeled entries
whole-workspace invalidation by construction
```

A practical stress case can use dozens or hundreds of segments/elements. The important properties are flat structure, linear node growth, and completion under ordinary query budgets.

---

# 26. Fixed-generic compatibility matrix

Existing fixed generic programs must retain pre-C2 semantics:

```phalcom
class Box<T> {}
Box<Int>
```

and:

```phalcom
class Holder<P: TypePack> {}
Holder<P>
```

continues to mean one `TypePack` argument.

It remains distinct from:

```phalcom
class Holder<*P> {}
Holder<*P>
```

Higher-kinded fixed parameters must remain kind-driven and must not acquire variadic semantics merely because their kind equals `TypePack`.

`Fixed<>` remains invalid while fixed binders are unsatisfied.

---

# 27. C1 regression compatibility

P3 closure must re-certify representative C1 laws:

```text
Tuple::Positionals
Tuple::Labeled
Tuple::At<I>
Tuple::Map<F>

Record::Map<F>

TypePack::Map<F>
LabeledTypePack::Map<F>

RecordRow remains distinct from LabeledTypePack

Tuple([],[]) == Unit
Record({}) remains a distinct static Record
```

---

# 28. Runtime-boundary certification

P3 must prove absence of C2-driven:

```text
runtime TypePack objects
runtime LabeledTypePack objects
runtime CompleteTypePack objects
VM pack projection/composition opcode
VM generic pack solver
runtime type-level concat evaluator
call-frame ABI redesign
runtime ArgumentPack as a TypePack representation
```

`CallableFromPacks` is a **static type-formation node**. Concrete lanes canonicalize to ordinary `CallableType`; this does not authorize runtime pack solving.

Metadata inspection/display of pack nodes is permitted. Runtime metadata transport is not type-level execution.

---

# 29. Runtime metadata consumer requirement

Runtime/public metadata consumers may:

```text
retain nodes
inspect nodes
format nodes
use stable fingerprints/handles
```

but may not:

```text
beta-reduce arbitrary pack lambdas in the VM
solve pack variables
execute TypePackConcat
materialize runtime CompleteTypePack values
```

---

# 30. Metadata schema compatibility certification

If P3 lands schema v4, tests must cover:

```text
v1 fixture still decodes where historically supported
v2 fixture still decodes where historically supported
v3 fixture still decodes
v4 C2 scoped-pack bundle decodes and validates
```

v4 must reject malformed node indexes, bad kinds, wrong mode counts, mode/kind mismatches, malformed projection result kinds, forward references, and invalid scope indices.

The validator fails closed.

---

# 31. Cross-module/public fingerprint requirements

Public interface identity must differ for:

```text
Single(TypePack)
PositionalPack(TypePack)
LabeledPack
CompletePack
```

For type lambdas:

```text
lambda(TypePack, mode Single, body X)
    !=
lambda(TypePack, mode PositionalPack, body X)
```

even when parameter kinds and body otherwise match.

Scoped pack body changes must also change structural fingerprint.

---

# 32. Inference hostile cases

P3 must include negative probes that catch accidental solver expansion.

### No concat subtraction

```text
Concat(P, [String]) == [Int, Bool, String]
```

must not infer `P`.

### No arbitrary two-pack split

```text
Concat(P, Q) == [Int, String, Bool]
```

must not choose a split.

### No labeled partition search

```text
ConcatLabeled(L1, L2) == [a:Int, b:String]
```

must not decide partition ownership without independent whole-pack evidence.

### Kind mismatch remains exact

A `TypePack` inference variable cannot bind to `LabeledTypePack`, `CompleteTypePack`, or Tuple.

---

# 33. Whole-pack positive inference cases

Required positive evidence:

```text
Var(TypePack) == concrete TypePack
Var(LabeledTypePack) == concrete LabeledTypePack
Var(CompleteTypePack) == concrete CompleteTypePack
```

Also certify one generic constructor case and one nested C2 structural case so the recursive inference representation is genuinely exercised.

---

# 34. Materialization-through-all-forms matrix

Certify `GenericInstantiation` binding inside:

```text
TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks
Positionals/Labeled complete-pack projections
Applied generic arguments
transparent alias lambda bodies
nested aliases
```

For each, prove symbolic preservation, concrete canonicalization, and fallible invalid labeled composition as applicable.

---

# 35. Scoped alias persistence matrix

At least these forms must be serializable after P3:

```phalcom
type Identity<*Ts> = Target<*Ts>
type AddHead<*Ts> = (Int, *Ts)
type AddLabels<**Ls> = (**Ls,)
type Complete<***P> = (***P,)
type Callable<*Ts> = (*Ts,) -> Result
```

plus a partial/nested lambda case sufficient to prove residual binder modes survive durable metadata.

The stable representation must be structural, not debug/source strings.

---

# 36. No new public pack operations by implication

P3 persistence work does not add public syntax for:

```text
Pack::Concat
Pack::Length
Pack::Slice
```

or other sequence calculus.

Internal concat/projection terms remain implementation machinery.

Metadata persistence of `CompleteTypePack` does not establish a public source-level kind name.

---

# 37. No elementwise constraints

P3 must not reinterpret a constraint on a pack as quantified over its elements.

Whole-pack inference concerns the pack term itself, not implicit iteration over its members.

---

# 38. Inference/source declaration coverage

Where existing generic inference instantiates a `GenericSignature`, pack-kind parameters must participate as ordinary kind-aware inference variables and remain visible through all C2 compound forms.

P3 does not have to invent inference for declaration categories that did not previously infer ordinary generic arguments.

---

# 39. Expected implementation impact map

Likely primary P3 implementation areas:

```text
phalcom-semantic/src/checker/inference.rs
    recursive C2 inference terms
    lifting
    traversal
    materialization
    conservative unification

phalcom-type-meta/src/scoped_type.rs
    stable scoped C2 nodes
    stable lane enum
    lambda parameter modes

phalcom-type-meta/src/header.rs
    metadata schema revision

phalcom-type-meta/src/validate.rs
    v4 feature floor
    scoped pack validation
    lambda mode validation

phalcom-semantic/src/metadata/export.rs
    export scoped C2 nodes
    export lambda modes
    structural fingerprints

phalcom-core/src/typing/inspect.rs
    structural inspection/display only if required

phalcom-semantic/tests/semantic/...
    inference
    metadata
    imported/cross-module
    incremental
    closure/stress
```

P3 should not reflexively touch parser/AST/runtime dispatch files unless a real regression proves need.

---

# 40. Required verification layers

## Layer 1 — inference owner-local

```text
pack-kind fresh variables
whole-pack binding
C2 term lifting
C2 term materialization
no concat decomposition
projection resolution after solution
```

## Layer 2 — metadata owner-local

```text
stable scoped C2 encode/decode
lambda mode persistence
schema compatibility
malformed validation
fingerprint mode sensitivity
```

## Layer 3 — semantic integration

```text
variadic alias public metadata
TupleFromPacks inference
CallableFromPacks inference/materialization
duplicate-label transitions
C1 compatibility
```

## Layer 4 — cross-module/incremental

```text
provider/consumer pack API
mode-only edit
alias contract invalidation
cold parity
source target parity
metadata fingerprint parity
```

## Layer 5 — closure

Affected crate checks, closure suite, static architecture audits, lifecycle record updates.

Broad release certification remains separate unless repository convention explicitly requires it.

---

# 41. Baseline failure policy

C2 inherits:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

If broad P3 verification encounters the same signatures unchanged, classify them as inherited baseline.

Do not modify trait/default-body evidence architecture merely to obtain a globally green test count.

---

# 42. P3 non-goals

P3 does not implement:

```text
pack prefix/suffix solver
pack length arithmetic
unknown pack partitioning
sequence subtraction
public pack slicing
public pack concatenation syntax
general variadic runtime-call inference
runtime argument-pack ABI redesign
runtime TypePack values
runtime CompleteTypePack values
VM pack normalization
pack covariance/contravariance
elementwise quantified constraints
GATs
open type families
general const generics
union-distributive pack algebra
general compile-time execution
```

---

# 43. P3 durable acceptance laws

```text
P3-01
Pack generic parameters use ordinary InferVarId(kind); no PackVarId exists.

P3-02
Every source-reachable C2 compound type can expose nested generic parameters
to the query-local inference calculus.

P3-03
Whole-pack inference may solve one entire pack/lane term.

P3-04
Pack decomposition, prefix/suffix inference, and arbitrary partitioning remain unsupported.

P3-05
Inference materialization re-enters canonical TypeStore constructors.

P3-06
Concrete labeled duplication discovered after inference/substitution is an error.

P3-07
No InferVarId or solver-local term enters stable metadata.

P3-08
All public scoped C2 forms have durable structural metadata.

P3-09
Type-lambda parameter mode is serialized and fingerprinted independently of kind.

P3-10
Mode-only lambda/declaration edits change public semantic identity.

P3-11
Metadata schema compatibility is explicit.

P3-12
Cold and incremental compilation converge for binder-mode, pack-composition,
alias-forwarding, Tuple, callable, and duplicate-label edits.

P3-13
Alias-to-nominal binder-contract invalidation remains separate from alias-cycle ordering.

P3-14
Source references inside expansion operands retain canonical TypeParameterId identity.

P3-15
Concat representation remains flat and resource growth remains linear/bounded.

P3-16
Single(TypePack) remains distinct from PositionalPack(TypePack).

P3-17
LabeledTypePack remains distinct from RecordRow.

P3-18
CompleteTypePack remains distinct from Tuple.

P3-19
Concrete TupleFromPacks and CallableFromPacks collapse to existing concrete identities.

P3-20
No C2 type-level pack operation gains VM execution semantics.

P3-21
Existing fixed-arity generic programs retain pre-C2 semantics.

P3-22
Representative C1 structural operations remain green.

P3-23
C2 closes only after inference, persistence, incremental, runtime-negative,
and compatibility evidence are recorded.

P3-24
Completion records reflect actual verification, not intended future work.
```

---

# 44. Required hostile acceptance matrix

| Class | Representative probe | Expected |
|---|---|---|
| whole positional pack | `Var(TypePack) == [Int,String]` | solve whole var |
| whole labeled pack | `Var(LabeledTypePack) == [a:Int]` | solve whole var |
| whole complete pack | `Var(CompleteTypePack) == <P,L>` | solve whole var |
| kind mismatch | `Var(TypePack) == LabeledTypePack` | reject |
| concat subtraction | `Concat(P,[S]) == [I,S]` | do not infer `P` |
| two-pack split | `Concat(P,Q) == [I,S]` | do not partition |
| symbolic duplicate | `L1 ++ L2` unknown | retain residual |
| concrete duplicate later | substitution gives duplicate label | reject |
| concrete disjoint later | substitution proves disjoint | canonicalize |
| variadic lambda metadata | `type A<*Ts> = ...` | modes survive JSON round-trip |
| scoped TupleFromPacks metadata | `type A<*Ts> = (Int,*Ts)` | export/decode/validate |
| scoped CallableFromPacks metadata | `type F<*Ts> = (*Ts,)->R` | export/decode/validate |
| mode-only edit | `P:TypePack -> *P` | invalidate + cold parity |
| alias contract edit | provider binder mode changes | dependent alias recomputed |
| fixed generic | `Box<T>` | unchanged |
| single pack arg | `Holder<P:TypePack>` | one argument, not expansion |
| runtime audit | no pack opcode/object/solver | pass |
| density | many flat concat segments | bounded/flat |

---

# 45. Source-level representative acceptance programs

```phalcom
class Positional<*Ts> {}
class Labeled<**Ls> {}
class Complete<***P> {}

type AddHead<*Ts> = (Int, *Ts)

type Forward<*Ts> = Positional<*Ts>

type CompleteForward<***P> = Complete<***P>

type CallableOf<*Ts> = (*Ts,) -> Result
```

Cross-module variants should prove these semantics survive module/public metadata boundaries.

---

# 46. Closure truth table

C2 must not be marked complete until all rows are true.

| Area | Required closure state |
|---|---|
| P1 semantic algebra | complete / frozen |
| P2 source syntax/elaboration | complete / frozen |
| pack-kind inference variables | certified |
| recursive C2 inference lifting | implemented and tested |
| no pack decomposition | negative-certified |
| canonical materialization | certified through every C2 form |
| stable canonical metadata | certified |
| stable scoped C2 metadata | implemented and certified |
| type-lambda binder modes | durably persisted |
| metadata schema | compatibility-certified |
| public fingerprints | mode/body sensitive |
| cross-module products | certified |
| incremental invalidation | certified |
| cold parity | certified |
| source-index parity | certified |
| duplicate-label deferred invalidity | certified |
| density/resource bounds | certified |
| fixed generic compatibility | certified |
| C1 regression compatibility | certified |
| runtime pack execution | absent / certified |
| lifecycle docs | current |
| C2 walkthrough | complete |
| C2 handoff | complete |
| roadmap | re-baselined |

---

# 47. Lifecycle and roadmap closure requirements

At successful P3 completion:

```text
LANG007.C2.P3
    COMPLETE
    IMPLEMENTED
    FOCUSED_TESTED

LANG007.C2
    COMPLETE
    IMPLEMENTED
    FOCUSED_TESTED
```

`LANG007.C2-CHECKPOINT.md` must record the completed plan ledger, final inference/persistence architecture, incremental parity matrix, metadata schema state, runtime-negative evidence, inherited baselines, and next checkpoint.

`STATUS.md` and `PROGRAM.md` must stop describing P2/P3 as future work.

Current stale future-facing lifecycle prose must be updated in active sections.

P3 must produce:

```text
LANG007.C2.P3-walkthrough.md
LANG007.C2.P3-handoff.md
```

Do not rewrite closed historical walkthroughs merely to make past descriptions read as current.

---

# 48. STOP / CONSULT triggers for P3

Stop and consult if:

1. whole-pack inference appears to require sequence decomposition or split search;
2. a proposed design requires persistent `PackVarId`/`PackId`;
3. inference cannot expose C2 nested parameters without replacing the accepted inference architecture wholesale;
4. scoped pack metadata cannot be represented structurally without serializing source syntax;
5. preserving lambda binder mode would require a second alias-signature store;
6. metadata compatibility policy forbids a schema revision while old consumers would silently lose variadic lambda semantics;
7. cross-module persistence appears to require inventing a semantic metadata importer no existing architecture owns;
8. incremental parity can only be restored by whole-workspace invalidation;
9. fixed generic semantics change;
10. labeled packs would need conversion to `RecordRow`;
11. complete packs would need conversion to Tuple;
12. a static pack operation appears to require a VM opcode/runtime object;
13. a broad failure is causally entangled with unrelated trait/default-body architecture;
14. C2 closure would require claiming release certification not actually performed.

---

# 49. Implementation-planning implications

A patch-grade P3 plan should use five dependency-ordered gates:

```text
G1
Whole-pack inference-term and materialization closure

G2
Stable scoped metadata + type-lambda mode persistence + schema revision

G3
Cross-module products + incremental/cold/source-index parity

G4
Hostile duplicate/inference boundaries + density + fixed/C1 compatibility

G5
Runtime-negative certification + lifecycle/roadmap closure
```

The first two gates contain actual missing implementation.

The final three are primarily integration, certification, and closure, with corrective code only when tests reveal a real owned defect.

P3 should not rewrite parser/AST, generic binder elaboration, TypeStore pack algebra, or runtime call ABI.

---

# 50. Final requirements conclusion

The post-P2 architecture is coherent and largely complete, but C2 cannot honestly close yet.

Two remaining gaps are structural:

```text
Inference:
    C2 compound canonical terms are too opaque to the solver-local term graph.

Persistence:
    C2 scoped lambda forms and lambda binder modes are not durably represented.
```

P3 must close those gaps using existing authorities:

```text
InferVarId(kind)
InferenceTerm
GenericInstantiation
TypeStore canonical constructors
TypeLambdaData
SemanticMetadataBundle
incremental semantic dependency graph
```

and then prove:

```text
source * / ** / ***
        ↓
P1/P2 canonical pack terms
        ↓
whole-pack query-local inference
        ↓
canonical materialization
        ↓
stable canonical + scoped metadata
        ↓
cross-module/incremental identity
        ↓
no runtime pack execution
```

The central P3 architectural achievement should be:

> **Every C2 pack construct remains visible to compile-time inference and durable semantic identity without becoming a sequence-solving language or a runtime object model.**

Once that statement is demonstrated by inference, metadata, incremental, compatibility, density, and runtime-negative evidence, `LANG007.C2` can be closed as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

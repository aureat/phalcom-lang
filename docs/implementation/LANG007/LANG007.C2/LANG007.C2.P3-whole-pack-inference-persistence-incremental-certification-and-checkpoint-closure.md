---
id: LANG007.C2.P3
category: LANG
program: LANG007
checkpoint: LANG007.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG007.C2.P1
  - LANG007.C2.P2
follows: LANG007.C2.P2
supersedes: null
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 7da5b04378dbd68fc20df6cda9a890cce0e0afa0
prepared: 2026-09-22
---

# LANG007.C2.P3 — Whole-Pack Inference, Durable Persistence, Incremental Certification, and Checkpoint Closure

## 0. Executor contract

This plan is written for GPT-5.6 Luna High/Extra High as the primary implementer under constrained architectural authority.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture, semantic ownership, and invariants below;
- must not improvise a new pack model, inference model, persistence model, or runtime contract;
- must use the smallest useful test at each implementation step;
- must not run broad suites merely because an edit occurred;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep `LANG007.C2-CHECKPOINT.md` current at meaningful state transitions;
- must produce the required P3 walkthrough and handoff;
- must close `LANG007.C2` only if the final closure truth table is satisfied.

Testing is evidence gathering, not a ritual.

The plan is intentionally explicit because P3 contains two real implementation closures followed by checkpoint-wide certification. Most time should be spent implementing G1/G2, not repeatedly re-auditing P1/P2 or running the workspace.

---

## 1. Goal

Complete C2 by making every source-reachable pack construct visible to ordinary kind-aware inference and durable stable metadata, then certify cross-module persistence, incremental/cold parity, hostile symbolic behavior, fixed-generic compatibility, bounded resource behavior, source identity, and the static compiler/runtime boundary.

---

## 2. Checkpoint acceptance objective

`LANG007.C2` owns **variadic generic pack binding and type-level pack expansion** over the canonical type-level substrate established by C1.

P1 established:

```text
canonical pack kinds and terms
complete-pack identity
flat positional/labeled concat
binder mode as semantic identity
checked substitution/materialization
stable canonical metadata
```

P2 established:

```text
source * / ** / *** generic binders
binder-aware generic application
explicit empty specialization
scoped pack algebra for aliases/type lambdas
Tuple pack expansion
CallableFromPacks
staged GenericBinderHeader publication
source-index/dependency traversal
focused persistence/incremental foundations
```

P3 is the **full checkpoint-closing plan**.

C2 may be accepted only when the following bounded statement is true:

> Every C2 source construct lowers to the shared canonical pack algebra, participates in query-local whole-pack inference without sequence decomposition, persists with complete stable semantic identity across metadata/module boundaries, invalidates incrementally with cold-build parity, preserves fixed-generic and C1 behavior, remains bounded/flat, and creates no runtime pack execution semantics.

P3 is therefore:

```text
corrective integration closure
+ persistence migration
+ soundness/incremental hardening
+ checkpoint certification
```

It is **not** a new variadic feature plan.

---

## 3. Repository grounding

Prepared against:

```text
repository:          aureat/phalcom-lang
branch:              main
revision:            7da5b04378dbd68fc20df6cda9a890cce0e0afa0
revision subject:    feat(lang007): implement variadic generic binding and structural type expansion (LANG007.C2.P2)
relevant predecessor: LANG007.C2.P2 — COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

### 3.1 Verified repository facts

The following facts were verified during planning.

1. `LANG007.C2.P2` is committed and recorded complete.
2. `LANG007.C2-CHECKPOINT.md` names `LANG007.C2.P3` as the next plan.
3. The current source has:
   - `KindId::{TYPE_PACK, LABELED_TYPE_PACK, COMPLETE_TYPE_PACK}`;
   - `TypeData::{TypePack, LabeledTypePack, CompleteTypePack, TypePackConcat, LabeledTypePackConcat, TupleFromPacks, CallableFromPacks}`;
   - mode-sensitive `GenericParameterMode`;
   - staged `GenericBinderHeader`;
   - scoped P2 pack algebra in `types/type_lambda.rs`;
   - checked canonical materialization in `types/instantiation.rs`;
   - stable canonical C2 metadata nodes in `phalcom-type-meta`.
4. `InferenceSession::instantiate_generic_signature*` already allocates ordinary `InferVarId` using each parameter's exact `KindId`, excluding only `RecordRow`.
5. `InferenceTerm` does **not** yet structurally represent C2 pack terms.
6. `type_id_to_inference` and `type_id_to_inference_with_rows` currently fall through to opaque `InferenceTerm::Canonical` for C2 pack terms.
7. `scoped_to_inference` explicitly returns `None` for the P2 scoped pack forms.
8. `InferenceMaterializationFailure` currently has no pack-formation failure variant.
9. `phalcom-semantic/src/metadata/export.rs::export_scoped_type` explicitly rejects P2 scoped pack forms through `MetadataExportError::ScopedPackForm`.
10. Stable `TypeLambdaRef` and `ScopedTypeNode::Lambda` currently preserve parameter kinds but not `GenericParameterMode`.
11. `TYPE_METADATA_SCHEMA_VERSION` is currently `3`.
12. Canonical `TypeNode::CallableFromPacks`, `TupleFromPacks`, concat, complete-pack, and generic parameter mode metadata already exist under schema v3.
13. P2 already has focused tests for:
    - variadic alias forwarding;
    - `Single(TypePack)` versus pack mode;
    - nominal binder-header staging;
    - provider-side invalidation;
    - callable-domain symbolic formation;
    - callable alias beta reduction.
14. The semantic integration test binary is `phalcom-semantic/tests/semantic.rs`; responsibility-specific modules live under `tests/semantic/`.
15. Existing repository workflow explicitly requires narrow BUILD-mode testing and checkpoint state maintenance.

### 3.2 Live-tree rule

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

Before editing, execute locally:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Preserve unrelated modified, staged, and untracked files.

Do not reset, clean, restore, or broadly format unrelated work.

---

## 4. Required reads before implementation

Read these before substantive edits.

### 4.1 Workflow and lifecycle

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/luna-patch-grade-plan-schema.md
```

The user-supplied Luna plan schema and principal-architect doctrine supplied with this plan are also authoritative execution guidance.

### 4.2 Program/checkpoint state

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2-GUIDANCE.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P2-handoff.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-requirements-analysis.md
```

If the P3 requirements-analysis artifact has not yet been committed into the repository, use the supplied/downloaded requirements analysis as planning authority and record its adoption in the checkpoint when P3 starts.

### 4.3 Normative language/design authority

Read only the relevant sections of:

```text
docs/specs/README.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
docs/specs/generic-constraints.md

docs/spec/collections-next/argument-packs-and-expansion-spec.md
docs/spec/collections/06-rest-spread-and-pack-operators.md
docs/spec/collections/07-callable-domains.md

docs/decisions/accepted/0083-type-level-domains-and-typed-compile-time-parameters.md
docs/decisions/accepted/0084-canonical-type-level-terms-and-inference-lifetimes.md
docs/decisions/accepted/0085-parameterized-type-members-and-symbolic-projections.md
docs/decisions/accepted/0088-native-type-operations-use-closed-compiler-normalization.md
```

Where historical `../../../__unstructured__/spec/` material conflicts with a current owner in `docs/specs/`, the current owning specification wins.

### 4.4 Exact source paths

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/row_inference.rs

phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/parameter.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/builder.rs

phalcom-type-meta/src/header.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/encode.rs

phalcom-core/src/typing/inspect.rs
```

### 4.5 Exact test conventions/surfaces

```text
phalcom-semantic/tests/semantic/README.md

phalcom-semantic/tests/semantic/foundations/inference.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/substitution.rs
phalcom-semantic/tests/semantic/foundations/type_members.rs

phalcom-semantic/tests/semantic/capabilities/aliases.rs

phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/integration/type_member_closure.rs

phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
phalcom-semantic/tests/semantic/incremental/query_ownership.rs

phalcom-type-meta/tests/schema_compat.rs
```

Avoid broad exploration unless live evidence makes it necessary.

---

## 5. Normative authority

Keep these authority classes distinct.

### 5.1 Normative behavior

The current owning specifications and accepted decisions establish:

1. `TypePack`, `LabeledTypePack`, and `CompleteTypePack` are semantic type-level domains, not runtime object classes.
2. Canonical type-level terms use stable kinded `TypeId` identity.
3. Query-local inference identities do not enter durable semantic products.
4. Whole unknown pack values may use ordinary kind-aware inference variables.
5. A separate pack/sequence solver is justified only by decomposition features, which C2 does not own.
6. Symbolic projections can be reduced, stuck, or invalid; stuck is not `Dynamic`.
7. Type-level structural normalization is semantic/compiler-owned and does not execute in the VM.
8. Tuple/Record structural semantics and `LabeledTypePack`/`RecordRow` identity remain distinct.
9. Pack operators retain fixed meanings:
   - `*` positional;
   - `**` labeled;
   - `***` complete.
10. `...` is not generic/type-pack syntax.

### 5.2 User-ratified C2 behavior

The accepted C2 architecture additionally fixes:

```text
GenericParameterMode != KindId

Single(TypePack)
    !=
PositionalPack(TypePack)

canonical Applied argument count
    == declared binder count after saturation

TypePackConcat / LabeledTypePackConcat
    are flat canonical residuals

labeled concrete duplicates
    are invalid

symbolic overlap
    stays symbolic until checked rewriting proves validity/invalidity

CompleteTypePack
    != Tuple

LabeledTypePack
    != RecordRow

whole-pack inference
    allowed

pack decomposition/backtracking
    forbidden
```

### 5.3 Current implementation

The live repository is implementation evidence, not independent language authority.

P3 must adapt mechanically to live APIs but must not change the above semantics merely because an implementation helper is inconvenient.

### 5.4 Future/non-normative proposals

The following are not authorized by P3:

```text
pack length arithmetic
Pack::Slice
public pack-concat syntax
elementwise quantified pack constraints
pack variance
general runtime variadic-call type-pack inference
runtime pack objects
general compile-time execution
open type families
```

---

## 6. Takeover state

P3 inherits the following stable interfaces.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| positional pack kind | `KindId::TYPE_PACK` | `TypeStore` / semantic type system | exact non-proper pack domain |
| labeled pack kind | `KindId::LABELED_TYPE_PACK` | semantic type system | ordered labels; not RecordRow |
| complete pack kind | `KindId::COMPLETE_TYPE_PACK` | semantic type system | exact two-lane domain |
| positional pack | `TypeData::TypePack` | `TypeStore` | canonical concrete positional pack |
| labeled pack | `TypeData::LabeledTypePack` | `TypeStore` | canonical concrete ordered labeled pack |
| complete pack | `TypeData::CompleteTypePack` | `TypeStore::complete_type_pack` | positionals + labeled |
| positional concat | `TypeData::TypePackConcat` / `type_pack_concat` | `TypeStore` | flat ordered residual |
| labeled concat | `TypeData::LabeledTypePackConcat` / `labeled_type_pack_concat` | `TypeStore` | flat ordered residual; duplicates fail once concrete |
| Tuple formation | `TypeData::TupleFromPacks` / `tuple_from_packs` | `TypeStore` | concrete lanes collapse to Tuple/Unit |
| callable formation | `TypeData::CallableFromPacks` / `callable_from_packs` | `TypeStore` | concrete lanes collapse to ordinary Callable |
| lane projection | `project_positionals`, `project_labeled`, `project_complete` | `TypeStore` | domain-driven projection authority |
| opaque complete sublane | `TypeMemberProjection(Positionals/Labeled)` residual | type-member/projection subsystem | stable symbolic projection |
| binder mode | `GenericParameterMode` | parameter/signature subsystem | mode independent from kind |
| early nominal contract | `GenericBinderHeader` | declarations | published before alias lowering |
| final generic signature | `GenericSignature` | declarations/checker | completed after aliases |
| generic durable binding | `GenericInstantiation` | semantic type system | non-row kinds bind as `TypeId` |
| solver variable | `InferVarId(kind)` | `InferenceSession` | query-local only |
| row solver | `RecordRowVarId` / row inference | row subsystem | stays separate from packs |
| scoped pack algebra | `ScopedTypeData::*Pack*` | type-lambda arena | alpha-normalized alias body structure |
| lambda binder modes | `TypeLambdaData::parameter_modes` | type-lambda arena | participates in in-memory lambda identity |
| checked substitution | `TypeSubstitution::try_apply` | type system | re-canonicalizes C2 forms |
| canonical materialization | `materialize_type` | type system | re-enters canonical constructors |
| canonical metadata | `TypeNode::*Pack*`, `CallableFromPacks` | `phalcom-type-meta` | stable canonical graph |
| stable parameter mode | `TypeParameterRecord.mode` | `phalcom-type-meta` | public semantic identity |
| checkpoint state | `LANG007.C2-CHECKPOINT.md` | implementation records | single durable checkpoint authority |

### 6.1 P1/P2 behavior to treat as frozen

Do not reimplement:

```text
parser/AST * ** *** support
generic binder ordering legality
empty <> specialization
binder-aware source argument assembly
Tuple source expansion
CallableFromPacks source expansion
GenericBinderHeader publication
alias contract dependency separation
checked canonical substitution
P1 pack projection amendment
```

If a P3 test exposes a defect in those areas, classify the defect causally. Correct owned defects only; do not redesign those subsystems by default.

---

## 7. Architecture

### 7.1 End-to-end authority flow

P3 preserves this direction of authority:

```text
source
  * / ** / ***
        ↓
P2 source elaboration
        ↓
P1/P2 canonical TypeId pack algebra
        ↓
query-local InferenceTerm graph
  InferVarId(exact KindId)
        ↓
whole-pack solution only
        ↓
GenericInstantiation
        ↓
canonical materialization
        ↓
stable TypeNode / ScopedTypeNode metadata
        ↓
cross-module + incremental products
        ↓
compiler/runtime may inspect metadata
        ↓
NO runtime pack computation
```

The semantic layer owns every formal pack fact.

Compiler/runtime consumers do not reconstruct binder mode, lane meaning, canonical pack composition, or pack inference.

---

### 7.2 G1 architecture — solver-local C2 term vocabulary

The current inference graph must stop treating C2 compound types as atomic canonical leaves when they contain generic parameters.

Add solver-local representations equivalent to:

```rust
pub struct InferenceLabeledTypePackElement {
    pub label: Box<str>,
    pub term: InferenceTerm,
}

pub enum InferencePackProjectionLane {
    Positionals,
    Labeled,
    Complete,
}

pub enum InferenceTerm {
    // existing variants ...

    TypePack(Box<[InferenceTerm]>),

    LabeledTypePack(
        Box<[InferenceLabeledTypePackElement]>
    ),

    CompleteTypePack {
        positionals: Box<InferenceTerm>,
        labeled: Box<InferenceTerm>,
    },

    TypePackConcat(
        Box<[InferenceTerm]>
    ),

    LabeledTypePackConcat(
        Box<[InferenceTerm]>
    ),

    TupleFromPacks {
        positionals: Box<InferenceTerm>,
        labeled: Box<InferenceTerm>,
    },

    CallableFromPacks {
        positionals: Box<InferenceTerm>,
        labeled: Box<InferenceTerm>,
        return_type: Box<InferenceTerm>,
    },

    PackProjection {
        subject: Box<InferenceTerm>,
        lane: InferencePackProjectionLane,
    },
}
```

Exact private naming is mechanically flexible.

The semantic shape is FIXED.

Do not put `TypeId` children inside these compound variants merely to avoid recursive lifting. Every child that may contain one of the current inference signature's generic parameters must remain recursively representable as `InferenceTerm`.

---

### 7.3 Canonical `TypeId` → inference lifting

Extend both:

```rust
InferenceSession::type_id_to_inference
InferenceSession::type_id_to_inference_with_rows
```

for all C2 terms.

Required mapping:

```text
TypeData::TypePack
    -> InferenceTerm::TypePack

TypeData::LabeledTypePack
    -> InferenceTerm::LabeledTypePack

TypeData::CompleteTypePack
    -> InferenceTerm::CompleteTypePack

TypeData::TypePackConcat
    -> InferenceTerm::TypePackConcat

TypeData::LabeledTypePackConcat
    -> InferenceTerm::LabeledTypePackConcat

TypeData::TupleFromPacks
    -> InferenceTerm::TupleFromPacks

TypeData::CallableFromPacks
    -> InferenceTerm::CallableFromPacks
```

For canonical P1 opaque complete-lane residuals:

```text
TypeData::TypeMemberProjection
  subject kind CompleteTypePack
  zero arguments
  canonical Positionals/Labeled/Complete lane identity
```

lower to `InferenceTerm::PackProjection`.

Do **not** detect these residuals in the checker by ad hoc arbitrary source-text inspection.

Preferred implementation:

```text
types/projection or TypeStore
    owns a small classifier/helper for the internally canonical pack-lane residual

checker/inference
    consumes that semantic classifier
```

If the live projection subsystem already has an exact identity helper, reuse it.

Do not redesign the P1 residual representation.

---

### 7.4 Scoped type lambda → inference lifting

`InferenceSession::scoped_to_inference` currently returns `None` for every P2 scoped pack form.

Implement recursive lowering for:

```text
ScopedTypeData::TypePack
ScopedTypeData::LabeledTypePack
ScopedTypeData::CompleteTypePack
ScopedTypeData::TypePackConcat
ScopedTypeData::LabeledTypePackConcat
ScopedTypeData::TupleFromPacks
ScopedTypeData::CallableFromPacks
ScopedTypeData::PackProjection
```

Nested lambda behavior remains as currently bounded unless a source-reachable P3 case proves it must be extended.

The P2 alias:

```phalcom
type AddHead<*Ts> = (Int, *Ts)
```

must be able to participate in symbolic beta reduction when the argument is an `InferVarId(TypePack)`.

---

### 7.5 Inference traversals

Every function that recursively reasons over `InferenceTerm` must become C2-complete.

Audit and update at least:

```text
term_has_variables
term_has_unresolved_variables
collect_term_variables
record_term_proof_state
record_term_support
term_support
occurs_in_term
term_has_projection
materialize_for_expected
materialize
symbolic_beta_reduce/scoped_to_inference
unify_terms
subtype_terms
```

Also inspect any formatting/debug/explanation helpers that exhaustively match `InferenceTerm`.

Do not add a generic "other => ignore children" escape hatch.

A nested pack variable must remain visible to:

```text
occurs checks
proof/support propagation
underconstrained detection
materialization
constraint replay
projection normalization
```

---

### 7.6 Whole-pack inference law

A pack inference variable denotes one whole kinded term.

Allowed:

```text
Var(TypePack)
    == TypePack[Int, String]

Var(LabeledTypePack)
    == [name: String]

Var(CompleteTypePack)
    == CompleteTypePack(P, L)
```

Allowed structural whole-lane inference:

```text
TupleFromPacks(P, emptyLabeled)
    against concrete Tuple(Int, String)

=> P may be selected as the entire positional lane
```

Likewise for a finite `CallableFromPacks` domain.

Forbidden:

```text
Concat(P, [String])
    == [Int, Bool, String]

=> DO NOT infer P = [Int, Bool]
```

Forbidden:

```text
Concat(P, Q)
    == ConcretePack

=> DO NOT try split points
```

The distinction is:

```text
whole lane correspondence
    allowed

unknown sequence partition
    forbidden
```

---

### 7.7 C2 unification rules

Add only deterministic structural rules.

#### TypePack

```text
TypePack(A0..An)
vs
TypePack(B0..Bn)

same length required
unify corresponding element terms
```

No prefix/suffix logic.

#### LabeledTypePack

```text
same entry count required
same ordered labels required
unify corresponding payload terms
```

Do not treat labels as an unordered row.

#### CompleteTypePack

```text
unify positional lane
unify labeled lane
```

#### TypePackConcat / LabeledTypePackConcat

If both sides are the same residual constructor and have the same canonical segment count:

```text
unify segment i with segment i
```

This permits whole segment variables to solve where segment correspondence is already canonical.

If one side is a residual concat and the other is a flat concrete pack, **do not derive a partition**.

Let canonical materialization solve the relation only when all unknown segments have independently become known.

#### TupleFromPacks

For:

```text
TupleFromPacks(P,L)
vs
Tuple(...)
```

derive the concrete Tuple's exact positional and labeled lanes without guessing and unify:

```text
P with entire positional lane
L with entire labeled lane
```

Preserve Tuple lane order.

This is not arbitrary pack decomposition; Tuple already has one canonical lane split.

#### CallableFromPacks

For a finite ordinary callable with no open/rest parameter semantics:

```text
CallableFromPacks(P,L,R)
vs
Callable(...)
```

derive the exact finite callable positional/labeled lane structure and unify the pack lanes as whole lane terms.

Then apply the existing callable relation to the return type.

Do not reinterpret runtime `RestMode` as pack structure.

If the callable contains open rest semantics that cannot be represented by the finite C2 pack domain, do not coerce it into `CallableFromPacks`.

#### PackProjection

A pack projection with an unresolved subject remains deferred.

Once its subject materializes:

```text
Positionals(subject)
Labeled(subject)
Complete(subject)
```

must re-enter:

```text
TypeStore::project_positionals
TypeStore::project_labeled
TypeStore::project_complete
```

and normalize through the P1 authority.

---

### 7.8 Inference materialization

Extend `InferenceSession::materialize` and `materialize_for_expected` to reconstruct C2 terms only through canonical constructors.

Use:

```text
TypeStore::type_pack
TypeStore::labeled_type_pack
TypeStore::complete_type_pack
TypeStore::type_pack_concat
TypeStore::labeled_type_pack_concat
TypeStore::tuple_from_packs
TypeStore::callable_from_packs
TypeStore::project_positionals
TypeStore::project_labeled
TypeStore::project_complete
```

Do not directly intern raw `TypeData` variants from solver terms.

Extend `InferenceMaterializationFailure` with a structured pack failure, preferred shape:

```rust
Pack(PackFormationError)
```

or an exact equivalent.

Do not collapse a proven duplicate-label error into `UnsupportedDomain` or `InternalInvariant`.

The caller may still map it into existing generic-inference diagnostics, but the solver/materializer must preserve the semantic failure category.

---

### 7.9 G2 architecture — stable scoped metadata

Canonical C2 `TypeNode` variants already exist.

P3 must add stable scoped equivalents for every P2 scoped form that can appear in a durable public type lambda:

```rust
ScopedTypeNode::TypePack(...)
ScopedTypeNode::LabeledTypePack(...)
ScopedTypeNode::CompleteTypePack { ... }
ScopedTypeNode::TypePackConcat(...)
ScopedTypeNode::LabeledTypePackConcat(...)
ScopedTypeNode::TupleFromPacks { ... }
ScopedTypeNode::CallableFromPacks { ... }
ScopedTypeNode::PackProjection { ... }
```

Add stable helper records as needed, e.g.:

```rust
ScopedLabeledTypePackElementRef
ScopedPackProjectionLaneRef
```

Do not serialize source syntax or source operator tokens as semantic structure.

---

### 7.10 Stable lambda binder modes

`TypeLambdaData::parameter_modes` already participates in in-memory lambda identity.

Stable metadata must preserve it.

Change stable top-level lambda representation conceptually to:

```rust
pub struct TypeLambdaRef {
    pub parameter_kinds: Box<[KindNodeId]>,
    pub parameter_modes: Option<Box<[GenericParameterModeRef]>>,
    pub body: ScopedTypeNodeId,
}
```

and nested scoped lambda to equivalent shape:

```rust
ScopedTypeNode::Lambda {
    parameter_kinds: Box<[KindNodeId]>,
    parameter_modes: Option<Box<[GenericParameterModeRef]>>,
    body: ScopedTypeNodeId,
}
```

### Compatibility meaning

FIXED:

```text
None
    = legacy schema v1-v3 lambda
    = semantically all Single modes

Some(modes)
    = explicit mode vector
```

Current schema-v4 producers must always emit:

```text
Some(modes)
```

with:

```text
modes.len() == parameter_kinds.len()
```

This `Option` is a serialization compatibility representation, not semantic uncertainty.

Do not make a current v4 variadic lambda depend on implicit `None`.

Use serde defaults so v1-v3 JSON lacking the field still decodes.

Whether the field is omitted on serialization when `None` is mechanically flexible.

---

### 7.11 Metadata schema v4

Bump:

```text
TYPE_METADATA_SCHEMA_VERSION
3 -> 4
```

Do not change:

```text
MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION
```

unless live metadata policy gives a separate, evidenced reason.

Compatibility:

```text
v1/v2
    historical rules unchanged

v3
    existing canonical C2 pack terms and parameter mode support
    no scoped C2 pack nodes
    no explicit type-lambda mode vectors

v4
    scoped C2 pack nodes
    explicit type-lambda modes
```

Add a pre-v4 feature-floor validator so a manually constructed schema-v3 bundle cannot smuggle v4-only scoped forms/mode vectors.

Add a v4 validator requiring explicit mode vectors for current lambdas.

Do not retroactively remove canonical C2 nodes that P2 already emits under schema v3.

---

### 7.12 Stable metadata validation

Validate all new scoped nodes topologically and by kind.

Required laws:

```text
Scoped TypePack element children
    : Type

Scoped LabeledTypePack payload children
    : Type

Scoped CompleteTypePack.positionals
    : TypePack

Scoped CompleteTypePack.labeled
    : LabeledTypePack

Scoped TypePackConcat segment
    : TypePack

Scoped LabeledTypePackConcat segment
    : LabeledTypePack

Scoped TupleFromPacks.positionals
    : TypePack

Scoped TupleFromPacks.labeled
    : LabeledTypePack

Scoped CallableFromPacks.positionals
    : TypePack

Scoped CallableFromPacks.labeled
    : LabeledTypePack

Scoped CallableFromPacks.return_type
    : Type
```

Projection lane/result kind must agree.

Lambda validation:

```text
schema <= 3:
    mode vector absent/legacy
    effective modes = all Single

schema >= 4:
    explicit vector required
    vector length == parameter kind length

PositionalPack -> TypePack
LabeledPack    -> LabeledTypePack
CompletePack   -> CompleteTypePack
Single         -> any already-valid declared kind
```

---

### 7.13 Structural fingerprints

Do not renumber or reuse existing tags.

Every new stable scoped variant gets a new unique tag.

Fingerprint:

```text
ordered children
labels
projection lane
parameter mode vector
```

Top-level and nested type-lambda fingerprints must include mode.

Therefore:

```text
lambda(TypePack, Single, Body)
    !=
lambda(TypePack, PositionalPack, Body)
```

in current v4 structural identity.

Historical v1-v3 bundle fingerprints remain historical data; do not recompute/rewrite fixtures merely to make them look v4-native.

---

### 7.14 Metadata round-trip boundary

P3 does not invent a semantic `TypeStore` importer.

Required durable round trip is:

```text
semantic export
    ↓
SemanticMetadataBundle
    ↓
encode_metadata_json
    ↓
decode_metadata_json
    ↓
validate_metadata_bundle
    ↓
same stable semantic structure/fingerprints
```

Where existing runtime/core metadata consumers inspect the bundle, update them only as needed for structural preservation/display.

Do not make `phalcom-core` execute pack operations.

---

### 7.15 G3 architecture — incremental/cross-module certification

P2 already introduced alias-to-nominal binder-contract dependencies.

P3 proves the whole dependency graph.

Cold and incremental final state must agree for:

```text
mode-only binder edits
Single(TypePack) <-> PositionalPack(TypePack)
* <-> ** <-> ***
alias forwarding edits
Tuple pack edits
CallableFromPacks edits
concat structure edits
symbolic duplicate -> valid concrete
symbolic duplicate -> invalid concrete
provider-only edits
```

Compare semantically relevant outputs:

```text
diagnostics
canonical form/fingerprint
generic modes
alias target/lambda form
source-index targets
public metadata fingerprint
```

Do not compare only "has errors".

---

### 7.16 G4 architecture — hostile/resource/compatibility matrix

P3 certifies:

```text
flat concat behavior
bounded node growth
duplicate-label checked invalidity
no sequence decomposition
no row conflation
fixed-generic compatibility
C1 structural operation compatibility
source identity
```

Use structural evidence rather than microbenchmark ceremony.

A dense test may construct approximately 128–512 symbolic segments and assert:

```text
one flat concat residual
no concat child is itself a concat
node growth is linear in inputs
normalization completes within ordinary query budget
```

Do not assert unstable wall-clock timings.

---

### 7.17 G5 architecture — runtime-negative closure

The final architecture must still be:

```text
type-level pack computation
    semantic/compiler host only
```

P3 must not add:

```text
runtime TypePack object
runtime LabeledTypePack object
runtime CompleteTypePack object
pack bytecode
pack-solving VM dispatch
call-frame ABI change
runtime ArgumentPack reuse as TypePack
```

Core metadata inspection may gain exhaustiveness/display cases only.

---

## 8. Ownership boundaries

### 8.1 Owns

P3 owns:

- query-local inference representation for C2 pack structures;
- whole-pack exact-kind inference;
- deterministic lane-exact structural matching needed by C2;
- C2 inference materialization;
- inference-visible P2 alias/scoped algebra;
- stable scoped pack metadata;
- stable lambda binder-mode metadata;
- schema-v4 compatibility rules;
- stable fingerprints for the new metadata;
- cross-module persistence proof;
- incremental/cold parity for the complete C2 surface;
- source-index parity certification;
- hostile duplicate-label and no-decomposition proof;
- density/flatness evidence;
- fixed-generic/C1 compatibility proof;
- static/runtime boundary certification;
- C2 lifecycle closure.

### 8.2 Does not own

P3 does not own:

- new parser grammar;
- a new pack operator;
- runtime rest/spread redesign;
- arbitrary variadic call-argument inference;
- pack sequence splitting;
- pack length/slicing;
- quantified pack constraints;
- user-defined pack protocols;
- runtime pack reification;
- general metadata-store import architecture;
- unrelated trait/default-body failures;
- release-wide certification.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| pack kind | `TypeStore` kind graph | inference, metadata, diagnostics | solver-local pack kind enum |
| pack canonical term | `TypeStore` | inference/materialization/export | inference-owned canonical pack arena |
| binder mode | `TypeParameterData` / `GenericSignature` / `TypeLambdaData` | application, metadata, fingerprints | infer mode from kind |
| nominal early application contract | `GenericBinderHeader` | alias/application lowering | alias-side signature cache |
| whole-pack unknown | `InferVarId(kind)` | solver | `PackVarId` |
| row unknown | `RecordRowVarId` | row solver | pack solver reuse |
| lane projection semantics | `TypeStore` + type-member projection authority | inference, substitution | checker-local projection rules |
| pack concat canonicalization | `TypeStore` | materialization, inference | solver-local canonical concat |
| durable generic binding | `GenericInstantiation` | materialization | pack-specific binding map |
| in-memory lambda modes | `TypeLambdaData` | beta reduction/application | reconstruct from kind |
| stable lambda modes | `TypeLambdaRef` / scoped metadata | metadata consumers | source-text reconstruction |
| metadata version policy | `phalcom-type-meta` header/validator | producer/consumer | producer-specific hidden version |
| checkpoint state | `LANG007.C2-CHECKPOINT.md` | future plans/agents | plan-local implementation-state file |

---

## 9. Global invariants

### Semantic representation

```text
INV-01 — * always means positional lane.
INV-02 — ** always means labeled lane.
INV-03 — *** always means complete two-lane pack.
INV-04 — ... is never C2 pack syntax.
INV-05 — TypePack, LabeledTypePack, and CompleteTypePack are distinct domains.
INV-06 — LabeledTypePack is not RecordRow.
INV-07 — CompleteTypePack is not Tuple.
INV-08 — GenericParameterMode is not inferred from KindId.
INV-09 — Single(TypePack) is distinct from PositionalPack(TypePack).
INV-10 — canonical generic application remains binder-aligned ordinary Applied.
```

### Inference

```text
INV-11 — pack unknowns use ordinary InferVarId(exact kind).
INV-12 — no PackVarId/PackId inference identity is introduced.
INV-13 — every C2 compound inference term exposes nested query-local variables.
INV-14 — no query-local inference identity enters TypeStore or durable metadata.
INV-15 — whole-pack/lane inference is allowed.
INV-16 — sequence subtraction, prefix/suffix inference, and arbitrary partition search are forbidden.
INV-17 — pack inference enforces exact KindId.
INV-18 — solver materialization re-enters TypeStore canonical constructors.
INV-19 — pack projection inference reuses the P1 projection authority.
INV-20 — concrete duplicate labeled entries discovered during inference/materialization are errors.
INV-21 — symbolic labeled overlap may remain residual while disjointness is unknown.
INV-22 — finite Tuple/callable lane extraction may constrain an entire lane without arbitrary partitioning.
```

### Persistence

```text
INV-23 — every public P2 scoped pack form has a structural stable metadata form.
INV-24 — type-lambda binder modes are durable stable semantics.
INV-25 — mode-only lambda edits change current structural/public fingerprints.
INV-26 — schema-v4 producers emit explicit aligned lambda mode vectors.
INV-27 — legacy v1-v3 missing lambda mode vectors mean all-Single only.
INV-28 — schema compatibility is explicit and validator-enforced.
INV-29 — stable metadata serializes structure, never InferVarId/query frames.
INV-30 — stable labels and child order participate in labeled-pack identity.
```

### Incremental/source

```text
INV-31 — mode-only declaration edits invalidate all semantic dependents that consume the application contract.
INV-32 — alias-to-nominal contract invalidation remains separate from alias-cycle ordering.
INV-33 — cold and incremental final semantic products agree.
INV-34 — expansion operand references retain canonical TypeParameterId identity.
INV-35 — unrelated modules/products remain reusable when their dependencies are unchanged.
```

### Performance/runtime/compatibility

```text
INV-36 — concat residuals remain flat.
INV-37 — normal pack composition has linear/bounded structural growth.
INV-38 — no exponential/backtracking pack partition algorithm exists.
INV-39 — fixed generic semantics remain unchanged.
INV-40 — C1 structural type-member semantics remain unchanged.
INV-41 — no C2 runtime pack objects/opcodes/solver are introduced.
INV-42 — CallableFromPacks remains a static type formation node, not runtime rest semantics.
INV-43 — P3 does not weaken a semantic invariant merely to pass broad tests.
INV-44 — C2 closes only with truthful focused evidence; RELEASE_COMPLETE is not implied.
```

All tasks and coverage obligations below reference these invariants.

---

## 10. Non-goals

Explicitly out of scope:

```text
pack decomposition solver
prefix/suffix pack inference
pack-length arithmetic
Pack::Length
Pack::Slice
public generic pack-concat operator
arbitrary user-defined expansion protocols
general nominal const generics
trait const generics
GATs
open type families
implicit union distribution
pack-wide quantified trait constraints
pack covariance/contravariance
runtime TypePack values
runtime LabeledTypePack values
runtime CompleteTypePack values
runtime pack-solving bytecode
runtime callable ABI redesign
runtime rest-capture migration
general runtime variadic argument inference
general semantic metadata -> TypeStore importer
release certification unrelated to C2
broad LSP/UI pack feature work
```

Do not expand P3 merely to make it feel larger than a closure plan.

---

## 11. Expected impact map

### 11.1 Expected source areas

Primary expected edits:

```text
phalcom-semantic/src/checker/inference.rs
    solver-local C2 terms, traversals, lifting, materialization, unification

phalcom-semantic/src/types/projection.rs
or
phalcom-semantic/src/types/store.rs
    small canonical pack-projection classification helper if needed

phalcom-type-meta/src/scoped_type.rs
    stable scoped pack nodes and lambda mode fields

phalcom-type-meta/src/header.rs
    schema version 4

phalcom-type-meta/src/validate.rs
    pre-v4/v4 feature floors, kind/scope/mode validation

phalcom-semantic/src/metadata/export.rs
    scoped C2 export and mode-sensitive fingerprints
```

Likely secondary mechanical edits:

```text
phalcom-semantic/src/checker/mod.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/row_inference.rs
    only if exhaustiveness/error propagation requires

phalcom-core/src/typing/inspect.rs
    structural display/exhaustiveness for new stable nodes

phalcom-type-meta/src/lib.rs
    exports if new stable helper types are public
```

Expected checkpoint/lifecycle docs:

```text
docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md

docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-walkthrough.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-handoff.md
```

### 11.2 Expected tests

```text
phalcom-semantic/tests/semantic/foundations/inference.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/substitution.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs

phalcom-semantic/tests/semantic/capabilities/aliases.rs

phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/integration/type_member_closure.rs

phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs

phalcom-type-meta/tests/schema_compat.rs
```

### 11.3 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem—especially VM execution, bytecode, runtime argument-pack ABI, trait proof machinery, parser grammar, or general module-linker architecture—when that entry changes architecture or ownership.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. `InferVarId(kind)` is the only C2 pack inference-variable model.
2. C2 compound inference terms are solver-local structure, not canonical identity.
3. TypeStore constructors remain canonicalization authority.
4. Whole-pack inference is allowed; sequence decomposition is forbidden.
5. Tuple/callable lane extraction may use only deterministic existing lane structure.
6. `LabeledTypePack` remains ordered and distinct from `RecordRow`.
7. `CompleteTypePack` remains distinct from Tuple.
8. `GenericInstantiation` remains durable non-row binding authority.
9. P1 projection functions remain lane semantics authority.
10. Stable scoped pack metadata is structural.
11. Lambda binder mode must persist independently from kind.
12. Metadata schema advances to v4 unless a verified repository-wide policy makes that mechanically impossible; a silent schema-v3 reinterpretation is forbidden.
13. Legacy lambda mode absence means all `Single`.
14. v4 producer output must contain explicit modes.
15. No runtime pack execution is introduced.
16. The checkpoint remains `FOCUSED_TESTED` unless broader release gates actually run.

### 12.2 MECHANICALLY FLEXIBLE

Luna may adapt:

- private helper names;
- whether C2 inference helper structs live beside `InferenceTerm` or in a nearby private module;
- iterator versus temporary-vector mechanics;
- exact enum names for solver-local pack lanes;
- exact helper API for recognizing canonical pack residual projections;
- local factoring of recursive traversal code;
- whether serde omits `None` legacy fields on serialization;
- test fixture names/IDs;
- private metadata validation helper names;
- mechanical imports/borrowing;
- exact checkpoint prose layout.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| latest local base contains P2 commit | `git rev-parse HEAD`, `git log` | adapt if descendant; STOP if P2 architecture missing |
| no newer stable scoped pack nodes exist | `phalcom-type-meta/src/scoped_type.rs` | reuse newer forms mechanically |
| schema version remains 3 | `phalcom-type-meta/src/header.rs` | choose next version mechanically; preserve same compatibility policy |
| scoped exporter still rejects pack forms | `metadata/export.rs` | consume newer support; do not duplicate |
| `TypeLambdaRef` still lacks modes | `scoped_type.rs` | reuse newer field and adjust T3 |
| `InferenceTerm` still lacks C2 forms | `checker/inference.rs` | reuse newer forms if architecture matches |
| `scoped_to_inference` still rejects C2 forms | `checker/inference.rs` | narrow T1 to missing cases |
| no general semantic metadata importer exists | search `phalcom-type-meta` / semantic metadata consumers | if one exists, integrate; do not build parallel importer |
| `InferenceMaterializationFailure` has no pack variant | `checker/inference.rs` | reuse existing structured failure if present |
| P2 alias dependency index remains separate from alias cycle graph | `session.rs` | STOP if architecture changed |
| runtime has no pack opcode/object | `phalcom-core` audit | if present intentionally after P2, STOP and reconcile authority |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing the affected issue and build a consultation packet if any of the following occurs.

1. Whole-pack inference appears to require sequence decomposition or split search.
2. A proposed solution requires persistent `PackVarId`, `PackId`, or a second canonical pack arena.
3. `InferenceTerm` cannot expose C2 nested generic parameters without replacing the accepted generic inference architecture wholesale.
4. Two plausible unification strategies have materially different soundness/variance consequences beyond the exact lane rules in this plan.
5. Supporting `CallableFromPacks` inference appears to require runtime rest/ABI semantics.
6. Pack projection inference would require a second projection semantics rather than reusing P1.
7. Stable scoped pack metadata cannot be represented structurally in the existing graph.
8. Lambda binder modes cannot be made durable without a duplicate alias/signature store.
9. The repository's metadata compatibility policy makes v4 impossible while older consumers would silently lose new semantics.
10. A real semantic metadata importer exists with a materially different ownership model than assumed here.
11. Incremental correctness appears possible only through whole-workspace invalidation.
12. Alias contract dependencies must enter alias-cycle ordering.
13. A fixed generic program changes meaning.
14. A solution converts `LabeledTypePack` to `RecordRow`.
15. A solution converts `CompleteTypePack` to Tuple as canonical identity.
16. A static pack operation appears to require VM execution.
17. A broad failure indicates causal interaction with trait/default-body architecture.
18. A required test oracle conflicts with accepted specs/user rulings.
19. The same nontrivial semantic failure persists after one serious correction.
20. Passing tests appears to require weakening any `INV-*`.
21. Patch scope becomes materially more cross-cutting than this plan.
22. P3 cannot close C2 without adding a new user-visible pack feature.

A triggered consultation cannot be self-waived.

### Consultation packet

Use:

```markdown
# IMPLEMENTATION INCIDENT

## Identity
- Program/checkpoint/plan/task/gate
- revision/worktree state

## Required invariant
- exact INV-* IDs

## Observed
- symptom, not hypothesis

## Minimal reproduction
```sh
...
```

## Relevant path
A -> B -> C

## Evidence
- exact code/test facts

## Hypothesis
...

## Serious correction already attempted
...

## Competing interpretations
1. ...
2. ...

## Decision requested
One narrow architectural question.
```

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Examples:

```text
non-exhaustive match after new InferenceTerm variant
missing import
serde derive bound
borrow conflict
renamed test helper
wrong Cargo filter
```

Do not consult for these unless the problem stops being mechanical.

### 14.2 Semantic failures

Before editing:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic problem.

If it persists: STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

### 14.4 Rerun rule

Never rerun an unchanged failing test unless something relevant to the failure changed.

A zero-selected Cargo filter is not a PASS; inspect `-- --list` and correct the selector.

---

## 15. Testing surface analysis

### 15.1 Coverage ledger

| Coverage ID | Dimension | Invariants | Required case |
|---|---|---|---|
| CV-01 | canonical success | INV-11,17 | `InferVarId(TypePack)` binds concrete TypePack |
| CV-02 | canonical success | INV-11,17 | labeled-pack var binds concrete labeled pack |
| CV-03 | canonical success | INV-11,17 | complete-pack var binds concrete complete pack |
| CV-04 | negative kind | INV-17 | TypePack var rejects labeled/complete/Tuple |
| CV-05 | recursive lifting | INV-13 | parameter under TypePack becomes Var |
| CV-06 | recursive lifting | INV-13 | parameter under LabeledTypePack becomes Var |
| CV-07 | recursive lifting | INV-13 | parameter under CompleteTypePack becomes Var |
| CV-08 | recursive lifting | INV-13 | parameters under both concat forms remain visible |
| CV-09 | recursive lifting | INV-13 | TupleFromPacks lanes remain inference terms |
| CV-10 | recursive lifting | INV-13,42 | CallableFromPacks lanes/return remain inference terms |
| CV-11 | scoped beta | INV-13 | `AddHead<*Ts>` symbolic lambda lowers with pack Var |
| CV-12 | pack projection | INV-19 | unresolved complete subject stays deferred |
| CV-13 | pack projection | INV-19 | solved subject reduces Positionals/Labeled |
| CV-14 | whole-lane Tuple | INV-15,16,22 | entire positional lane solves one pack var |
| CV-15 | whole-lane labeled Tuple | INV-15,22 | entire labeled lane solves one pack var |
| CV-16 | finite callable lane | INV-15,22,42 | exact finite domain solves whole lane |
| CV-17 | concat aligned | INV-15,16 | same-segment residuals can unify deterministically |
| CV-18 | no concat subtraction | INV-16,38 | residual vs flat concrete does not infer prefix |
| CV-19 | no split search | INV-16,38 | `Concat(P,Q)==Concrete` remains unsupported |
| CV-20 | no labeled split | INV-16 | labeled unknowns are not partitioned |
| CV-21 | materialization | INV-18 | TypePack inference term canonicalizes through store |
| CV-22 | materialization | INV-18,20 | labeled duplicate yields structured pack error |
| CV-23 | materialization | INV-18 | TupleFromPacks concrete collapses to Tuple/Unit |
| CV-24 | materialization | INV-18,42 | CallableFromPacks concrete collapses to Callable |
| CV-25 | query lifetime | INV-14 | no inference ID reaches metadata |
| CV-26 | stable scoped pack | INV-23,29 | TypePack/Labeled stable scoped export |
| CV-27 | stable complete/concat | INV-23,29 | complete + concat stable export |
| CV-28 | stable Tuple callable | INV-23 | TupleFromPacks/CallableFromPacks stable export |
| CV-29 | stable projection | INV-23 | PackProjection lane survives round trip |
| CV-30 | lambda mode | INV-24,26 | v4 top-level type lambda explicit modes |
| CV-31 | nested lambda mode | INV-24,26 | scoped nested lambda explicit modes |
| CV-32 | fingerprint | INV-25 | Single(TypePack) lambda != PositionalPack lambda |
| CV-33 | schema legacy | INV-27,28 | v1/v2/v3 valid fixtures still decode |
| CV-34 | schema floor | INV-28 | v3 rejects v4-only scoped/mode features |
| CV-35 | schema malformed | INV-28 | v4 rejects mode length/kind mismatch |
| CV-36 | JSON round trip | INV-23,24,28 | v4 pack alias bundle encode/decode/validate |
| CV-37 | cross-module | INV-23,24 | provider pack alias metadata remains structural |
| CV-38 | mode edit | INV-31,33 | Single(TypePack)->* invalidates and cold-parity |
| CV-39 | lane mode edit | INV-31,33 | `*`/`**`/`***` relevant edit parity |
| CV-40 | alias dependency | INV-31,32,33 | provider header edit recomputes Forward |
| CV-41 | concat edit | INV-33 | incremental concat edit equals cold |
| CV-42 | callable edit | INV-33,42 | CallableFromPacks edit parity |
| CV-43 | duplicate transition | INV-20,21,33 | symbolic -> duplicate parity |
| CV-44 | disjoint transition | INV-21,33 | symbolic -> valid concrete parity |
| CV-45 | source identity | INV-34 | expansion operand target stays TypeParameterId |
| CV-46 | unrelated reuse | INV-35 | unaffected module/product reused |
| CV-47 | flat positional density | INV-36,37 | many symbolic segments yield one flat concat |
| CV-48 | flat labeled density | INV-36,37 | many labeled segments remain flat |
| CV-49 | fixed generic | INV-39 | `Box<T>` semantics unchanged |
| CV-50 | fixed TypePack arg | INV-09,39 | `Holder<P:TypePack>` remains one fixed arg |
| CV-51 | empty specialization | INV-10,39 | fixed `<>` still invalid; pack-empty stays P2 behavior |
| CV-52 | C1 type members | INV-40 | Positionals/Labeled/At/Map representative suite green |
| CV-53 | domain distinction | INV-06,07 | labeled pack not row; complete not Tuple |
| CV-54 | runtime negative | INV-41 | no pack bytecode/object/solver added |
| CV-55 | callable boundary | INV-42 | open rest not coerced to finite pack callable |
| CV-56 | closure state | INV-44 | docs claim only executed verification |
| CV-57 | baseline policy | INV-43 | inherited trait failures not opportunistically repaired |
| CV-58 | cancellation/budget | INV-37 | dense normalization respects existing query controls if source path uses them |

Not every coverage item requires a unique test function. One well-designed test may satisfy several IDs.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Goal: implement G1/G2 quickly with sparse, discriminating feedback.

Do not climb the test ladder automatically.

#### STABILIZE MODE

Goal: prove coherent inference/persistence/incremental behavior after implementation is complete.

#### CERTIFY MODE

Goal: close C2 with the named focused checkpoint evidence.

`FOCUSED_TESTED` is the intended final verification state unless the user separately requests release certification.

---

### 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

G1 implementation:

```text
new `p3_` inference regressions only
one semantic generic-application probe if needed
cargo check -p phalcom-semantic only at coherent compile boundary
```

G2 implementation:

```text
new `p3_` schema-compat tests
new `p3_` semantic metadata tests
cargo check -p phalcom-type-meta
cargo check -p phalcom-semantic
```

Do not run affected suites after every enum/match edit.

### 16.4 Mandatory at named gates

G1:
- targeted inference tests;
- focused `foundations::inference` once coherent.

G2:
- `phalcom-type-meta` schema tests;
- focused semantic metadata tests.

G3:
- exact incremental/cross-module/source-index matrix.

G4:
- focused C1/C2 compatibility and density tests.

G5:
- affected crate checks;
- selected closure suites;
- formatting/diff hygiene;
- static runtime-negative audit.

### 16.5 Mandatory during STABILIZE

At the end of T5/G4, run the directly affected semantic modules listed in G4 below.

### 16.6 Only if evidence demands

Run:

```text
full phalcom-semantic integration binary
full phalcom-core tests
full language corpus
LSP tests
VM runtime suites
```

only if a focused failure creates a plausible cross-layer concern.

### 16.7 Explicitly deferred / DO NOT RUN during BUILD

Do not run during BUILD:

```sh
cargo test --workspace
cargo clippy --workspace
cargo test -p phalcom-core
```

unless a concrete affected boundary demands them.

Also defer:

```text
full language corpus
unrelated trait suites
broad VM suites
LSP/editor suites
performance benchmark suites
release packaging
```

Reason:

P3's active implementation gaps are localized to semantic inference and type metadata. Broad execution during BUILD would spend time rediscovering unrelated state and is not useful evidence.

---

## 17. Baseline / unrelated failure policy

Known inherited C1/C2 baseline failures:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Classify unexpected failures:

```text
A — definitely caused by P3
B — probably caused by P3
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active responsibility.
- C: one bounded classification pass; if still unclear, nonblocking, and outside active acceptance, record/defer.
- D: record and continue immediately.
- Do not use adviser compute on D unless it blocks active acceptance.
- Do not weaken assertions or skip required tests to make a broad gate green.
- If baseline confirmation is needed, compare non-destructively against the P2 revision `7da5b043...`.

Record deferred failures in the C2 checkpoint ledger.

---

# 18. Tasks

## T0 — Activate P3 and verify the bounded takeover state

### Purpose

Make the checkpoint state truthful and verify that the live local repository still satisfies the architectural premises used by this plan.

### Preconditions

- P2 is committed/pushed.
- P2 walkthrough and handoff exist.
- No architectural consultation is currently unresolved.

### Consumes

- P2 handoff/walkthrough.
- C2 checkpoint/guidance.
- this P3 plan.

### Produces

- active P3 checkpoint state;
- confirmed local starting revision/worktree;
- verified live symbols/version assumptions;
- no implementation change.

### Files and symbols

**Read:**

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P2-handoff.md
checker/inference.rs
metadata/export.rs
phalcom-type-meta/src/{header,scoped_type,validate}.rs
```

**Modify:**

```text
LANG007.C2-CHECKPOINT.md
```

Only lifecycle state.

### Required implementation shape

1. Run local repository status commands.
2. Confirm descendant/equivalent of P2 revision.
3. Confirm the two P3 implementation gaps still exist:
   - opaque C2 inference terms;
   - rejected scoped pack metadata / absent lambda stable modes.
4. Mark P3 active:
   ```yaml
   active_plan: LANG007.C2.P3
   next_plan: null
   ```
   while checkpoint remains:
   ```yaml
   status: IN_PROGRESS
   completion: PARTIAL
   verification: FOCUSED_TESTED
   ```
5. Add P3 to the checkpoint plan ledger as `IN_PROGRESS / PARTIAL / UNVERIFIED` or repository-equivalent truthful state.

### Forbidden approaches

- no semantic edits;
- no broad test run;
- no rewrite of P1/P2 historical records;
- no re-audit of unrelated subsystems.

### Test changes required

None.

### Tests to run now

None.

Use source inspection only.

### Tests explicitly deferred

All runtime/semantic suites.

### Acceptance

The checkpoint accurately records P3 active state and all architectural premises either remain true or have been classified as mechanical drift.

### Local STOP / CONSULT triggers

STOP if:

- local main lacks completed P2 architecture;
- a newer commit already implements a materially different P3 inference/persistence architecture;
- worktree state makes it unsafe to distinguish P3 edits from unrelated work.

### Checkpoint update

Mandatory: plan activation and starting revision.

---

## T1 — Add solver-local C2 pack structure and recursive inference lifting

### Purpose

Make all C2 compound types and P2 scoped pack aliases structurally visible to query-local inference variables without adding solving behavior yet.

### Preconditions

- T0 complete.
- P1/P2 canonical representation unchanged.

### Consumes

- `InferenceTerm`;
- P1/P2 `TypeData`;
- P2 `ScopedTypeData`;
- exact parameter substitution map.

### Produces

- solver-local pack term vocabulary;
- recursive `TypeId -> InferenceTerm` lifting;
- recursive scoped lambda -> inference lifting;
- complete traversal visibility for nested pack variables.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/checker/inference.rs
```

**Read, modify only if semantic classifier is needed:**

```text
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_member.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/inference.rs
```

### Required implementation shape

1. Add solver-local C2 variants described in §7.2.
2. Add `InferenceLabeledTypePackElement` or exact equivalent.
3. Add a solver-local pack projection lane discriminator.
4. Extend `type_id_to_inference`.
5. Extend `type_id_to_inference_with_rows`.
6. Extend `scoped_to_inference`.
7. Extend all non-solving traversals:
   - variable collection;
   - unresolved-variable detection;
   - support/proof propagation;
   - occurs traversal;
   - projection detection.
8. Add an owning-layer helper to classify the canonical P1 pack lane residual if no existing helper exists.
9. Do not implement arbitrary pack sequence matching in this task.

### Required projection-classifier behavior

It may classify only the canonical internal residual family that P1 creates:

```text
subject kind = CompleteTypePack
selector = canonical Positionals/Labeled/Complete lane identity
arguments = empty
result kind = matching lane kind
```

The helper must live with semantic type/projection ownership, not in source syntax.

### Forbidden approaches

- no `PackVarId`;
- no canonical `TypeId` containing query vars;
- no string checks scattered through inference;
- no generic type-member solver expansion beyond the required pack residual;
- no new parser/source syntax;
- no unification by pack length.

### Test changes required

Add targeted tests with a `p3_` prefix or equivalent clear grouping.

Required probes:

```text
CV-05..CV-13
```

Suggested tests:

```text
p3_c2_type_forms_lift_pack_parameters_into_inference_terms
p3_c2_inference_traversals_keep_nested_pack_variables_visible
p3_scoped_pack_lambda_lowers_to_inference_terms
p3_complete_pack_lane_projection_keeps_inference_subject
```

Use direct semantic construction where that most precisely tests the solver boundary.

### Tests to run now

Run only new regressions:

```sh
cargo test -p phalcom-semantic --test semantic foundations::inference::p3_c2_
```

If the filter selects zero tests, use:

```sh
cargo test -p phalcom-semantic --test semantic -- --list
```

then correct the filter.

After all T1 regressions pass and the enum is mechanically coherent:

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
```

### Tests explicitly deferred

Do not run:

```text
generic_application suite
metadata suites
incremental suites
workspace tests
core tests
```

T1 has not yet implemented full solving/materialization.

### Acceptance

- every C2 canonical/scoped form recursively exposes relevant `InferVarId`s;
- proof/support/occurs/unresolved traversals see them;
- no new solving semantics or durable identity is introduced;
- semantic crate compiles.

### Local STOP / CONSULT triggers

STOP if:

1. a C2 term cannot be represented without query vars entering `TypeStore`;
2. pack residual recognition requires changing canonical P1 identity;
3. row inference must be merged with pack inference;
4. nested lambda support becomes architecturally required rather than mechanically reachable.

### Checkpoint update

Record the new durable inference-term vocabulary only after T1 acceptance.

---

## T2 — Implement whole-pack solving, checked materialization, and lane-exact structural matching

### Purpose

Complete G1 by allowing whole pack/lane variables to solve through C2 structures while proving no sequence decomposition is introduced.

### Preconditions

- T1 recursive representation complete.
- nested pack variables are visible in solver terms.

### Consumes

- T1 inference terms;
- existing `unify_terms` / `subtype_terms`;
- TypeStore pack constructors/projections;
- existing projection fixed-point/replay machinery.

### Produces

- exact-kind whole-pack inference;
- deterministic Tuple/callable lane matching;
- C2 inference materialization;
- structured pack-formation failure;
- negative no-decomposition boundary.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/checker/inference.rs
```

**Possibly mechanical error propagation:**

```text
phalcom-semantic/src/checker/mod.rs
phalcom-semantic/src/checker/call.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/inference.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/capabilities/aliases.rs
```

### Required implementation shape

#### A. Exact whole-value binding

Reuse existing:

```rust
InferenceSession::bind(var, canonical_ty, store)
```

and its exact kind check.

No special pack binding map.

#### B. Structural pack unification

Implement deterministic rules from §7.7.

For concrete pack terms, compare exact length/order/labels.

For concat terms, compare segment-wise only when segment correspondence already exists structurally.

Do not flatten an unknown concat against a flat concrete pack to infer missing segments.

#### C. Tuple whole-lane view

Add a private helper equivalent to:

```rust
fn inference_tuple_lanes(
    tuple: &[InferenceTupleElement]
) -> (InferenceTerm /*TypePack*/, InferenceTerm /*LabeledTypePack*/)
```

or construct canonical/inference lane terms directly.

The helper must preserve:

```text
positional order
labeled order
labels
```

No search.

#### D. Callable whole-lane view

For finite `RestMode::None` parameters only, construct positional/labeled lane terms.

If open/rest semantics are present, return "not structurally matchable as finite pack callable" rather than coercing.

#### E. Materialization

Extend:

```text
materialize
materialize_for_expected
```

for every new C2 inference variant.

All materialization must call `TypeStore` canonical constructors.

#### F. Pack failures

Add a structured pack failure to `InferenceMaterializationFailure`.

Propagate duplicate-label invalidity without turning it into internal failure.

#### G. Projection replay

Mark solver-local `PackProjection` as projection-bearing.

Once its subject is solved, materialization/projection replay must reduce it through the existing P1 TypeStore projection authority.

#### H. Subtype handling

When a proper structural relation reaches `TupleFromPacks` or `CallableFromPacks`, use exact lane matching to select whole pack variables only where the lane correspondence is predetermined.

Do not define a new `TypePack` subtyping lattice.

After lane selection, preserve existing proper-type subtype rules for the surrounding Tuple/callable relation.

### Forbidden approaches

- no prefix/suffix subtraction;
- no split-point loop;
- no dynamic-programming pack matcher;
- no heuristic "shortest/longest pack wins";
- no converting labeled lanes to maps/rows;
- no mapping duplicate labels last-wins;
- no runtime rest reuse;
- no "if unsolved, Dynamic";
- no swallowing `PackFormationError`.

### Test changes required

Required coverage:

```text
CV-01..CV-04
CV-14..CV-24
CV-55
```

Suggested test names:

```text
p3_whole_pack_inference_binds_exact_pack_kinds
p3_tuple_from_packs_infers_one_whole_positional_lane
p3_callable_from_packs_infers_finite_whole_domain_lane
p3_complete_pack_projection_reduces_after_subject_solution
p3_pack_concat_inference_does_not_subtract_concrete_suffix
p3_two_unknown_pack_concat_does_not_partition_concrete_pack
p3_labeled_pack_concat_does_not_partition_labels
p3_inference_materialization_surfaces_duplicate_label_pack_error
p3_open_callable_rest_is_not_coerced_to_callable_from_packs
```

Add one source-level or type-application integration probe where an existing generic-inference path consumes a C2 compound signature.

Do not invent runtime variadic argument inference simply to create such a test.

### Tests to run now

During BUILD:

```sh
cargo test -p phalcom-semantic --test semantic foundations::inference::p3_
```

Then one integration probe:

```sh
cargo test -p phalcom-semantic --test semantic foundations::generic_application::p3_
```

At T2 coherent completion:

```sh
cargo test -p phalcom-semantic --test semantic foundations::inference
```

Do not broaden automatically after PASS.

### Tests explicitly deferred

```text
metadata
incremental
type_member_closure
full aliases
workspace
core runtime
```

### Acceptance

- whole positional/labeled/complete vars solve by exact kind;
- nested C2 structures participate in inference;
- Tuple/callable exact lanes can select whole pack vars;
- projections reduce after subject solution;
- duplicate labels remain fallible;
- all no-decomposition tests remain unsupported/non-solving;
- `foundations::inference` passes.

### Local STOP / CONSULT triggers

STOP if:

1. a valid required positive case only works with sequence partitioning;
2. callable lane inference requires changing runtime rest semantics;
3. subtype correctness requires a new pack variance theory;
4. materialization failure cannot preserve pack invalidity without changing global diagnostic authority;
5. one serious correction does not resolve a semantic solver failure.

### Checkpoint update

After G1 only, record:

```text
ordinary InferVarId(kind) proven for whole packs
C2 inference terms established
no decomposition boundary proven
```

---

## T3 — Persist scoped pack algebra and lambda binder modes under schema v4

### Purpose

Complete G2 by making P2 variadic aliases/type lambdas fully durable and versioned across metadata encoding/decoding.

### Preconditions

- G1 passes.
- in-memory `TypeLambdaData::parameter_modes` remains canonical.
- current metadata schema verified.

### Consumes

- P2 scoped pack algebra;
- canonical C2 TypeNode nodes;
- `GenericParameterModeRef`;
- current metadata graph/validator/fingerprint architecture.

### Produces

- stable scoped pack nodes;
- stable pack projection lane;
- explicit lambda parameter mode vectors;
- schema v4;
- v1-v3 compatibility;
- v4 encode/decode/validation/fingerprint proof.

### Files and symbols

**Modify:**

```text
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/validate.rs
phalcom-semantic/src/metadata/export.rs
```

**Possibly modify exports/display:**

```text
phalcom-type-meta/src/lib.rs
phalcom-core/src/typing/inspect.rs
```

**Tests:**

```text
phalcom-type-meta/tests/schema_compat.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
```

### Required implementation shape

#### A. Stable node vocabulary

Add stable variants equivalent to §7.9.

All stable IDs reference earlier topologically ordered nodes.

#### B. Lambda modes with legacy compatibility

Use an explicit optional compatibility field:

```rust
parameter_modes: Option<Box<[GenericParameterModeRef]>>
```

on:

```text
TypeLambdaRef
ScopedTypeNode::Lambda
```

Semantics:

```text
None -> legacy all-Single
Some -> explicit current modes
```

Current v4 exporter always emits `Some`.

Do not infer a current variadic mode from kind.

#### C. Schema bump

```text
TYPE_METADATA_SCHEMA_VERSION = 4
```

Preserve minimum supported version unless repository policy requires otherwise.

#### D. Feature floors

For schemas `<4`, reject:

```text
new scoped C2 pack variants
explicit lambda mode vectors
```

For schema `4`, require explicit mode vectors for all emitted lambdas and validate alignment.

Historical decoded schemas may retain `None`.

#### E. Kind validation

Validate all child/result-kind contracts from §7.12.

#### F. Fingerprints

Allocate new unique stable tags.

Include lambda modes.

Do not renumber existing tags.

#### G. Export

Remove `MetadataExportError::ScopedPackForm` only when all formerly rejected P2 scoped forms have stable lowering.

If the error variant has no remaining callers, delete it mechanically.

Export:

```text
ScopedTypeData::TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks
PackProjection
Lambda(parameter_modes)
```

#### H. Round trip

Use the existing JSON encoder/decoder/validator.

Do not build a TypeStore importer.

### Forbidden approaches

- no source-text serialization as semantic identity;
- no kind-only reconstruction of binder mode;
- no unversioned schema-v3 mutation for the new scoped semantics;
- no dropping labels/order;
- no serializing TypeId/ScopedTypeId raw arena indexes as external identity;
- no inference IDs in metadata;
- no runtime evaluator for new nodes;
- no rewriting historical fixtures to v4 just to avoid compatibility work.

### Test changes required

Required coverage:

```text
CV-25..CV-36
```

Add/update `schema_compat.rs` tests for:

```text
historical v1 decode
historical v2 decode
historical/current v3 decode
v4 scoped pack JSON round trip
v4 top-level lambda mode
v4 nested lambda mode
v3 rejects v4-only feature
v4 rejects missing explicit current mode vector
v4 rejects mode length mismatch
v4 rejects mode/kind mismatch
v4 rejects wrong pack child kind
v4 rejects wrong projection lane/result kind
```

Add semantic metadata export tests for representative aliases:

```phalcom
type AddHead<*Ts> = (Int, *Ts)
type Callable<*Ts> = (*Ts,) -> Result
```

and a complete-pack alias/projection case.

### Tests to run now

During BUILD:

```sh
cargo test -p phalcom-type-meta --test schema_compat p3_
```

Then:

```sh
cargo test -p phalcom-semantic --test semantic integration::metadata::p3_
```

At coherent T3 completion:

```sh
RUSTFLAGS='' cargo check -p phalcom-type-meta
RUSTFLAGS='' cargo check -p phalcom-semantic
```

### Tests explicitly deferred

```text
incremental matrix
cross-module provider/consumer
C1 closure
core runtime suite
workspace
```

### Acceptance

- every public scoped C2 form exports structurally;
- type-lambda modes persist;
- schema v4 is explicit;
- v1-v3 compatibility remains;
- malformed v4 forms fail closed;
- mode-only lambda metadata changes fingerprint;
- no solver identity leaks.

### Local STOP / CONSULT triggers

STOP if:

1. existing metadata compatibility policy explicitly forbids the planned v4 semantics;
2. preserving old fixtures requires changing semantic meaning of old lambdas;
3. a stable scoped form cannot be validated without source/compiler state outside the bundle;
4. a runtime consumer demands executable pack semantics;
5. another stable mode authority already exists and conflicts with this plan.

### Checkpoint update

After G2 record:

```text
schema v4
stable scoped pack node set
legacy lambda mode rule
v4 explicit mode rule
```

---

## T4 — Certify cross-module persistence, incremental invalidation, cold parity, and source identity

### Purpose

Prove the complete C2 durable dependency graph after inference/persistence closure.

### Preconditions

- G1 and G2 pass.
- public pack aliases can export stable metadata.

### Consumes

- staged `GenericBinderHeader`;
- alias contract dependency index;
- schema-v4 metadata;
- source-index P2 recursion;
- workspace incremental infrastructure.

### Produces

- provider/consumer persistence evidence;
- mode-only invalidation evidence;
- concat/Tuple/callable cold parity;
- source-index parity;
- unrelated-reuse evidence.

### Files and symbols

**Primarily tests:**

```text
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/query_ownership.rs
```

**Production edits only if a test reveals an owned dependency omission:**

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/builder.rs
```

### Required implementation shape

Build a small matrix of source snapshots, not dozens of redundant tests.

#### Scenario A — mode-only same-kind edit

Revision 1:

```phalcom
class C<P: TypePack> {}
```

Revision 2:

```phalcom
class C<*P> {}
```

Prove:

```text
kind remains TypePack
mode changes
public/interface fingerprint changes
dependent application/alias recomputes
cold == incremental
```

#### Scenario B — alias contract dependency

Provider:

```phalcom
class Target<*Ts> {}
```

Consumer:

```phalcom
type Forward<*Ts> = Target<*Ts>
```

Change provider binder mode/contract.

Prove:

```text
Forward invalidates
alias cycle graph semantics remain unchanged
provider-only edit reaches consumer
cold == incremental
```

#### Scenario C — structural composition edit

Use a Tuple/alias with symbolic pack concat and change one segment.

Compare final canonical/stable structure.

#### Scenario D — callable pack edit

Change finite callable pack domain/return and prove `CallableFromPacks` product/fingerprint parity.

#### Scenario E — duplicate transition

Start symbolic, then edit provider/substitution so labels become:

```text
valid disjoint
```

and separately:

```text
invalid duplicate
```

Cold/incremental diagnostics must agree.

#### Scenario F — source identity

Verify binder references inside:

```text
*P
**L
***Q
```

still target one canonical `TypeParameterId` after an edit.

#### Scenario G — unrelated reuse

Edit one provider contract and assert unrelated module/product reuse using existing session stats/effects style.

### Forbidden approaches

- no whole-workspace invalidation as a "correctness fix";
- no adding nominal contract edges to alias-cycle detection;
- no source-text fingerprint substitute;
- no test that only compares error counts;
- no duplicated dependency cache.

### Test changes required

Required coverage:

```text
CV-37..CV-46
```

Suggested `p3_` tests:

```text
p3_mode_only_pack_binder_edit_invalidates_dependents_to_cold_parity
p3_variadic_alias_contract_edit_preserves_cold_parity
p3_pack_concat_edit_matches_cold_canonical_form
p3_callable_from_packs_edit_matches_cold_metadata
p3_symbolic_labeled_duplicate_transition_matches_cold
p3_pack_expansion_source_targets_survive_incremental_edit
p3_unrelated_module_reuses_products_after_pack_contract_edit
```

Prefer extending existing P2 incremental helpers instead of building another workspace harness.

### Tests to run now

Run only new P3 filters first:

```sh
cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions::p3_
cargo test -p phalcom-semantic --test semantic incremental::fingerprints::p3_
cargo test -p phalcom-semantic --test semantic integration::imported_resolution::p3_
```

If source-index coverage lives elsewhere, run that exact module only.

At G3, also rerun the P2 predecessor regression:

```sh
cargo test -p phalcom-semantic --test semantic nominal_binder_contract_edit_recomputes_variadic_alias_to_cold_parity
```

Use the actual full filter if needed.

### Tests explicitly deferred

```text
density
C1 compatibility
core runtime
workspace
```

### Acceptance

- every required edit produces cold/incremental equivalence;
- mode-only same-kind edit is observable;
- alias contract invalidation remains precise;
- source identity remains canonical;
- unrelated work is reusable.

### Local STOP / CONSULT triggers

STOP if:

1. correct invalidation requires moving semantic authority out of existing dependency ownership;
2. provider pack metadata cannot cross module boundary without a new module-format architecture;
3. alias contract dependencies must participate in cycle detection;
4. only global invalidation restores parity;
5. retained old snapshots violate stable denotation for pack terms.

### Checkpoint update

Record G3 parity matrix and any intentional dependency correction.

---

## T5 — Certify hostile normalization, density, fixed-generic compatibility, and C1 closure

### Purpose

Prove that P3 did not accidentally expand solver power, break canonicalization, or regress predecessor semantics.

### Preconditions

- G1–G3 pass.

### Consumes

- final P3 inference/persistence architecture;
- P1/P2 canonical constructors;
- C1 structural type-member suite.

### Produces

- hostile solver-negative proof;
- density/flatness evidence;
- fixed generic regression proof;
- C1 compatibility evidence;
- domain-distinction proof.

### Files and symbols

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/inference.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/substitution.rs
phalcom-semantic/tests/semantic/foundations/type_members.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/capabilities/aliases.rs
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
```

Production changes only for causally owned defects.

### Required implementation shape

#### A. No-decomposition hostile matrix

Ensure tests exist for:

```text
Concat(P,[String]) = concrete
Concat(P,Q) = concrete
LabeledConcat(L1,L2) = concrete
```

No arbitrary solutions.

#### B. Duplicate-label matrix

Prove:

```text
immediate concrete duplicate -> error
symbolic overlap -> residual
later duplicate -> error
later disjoint -> concrete ordered pack
```

#### C. Density

Construct large symbolic concat inputs using stable canonical parameter forms.

Assert:

```text
outer node is one flat concat
no child is same concat variant
segment order preserved
node/segment count linear
```

Use a size such as 256 unless live test budgets indicate a smaller/larger value is more appropriate.

Do not benchmark elapsed time.

#### D. Fixed generics

Prove:

```text
Box<T> unchanged
Holder<P: TypePack> remains Single
partial ordinary application unchanged
Fixed<> remains invalid
```

#### E. P2 pack generics

Prove empty pack specialization and binder-aligned canonical arguments still work.

#### F. C1 structural operations

Re-run representative C1 owner tests for:

```text
Tuple Positionals
Tuple Labeled
At / [] identity
Map
TypePack Map
LabeledTypePack Map
Record/RecordRow distinction
Tuple zero-product identity
```

### Forbidden approaches

- no relaxing negative tests because solver can now solve more;
- no converting unsupported decomposition into "ambiguous" if it previously should remain underconstrained/unsupported;
- no timer-based performance threshold;
- no random fuzzing requirement;
- no unrelated cleanup/refactor.

### Test changes required

Required coverage:

```text
CV-17..CV-20
CV-43..CV-44
CV-47..CV-53
CV-58 where applicable
```

Suggested density tests:

```text
p3_dense_positional_concat_remains_flat
p3_dense_labeled_concat_remains_flat_and_ordered
```

### Tests to run now

First new hostile/density tests only.

Then G4 focused stabilization:

```sh
cargo test -p phalcom-semantic --test semantic foundations::inference
cargo test -p phalcom-semantic --test semantic foundations::generic_application
cargo test -p phalcom-semantic --test semantic foundations::type_model
cargo test -p phalcom-semantic --test semantic foundations::substitution
cargo test -p phalcom-semantic --test semantic foundations::type_members
cargo test -p phalcom-semantic --test semantic capabilities::aliases
cargo test -p phalcom-semantic --test semantic integration::metadata
cargo test -p phalcom-semantic --test semantic integration::type_member_closure
```

Run serially.

If a module is large and a narrower established C1/C2 filter proves the same laws, the implementer may use that narrower exact filter and record it.

### Tests explicitly deferred

```text
all semantic tests
workspace tests
full core VM tests
LSP
language corpus
clippy workspace
```

### Acceptance

- no pack decomposition exists;
- hostile duplicate transitions remain correct;
- concat structures remain flat;
- fixed generics unchanged;
- C1 structural laws green;
- no domain conflation.

### Local STOP / CONSULT triggers

STOP if:

1. G1 behavior causes a legitimate fixed generic regression;
2. a C1 owner test reveals semantic conflict rather than mechanical exhaustiveness;
3. density exposes recursive/exponential behavior requiring a new canonical representation;
4. preserving no-decomposition conflicts with a ratified source example.

### Checkpoint update

Record G4 compatibility/resource evidence, not every test command.

---

## T6 — Certify the runtime-negative boundary and close LANG007.C2

### Purpose

Perform final focused checkpoint certification, update lifecycle state truthfully, and produce durable P3/C2 completion records.

### Preconditions

- G1–G4 pass.
- no unresolved P3-owned blocker.
- baseline failures classified.

### Consumes

- final implementation;
- coverage ledger;
- checkpoint verification evidence.

### Produces

- runtime-negative proof;
- affected crate checks;
- C2 closure;
- P3 walkthrough;
- P3 handoff;
- PROGRAM/STATUS rebaseline.

### Files and symbols

**Audit:**

```text
phalcom-core/src
phalcom-semantic/src
```

with focused searches only.

**Documentation:**

```text
docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-walkthrough.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-handoff.md
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
```

### Required implementation shape

#### A. Runtime-negative audit

Search for accidental P3 runtime additions:

```text
new bytecodes
new VM pack operations
runtime TypePack classes/objects
ArgumentPack used as semantic TypePack
runtime pack solver
CallableFromPacks execution branch
```

The acceptable P3 core changes are structural metadata inspection/display/exhaustiveness only.

#### B. Affected crate checks

Run:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' cargo check -p phalcom-type-meta
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo check -p phalcom-core
git diff --check
```

#### C. Type-meta focused package

Run:

```sh
cargo test -p phalcom-type-meta
```

The package is small and directly owned by G2.

#### D. Final C2 selected semantic gates

Re-run only the coherent closure modules necessary to prove C2.

Do not automatically run the full semantic integration binary if G1–G4 already produced the required evidence.

#### E. Checkpoint state

If all required evidence passes:

```yaml
LANG007.C2.P3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C2:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

Do not claim `RELEASE_COMPLETE`.

#### F. Roadmap rebaseline

Update active future-facing sections of:

```text
PROGRAM.md
STATUS.md
C2 CHECKPOINT
```

Remove stale statements saying P2/P3 is next.

Set the next action according to the live PROGRAM roadmap.

Do not invent a C3 title if PROGRAM does not currently define one; record the exact next accepted action that exists.

### Forbidden approaches

- no runtime implementation added to "prove" boundary;
- no broad historical documentation rewrite;
- no claim of workspace/release certification;
- no opportunistic unrelated baseline repair;
- no closing C2 with an unchecked truth-table row.

### Test changes required

No new feature tests should normally be added in T6.

If the static audit exposes a missing boundary regression, add the smallest owned test.

Required coverage:

```text
CV-54
CV-56
CV-57
```

### Tests to run now

G5 commands are in §19 below.

### Tests explicitly deferred

Unless user explicitly requests release certification:

```sh
cargo test --workspace
cargo clippy --workspace
```

Also defer:

```text
full language corpus
broad LSP
broad VM/runtime suite
release packaging
```

### Acceptance

- runtime-negative audit passes;
- affected crates compile;
- required focused suites pass or unrelated baselines are classified;
- checkpoint truth table is satisfied;
- P3 walkthrough/handoff exist;
- PROGRAM/STATUS/CHECKPOINT agree;
- C2 is truthfully closed.

### Local STOP / CONSULT triggers

STOP if:

1. a runtime pack execution path already exists intentionally and conflicts with C2 authority;
2. final focused acceptance requires a new architectural patch beyond G1/G2 scope;
3. a broad failure plausibly invalidates C2 soundness;
4. closure state cannot be made consistent without changing checkpoint objective.

### Checkpoint update

Mandatory final closure.

---

# 19. Verification gates

## G1 — Whole-pack inference and materialization closure

### Purpose

Prove ordinary kind-aware inference can see and solve whole C2 pack structures without decomposition.

### Required evidence

Run:

```sh
cargo test -p phalcom-semantic --test semantic foundations::inference::p3_
cargo test -p phalcom-semantic --test semantic foundations::generic_application::p3_
```

Then:

```sh
cargo test -p phalcom-semantic --test semantic foundations::inference
```

Expected:

```text
whole TypePack/LabeledTypePack/CompleteTypePack variables solve
C2 compound parameters are solver-visible
scoped C2 aliases beta-reduce into inference terms
Tuple/callable exact lanes can select whole pack values
projection resolves after subject solution
duplicate labeled materialization is fallible
concat subtraction/split tests remain non-solving
no PackVarId exists
```

Do not broaden after PASS.

On failure: classify before editing.

---

## G2 — Stable scoped metadata and schema-v4 gate

### Purpose

Prove public variadic type lambdas are durable and schema-compatible.

Run:

```sh
cargo test -p phalcom-type-meta --test schema_compat
cargo test -p phalcom-semantic --test semantic integration::metadata
```

Expected:

```text
v1-v3 valid fixtures still decode
v4 emits/validates scoped pack nodes
v4 lambda modes are explicit
legacy absent modes mean Single
malformed mode/kind/scope forms reject
pack aliases encode/decode structurally
mode-only lambda changes fingerprint
no inference IDs export
```

Then:

```sh
RUSTFLAGS='' cargo check -p phalcom-type-meta
RUSTFLAGS='' cargo check -p phalcom-semantic
```

Do not broaden after PASS.

---

## G3 — Cross-module and incremental parity gate

### Purpose

Prove durable dependencies and cold/incremental equivalence.

Run the exact new P3 tests:

```sh
cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions::p3_
cargo test -p phalcom-semantic --test semantic incremental::fingerprints::p3_
cargo test -p phalcom-semantic --test semantic integration::imported_resolution::p3_
```

Also run the existing P2 contract-edit regression by its actual filter.

Expected:

```text
mode-only same-kind edit invalidates
alias contract edit propagates
concat/Tuple/callable edit equals cold
duplicate transition equals cold
source targets remain canonical
unrelated products reuse
```

Do not run all incremental tests unless an owned failure justifies it.

---

## G4 — Hostile, density, fixed-generic, and C1 compatibility gate

### Purpose

Prove the final C2 architecture has not widened into decomposition or regressed predecessor semantics.

Run serially:

```sh
cargo test -p phalcom-semantic --test semantic foundations::inference
cargo test -p phalcom-semantic --test semantic foundations::generic_application
cargo test -p phalcom-semantic --test semantic foundations::type_model
cargo test -p phalcom-semantic --test semantic foundations::substitution
cargo test -p phalcom-semantic --test semantic foundations::type_members
cargo test -p phalcom-semantic --test semantic capabilities::aliases
cargo test -p phalcom-semantic --test semantic integration::type_member_closure
```

Expected:

```text
all P3 hostile tests pass
flat density tests pass
fixed generic behavior unchanged
Single(TypePack) distinction retained
C1 type-member semantics retained
no RecordRow/pack or Tuple/complete conflation
```

If inherited trait baselines appear only after a broader accidental filter, classify and defer.

---

## G5 — Runtime-negative and checkpoint closure gate

### Purpose

Establish final focused certification and close C2.

Run:

```sh
cargo test -p phalcom-type-meta

RUSTFLAGS='' cargo check -p phalcom-type-meta
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo check -p phalcom-core

cargo fmt --all -- --check
git diff --check
```

Perform focused static searches proving no new:

```text
pack VM opcode
pack runtime object
pack runtime solver
ArgumentPack-as-TypePack representation
CallableFromPacks runtime execution
```

Expected:

```text
all required plan coverage has evidence
affected crates compile
type-meta package passes
format/diff clean
runtime-negative audit clean
checkpoint records truthful
```

Do not run workspace certification automatically.

---

## 20. Final focused acceptance

At completion, the walkthrough must provide the result of every applicable coverage obligation.

Use a table:

| Coverage ID | Test/evidence | Result |
|---|---|---|
| CV-01 | exact test | PASS |
| ... | ... | ... |
| CV-58 | test/audit or N/A with reason | PASS / N/A |

"N/A" is valid only when the dimension truly does not apply and the walkthrough explains why.

### Mandatory acceptance summary

The implementer must explicitly certify:

```text
[ ] ordinary InferVarId(kind) handles whole packs
[ ] C2 compound terms recursively expose generic inference variables
[ ] scoped pack aliases lower through symbolic inference
[ ] no pack decomposition solver exists
[ ] pack materialization re-enters canonical TypeStore APIs
[ ] duplicate labels fail after inference/substitution
[ ] stable scoped pack metadata exists
[ ] type-lambda modes persist independently of kind
[ ] schema v4 compatibility tests pass
[ ] no inference IDs enter durable metadata
[ ] cross-module pack products survive metadata boundary
[ ] mode-only same-kind edits invalidate dependents
[ ] incremental final state equals cold final state
[ ] source references remain canonical
[ ] concat remains flat/bounded
[ ] fixed generic programs remain unchanged
[ ] C1 structural operations remain green
[ ] no runtime pack object/opcode/solver exists
[ ] baseline failures are classified
[ ] walkthrough and handoff exist
[ ] PROGRAM/STATUS/CHECKPOINT agree
[ ] C2 closure state is truthful
```

Do not automatically run workspace certification after all boxes are satisfied.

---

## 21. Performance/resource evidence

P3 has a bounded resource goal, not a wall-clock performance target.

### Required metric

Structural evidence:

```text
N symbolic concat inputs
    -> O(N) canonical children/nodes
    -> one flat concat residual
```

Recommended:

```text
N = 256
```

or another moderate size already conventional in semantic stress tests.

For labeled concat:

```text
N symbolic labeled-pack segments
    -> N ordered flat residual segments
    -> no nested LabeledTypePackConcat child
```

For metadata:

```text
reachable stable node count grows linearly with source semantic graph
validation stays within default ValidationLimits
```

### Unacceptable evidence

Do not introduce:

```text
unstable nanosecond thresholds
criterion benchmark requirement
large random fuzz workload
workspace-wide timing gate
```

unless live evidence reveals an actual performance defect requiring dedicated work.

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

T0 must:

- mark P3 active;
- record starting revision;
- add/update P3 ledger row;
- keep C2 `IN_PROGRESS / PARTIAL`.

### 22.2 During plan

Update checkpoint only for durable events.

After G1:

```text
whole-pack inference architecture established
no-decomposition proof recorded
```

After G2:

```text
schema v4
stable scoped pack metadata
lambda legacy/current mode rule
```

After G3:

```text
incremental/cold parity matrix
```

After G4:

```text
hostile/density/fixed/C1 compatibility evidence
```

Consultations:

record concise `INC-*` entries with trigger, decision, amendment, and required verification.

Deferred failures:

record only coherent baseline/deferred findings.

### 22.3 At plan completion

Update:

```text
P3 ledger -> COMPLETE / IMPLEMENTED / FOCUSED_TESTED
C2 -> COMPLETE / IMPLEMENTED / FOCUSED_TESTED
active_plan -> null
next action -> exact live roadmap successor
```

Record:

- final schema version;
- final inference term architecture;
- runtime-negative proof;
- final coverage/gate evidence;
- deferred baselines;
- consultations;
- next action.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-walkthrough.md
```

It must contain:

1. starting revision;
2. ending revision;
3. final plan/checkpoint state;
4. architecture implemented;
5. exact solver-local C2 variants;
6. exact whole-pack inference boundary;
7. exact no-decomposition behavior;
8. exact stable scoped metadata variants;
9. final metadata schema compatibility contract;
10. final lambda mode representation;
11. files changed;
12. important deviations;
13. consultations and decisions;
14. tests added;
15. tests actually executed;
16. tests deliberately deferred;
17. G1–G5 evidence;
18. incremental parity matrix;
19. density/resource observations;
20. runtime-negative audit;
21. baseline/deferred failures;
22. verification classification;
23. C2 closure truth table;
24. next program action.

Never claim a test passed if it did not run.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-handoff.md
```

Even though P3 closes C2, provide a compact successor handoff so the next LANG007 planning session does not rediscover C2.

Include:

- ending/current revision;
- C2 complete status;
- stable pack domains;
- final inference architecture;
- final schema version;
- lambda mode compatibility rule;
- canonical pack constructors/projections;
- cross-module/incremental guarantees;
- no-decomposition boundary;
- runtime-negative boundary;
- inherited baselines;
- must-read C2 records;
- next roadmap objective from live PROGRAM;
- first recommended next-session commands;
- explicit "do not redesign C2 pack substrate" guidance.

Do not invent a successor checkpoint objective if PROGRAM has not ratified one.

---

## 25. Completion truth table

Do not conflate these states.

| State | Meaning | Required for P3 completion? |
|---|---|---:|
| source written | implementation edits exist | yes |
| tests added | required regressions exist | yes |
| targeted tests passed | G1/G2/G3 exact evidence | yes |
| affected suites passed | G4/G5 focused evidence | yes |
| checkpoint accepted | C2 truth table satisfied | yes |
| workspace certified | entire workspace suite passed | no |
| release certified | release gates passed | no |

Final intended state:

```yaml
plan:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

checkpoint:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

If a required gate is baseline-blocked in a way that prevents proof of P3 behavior, do not claim `FOCUSED_TESTED`; record the truthful verification state and consult if acceptance is ambiguous.

---

## 26. Plan self-review

This plan has been reviewed against the Luna patch-grade schema and principal-architect doctrine.

### Architecture

- [x] one canonical owner exists for pack kinds/terms;
- [x] one query-local owner exists for inference variables;
- [x] `GenericInstantiation` remains durable binding authority;
- [x] pack projection authority remains P1/TypeStore;
- [x] stable metadata owns persistence, not source reconstruction;
- [x] lambda mode identity is explicit;
- [x] runtime boundary is explicit.

### Luna executability

- [x] P3's two actual implementation gaps are named precisely;
- [x] solver-local representation is pre-decided;
- [x] unification boundary is pre-decided;
- [x] schema compatibility strategy is pre-decided;
- [x] fixed/flexible/verify-first boundaries are explicit;
- [x] task dependency order is coherent;
- [x] STOP/CONSULT triggers are objective;
- [x] semantic debugging gets one serious correction;
- [x] mechanical debugging is bounded.

### Testing

- [x] relevant correctness dimensions have CV IDs;
- [x] BUILD execution is narrow;
- [x] G1–G5 deliberately broaden;
- [x] no workspace test is implicitly mandatory;
- [x] inherited baselines are handled;
- [x] structural density evidence replaces benchmark ceremony.

### Documentation lifecycle

- [x] checkpoint activation is T0;
- [x] durable gate updates are named;
- [x] walkthrough is required;
- [x] handoff is required;
- [x] C2 closure is tied to an explicit truth table.

### Premium-compute efficiency

- [x] likely architecture forks are pre-decided;
- [x] consultations require compressed incident packets;
- [x] no premium adviser is required for mechanical work;
- [x] unrelated failures are explicitly deferred.

No placeholders remain.

---

# 27. Compact Luna executor directive

```text
Implement LANG007.C2.P3 as the closure of C2, not as a new variadic feature.

Start from completed P1/P2.

G1:
Make every C2 pack compound recursively visible in InferenceTerm.
Keep ordinary InferVarId(kind) as the only pack unknown.
Support whole-pack/lane inference only.
Never solve unknown pack partitions.
Materialize only through TypeStore canonical constructors.
Preserve duplicate-label failure.

G2:
Make the P2 scoped pack algebra durable in phalcom-type-meta.
Persist TypeLambda parameter modes independently of kind.
Advance the metadata schema to v4 with explicit current modes and legacy v1-v3 all-Single compatibility.
Fingerprint every new structural fact.
Do not build a semantic metadata importer.

G3:
Prove mode-only and provider contract edits invalidate the right dependents and equal cold analysis.
Keep alias contract dependencies separate from alias-cycle ordering.
Preserve source TypeParameterId identity and unrelated reuse.

G4:
Prove no decomposition, checked duplicate behavior, flat/bounded concat, fixed-generic compatibility, and C1 structural compatibility.

G5:
Prove no runtime pack object/opcode/solver/ABI change.
Run only the named affected checks.
Close P3 and C2 as COMPLETE / IMPLEMENTED / FOCUSED_TESTED if and only if the closure truth table is satisfied.

Do not:
- invent PackVarId/PackId;
- infer mode from kind;
- convert labeled packs to RecordRow;
- convert complete pack identity to Tuple;
- use ... as pack syntax;
- introduce runtime pack semantics;
- repair unrelated trait baselines;
- run workspace tests/clippy reflexively.

STOP AND CONSULT immediately when a RED trigger fires.
```

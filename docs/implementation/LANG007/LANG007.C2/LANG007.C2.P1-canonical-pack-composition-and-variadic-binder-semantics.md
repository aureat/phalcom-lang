---
id: LANG007.C2.P1
category: LANG
program: LANG007
checkpoint: LANG007.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: main
planning_baseline_revision: 1a1e5ab022c84ad979a28994a5a135121baaaeaa
planning_baseline_subject: "feat(lang007): continue C2 universe implementation"
prepared: 2026-09-22
depends_on:
  - LANG007.C1 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
  - LANG007.C2 accepted requirements analysis
  - LANG007.C2-CHECKPOINT.md
  - LANG007.C2-GUIDANCE.md
follows: LANG007.C1.P3
supersedes: null
next_plan: LANG007.C2.P2
---

# LANG007.C2.P1 — Canonical Pack Composition and Variadic Binder Semantics

## 0. Executor contract

This is a **patch-grade Luna implementation plan** for the first numbered plan of `LANG007.C2`. It is intentionally narrower than the full checkpoint. The implementer is expected to perform routine Rust implementation, mechanical exhaustiveness updates, focused tests, lifecycle bookkeeping, and ordinary compiler cleanup without re-deriving the architecture.

The implementer:

- may adapt helper names, private file decomposition, imports, and equivalent local APIs to the live checkout;
- must preserve the semantic ownership, canonical identities, lifetime boundaries, and metadata contracts fixed below;
- must not improvise source syntax, pack decomposition inference, runtime pack objects, or VM execution;
- must not silently substitute `...` for `*` / `**` / `***` anywhere in generic-pack work;
- must use sparse, discriminating test feedback in BUILD MODE and must not run broad suites reflexively;
- must classify unrelated/baseline failures rather than repairing them;
- must STOP AND CONSULT when an objective escalation trigger fires;
- must keep the shared `LANG007.C2-CHECKPOINT.md` current at durable state transitions;
- must create the P1 walkthrough and P1→P2 handoff at completion.

Testing is evidence gathering, not ritual. A focused PASS is evidence to continue implementation, not a reason to climb automatically to broader suites.

---

# 1. Goal

Establish the **canonical semantic substrate** for C2: an exact complete two-lane type-pack domain, explicit generic-parameter binder modes, canonical flat positional/labeled symbolic concat terms, checked re-canonicalization through substitution/materialization, stable metadata/fingerprint representation, and compatibility with the completed C1 `TypePack` / `LabeledTypePack` / `TupleFromPacks` architecture.

P1 must leave P2 able to implement source `*` / `**` / `***` generic binders and expansions by elaborating directly into P1 terms without inventing parser-specific pack semantics.

---

# 2. Checkpoint acceptance objective and P1 role

The owning checkpoint `LANG007.C2` is accepted only when Phalcom can statically capture, compose, expand, substitute, persist, and incrementally re-evaluate positional, labeled, and complete type packs using the ratified lane algebra:

```text
*     positional lane
**    labeled lane
***   complete two-lane pack
```

P1 is **one of three dependency-ordered plans** and does **not** close the checkpoint.

The expected checkpoint decomposition is:

```text
C2.P1
    canonical pack domains/composition + binder semantics + metadata foundations
        ↓
C2.P2
    source generic binders/applications + Tuple/callable type unpacking
        ↓
C2.P3
    inference/persistence/incremental certification + checkpoint closure
```

P1 is successful when the compiler can represent and normalize the following **semantically**, even though the broad source grammar is not yet enabled:

```text
TypePack
LabeledTypePack
CompleteTypePack(TypePack, LabeledTypePack)

TypePackConcat(...)
LabeledTypePackConcat(...)

GenericParameterMode::Single
GenericParameterMode::PositionalPack
GenericParameterMode::LabeledPack
GenericParameterMode::CompletePack
```

with exact kinding, one canonical identity for concrete results, deferred labeled-overlap obligations for symbolic compositions, checked invalidation when substitution/materialization proves an overlap, durable metadata, and no runtime/VM execution path.

P1 does **not** need to parse `<*Ts>`, `C<*P>`, `(*P,)`, or `(***P,) -> R`. Those are P2 consumers of the substrate established here.

---

# 3. Repository grounding

Prepared against remote `main`:

```text
repository: aureat/phalcom-lang
branch: main
revision: 1a1e5ab022c84ad979a28994a5a135121baaaeaa
subject: feat(lang007): continue C2 universe implementation
previous checkpoint: LANG007.C1 — COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

The planning audit verified the following live facts.

1. `LANG007.C1` is closed and its C1 architecture is the substrate, not work to reimplement.
2. Remote `main` contains `docs/implementation/LANG007/LANG007.C2/LANG007.C2-requirements-analysis.md`, status `ACCEPTED`.
3. Remote `main` does **not yet** contain the newly prepared companion `LANG007.C2-CHECKPOINT.md` and `LANG007.C2-GUIDANCE.md` records. They are part of this planning package and must be materialized together with this plan before implementation begins.
4. `TypeId` is already the store-local canonical carrier for kinded semantic forms; `ProperTypeId` is the proof boundary for kind `Type`.
5. `KindData` currently has `Type`, `RecordRow`, `TypePack`, `LabeledTypePack`, and `Arrow`.
6. `KindId` has fixed base constants through `LABELED_TYPE_PACK`.
7. `TypeData` already contains `TypePack`, `LabeledTypePack`, and `TupleFromPacks`.
8. `TypeStore::tuple_from_packs` already collapses fully concrete positional/labeled packs into the existing concrete Tuple/Unit identity.
9. `GenericInstantiation` already stores non-row generic bindings as `TypeId` and keeps `RecordRowId` separate.
10. `GenericApplicationSession` already gives every non-`RecordRow` kind an ordinary kind-aware `InferVarId`; no pack-specific solver identity exists or is needed by P1.
11. `TypeParameterData` has owner/index/name/kind/variance/source but no binder mode.
12. `GenericSignature` has aligned parameter/kind/variance/shape data but no binder mode.
13. `TypeStore::intern_type_parameter` currently reuses an owner/index slot when name/kind/variance match; P1 must make mode part of that semantic-version check.
14. Stable metadata already represents C1 `TypePack`, `LabeledTypePack`, and `TupleFromPacks` through `KindNode` / `TypeNode`, but has no complete-pack/concat/binder-mode forms.
15. `TypeParameterRecord` currently contains stable id/name/kind/variance/source only.
16. `TYPE_METADATA_SCHEMA_VERSION` is currently `2`; the format is explicit, versioned JSON metadata.
17. `CompiledKindRef` in `phalcom-semantic/src/export.rs` exhaustively mirrors base kinds and therefore must be extended mechanically for an exact complete-pack kind.
18. `phalcom-core/src/typing/inspect.rs` exhaustively formats/inspects stable kind/type nodes; P1 metadata additions will require compatibility handling there, but **not** runtime pack objects or execution.
19. `TypeSubstitution::apply` is currently infallible. Labeled symbolic composition introduces a new case where a previously symbolic term may become invalid after substitution, so P1 needs an explicit checked path rather than `unwrap_or(original)` invalidity hiding.
20. Existing C1 metadata and type-member tests provide direct regression oracles for pack-domain identity and deterministic fingerprints.

The latest commit also touched several Universe source files, but those changes do not establish C2 type-pack syntax or a competing semantic pack representation. P1 should not modify Universe merely because the latest commit subject mentions C2.

**Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.**

---

# 4. Planning-package precondition

Before production edits, the local checkout must contain:

```text
docs/implementation/LANG007/LANG007.C2/
    LANG007.C2-requirements-analysis.md
    LANG007.C2-CHECKPOINT.md
    LANG007.C2-GUIDANCE.md
    LANG007.C2.P1-canonical-pack-composition-and-variadic-binder-semantics.md
```

At the planning baseline, only the requirements analysis is on remote `main`. The checkpoint/guidance/plan are companion planning artifacts produced after that revision.

If the checkpoint or guidance artifact is unavailable to the implementer:

> **STOP AND CONSULT. Do not reconstruct them from memory, this plan, or the older roadmap.**

Copying the supplied lifecycle artifacts into their canonical directory is documentation setup, not semantic implementation. Do not alter their accepted semantic content as part of that copy.

---

# 5. Required reads before implementation

Read in this order and stop once the active architecture is established; do not broadly rediscover the repository.

```text
1. AGENTS.md
2. docs/workflow/implementation-record-lifecycle-convention.md
3. docs/workflow/luna-patch-grade-plan-schema.md
4. docs/workflow/shared-consultation-escalation-protocol.md
5. docs/implementation/LANG007/PROGRAM.md
6. docs/implementation/LANG007/STATUS.md
7. docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
8. docs/implementation/LANG007/LANG007.C2/LANG007.C2-GUIDANCE.md
9. this plan
10. docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-walkthrough.md
11. docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-handoff.md
12. docs/implementation/LANG007/LANG007.C2/LANG007.C2-requirements-analysis.md
13. docs/spec/collections-next/argument-packs-and-expansion-spec.md
14. docs/spec/collections/06-rest-spread-and-pack-operators.md
15. docs/spec/collections/07-callable-domains.md
16. phalcom-semantic/tests/semantic/README.md
17. the exact production files named by the current task
```

Important authority correction:

`LANG007.C1.P3-handoff.md` still contains old future-work examples such as `...P` / `<...T>`. Those examples are superseded by the accepted C2 requirements and the ratified pack-operator specifications. They are **not** current syntax authority.

---

# 6. Normative authority

## 6.1 Ratified lane algebra

The operator meanings are fixed throughout the language:

```text
*     positional lane
**    labeled lane
***   complete two-lane pack
```

The surrounding grammar may mean capture, expansion, or type unpacking, but the lane selected by each operator does not change.

`...` remains independent syntax and must not be introduced or accepted as generic pack syntax.

## 6.2 Complete pack semantics

A complete type pack is an exact type-level lane product:

```text
CompleteTypePack(P, L)

P : TypePack
L : LabeledTypePack
```

It is neither:

```text
Type
Tuple
TypePack
LabeledTypePack
RecordRow
runtime ArgumentPack
```

Forming a Tuple from a complete pack must go through C1's existing `TupleFromPacks` authority.

## 6.3 Binder mode is orthogonal to kind

These are distinct public generic interfaces:

```phalcom
P: TypePack
*P
```

Both involve the `TypePack` domain, but the first consumes one generic argument whose value is a pack while the second captures a variable-length positional source sequence.

P1 must therefore store binder mode independently of kind.

## 6.4 Concrete canonicalization

When symbolic composition becomes fully concrete:

```text
TypePackConcat(concrete...)          -> TypePack(...)
LabeledTypePackConcat(concrete...)   -> LabeledTypePack(...)
```

There must never be two durable identities for the same concrete pack.

## 6.5 Labeled uniqueness

Concrete duplicate labels are invalid. Symbolic unknown overlap remains an unresolved composition term until later evidence makes it either concrete-valid or concrete-invalid.

No “last wins” rule exists.

## 6.6 C1 authority remains intact

P1 must preserve C1's accepted invariants, including:

```text
TypePack != Tuple
LabeledTypePack != RecordRow
Tuple([], []) == Unit
TypeId is kinded; ProperTypeId proves kind Type
query-local inference IDs never become durable semantic IDs
compiler-intrinsic type computation does not execute in the VM
intrinsic is a source keyword; @intrinsic remains retired
```

---

# 7. Takeover state

| Concept | Current path / symbol | Owner | P1 invariant |
|---|---|---|---|
| kind identity | `phalcom-semantic/src/types/{id,kind,store}.rs` — `KindId`, `KindData`, `TypeStore` | semantic type store | extend existing base-kind authority; no second kind registry |
| proper-type proof | `ProperTypeId` | semantic type store | complete/concat terms are not proper types merely because they are `TypeId`s |
| positional pack | `TypeData::TypePack` | semantic type store | preserve ordered concrete pack identity |
| labeled pack | `TypeData::LabeledTypePack` | semantic type store | preserve ordered labels and uniqueness |
| Tuple formation | `TypeData::Tuple`, `TypeData::TupleFromPacks`, `TypeStore::tuple_from_packs` | semantic type store | complete pack never becomes a second Tuple identity |
| generic parameter identity | `TypeParameterId`, `TypeParameterData` | semantic type store | binder mode becomes part of parameter semantics/versioning |
| generic signature | `GenericSignature` | semantic generic subsystem | parameter mode is aligned with parameter/kind/variance slot |
| generic binding store | `GenericInstantiation` | semantic generic subsystem | non-row pack bindings remain `TypeId`; no `PackInstantiation` |
| substitution | `TypeSubstitution` | semantic type subsystem | new terms recurse and re-canonicalize; invalid concrete labeled overlap is surfaced honestly |
| materialization | `materialize_type` / `GenericInstantiation` | semantic type subsystem | remains fallible and becomes pack-aware |
| generic inference | `InferenceSession`, `GenericApplicationSession` | checker | complete-pack kind uses ordinary `InferVarId`; no `PackVarId` |
| row inference | `RecordRowSolver`, `RecordRowVarId` | checker | remains distinct; labeled pack never coerces to row |
| C1 projections | `types/projection.rs`, `TypeLevelProjection` | semantic type subsystem | reuse/factor existing Tuple lane extraction logic; do not duplicate semantics |
| stable kind metadata | `phalcom-type-meta/src/kind.rs` | metadata schema | add exact complete-pack kind without renumbering old fingerprints |
| stable type metadata | `phalcom-type-meta/src/type_node.rs` | metadata schema | add complete/concat nodes with explicit tags |
| stable parameter metadata | `phalcom-type-meta/src/generic.rs` | metadata schema | binder mode becomes durable public interface data |
| metadata validation | `phalcom-type-meta/src/validate.rs` | metadata schema | enforce schema and new-node shape/kind invariants |
| metadata export | `phalcom-semantic/src/metadata/export.rs` | semantic publication | export new terms/mode and hash them structurally |
| compiler seam kind export | `phalcom-semantic/src/export.rs` — `CompiledKindRef` | semantic→compiler seam | exact complete kind represented; pack type terms may remain non-exportable through this legacy seam unless live evidence requires more |
| runtime typing inspection | `phalcom-core/src/typing/inspect.rs` | metadata consumer | inspect/format stable nodes only; no runtime pack execution |

---

# 8. Architecture

## 8.1 Semantic flow

P1 establishes this flow:

```text
canonical C1 pack terms / generic parameter facts
        │
        ├───────────────┐
        ▼               ▼
CompleteTypePack     GenericParameterMode
        │               │
        ▼               ▼
TypePackConcat / LabeledTypePackConcat
        │
        ▼
checked normalization after substitution/materialization
        │
        ▼
canonical TypeId graph
        │
        ▼
stable KindNode / TypeNode / TypeParameterRecord
        │
        ▼
metadata validation + structural/interface fingerprints
```

No VM/runtime stage participates in evaluating or solving these type-level terms.

## 8.2 Exact complete-pack kind

Unless T0 discovers an already-landed exact equivalent abstraction, add a new internal base kind conceptually equivalent to:

```rust
KindData::CompleteTypePack
KindId::COMPLETE_TYPE_PACK
```

The preferred deterministic base-kind order is:

```text
TYPE                 = 0
RECORD_ROW           = 1
TYPE_PACK            = 2
LABELED_TYPE_PACK    = 3
COMPLETE_TYPE_PACK   = 4
```

Arrow kinds remain dynamically interned after the base kinds.

The exact Rust spelling is mechanically flexible; the semantic distinction is FIXED.

Do **not** add a public `KindSyntax::CompleteTypePack` in P1. No public source name for the complete-pack kind is ratified.

## 8.3 Complete pack canonical term

Add one canonical `TypeData` form conceptually equivalent to:

```rust
TypeData::CompleteTypePack {
    positionals: TypeId, // kind TypePack
    labeled: TypeId,     // kind LabeledTypePack
}
```

Construction must be checked:

```rust
complete_type_pack(positionals, labeled) -> Result<TypeId, PackFormationError>
```

Required laws:

```text
kind(positionals) == TypePack
kind(labeled)     == LabeledTypePack
kind(result)      == CompleteTypePack
```

No zero-product collapse occurs at this level:

```text
CompleteTypePack([], [])
```

remains an empty complete **pack term**. Only explicit Tuple formation through `tuple_from_packs([], [])` becomes `Unit`.

## 8.4 Shared lane projection helpers

P1 needs semantic helpers that P2 can consume for type-level `*` / `**` / `***` projection without duplicating C1 Tuple semantics.

Conceptual behavior:

```text
positionals(TypePack P)              -> P
positionals(CompleteTypePack(P,L))    -> P
positionals(Tuple(P,L))               -> P
positionals(Unit)                     -> []

labeled(LabeledTypePack L)            -> L
labeled(CompleteTypePack(P,L))        -> L
labeled(Tuple(P,L))                   -> L
labeled(Unit)                         -> []

complete(CompleteTypePack(P,L))        -> same term
complete(Tuple(P,L))                   -> CompleteTypePack(P,L)
complete(Unit)                         -> CompleteTypePack([],[])
```

`complete(TypePack)` and `complete(LabeledTypePack)` are invalid: `***` is not an alias for single-lane expansion.

Implementation rule:

- inspect existing C1 Tuple `Positionals` / `Labeled` normalization first;
- if the extraction logic is private and duplicated would create a second semantic rule, factor a shared internal helper and have both C1 type-member normalization and C2 pack projection call it;
- do not create a second source-level member/projection identity for this helper.

## 8.5 Generic parameter mode

Add one canonical semantic enum conceptually equivalent to:

```rust
enum GenericParameterMode {
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}
```

Required effective domains:

```text
Single            -> declared kind/domain
PositionalPack    -> TypePack
LabeledPack       -> LabeledTypePack
CompletePack      -> CompleteTypePack
```

Mode is a semantic/public-interface fact, not a parser decoration.

`TypeParameterData::new(...)` should remain source-compatible for existing callers by defaulting to `Single`. Add a builder/constructor seam such as `with_mode` rather than forcing every existing caller to spell `Single` mechanically unless the live architecture makes another approach cleaner.

Add mode to the semantic equality/versioning test in `TypeStore::intern_type_parameter`:

```text
same owner/index + same name/kind/variance + different mode
    => new TypeParameterId semantic version
```

Do not infer mode from kind.

## 8.6 Generic signature alignment

`GenericSignature` must carry binder modes aligned with its parameter slots, conceptually:

```text
parameters[i]
parameter_kinds[i]
parameter_modes[i]
parameter_variances[i]
```

P1 source formation still creates only `Single` modes because P2 has not introduced variadic generic syntax yet.

Publication validation must enforce at least:

```text
PositionalPack => kind TypePack
LabeledPack    => kind LabeledTypePack
CompletePack   => kind CompleteTypePack
pack modes     => invariant variance
```

`Single` may legitimately have kind `TypePack` or `LabeledTypePack`; this is exactly why mode is separate from kind.

Binder ordering/multiplicity rules are P2 source-signature work. P1 should not invent parser-order policy beyond individual mode↔kind validity.

## 8.7 Positional concat term

Add a canonical term conceptually equivalent to:

```rust
TypeData::TypePackConcat(Box<[TypeId]>)
```

where every child has kind `TypePack`.

The public constructor/normalizer must enforce a **flat n-ary canonical representation**.

Required normalization laws:

```text
concat()                  == TypePack[]
concat(P)                 == P
concat([], P)             == P
concat(P, [])             == P
concat(concat(A,B), C)    == concat(A,B,C)
concat([A], [B,C])        == [A,B,C]
```

Algorithmic requirements:

1. iterate source segments in order;
2. recursively/iteratively flatten nested concat children into one output sequence;
3. drop empty concrete `TypePack` children;
4. merge adjacent concrete `TypePack` children by concatenating their elements;
5. preserve symbolic segment order exactly;
6. return the empty concrete pack when no segments remain;
7. return the single remaining child when one segment remains;
8. return one concrete C1 `TypePack` when all remaining content is concrete;
9. otherwise intern one flat `TypePackConcat` node.

Complexity must be linear in segment count plus concrete elements. Do not construct a recursive binary concat spine.

## 8.8 Labeled concat term

Add a canonical term conceptually equivalent to:

```rust
TypeData::LabeledTypePackConcat(Box<[TypeId]>)
```

where every child has kind `LabeledTypePack`.

The normalizer uses the same flatten/drop/adjacent-concrete-merge strategy plus uniqueness validation.

Required duplicate rule:

```text
if two labels are already known concrete anywhere in the composition:
    duplicate => Invalid
```

This remains true even when symbolic segments occur between the two concrete segments.

Unknown symbolic overlap is not rejected and is not considered proven disjoint. The residual concat node itself carries the unresolved uniqueness obligation.

Example:

```text
L1(symbolic) ++ [(x, Int)] ++ L2(symbolic)
```

remains symbolic.

After later substitution:

```text
L1 -> [(y, Bool)]
L2 -> [(x, String)]
```

re-canonicalization must become invalid because `x` is now concrete twice.

No “last wins,” row merge, map merge, or label sorting is permitted.

## 8.9 Checked re-canonicalization seam

C1 substitution is currently infallible. C2 introduces a semantic operation that may become invalid only after substitution.

P1 must add an explicit checked seam, conceptually:

```rust
TypeSubstitution::try_apply(...) -> Result<TypeId, TypeSubstitutionError>
```

or an equivalent narrowly scoped checked normalization API.

Required semantics for new pack terms:

```text
substitute children
    ↓
re-run complete/concat canonical constructors
    ↓
Reduced canonical term
or Symbolic canonical term
or Invalid error
```

The forbidden implementation is any equivalent of:

```rust
labeled_concat(substituted).unwrap_or(original)
```

or:

```text
concrete duplicate becomes an unchanged symbolic concat
```

### P1 compatibility rule for existing infallible `apply`

P1 is not authorized to force a repository-wide error-channel redesign merely to make the new source syntax reachable; P2 source syntax is not enabled yet.

Therefore the preferred staged shape is:

1. introduce one checked substitution path that fully supports C2 terms;
2. keep existing `apply` for pre-C2 call sites whose reachable terms are still guaranteed not to produce pack-composition invalidity;
3. make the unchecked path's precondition explicit in code/docs/tests rather than silently swallowing an error;
4. P1-created C2 tests use the checked path when invalidity is possible;
5. record in the P1 handoff that **P2 must not expose C2 source forms through any unchecked substitution path**; source-reachable callers must use/propagate the checked path before syntax rollout.

If the live architecture shows that a safe checked seam cannot coexist with existing `apply` without a production panic or invalid-term escape, STOP AND CONSULT rather than globally changing dozens of semantic APIs speculatively.

`materialize_type` is already fallible and should directly propagate a pack-normalization/materialization error when re-canonicalization proves invalidity.

## 8.10 Semantic visitors

New `TypeData` variants will force or require updates across recursive semantic visitors. Use the compiler plus a targeted search for C1 pack cases.

Expected production paths include live equivalents of:

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/impls.rs
```

Classify each visitor rather than adding catch-all wildcards:

```text
recursive structural visitor
    -> recurse into complete/concat children

proper-Type-only relation/check
    -> preserve kind boundary; do not coerce pack terms into Type

stable publication walker
    -> serialize children structurally

legacy compiled-type seam that already rejects C1 pack terms
    -> add new forms to the explicit non-exportable branch unless downstream contract requires representation
```

Do not make new terms opaque to dependency/parameter traversal merely to satisfy exhaustiveness.

## 8.11 Metadata schema decision

P1 adds serialized semantic constructs and a new durable binder-mode field. This is an intentional metadata schema extension.

**FIXED P1 decision:** bump:

```text
TYPE_METADATA_SCHEMA_VERSION: 2 -> 3
```

while preserving support for existing schema versions according to the current supported-range policy.

Required compatibility behavior:

- new P1-produced bundles use schema `3`;
- v1/v2 bundles containing only their historically legal forms remain decodable/validatable by the new compiler;
- `TypeParameterRecord.mode` (or equivalent) must deserialize as `Single` when absent from old JSON records, using an explicit serde/default compatibility rule;
- schemas `< 3` must reject C2-only complete-pack/concat nodes if such a bundle is manually forged;
- schemas `< 3` must reject non-`Single` binder modes if manually forged;
- do not reuse or renumber old structural fingerprint tags.

If repository policy discovered at T0 explicitly forbids schema `3` in this checkpoint and provides another accepted compatibility mechanism, STOP AND CONSULT; do not silently keep schema `2` for new serialized semantics.

## 8.12 Stable metadata forms

Add stable forms conceptually equivalent to:

```rust
KindNode::CompleteTypePack

TypeNode::CompleteTypePack {
    positionals: TypeNodeId,
    labeled: TypeNodeId,
}

TypeNode::TypePackConcat(Box<[TypeNodeId]>)
TypeNode::LabeledTypePackConcat(Box<[TypeNodeId]>)

enum GenericParameterModeRef {
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}

TypeParameterRecord {
    ...
    mode: GenericParameterModeRef,
}
```

The exact names are flexible.

Metadata validation must enforce for the new forms:

```text
CompleteTypePack node entry kind == CompleteTypePack
    positional child kind == TypePack
    labeled child kind == LabeledTypePack

TypePackConcat node entry kind == TypePack
    every child kind == TypePack

LabeledTypePackConcat node entry kind == LabeledTypePack
    every child kind == LabeledTypePack

PositionalPack mode => parameter kind TypePack
LabeledPack mode    => parameter kind LabeledTypePack
CompletePack mode   => parameter kind CompleteTypePack
pack mode           => invariant variance
```

The validator need not solve symbolic label disjointness; semantic construction already owns that obligation. It must reject structurally malformed metadata.

## 8.13 Fingerprints and public interface identity

New stable nodes need explicit unique fingerprint tags. Preserve all existing tag values; append unused tags.

Fingerprint structure:

```text
CompleteTypePack:
    tag + positional child fingerprint + labeled child fingerprint

TypePackConcat:
    tag + segment count + ordered child fingerprints

LabeledTypePackConcat:
    tag + segment count + ordered child fingerprints
```

Binder mode must affect the **public/interface semantic fingerprint**.

A mode-only edit:

```text
P: TypePack
    -> *P
```

must change published interface identity even if owner/name/index/kind remain otherwise compatible.

The current `MetadataExporter` dedup key uses owner + `TypeParameterId`s + constraints. Do not assume this alone is sufficient across revisions because stable parameter references use owner/index. Audit the interface-fingerprint writer and ensure the stable parameter mode is hashed through the actual publication path.

## 8.14 Compiler seam and runtime inspector

Extend `CompiledKindRef` with the exact complete-pack kind so kinded generic signatures can cross the semantic/compiler seam without lying about their domain.

`CompiledTypeRef` currently rejects C1 pack/TupleFromPacks/internal terms. P1 should keep the new complete/concat terms in that explicit non-exportable class **unless** a current compiler consumer actually requires them. Do not expand the legacy compiled-type representation speculatively.

Update `phalcom-core/src/typing/inspect.rs` only as a metadata consumer:

- format/inspect the new stable kind/type nodes;
- expose binder mode if the existing reflection inspector surfaces parameter metadata;
- do not create runtime `TypePack`, `CompleteTypePack`, concat objects, methods, bytecodes, or native operations.

---

# 9. Ownership boundaries

## 9.1 P1 owns

P1 owns implementation of:

```text
exact internal complete-pack kind/domain
complete-pack canonical TypeId term
shared semantic lane projection helpers
semantic GenericParameterMode
mode-sensitive TypeParameterId versioning
GenericSignature parameter-mode storage/validation
TypePackConcat canonical term + normalizer
LabeledTypePackConcat canonical term + normalizer
concrete duplicate-label rejection
symbolic overlap residual preservation
checked pack-aware substitution seam
pack-aware fallible materialization
semantic visitor integration required by new canonical terms
metadata schema v3 extension
stable complete/concat kind/type nodes
stable binder-mode metadata
metadata validation/fingerprints for new forms
compiled kind seam compatibility
runtime typing metadata-inspector compatibility
focused semantic/metadata tests
```

## 9.2 P1 does not own

P1 does not implement:

```text
* / ** / *** generic binder parsing
pack expansion AST in generic arguments
binder ordering syntax diagnostics
explicit empty <> saturation semantics
generic argument lane assembly
binder-aware source generic application
Tuple type expansion syntax
callable-domain type unpack syntax
source target/index behavior for pack operators
source-oriented diagnostics for pack expansions
whole pack decomposition inference
prefix/suffix sequence solving
Pack::Length / Pack::Slice
public pack Concat operator/member
pack elementwise/quantified constraints
public CompleteTypePack kind syntax
runtime ArgumentPack ABI
runtime rest capture changes
VM pack solving or type-level execution
runtime TypePack/LabeledTypePack/CompleteTypePack objects
unrelated Universe redesign
release/workspace certification
```

## 9.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| complete-pack kind | `KindData` / `TypeStore` | signatures, inference, metadata | parser-only marker with informal kind |
| complete-pack term | `TypeData` / `TypeStore` | concat/projection, substitution, metadata | Tuple or runtime ArgumentPack surrogate |
| binder mode | `TypeParameterData` + aligned `GenericSignature` data | P2 binder/application, metadata | infer-from-kind logic |
| positional composition | `TypeStore` concat normalizer | P2 generic/Tuple/callable assembly | parser-specific vector flattener |
| labeled composition | `TypeStore` labeled concat normalizer | P2 assembly, materialization | RecordRow/map merge machinery |
| duplicate-label obligation | labeled concat semantic term + checked normalizer | substitution/materialization | “last wins” or silent acceptance |
| generic binding lifetime | existing `GenericInstantiation` | materialization | PackInstantiation |
| query-local unknown pack | existing `InferVarId(kind)` | checker | PackVarId |
| stable representation | `phalcom-type-meta` + metadata exporter | cross-module/runtime typing inspector | raw store IDs / source strings |
| runtime execution | none | none | VM opcode/native pack solver |

---

# 10. Global invariants

`INV-01` — `*`, `**`, and `***` retain positional, labeled, and complete-lane meaning respectively. P1 must not introduce `...` pack semantics.

`INV-02` — `TypeId` remains the canonical carrier for non-row kinded pack terms. No `PackId` / pack arena is introduced.

`INV-03` — `CompleteTypePack` is an exact type-level domain distinct from `Type`, `Tuple`, `TypePack`, `LabeledTypePack`, `RecordRow`, and runtime `ArgumentPack`.

`INV-04` — `CompleteTypePack(P,L)` requires `P : TypePack` and `L : LabeledTypePack`.

`INV-05` — An empty complete pack remains a complete-pack term; only explicit Tuple formation through C1 becomes `Unit`.

`INV-06` — Generic binder mode is orthogonal to kind. `P: TypePack` and `*P` are not interchangeable semantics.

`INV-07` — Binder mode participates `TypeParameterId` semantic versioning and durable public interface identity.

`INV-08` — Pack binders are invariant in C2.P1; no variance semantics for pack capture are invented.

`INV-09` — `GenericSignature` retains one slot-aligned mode per parameter and validates mode↔kind coherence.

`INV-10` — Positional concat is a flat, ordered, n-ary canonical term and normalizes empty/single/all-concrete cases back to C1 `TypePack` identity.

`INV-11` — Labeled concat is a flat, ordered, n-ary canonical term and normalizes empty/single/all-concrete valid cases back to C1 `LabeledTypePack` identity.

`INV-12` — Labeled pack order is preserved; no sorting through `RecordRow` or map semantics occurs.

`INV-13` — Concrete duplicate labels are invalid. Unknown symbolic overlap remains a residual obligation, not proven validity or invalidity.

`INV-14` — Re-canonicalization after substitution/materialization may return invalidity; no API may hide a newly proven duplicate by returning the pre-substitution symbolic term.

`INV-15` — `LabeledTypePack` remains distinct from `RecordRow` at every semantic, metadata, and inspection layer.

`INV-16` — Complete and concat terms remain kinded compiler semantics; no runtime objects, VM bytecodes, or native execution path are added.

`INV-17` — Existing `GenericInstantiation` remains the single canonical non-row binding authority.

`INV-18` — Existing `InferVarId(kind)` remains the only query-local inference identity for non-row pack kinds; no `PackVarId` is added.

`INV-19` — Stable metadata contains semantic structure and stable references only; no `TypeId`, `InferVarId`, or `RecordRowVarId` is serialized as durable identity.

`INV-20` — Existing metadata fingerprint tags are never renumbered/reused; new P1 nodes receive new explicit tags.

`INV-21` — Binder mode changes the published interface fingerprint even if the stable owner/index/name/kind are otherwise unchanged.

`INV-22` — Schema v3 is the first schema permitted to encode C2 complete/concat forms and non-`Single` binder mode. Older schemas remain readable for their historical forms.

`INV-23` — C1 `TypePack`, `LabeledTypePack`, `TupleFromPacks`, `Map<F>`, type-member projection, and zero-product semantics remain unchanged.

`INV-24` — Type-level pack canonicalization is deterministic and linear in input segments/elements; no backtracking or sequence partition search exists.

`INV-25` — No user-visible public name for the complete-pack kind is introduced in P1.

`INV-26` — `intrinsic` remains the source keyword; `@intrinsic` is not resurrected.

---

# 11. Non-goals

Do not expand P1 into any of the following:

```text
source generic binder grammar
source generic expansion grammar
public CompleteTypePack syntax
labeled generic-argument syntax
variadic callable ABI
runtime rest capture
pack decomposition solver
prefix/suffix inference
sequence arithmetic
GATs
nominal const generics
pack quantified constraints
public Concat type member/operator
RecordRow conversion
new Tuple identity
new runtime reflection object model
LSP UI for packs
workspace-wide cleanup
unrelated trait baseline repair
```

If a seemingly necessary solution starts requiring one of these, treat that as evidence of an architectural mistake or later-plan dependency, not permission to widen P1.

---

# 12. Expected impact map

## 12.1 Expected primary source areas

```text
phalcom-semantic/src/types/id.rs
    new base KindId constant

phalcom-semantic/src/types/kind.rs
    exact complete-pack KindData

phalcom-semantic/src/types/store.rs
    new TypeData forms, constructors, canonicalizers, formatting/visitors

phalcom-semantic/src/types/parameter.rs
    GenericParameterMode, TypeParameterData.mode, GenericSignature.parameter_modes

phalcom-semantic/src/types/substitution.rs
    checked C2 re-canonicalization path

phalcom-semantic/src/types/instantiation.rs
    fallible complete/concat materialization

phalcom-semantic/src/types/projection.rs
    shared/factored lane extraction if needed to avoid C1 semantic duplication

phalcom-semantic/src/export.rs
    CompiledKindRef complete-pack kind; explicit non-exportable new type terms as appropriate

phalcom-semantic/src/metadata/export.rs
    stable node export, binder mode, structural fingerprints, interface fingerprint integration

phalcom-type-meta/src/header.rs
    metadata schema v3

phalcom-type-meta/src/kind.rs
    stable complete-pack kind node

phalcom-type-meta/src/type_node.rs
    stable complete/concat type nodes

phalcom-type-meta/src/generic.rs
    stable generic binder mode / TypeParameterRecord field

phalcom-type-meta/src/validate.rs
    schema floors + structural validation

phalcom-core/src/typing/inspect.rs
    stable metadata inspection/formatting compatibility only
```

## 12.2 Expected mechanical visitor updates

New `TypeData` variants may require updates in live equivalents of:

```text
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/impls.rs
```

These are expected only where exhaustiveness or recursive semantic traversal requires them. Do not refactor these subsystems broadly.

## 12.3 Expected tests

Primary additions/updates should land in existing semantic/metadata test trees, likely:

```text
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/foundations/substitution.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs   # only if the live test owner fits mode-versioning best
phalcom-type-meta unit tests / validation tests as currently organized
```

Do not add parser tests in P1 unless T0 discovers that an existing parser path must mechanically compile against a semantic enum change. Source behavior is not a P1 acceptance surface.

## 12.4 Expected documentation/state

```text
LANG007.C2-CHECKPOINT.md
    plan activation, durable interfaces/invariants, gate evidence, deferred failures

LANG007.C2.P1-walkthrough.md
    retrospective implementation/evidence

LANG007.C2.P1-handoff.md
    P2 takeover contract

PROGRAM.md / STATUS.md
    only lifecycle/next-action updates required by repository convention
```

## 12.5 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary to keep exhaustive matches or stable consumers compiling.

STOP AND CONSULT before entering a materially different subsystem if doing so changes semantic ownership, runtime responsibility, metadata compatibility, inference identity, or public language behavior.

---

# 13. Implementer decision authority

## 13.1 FIXED

The implementer must not change:

1. the `*` / `**` / `***` lane meanings;
2. the prohibition on using `...` as C2 pack syntax;
3. `TypeId` as the canonical kinded carrier for non-row pack terms;
4. the distinction `LabeledTypePack != RecordRow`;
5. the distinction `CompleteTypePack != Tuple`;
6. the exact complete-pack semantic domain;
7. binder mode as a semantic fact separate from kind;
8. mode-sensitive parameter identity/versioning;
9. one canonical `GenericInstantiation` authority;
10. ordinary kind-aware `InferVarId` for complete-pack unknowns;
11. flat n-ary concat canonicalization;
12. concrete duplicate labels are invalid;
13. unknown overlap remains symbolic;
14. checked invalidity after substitution/materialization;
15. schema v3 for the new serialized semantics, subject only to an explicit repository-policy conflict found at T0;
16. stable fingerprint participation for mode and new structural nodes;
17. no public complete-pack kind syntax in P1;
18. no runtime object/ABI/VM execution for these terms;
19. no parser/application rollout in P1;
20. no pack decomposition solver.

## 13.2 MECHANICALLY FLEXIBLE

The implementer may adapt without consultation:

```text
exact enum/struct/helper names
private module/file decomposition
whether concat constructors accept slices/iterators/boxed arrays
the precise internal error enum names
local iterator/HashSet choices
helper visibility
whether lane projection helpers live in store.rs, a small pack module, or factored projection utility
exact test function names
imports and borrow-checker cleanup
equivalent stable-node field names
mechanical exhaustive-match edits
```

Provided the fixed semantics and ownership remain unchanged.

## 13.3 VERIFY-FIRST

| Assumption | Verify in live tree | If false |
|---|---|---|
| no exact complete-pack domain already exists | `types/{kind,id,store}.rs`, metadata nodes | if exact equivalent exists, reuse it; if semantics differ, STOP AND CONSULT |
| `KindId` base constants still end at labeled pack | `types/id.rs`, `TypeStore::new` | adapt mechanically if only numbering drifted; consult if identity model changed |
| `TypeStore` remains canonical kinded-form owner | `types/store.rs` | STOP AND CONSULT if ownership changed |
| `TypeParameterData` semantic-reuse check is still in `intern_type_parameter` | `types/store.rs` | adapt equivalent owner; consult if stable identity model changed |
| `GenericSignature` still owns aligned parameter metadata | `types/parameter.rs` | adapt mechanically; consult if mode must live elsewhere canonically |
| `GenericInstantiation` still uses `TypeId` + row bindings | `types/instantiation.rs` | STOP if a new pack-binding authority already landed |
| non-row generic inference still uses `InferVarId(kind)` | `checker/{inference,row_inference}.rs` | STOP if a pack solver identity already exists or is required |
| C1 Tuple lane extraction remains in projection/store code | `types/projection.rs`, `types/store.rs` | factor/reuse current owner; do not duplicate |
| metadata current version still 2 | `phalcom-type-meta/src/header.rs` | use next version number mechanically; preserve old-schema floor semantics |
| metadata is still JSON serde + validator | `encode.rs`, `validate.rs` | adapt compatibility mechanics; consult if schema contract changed materially |
| `CompiledTypeRef` still rejects C1 pack/internal terms | `phalcom-semantic/src/export.rs` | if already expanded, extend consistently; otherwise do not invent expansion |
| runtime typing inspector is still metadata-only | `phalcom-core/src/typing/inspect.rs` | STOP if implementation would require runtime execution/state |
| no C2 source syntax is currently enabled | AST/parser + semantic annotation lowering | if source already creates P1 terms, checked substitution/materialization must be source-safe before proceeding; STOP if plan staging is invalid |

---

# 14. Global STOP / CONSULT triggers

The implementer must stop editing the affected issue and produce a consultation packet if any of these occurs.

1. The accepted specs or current C2 requirements give a different `*` / `**` / `***` lane meaning.
2. Live `main` already contains another exact complete-pack domain with materially different identity/ownership.
3. Supporting complete packs appears to require treating them as proper `Type` or Tuple.
4. A canonical pack implementation appears to require a second `PackId`/arena.
5. A generic unknown pack appears to require persistent `PackVarId` rather than ordinary kind-aware `InferVarId`.
6. Correct concat normalization appears to require backtracking, unknown sequence partitioning, or pack decomposition.
7. Preserving labeled duplicate uncertainty appears impossible without changing a major semantic normalization contract.
8. A correct implementation would need to convert `LabeledTypePack` into `RecordRow`.
9. P1 appears to require a user-visible public `CompleteTypePack` kind spelling.
10. P1 appears to require parser/source generic-pack support to prove the semantic substrate.
11. Making substitution honest appears to require an unbounded repository-wide signature/error redesign rather than a bounded checked seam.
12. A checked substitution seam cannot coexist safely with existing source-reachable paths without panic, stale invalid terms, or hidden errors.
13. Metadata schema versioning policy conflicts with the fixed schema-v3 decision or cannot preserve reading historical v1/v2 JSON.
14. Binder mode cannot affect stable public interface fingerprints through the current metadata architecture without a new global identity scheme.
15. A mode-only edit cannot receive a new semantic parameter version without destabilizing unrelated persistent IDs.
16. P1 changes would require runtime ArgumentPack ABI, VM opcodes, native pack operations, or runtime heap objects.
17. A material semantic test oracle conflicts with the accepted requirements/specs.
18. The same nontrivial semantic failure survives one serious corrective attempt.
19. Two plausible fixes have meaningfully different ownership/identity/persistence consequences.
20. Tests can pass only by weakening an invariant or preserving an invalid concrete pack as symbolic.
21. Scope becomes materially broader than the semantic+metadata substrate described here.

A triggered consultation is not self-waivable.

### Consultation evidence packet

Collect cheaply before escalating:

```markdown
## LANG007.C2.P1 consultation

### Task / gate
T? / G?

### Required invariant
INV-??

### Observed
<exact behavior/code/test>

### Expected architecture
<owner + representation>

### Minimal reproducer
<test or TypeStore construction>

### Relevant symbols / call path
- ...

### Scoped diff
<files / concise patch summary>

### Causal hypothesis
...

### Competing fixes
1. ...
2. ...

### Architectural consequences
<identity / substitution / metadata / runtime>

### Serious correction already attempted
<if any>

### Trigger
<exact global/local trigger>

### Decision requested
<one precise architecture question>
```

---

# 15. Debugging budget

## 15.1 Mechanical failures

Allow up to three coherent mechanical correction cycles while each cycle has a concrete cause and shows progress.

A cycle is:

```text
inspect failure
identify concrete mechanical cause
make one coherent correction
rerun smallest discriminating check/test
```

Examples:

```text
missing exhaustive match arm
renamed helper
borrow conflict in canonicalizer
forgotten stable enum match
wrong test filter
```

Do not turn ordinary exhaustiveness fixes into architecture consultation.

## 15.2 Semantic failures

Before a nontrivial correction, record locally:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow **one serious corrective attempt** for the same underlying semantic failure.

If the same failure persists, STOP AND CONSULT.

## 15.3 Architectural failures

Zero speculative architecture attempts. Consult immediately.

## 15.4 Diagnostic order

For P1 semantic failures, inspect in this order:

```text
1. operand/child kind
2. canonical constructor chosen
3. flat segment order
4. concrete-buffer merging
5. known-label duplicate set
6. TypeParameterData mode/kind/version
7. substitution/materialization result
8. metadata node/kind
9. fingerprint/interface publication
10. runtime inspector only if the stable metadata is already correct
```

Do not debug the VM for a P1 canonical type error.

---

# 16. Testing surface analysis

The following coverage obligations are mandatory for P1. Tests may be combined where one case proves multiple obligations, but every coverage ID must have explicit evidence in the walkthrough.

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| `CV-01` | exact domain identity | INV-02, INV-03 | complete pack has exact complete-pack kind and is not proper `Type` |
| `CV-02` | child-domain validation | INV-04 | reject wrong positional/labeled child kinds |
| `CV-03` | zero boundary | INV-05 | empty complete pack remains complete term; Tuple formation separately becomes Unit |
| `CV-04` | lane projection | INV-01, INV-03 | positional/labeled/complete projection from complete/Tuple/Unit; reject `***TypePack` / `***LabeledTypePack` equivalent helper cases |
| `CV-05` | binder mode vs kind | INV-06 | `Single(TypePack)` differs from `PositionalPack(TypePack)` |
| `CV-06` | semantic identity lifecycle | INV-07 | owner/index mode-only change allocates a new semantic parameter version |
| `CV-07` | mode-kind validity | INV-08, INV-09 | reject mismatched pack mode/kind and non-invariant pack mode |
| `CV-08` | positional empty/single | INV-10 | concat zero/single/empty-segment laws |
| `CV-09` | positional flatten/merge | INV-10, INV-24 | nested concat flattens; adjacent concrete segments merge |
| `CV-10` | positional symbolic identity | INV-10 | symbolic segments remain ordered one flat concat |
| `CV-11` | labeled empty/single | INV-11 | concat zero/single/empty-segment laws |
| `CV-12` | labeled flatten/merge | INV-11, INV-12, INV-24 | nested concat flattens; adjacent concrete segments merge without sorting |
| `CV-13` | concrete duplicate negative | INV-13 | duplicate concrete label across separate segments is invalid |
| `CV-14` | symbolic overlap | INV-13 | symbolic segment overlap uncertainty remains residual concat |
| `CV-15` | post-substitution invalidity | INV-14 | symbolic concat becomes duplicate concrete after checked substitution and returns error |
| `CV-16` | post-substitution reduction | INV-10, INV-11, INV-14 | fully concrete valid substitution collapses to C1 concrete pack identity |
| `CV-17` | materialization | INV-14, INV-17 | GenericInstantiation materialization re-canonicalizes and surfaces invalid duplicate |
| `CV-18` | inference lifetime | INV-18 | complete-pack parameter receives ordinary `InferVarId(kind CompletePack)`; no pack solver type introduced |
| `CV-19` | metadata stable kind/type | INV-19, INV-20 | complete/concat nodes serialize, validate, and use stable child refs |
| `CV-20` | stable binder metadata | INV-07, INV-21 | mode serializes and mode-only public interface fingerprint changes |
| `CV-21` | metadata backward compatibility | INV-22 | v1/v2 historical record with missing mode decodes as Single; C2 forms are schema-v3-only |
| `CV-22` | malformed metadata negative | INV-03, INV-04, INV-19, INV-22 | wrong child-kind / mode-kind C2 metadata is rejected |
| `CV-23` | C1 compatibility | INV-23 | existing TypePack/LabeledTypePack/TupleFromPacks/type-member closure tests stay green |
| `CV-24` | row distinction | INV-12, INV-15 | labeled concat retains order and never becomes RecordRow |
| `CV-25` | runtime boundary | INV-16 | no new VM/native execution path; core changes are inspection-only |
| `CV-26` | moderate density | INV-10, INV-11, INV-24 | e.g. 128 mixed concrete/symbolic segments canonicalize flat without deep recursive structure |
| `CV-27` | keyword boundary | INV-26 | static audit shows no `@intrinsic` resurrection in P1 patch |

Not required in P1:

```text
parser recovery
source diagnostic wording
empty <> source saturation
cold/incremental source-edit parity
cross-module source elaboration
callable ABI behavior
runtime spread/capture behavior
pack decomposition inference
```

Those belong to P2/P3 or other checkpoints.

---

# 17. Verification execution budget

## 17.1 Modes

### BUILD MODE

Default for T1–T4. Use only the smallest tests/checks that answer the current semantic question.

### STABILIZE MODE

Enter at G5 after the coherent P1 substrate exists. Run focused C1/P1 regression suites and affected-crate checks.

### CERTIFY MODE

Not part of P1 unless explicitly requested later. Workspace/release certification belongs to checkpoint/release work.

## 17.2 Verification ladder

```text
T0 — exact new regression/test function
T1 — directly affected foundation/metadata module
T2 — owning semantic/type-meta suite
T3 — adjacent metadata/core inspector integration
T4 — affected crate checks
T5 — workspace/release verification
```

## 17.3 Mandatory during BUILD

Use serial Cargo commands.

At meaningful task boundaries, run only the directly affected tests named in the task/gate.

Do not run `cargo test --workspace` or broad language corpora during BUILD.

## 17.4 Mandatory during STABILIZE

At G5 run the focused set specified there plus affected-crate checks and hygiene.

## 17.5 Only if evidence demands

```text
phalcom-core focused reflection/typing suite
broader semantic integration tests
full phalcom-semantic test target
additional metadata compatibility tests
```

Run these only if the changed path or a failure creates a real unresolved concern.

## 17.6 Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace
full language corpus
all phalcom-core tests
all VM tests
all LSP tests
performance benchmark suites
trait capability suite unrelated to C2 P1
release packaging/certification
```

Reason: none of these improves the information available while the semantic substrate is being assembled; they create noise and may hit inherited unrelated baselines.

---

# 18. Baseline / unrelated failure policy

C1 left two documented unrelated trait-suite baseline failures:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Unless P1 directly changes the relevant trait/default-body dependency architecture, treat those as inherited baseline failures.

Classify any unexpected failure:

```text
A — definitely caused by P1
B — probably caused by P1
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active responsibility; fix within plan scope.
- C: one bounded classification pass; if still unclear, nonblocking, and outside P1 acceptance, record and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks required P1 evidence.
- Do not weaken assertions, add skips, or change expected semantics to make a broad test green.

Record material deferred failures in `LANG007.C2-CHECKPOINT.md`.

---

# 19. Tasks

## T0 — Bootstrap lifecycle state and verify P1 architectural premises

### Purpose

Confirm that the implementation checkout still matches the repository facts this plan depends on, install/confirm the C2 planning records, and activate P1 without performing production semantic edits.

### Preconditions

- supplied C2 checkpoint/guidance/plan artifacts are available;
- no implementation work has started under a competing C2 plan.

### Consumes

- planning baseline `1a1e5ab...`;
- C1 P3 walkthrough/handoff;
- accepted C2 requirements and ratified pack specs.

### Produces

- verified local start revision/worktree state;
- active C2 lifecycle state;
- a concise drift note if helper/path mechanics changed;
- confirmation that no competing exact complete-pack/pack-solver architecture landed.

### Files and symbols

**Read:**

```text
AGENTS.md
docs/workflow/{implementation-record-lifecycle-convention,luna-patch-grade-plan-schema,shared-consultation-escalation-protocol}.md
docs/implementation/LANG007/{PROGRAM,STATUS}.md
docs/implementation/LANG007/LANG007.C2/*
phalcom-semantic/src/types/{id,kind,store,parameter,substitution,instantiation,projection}.rs
phalcom-semantic/src/checker/{inference,row_inference}.rs
phalcom-semantic/src/{export.rs,metadata/export.rs}
phalcom-type-meta/src/{header,kind,type_node,generic,validate,encode}.rs
phalcom-core/src/typing/inspect.rs
```

**Modify:** lifecycle records only.

### Required implementation shape

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then targeted searches:

```sh
rg -n 'CompleteTypePack|PackVarId|PackId|TypePackConcat|LabeledTypePackConcat|GenericParameterMode' \
  phalcom-semantic phalcom-type-meta phalcom-core

rg -n 'KindData::(TypePack|LabeledTypePack)|KindId::(TYPE_PACK|LABELED_TYPE_PACK)' \
  phalcom-semantic/src

rg -n 'TypeData::(TypePack|LabeledTypePack|TupleFromPacks)' \
  phalcom-semantic/src
```

Update checkpoint state at plan start:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG007.C2.P1
```

Add/activate the P1 ledger entry. Keep overall C2 open.

### Forbidden approaches

- no production code edits before the premise audit;
- no broad repository scan;
- no re-audit of C1 semantics already proven unless live source contradicts the handoff;
- no reinterpretation of stale `...P` examples.

### Test changes required

None.

### Tests to run now

None unless local build state is already suspicious. Do not establish a ceremonial full baseline.

### Tests explicitly deferred

All feature/runtime suites.

### Acceptance

T0 passes when:

- local revision/worktree are known;
- planning records are present;
- no conflicting C2 architecture has landed;
- current base kinds/TypeData/generic/metadata owners still match the plan mechanically;
- checkpoint marks P1 active.

### Local STOP / CONSULT triggers

- exact complete-pack or pack-concat architecture already exists with materially different semantics;
- source C2 syntax is already user-reachable, invalidating the staged checked-substitution assumption;
- metadata schema policy explicitly prevents the planned versioning model;
- a second pack solver/arena already owns canonical state.

### Checkpoint update

Required: plan activation/start revision and any material drift finding.

---

## T1 — Add the exact complete-pack domain, canonical term, and shared lane projections

### Purpose

Create the exact semantic domain required by `***` and the canonical two-lane term that P2 will bind/expand, without adding source syntax or a runtime object.

### Preconditions

- T0 passed;
- no competing exact complete-pack representation exists.

### Consumes

- C1 `TypeId`/`KindId` architecture;
- C1 `TypePack`, `LabeledTypePack`, `TupleFromPacks`;
- existing Tuple lane extraction semantics.

### Produces

- exact complete-pack `KindId`/`KindData`;
- checked canonical complete-pack constructor;
- semantic lane projection helpers;
- stable complete-pack kind node/schema-v3 foundation sufficient for compilation;
- compiler seam complete kind;
- CV-01..CV-04 evidence.

### Files and symbols

**Primary modify:**

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/projection.rs          # only if sharing C1 extraction logic requires it
phalcom-semantic/src/export.rs
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/validate.rs
phalcom-semantic/src/metadata/export.rs           # kind export/fingerprint only at this task
```

**Mechanical compile-followups:** only exhaustive `KindData` / stable-kind consumers proven by compiler/search.

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-type-meta validation tests as currently organized
```

### Required implementation shape

1. Add the new base kind after existing C1 base kinds; preserve old constant meanings.
2. Initialize/assert it deterministically in `TypeStore::new`.
3. Extend `format_kind` / `apply_kind` base-kind non-applicability.
4. Add `TypeData::CompleteTypePack { positionals, labeled }` with kind `COMPLETE_TYPE_PACK`.
5. Add checked formation error(s) for wrong positional/labeled child kind.
6. Do not make the complete pack a proper type.
7. Add/factor lane projection helpers according to §8.4.
8. Extend `CompiledKindRef` only for the kind. Keep new `TypeData` term in explicit non-exportable compiled-type forms unless a current consumer proves otherwise.
9. Add stable `KindNode::CompleteTypePack`.
10. Bump metadata schema to `3` now because the stable kind enum is extended. Preserve decoding/validation of historical v1/v2 forms.
11. Add new kind fingerprint tag without changing existing tags.

### Forbidden approaches

- no `KindSyntax::CompleteTypePack`;
- no nominal `CompleteTypePack` class;
- no Tuple aliasing;
- no `***TypePack` fallback;
- no pack arena;
- no VM/native type operation.

### Test changes required

Add tests proving:

- CV-01 exact kind/not proper Type;
- CV-02 wrong child kind rejection;
- CV-03 empty complete pack vs `tuple_from_packs`→Unit distinction;
- CV-04 lane projections including invalid complete projection from single-lane packs;
- schema constant now emits v3 while v1/v2 historical metadata without C2 forms remains supported.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
```

Do not broaden after PASS.

### Tests explicitly deferred

Metadata type-node round-trip, binder mode, substitution, C1 closure suite, core inspector.

### Acceptance

- complete-pack kind/term is exact and interned canonically;
- wrong child domains cannot form it;
- empty complete pack remains complete-pack kind;
- C1 Tuple/Unit identity is unchanged;
- projection laws are centralized/reused, not duplicated;
- semantic/type-meta crates compile.

### Local STOP / CONSULT triggers

- adding the base kind destabilizes a supposedly stable external numeric kind identity rather than store-local IDs;
- C1 lane extraction cannot be reused without duplicating a second semantic projection authority;
- metadata version bump conflicts with a documented current schema policy.

### Checkpoint update

Record the exact complete-pack domain/term and schema-v3 decision after G1 passes.

---

## G1 — Exact complete-pack domain gate

### Purpose

Prove that the complete lane product is a genuine kinded semantic domain, not a Tuple/proper-Type alias, and that the first serialized kind extension is structurally sound.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Expected

```text
CV-01..CV-04 PASS
no proper-Type coercion
no Tuple identity duplication
no pack runtime path
```

Do not broaden after PASS.

On failure, classify before editing.

---

## T2 — Add semantic generic binder mode and mode-sensitive parameter/signature identity

### Purpose

Represent variadic generic binder shape as a first-class semantic fact separate from kind so P2 can bind source lanes deterministically without reconstructing mode from syntax or kind.

### Preconditions

- G1 passed;
- exact complete-pack kind exists.

### Consumes

- `TypeParameterData` / `TypeParameterId` semantic-version model;
- `GenericSignature` aligned parameter metadata;
- existing generic signature publication validation.

### Produces

- semantic `GenericParameterMode`;
- mode on `TypeParameterData`;
- mode-sensitive `intern_type_parameter` reuse/versioning;
- aligned `GenericSignature.parameter_modes` or exact equivalent;
- individual mode↔kind/variance publication validation;
- all existing source generics still default to `Single`;
- CV-05..CV-07 evidence.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/annotation.rs       # only to populate Single in current source resolution if needed
phalcom-semantic/src/types/native.rs           # mechanical signature construction compatibility if needed
phalcom-semantic/src/declarations.rs           # mechanical signature construction compatibility if needed
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs  # if this is the existing owner for parameter-version semantics
```

### Required implementation shape

Preferred API shape:

```rust
TypeParameterData::new(...)
    // defaults mode = Single

TypeParameterData::with_mode(mode)
```

This minimizes mechanical churn for existing source/tests while making mode explicit when needed.

`intern_type_parameter` semantic equivalence must include:

```text
name
kind
variance
mode
```

A mode-only change on the same owner/index must allocate a new parameter version.

`GenericSignature` must store aligned modes. Existing constructors may default/populate `Single` as long as the resulting signature has explicit mode data before publication.

Add publication errors or equivalent checks for:

```text
PositionalPack with kind != TypePack
LabeledPack with kind != LabeledTypePack
CompletePack with kind != CompleteTypePack
pack mode with variance != Invariant
```

Do not enforce P2 binder ordering/multiplicity here.

### Forbidden approaches

- no mode inference from kind;
- no AST-only mode;
- no reuse of runtime `RestMode` as the canonical semantic generic mode merely to save an enum;
- no source parsing;
- no pack variance semantics;
- no elementwise interpretation of generic constraints.

### Test changes required

Add tests for:

- CV-05 same kind/different mode distinction;
- CV-06 mode-only parameter version change;
- CV-07 mode-kind and pack-variance rejection;
- existing fixed generic signatures remain `Single` and unchanged semantically.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generics_core -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

If the mode-version test belongs in `incremental::type_store_revisions`, run only that module additionally.

### Tests explicitly deferred

Metadata/public fingerprint proof is T5/G5. Parser/source binder tests are P2.

### Acceptance

- generic mode exists in canonical semantic parameter state;
- signature mode is slot-aligned and publishable;
- mode-only semantic edits do not reuse stale `TypeParameterId` semantics;
- fixed generics remain `Single` without source behavior change.

### Local STOP / CONSULT triggers

- current parameter identity semantics cannot version a mode-only slot without mutating old snapshots;
- mode must be stored only in syntax to keep the compiler working;
- pack variance is required by an accepted current example.

### Checkpoint update

Record `GenericParameterMode` as a durable P1 interface and mode-sensitive parameter versioning after G2.

---

## G2 — Binder-mode semantic identity gate

### Purpose

Prove that variadic binder shape is semantic/public identity rather than kind-derived syntax trivia.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generics_core -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
```

Run the focused type-store revision module only if T2 placed the mode-version regression there.

### Expected

```text
CV-05..CV-07 PASS
existing source-created generic parameters remain Single
no parser syntax added
```

---

## T3 — Implement canonical positional and labeled pack composition

### Purpose

Create the symbolic sequence terms that let later source expansion compose unknown-cardinality pack contributions without concrete-vector forcing or pack decomposition.

### Preconditions

- G2 passed;
- canonical C1 concrete pack forms and exact complete-pack term exist.

### Consumes

- `TypeData::TypePack` / `LabeledTypePack`;
- `TypeStore` interning;
- `PackFormationError` or equivalent checked formation framework.

### Produces

- flat canonical `TypePackConcat`;
- flat canonical `LabeledTypePackConcat`;
- linear normalizers;
- concrete duplicate-label rejection;
- symbolic overlap residual semantics;
- no public concat operator;
- CV-08..CV-14 and CV-26 evidence.

### Files and symbols

**Primary modify:**

```text
phalcom-semantic/src/types/store.rs
```

**Expected visitor compile-followups:**

```text
phalcom-semantic/src/types/{substitution,instantiation,environment,relation,projection}.rs
phalcom-semantic/src/checker/{gadt_proof,row_inference}.rs
phalcom-semantic/src/{advisory/formal.rs,impls.rs,export.rs,metadata/export.rs}
```

At T3, only add semantically correct minimal match handling required to compile; full checked rewrite behavior belongs to T4 and metadata nodes to T5.

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/type_model.rs
```

### Required implementation shape

Implement the canonicalizers described in §8.7 and §8.8.

Important details:

- nested concat nodes must be flattened during construction;
- use an adjacent concrete buffer, not a recursive binary tree;
- positional concrete buffers merge element order exactly;
- labeled concrete buffers merge label/element order exactly;
- maintain one global `known_labels` set across all concrete segments, even when symbolic segments intervene;
- symbolic segments retain their original relative order;
- concrete known duplicates fail immediately;
- symbolic-vs-symbolic and symbolic-vs-concrete unknown overlap remains residual;
- all-concrete valid results return the existing C1 concrete pack term;
- result kind is the lane kind, never `Type`.

### Forbidden approaches

- no recursive binary concat spine;
- no `Vec<TypeId>` eager expansion requirement for symbolic packs;
- no label sorting;
- no RecordRow conversion;
- no “last wins” duplicate behavior;
- no backtracking/decomposition;
- no public source `Concat` member/operator.

### Test changes required

Add targeted tests for CV-08..CV-14 and CV-26.

A moderate-density test should prove structural flatness directly, for example by composing ~128 alternating concrete/symbolic segments and asserting the canonical node has one flat child array rather than nested concat depth.

This is a structural resource test; do not add a benchmark harness.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Tests explicitly deferred

Substitution/materialization invalidation and metadata export.

### Acceptance

- both concat forms obey all canonical laws;
- fully concrete results have exactly C1 concrete pack identity;
- duplicate concrete labels are rejected;
- symbolic overlap remains a residual concat;
- canonicalization is flat and linear;
- semantic crate compiles with correctly classified new variants.

### Local STOP / CONSULT triggers

- symbolic overlap requires introducing a theorem prover or row solver to preserve correctness;
- canonicalization appears to need backtracking;
- a visitor can only compile by pretending the new pack form is a proper type.

### Checkpoint update

Record the flat concat representations and duplicate-label residual law after G3.

---

## G3 — Canonical pack composition gate

### Purpose

Prove canonical identity, order, duplicate semantics, symbolic residual behavior, and flat resource shape before wiring substitution.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Expected

```text
CV-08..CV-14 PASS
CV-26 structural flatness PASS
no nested concat identity for canonical construction
no RecordRow involvement
```

---

## T4 — Add checked substitution/materialization and recursive semantic preservation

### Purpose

Make P1 terms safe under semantic rewriting so symbolic compositions re-canonicalize when bindings become concrete and newly proven duplicate labels become explicit invalidity rather than stale symbolic terms.

### Preconditions

- G3 passed;
- concat canonicalizers are stable.

### Consumes

- complete/concat constructors;
- existing `TypeSubstitution`;
- existing fallible `materialize_type` / `GenericInstantiation`;
- existing ordinary kind-aware inference session.

### Produces

- checked substitution path for C2 terms;
- pack-aware materialization errors;
- recursive new-term handling in semantic walkers where required;
- proof that complete-pack generic parameters use ordinary `InferVarId`;
- CV-15..CV-18 evidence.

### Files and symbols

**Primary modify:**

```text
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/checker/inference.rs                # only if a focused kind-preservation test needs an explicit case
phalcom-semantic/src/checker/row_inference.rs            # only for GenericApplicationSession focused proof / traversal
```

**Recursive visitor followups as required:**

```text
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/advisory/formal.rs
phalcom-semantic/src/impls.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/substitution.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
```

### Required implementation shape

1. Add one explicit checked substitution API/error channel for C2 terms.
2. Recurse through `CompleteTypePack`, positional concat, and labeled concat children.
3. After child substitution, call the canonical constructor/normalizer again.
4. A valid all-concrete result collapses to C1 pack identity.
5. A newly concrete duplicate labeled result returns semantic invalidity.
6. `materialize_type` performs the same checked re-canonicalization and maps failure into `TypeMaterializationError` (or exact equivalent).
7. New recursive semantic visitors that inspect contained parameters/dependencies must traverse children rather than treating the term as opaque.
8. Do not add a pack solver. To prove lifetime correctness, construct a generic signature containing a complete-pack parameter and verify `GenericApplicationSession` allocates ordinary `InferVarId` with the exact kind.
9. Keep the existing unchecked substitution API limited to pre-C2-safe callers as described in §8.9; do not silently swallow C2 errors.

### Forbidden approaches

- no `unwrap_or(original_concat)`;
- no invalid `TypeId` sentinel;
- no conversion to `Dynamic`, `Unknown`, or `Never`;
- no query-local inference ID in canonical `TypeData`;
- no `PackVarId`;
- no global speculative substitution API rewrite if a bounded checked seam works;
- no source C2 syntax enablement in this task.

### Test changes required

Add tests for:

- CV-15 symbolic labeled concat becoming invalid after substitution;
- CV-16 valid symbolic concat becoming concrete C1 pack identity;
- CV-17 materialization producing both reduced-valid and invalid duplicate outcomes;
- CV-18 complete-pack parameter gets ordinary kind-aware inference variable;
- recursive parameter/dependency walker sees parameters inside complete/concat terms where that walker is part of P1 semantics.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::substitution -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generic_application -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Tests explicitly deferred

Broad trait/enum/generic source integration and all parser/runtime tests.

### Acceptance

- no C2 canonical term becomes stale/invalid silently after checked substitution/materialization;
- valid concrete rewrites converge to existing C1 identity;
- invalid duplicate rewrite is surfaced as an error;
- complete-pack generic unknowns use ordinary inference variables;
- no new solver identity or durable query-local ID exists.

### Local STOP / CONSULT triggers

- implementing checked substitution safely requires changing a broad set of source-reachable API contracts with materially different error semantics;
- a production path can already feed C2 terms through unchecked substitution and would panic or hide invalidity;
- inference needs sequence decomposition to make the focused complete-kind test work.

### Checkpoint update

Record the checked re-canonicalization interface and the explicit P2 obligation: no source C2 term may enter an unchecked substitution path.

---

## G4 — Checked rewrite and inference-lifetime gate

### Purpose

Prove the hardest semantic correctness condition in P1: symbolic labeled overlap can later become invalid without leaking an invalid canonical term, while pack unknowns still use existing inference lifetime machinery.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::substitution -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generic_application -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Expected

```text
CV-15..CV-18 PASS
no invalid concrete labeled pack preserved as symbolic
no PackVarId / PackId introduced
```

---

## T5 — Complete durable metadata, fingerprints, compatibility consumers, and focused regression closure

### Purpose

Make every P1 canonical term and binder-mode fact durable, deterministic, schema-valid, and consumable across the existing metadata/reflection inspection seams, then prove P1 does not regress C1.

### Preconditions

- G4 passed;
- semantic representations and checked rewrite behavior are stable.

### Consumes

- exact complete kind/term;
- binder modes;
- concat forms;
- schema v3 foundation;
- existing metadata exporter/validator/fingerprint architecture.

### Produces

- stable complete/concat `TypeNode`s;
- stable binder mode in parameter metadata;
- schema-v3 validation floors;
- deterministic structural fingerprints;
- mode-sensitive public interface fingerprint;
- backward v1/v2 decode behavior for historical forms;
- runtime typing inspector formatting compatibility;
- C1 focused compatibility evidence;
- final P1 lifecycle records.

### Files and symbols

**Modify:**

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/validate.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/export.rs
phalcom-core/src/typing/inspect.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-type-meta tests
```

### Required implementation shape

1. Add stable node forms from §8.12.
2. Add `GenericParameterModeRef` or equivalent stable mode.
3. Add mode to `TypeParameterRecord` with backward-deserialization default `Single` for older records.
4. Complete schema-floor validation:
   - schemas 1/2 reject C2-only forms/modes;
   - schema 3 accepts them subject to structural checks.
5. Validate child kind relationships for complete/concat nodes.
6. Validate mode↔kind/variance coherence in parameter records.
7. Export semantic complete/concat terms recursively and topologically.
8. Preserve stable child ordering exactly.
9. Allocate new explicit structural fingerprint tags without altering any old tag.
10. Ensure interface/public fingerprint includes binder mode through the actual stable publication path.
11. Add deterministic fresh-store fingerprint tests for complete/concat terms.
12. Extend `phalcom-core/src/typing/inspect.rs` for metadata inspection/formatting only.
13. Keep `CompiledTypeRef` new pack terms explicitly non-exportable unless live code proves a current consumer needs a representation; do extend `CompiledKindRef` for complete kind.
14. Run static audits for forbidden runtime/source shortcuts.

### Forbidden approaches

- no raw `TypeId` or infer IDs in metadata;
- no schema-2 output containing C2 forms;
- no serde-ordinal-as-compatibility assumption;
- no mode derived from kind during reflection;
- no old fingerprint tag renumbering;
- no runtime pack object or VM/native operation;
- no source parser changes;
- no C1 metadata weakening.

### Test changes required

Add tests for:

- CV-19 stable complete/concat metadata nodes + validation;
- CV-20 binder mode metadata + interface fingerprint change;
- CV-21 v1/v2 historical decode/default behavior + schema-v3 C2 floor;
- CV-22 malformed complete/concat/mode metadata rejection;
- CV-23 existing C1 closure metadata/type-member tests;
- CV-24 labeled order/row distinction visible in metadata;
- CV-25 static runtime-boundary audit;
- CV-27 no `@intrinsic` resurrection.

### Tests to run now

Run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::type_member_closure -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generics_core -- --nocapture --test-threads=1
```

Then affected-crate checks:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Hygiene:

```sh
cargo fmt --all -- --check
git diff --check
```

### Static audits

```sh
rg -n 'PackVarId|struct PackId|enum PackId|CompletePackArena' \
  phalcom-semantic phalcom-type-meta phalcom-core

rg -n 'TypePack.*RecordRow|LabeledTypePack.*RecordRow|LabeledTypePackConcat.*RecordRow' \
  phalcom-semantic/src phalcom-type-meta/src

rg -n 'InferVarId|RecordRowVarId' phalcom-type-meta/src

rg -n '@intrinsic' \
  phalcom-semantic phalcom-type-meta phalcom-core/core/universe/src
```

Interpret audit hits; do not demand literal zero where the term appears in comments/tests explicitly proving the prohibition.

### Tests explicitly deferred

```text
workspace test/clippy
full semantic target
full core/VM suites
full language corpus
P2 parser/type-annotation integration
P3 incremental/cross-module source parity
```

### Acceptance

- every P1 durable semantic fact has stable metadata representation;
- schema v3 is explicit and backward-safe for historical forms;
- binder mode changes interface identity;
- metadata rejects malformed C2 structure;
- core inspector can observe metadata without runtime pack execution;
- focused C1 structural/type-member metadata regressions remain green;
- affected crates compile and hygiene passes.

### Local STOP / CONSULT triggers

- stable metadata cannot express binder mode without breaking old decode in a way schema v3 cannot isolate;
- interface fingerprint architecture cannot include mode without a new global identity design;
- core consumer demands runtime executable pack representation rather than stable inspection;
- a C1 fingerprint/tag must be changed to fit the new variants.

### Checkpoint update

Required after G5: record metadata schema, stable node/interface facts, verification evidence, deferred failures, P1 completion, and next action P2.

---

# 20. Verification gates

G1–G4 are defined inline after their owning tasks. G5 is the plan-level stabilization gate.

## G5 — P1 focused acceptance and C1 compatibility gate

### Purpose

Prove the entire P1 substrate as one coherent implementation and establish honest `FOCUSED_TESTED` evidence without workspace/release over-testing.

### Mandatory run

Run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generics_core -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::substitution -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generic_application -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::type_member_closure -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
cargo fmt --all -- --check
git diff --check
```

Confirm every filter selects actual tests. If a named module has moved mechanically, discover its current exact filter and record the adapted command in the walkthrough.

### Mandatory static audits

Use the T5 audits and record conclusions, especially:

```text
no PackId/PackVarId authority
no labeled-pack→RecordRow shortcut
no query-local IDs in stable schema
no @intrinsic resurrection
no VM/native pack execution additions
```

For the last item, review the scoped diff in `phalcom-core`: P1 should touch only metadata inspection/formatting if at all.

### Expected

```text
all CV-01..CV-27 have evidence
all P1 focused tests pass or are truthfully baseline-classified
C1 structural/type-member closure remains green
all affected crates compile
format/diff hygiene clean
no active architectural incident
workspace/release testing still deferred
```

### Do not broaden automatically

A G5 PASS ends required P1 verification. Do not run workspace tests/clippy/full corpora solely because G5 passed.

---

# 21. Final focused acceptance matrix

At P1 completion, the walkthrough must contain a table of this form populated with actual test names/results:

| Coverage ID | Evidence | Result |
|---|---|---|
| CV-01 | exact complete-kind test | PASS |
| CV-02 | complete child-kind negative tests | PASS |
| CV-03 | empty complete vs Tuple/Unit test | PASS |
| CV-04 | lane projection matrix | PASS |
| CV-05 | Single(TypePack) vs PositionalPack test | PASS |
| CV-06 | mode-only parameter version test | PASS |
| CV-07 | mode-kind/variance publication negative tests | PASS |
| CV-08 | positional empty/single laws | PASS |
| CV-09 | positional flatten/merge laws | PASS |
| CV-10 | positional symbolic ordering | PASS |
| CV-11 | labeled empty/single laws | PASS |
| CV-12 | labeled flatten/merge/order laws | PASS |
| CV-13 | concrete duplicate rejection | PASS |
| CV-14 | symbolic overlap residual | PASS |
| CV-15 | post-substitution duplicate invalidity | PASS |
| CV-16 | post-substitution concrete reduction | PASS |
| CV-17 | materialization reduction/invalidity | PASS |
| CV-18 | ordinary InferVarId complete-kind test | PASS |
| CV-19 | stable complete/concat metadata | PASS |
| CV-20 | stable binder mode/interface fingerprint | PASS |
| CV-21 | schema backward compatibility | PASS |
| CV-22 | malformed C2 metadata rejection | PASS |
| CV-23 | C1 compatibility suite | PASS |
| CV-24 | ordered labeled pack remains non-row | PASS |
| CV-25 | runtime-negative architecture audit | PASS |
| CV-26 | moderate-density flat concat test | PASS |
| CV-27 | intrinsic keyword/attribute boundary audit | PASS |

Do not pre-fill results in the actual walkthrough before execution.

---

# 22. Performance / resource evidence

P1 has a structural complexity requirement but does not require benchmarking.

Required algorithmic bounds:

```text
complete pack construction       O(1) + child validation
positional concat normalization  O(segments + concrete elements)
labeled concat normalization     O(segments + concrete entries)
duplicate known-label lookup     expected O(1) per concrete label
metadata export                  O(reachable new-node graph)
```

Required evidence:

- code review shows one flat canonicalization pass rather than partition/backtracking;
- CV-26 constructs a moderate number of segments and proves the interned residual is flat, not a deep recursive concat tree;
- no benchmark harness or performance suite is required.

If implementation requires repeated full-vector copying that makes canonicalization obviously quadratic in segment count, correct it before G3/G5.

---

# 23. Checkpoint bookkeeping

## 23.1 At plan start

Update `LANG007.C2-CHECKPOINT.md`:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG007.C2.P1
next_plan: null
```

Add P1 to the plan ledger as active and record the actual local starting revision.

Update `PROGRAM.md` / `STATUS.md` only as required to stop claiming that C2 merely has “no active plan.”

## 23.2 During plan

Update the checkpoint only for durable events:

```text
G1: exact complete-pack kind/term established
G2: binder-mode identity/versioning established
G3: concat canonical forms established
G4: checked rewrite seam established
schema/metadata compatibility decision if amended
consultation/amendment
meaningful deferred failure
coherent verification gate
```

Do not turn the checkpoint into a chronological test log.

## 23.3 At plan completion

If G5 passes:

```yaml
P1 status: COMPLETE
P1 completion: IMPLEMENTED
P1 verification: FOCUSED_TESTED

C2 status: IN_PROGRESS
C2 completion: PARTIAL
C2 verification: FOCUSED_TESTED
active_plan: null
next_plan: LANG007.C2.P2
```

Record:

- final P1 interfaces/invariants;
- actual verification commands/results;
- consultations/amendments;
- deferred/baseline failures;
- explicit P2 checked-substitution takeover obligation;
- next action: plan/execute P2 source elaboration.

Do **not** mark the C2 checkpoint complete.

---

# 24. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P1-walkthrough.md
```

It must contain:

1. starting and final revision;
2. final plan state;
3. exact as-built complete-pack kind/term names;
4. exact binder-mode representation and mode-versioning behavior;
5. exact positional/labeled concat representation and normalization laws;
6. checked substitution/materialization interface actually implemented;
7. metadata schema/version change and compatibility behavior;
8. stable node/fingerprint implementation;
9. compiler/core compatibility edits;
10. important mechanical deviations from this plan;
11. consultations/amendments;
12. tests added;
13. tests actually run and selected counts;
14. tests deliberately deferred;
15. CV-01..CV-27 evidence matrix;
16. residual risks/deferred failures;
17. verification classification.

Do not claim workspace/release certification.

---

# 25. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P1-handoff.md
```

The P2 handoff must contain:

```text
current revision and worktree status
P1 completion/verification state
exact stable semantic symbols/APIs
complete-pack kind/domain law
GenericParameterMode law
concat canonicalization law
metadata schema v3 law
mode-sensitive parameter/public fingerprint law
known baseline/deferred failures
P2 objective and must-read files
first recommended commands
```

It must also state these **P2 prohibitions** explicitly:

> P2 must elaborate source `*` / `**` / `***` directly into P1 canonical terms; it must not create source-only pack identities.

> P2 must not expose any C2 source form through an unchecked substitution/materialization path that can hide or panic on newly concrete duplicate labels. Migrate the source-reachable call sites to the P1 checked seam before enabling the syntax that can reach them.

> P2 must not infer generic binder mode from kind.

> P2 must not use `...` as pack syntax.

> P2 must not add pack decomposition inference or runtime ABI work.

---

# 26. Completion truth table

Do not conflate these states:

| State | Meaning |
|---|---|
| source written | Rust changes exist |
| tests added | regression cases exist |
| targeted tests passed | exact behavior has evidence |
| affected-crate checks passed | compile integration is coherent |
| P1 implemented | all P1 tasks complete |
| P1 focused-tested | G5 required evidence passed/classified |
| C2 checkpoint complete | **false after P1**; requires P2/P3 |
| release certified | **false unless separately performed** |

Expected honest P1 finish:

```yaml
LANG007.C2.P1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C2:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: FOCUSED_TESTED
```

---

# 27. Ordered execution summary

Execute in this order:

```text
T0  live authority / planning-package / drift verification
 ↓
T1  exact complete-pack kind + term + shared lane projection
 ↓
G1  complete-domain gate
 ↓
T2  semantic binder mode + parameter/signature identity
 ↓
G2  binder-mode gate
 ↓
T3  positional/labeled concat canonicalization
 ↓
G3  composition gate
 ↓
T4  checked substitution/materialization + inference lifetime
 ↓
G4  rewrite/lifetime gate
 ↓
T5  metadata/fingerprints/compatibility + C1 regression closure
 ↓
G5  focused P1 acceptance
 ↓
walkthrough + P2 handoff + checkpoint transition
```

Do not reorder source/parser work into P1.

Do not postpone metadata until after P2; P2 must consume stable P1 terms with known persistence semantics.

Do not start broad regression certification before G5.

---

# 28. Plan self-review

## Architecture

- [x] one canonical owner exists for complete/concat pack terms (`TypeStore`);
- [x] complete pack is not a proper Type/Tuple;
- [x] labeled packs remain distinct from rows;
- [x] binder mode is explicit and not kind-derived;
- [x] mode changes semantic and public interface identity;
- [x] concat canonicalization is flat/deterministic;
- [x] concrete duplicate invalidity is explicit;
- [x] symbolic overlap remains representable;
- [x] checked rewriting prevents stale invalid terms;
- [x] existing GenericInstantiation/InferVarId lifetimes are reused;
- [x] metadata schema/versioning is explicit;
- [x] type-level work remains outside the VM.

## Luna executability

- [x] tasks are dependency-ordered and coherent;
- [x] source files/symbols are grounded in the live baseline;
- [x] fixed/flexible/verify-first decisions are explicit;
- [x] likely architecture forks are pre-decided;
- [x] checked-substitution staging is bounded and has an explicit P2 takeover rule;
- [x] STOP/CONSULT conditions are objective;
- [x] debugging loops are bounded.

## Testing

- [x] every major invariant has a coverage obligation;
- [x] canonical/zero/single/many/negative/symbolic/persistence/resource cases are represented;
- [x] C1 compatibility is included;
- [x] broad workspace testing is explicitly deferred;
- [x] no parser/runtime tests are required without relevant P1 changes.

## Scope

- [x] no source variadic syntax is implemented;
- [x] no pack decomposition solver is implemented;
- [x] no runtime pack object/ABI is implemented;
- [x] no RecordRow conflation is planned;
- [x] no public complete kind syntax is planned;
- [x] no `@intrinsic` regression is permitted;
- [x] no unrelated baseline repair is required.

## Lifecycle

- [x] checkpoint activation is required;
- [x] durable gate updates are specified;
- [x] completion state remains checkpoint-partial;
- [x] walkthrough and P2 handoff are mandatory.

---

# 29. Compact executor directive

Keep these sentences visible while implementing:

> `TypeId` is the canonical carrier for kinded pack terms; do not create a pack arena.

> `CompleteTypePack(P,L)` is a type-level lane product, not Tuple and not runtime ArgumentPack.

> Binder mode is not kind: `P: TypePack` and `*P` are different interfaces.

> Positional/labeled concat is flat and canonical; all-concrete valid results collapse to C1 pack identity.

> Concrete duplicate labels are invalid; unknown overlap stays symbolic until evidence resolves it.

> Rewriting must be checked: never preserve a newly invalid concrete composition as an old symbolic term.

> Use ordinary `InferVarId(kind)` and existing `GenericInstantiation`; no `PackVarId`, no `PackInstantiation`.

> Stable metadata is part of P1, including schema v3 and binder-mode fingerprints.

> P1 does not add parser syntax, public complete-kind spelling, pack decomposition, runtime ABI, or VM execution.

> `*`, `**`, `***` are the only C2 pack operators; `...` is not pack syntax.

> If correctness seems to require Tuple/RecordRow conflation, a second semantic authority, a pack solver, a runtime pack object, or a broad substitution redesign, STOP AND CONSULT.

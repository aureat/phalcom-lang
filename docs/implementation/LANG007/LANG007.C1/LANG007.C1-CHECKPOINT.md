---
id: LANG007.C1
category: LANG
program: LANG007
checkpoint: LANG007.C1
kind: checkpoint-record
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
planning_baseline_revision: 5b63ade8073d13d6b528af4774181665c30cbd47
planning_baseline_subject: "fix(core): restore standalone semantic universe"
prepared: 2026-09-21
p1_implementation_revision: 479f8fb82fa58730fb2c40c2652b5e07016549ec
p1_implementation_subject: "feat(semantic): add intrinsic type-level structural operations"
p3_start_revision: 6551576035a6e55718497c457fac6073b322d599
p3_start_subject: "feat(lang007): implement P2 authoritative specification and intrinsic keyword convergence"
active_plan: null
requires:
  - accepted LANG007 type-level domain semantics
  - accepted LANG007 type-member semantics
  - accepted LANG007 structural Tuple/Record operation semantics
  - accepted Tuple/Record structural-product decisions
---

# LANG007.C1 — Semantic Domains and Compiler-Intrinsic Structural Type Operations

## 0. Checkpoint role

This file is the **single durable implementation-state authority** for `LANG007.C1`.

It records:

- the bounded C1 acceptance objective;
- what C1 owns and defers;
- the numbered-plan ledger;
- accepted architectural invariants;
- stable takeover interfaces;
- actual verification gates once executed;
- deferred/baseline failures;
- consultation/amendment decisions;
- the active plan and next action.

Numbered plans describe intended work. Walkthroughs record what a plan actually implemented and proved. Handoffs provide continuation context. Neither replaces this checkpoint record.

Current lifecycle truth:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

P1, P2, and P3 are complete and focused-tested. `LANG007.C1` is complete and closed.

---

# 1. Bounded acceptance objective

C1 is accepted when Phalcom has one coherent compiler implementation for the first closed structural type-operation vertical:

```phalcom
(Int, String)::Positionals
(Int, String, name: Symbol)::Labeled

(Int, String)[1]
(Int, String)::At<1>
(Int, String)::At<-1>

(Int, String)::Map<Option>
(Int, name: String)::Map<Option>

#{name: String, age: Int}::Map<Option>
```

plus the accepted symbolic forms over `TypePack`, `LabeledTypePack`, and `RecordRow`.

C1 must establish the semantic substrate required to form, preserve, normalize, infer/substitute, serialize, and source-index those operations while preserving existing associated-type semantics and keeping type computation out of runtime VM dispatch.

C1 is not accepted merely because syntax parses or individual concrete examples reduce. The checkpoint requires the full bounded vertical:

```text
source syntax
    ↓
canonical semantic domains
    ↓
type-member identity / typed arguments
    ↓
symbolic projection
    ↓
intrinsic structural normalization
    ↓
substitution / inference / materialization
    ↓
stable metadata / source identity
    ↓
canonical Universe source integration
    ↓
no VM type-member solver
```

---

# 2. Ownership boundary

## 2.1 C1 owns

C1 owns code implementation for:

- `TypePack` and `LabeledTypePack` semantic kinds/domains;
- canonical ordered pack terms;
- symbolic Tuple formation from pack-valued terms where required;
- native/compiler-intrinsic type-member declaration and projection syntax;
- canonical type-member identity;
- typed type-member parameter domains;
- signed integer compile-time member arguments for Tuple indexing;
- canonical identity equivalence between bracket Tuple indexing and `At`;
- Tuple `Positionals`, `Labeled`, and integer `At`;
- shape-preserving `Map<F>` for the accepted Tuple/Record/pack/row domains;
- symbolic residual projections when reduction is well formed but blocked on abstract inputs;
- open-row mapping that transforms the symbolic tail;
- substitution/materialization/inference integration;
- durable stable metadata for new kinds/terms/projections;
- source target identity for type members;
- canonical Universe source anchors;
- explicit proof that type-member computation requires no new VM bytecode/runtime dispatch.

## 2.2 C1 does not own

C1 does not own:

```text
general nominal const generics
trait const generics
generic associated types
user-defined open type families
arbitrary compile-time evaluation
const arithmetic
pack arithmetic
general pack concatenation
multiple unknown-length pack split inference
variadic call-argument inference
variadic callable lowering
runtime TypePack/LabeledTypePack objects
runtime type-member methods
union-distributive structural operations
type-level Symbol indexing
type-level Range slicing
general reflection UI/API design
unrelated trait/conformance redesign
broad runtime collection behavior redesign
```

If P1 appears to require one of these to satisfy a C1 requirement, that is architectural drift and must trigger consultation rather than silent scope expansion.

---

# 3. Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG007.C1.P1` | semantic domains, type-member identity/projection, structural operations, inference/persistence, Universe/runtime boundary | `COMPLETE` | `IMPLEMENTED` | `FOCUSED_TESTED` | completed at `479f8fb8`; final scoped validation remains uncommitted |
| `LANG007.C1.P2` | authoritative specification and `intrinsic` keyword convergence | `COMPLETE` | `IMPLEMENTED` | `FOCUSED_TESTED` | completed; TDR-0089 accepted, @intrinsic retired, intrinsic keyword preserved, normative specs converged |
| `LANG007.C1.P3` | architecture baseline, cross-layer certification, roadmap re-baselining, and C1 closure | `COMPLETE` | `IMPLEMENTED` | `FOCUSED_TESTED` | completed; architecture baseline established, cross-layer integration certified, incremental parity proven, acceptance matrix CV-01..CV-28 verified, roadmap re-baselined, C1 closed |

P1 lifecycle closure is recorded by:

- `LANG007.C1.P1-walkthrough.md`
- `LANG007.C1.P1-handoff.md`

P2 lifecycle closure is recorded by:

- `LANG007.C1.P2-walkthrough.md`
- `LANG007.C1.P2-handoff.md`

P3 lifecycle closure is recorded by:

- `LANG007.C1.P3-walkthrough.md`
- `LANG007.C1.P3-handoff.md`

Latest completed plan: **`LANG007.C1.P3`**.

Active plan: **none**.

Next action: define and plan follow-on checkpoint `LANG007.C2`.

---

# 4. Accepted architectural invariants

These are durable C1 architectural facts accepted by the LANG007 design/planning
work. P1's implementation and focused evidence now establish the corresponding
code path; P2 must converge the authority corpus on these facts.

## 4.1 Canonical semantic domains

1. `Type`, `RecordRow`, `TypePack`, and `LabeledTypePack` are semantically distinct domains/kinds.
2. `TypePack` is an ordered sequence of proper type terms.
3. `LabeledTypePack` is an ordered sequence of `(label, proper type)` entries.
4. `RecordRow` remains row/key structure with order-independent field identity.
5. Tuple labeled-lane ordering must never be represented through sorted `RecordRow` machinery.
6. Concrete structural Tuple remains the existing structural proper-type representation; no second nominal `Tuple<...>` identity is introduced.

## 4.2 `TypeId` and proper-type boundary

7. `TypeId` is the canonical identity of a kinded type-level form, not semantically synonymous with a runtime proper type.
8. `ProperTypeId` remains the proof boundary that a kinded form has kind `Type`.
9. Pack-kinded canonical terms may therefore use the existing `TypeId` arena without becoming proper types.
10. `RecordRow` remains on its existing distinct canonical row path.

## 4.3 Type-member identity and arguments

11. Type-member identity is semantic and typed; it is not member-name text.
12. Bracket Tuple indexing and the accepted `At` spelling denote one canonical subscript type-member identity.
13. `F` in `Map<F>` is a type-level form constrained by kind `Type -> Type`.
14. `I` in integer `At<I>` is a typed compile-time `Int` term; `Int` is not a new kind.
15. C1 needs heterogeneous **type-member** arguments, not a language-wide nominal/trait const-generic migration.

## 4.4 Projection and normalization

16. Type-member projection identity includes subject, member identity, canonical arguments, and result domain.
17. A well-formed abstract projection may remain symbolic.
18. `Stuck` and `Invalid` are semantically distinct.
19. Invalid concrete projections must not be approximated as `Dynamic`, `Never`, or unrelated unknown types.
20. Existing trait associated-type identity/evidence remains authoritative and semantically unchanged.

## 4.5 Tuple/Record structural operations

21. Tuple component order is positional lane followed by ordered labeled lane.
22. Concrete Tuple integer indexing supports negative indexes and indexes that complete linear component order.
23. Tuple `Map<F>` preserves arity, lane boundary, labels, label order, and component order.
24. `TypePack::Map<F>` preserves pack order.
25. `LabeledTypePack::Map<F>` preserves labels and label order.
26. Record mapping preserves key identity.
27. Mapping an open row maps the unknown tail; retaining the original tail unchanged is incorrect.
28. Intrinsic `Map` forms `F<T>` through the existing ordinary type-form application authority rather than a Map-specific application engine.
29. C1 does not implicitly distribute structural operations across unions.

## 4.6 Lifetime and persistence

30. Query-local `InferVarId` and `RecordRowVarId` must never enter canonical type state, durable metadata, or snapshots.
31. Fully concrete symbolic Tuple-from-pack formation must collapse to the existing canonical Tuple/zero-product identity.
32. Stable row-valued projection terms must remain distinct from row-solver metavariables.
33. Durable metadata encodes stable semantic structure, never raw store/query-local IDs.

## 4.7 Compiler/runtime boundary

34. Compiler-intrinsic type operations normalize in semantic/type-system code.
35. C1 introduces no type-member VM bytecode.
36. C1 introduces no runtime type-member message send or conformance/projection solver.
37. Canonical Universe source may anchor intrinsic members for source identity/navigation without making them runtime native callables.
38. Native and Intrinsic are distinct `ImplementationKind` values. `@intrinsic` is retired as a source/builtin attribute. `NativeIntrinsicId` is separate internal operation metadata and does not determine source `ImplementationKind`.

---

# 5. Stable interface / takeover map

This table describes the **planning baseline** at revision `5b63ade8073d13d6b528af4774181665c30cbd47`. The P1 implementer must verify the live checkout before relying on symbol mechanics.

| Concern | Planning-baseline owner / symbol | C1 rule |
|---|---|---|
| kind identity | `phalcom-semantic/src/types/{id,kind,store}.rs` — `KindId`, `KindData`, `TypeStore` | extend existing kind authority |
| proper-type proof | `ProperTypeId` | preserve as kind-`Type` proof boundary |
| canonical kinded forms | `TypeId` / `TypeStore` | use for TypePack/LabeledTypePack forms |
| structural Tuple | `TypeData::Tuple`, `TupleTypeElement` | preserve ordered positional+labeled identity |
| structural Record | `TypeData::Record(RecordRowId)` | preserve row/key authority |
| row canonicalization | `RecordRowData`, `RecordRowTail` | extend only as required for stable mapped-tail projections |
| higher-kinded application | `TypeStore::apply_type_form` | canonical `F<T>` authority for Map |
| generic parameter kind | `TypeParameterData.kind` | reuse for pack-kinded binders |
| generic substitution | `GenericInstantiation` | kinded bindings may carry pack TypeIds; row bindings remain separate |
| lexical type-level binding | `TypeLevelBinding` | preserve row/kinded distinction |
| ordinary inference | `InferenceSession`, `InferVarId` | pack kinds remain kind-aware query-local inference |
| row inference | `RecordRowSolver`, `RecordRowVarId` | no pack solver substitution |
| combined generic application | `GenericApplicationSession` | reuse coordinator; do not create parallel call solver |
| associated projection | current associated projection + `types::projection` | generalize without weakening trait semantics |
| stable kind metadata | `phalcom-type-meta::KindNode` | add stable pack kinds |
| stable type metadata | `phalcom-type-meta::TypeNode` | add stable pack/projection forms |
| type source lowering | `phalcom-semantic/src/types/annotation.rs` | semantic expected-domain elaboration |
| source target identity | `phalcom-semantic/src/source_index/*` | type-member occurrences target canonical semantic member |
| Tuple runtime behavior | `universe/collections/tuple.ph` | behavior reference, not type solver |
| Record runtime behavior | `universe/collections/record.ph` | behavior reference, not type solver |
| runtime native source census | `phalcom-core/src/native/source.rs` | type members must not become runtime callable anchors |

Mechanical paths may drift. Ownership/identity drift requires consultation.

---

# 6. P1 evidence/gate map

The P1 plan divides C1 implementation evidence into coherent gates.

| Gate | Scope | Required semantic proof | State |
|---|---|---|---|
| `LANG007.C1.P1.G1` | canonical domains | TypePack/LabeledTypePack kinding, proper-type boundary, pack canonicalization, checked Tuple formation | `PASS` |
| `LANG007.C1.P1.G2` | member identity/projection | typed arguments, `[]`/`At` identity, symbolic projections, existing associated-type compatibility | `PASS` |
| `LANG007.C1.P1.G3` | structural algebra | Positionals/Labeled/At/Map including negative indexes and open-row mapped tails | `PASS` |
| `LANG007.C1.P1.G4` | inference/persistence | no query-local leakage; stable metadata/source identity; row residual preservation | `PASS` |
| `LANG007.C1.P1.G5` | focused plan acceptance | complete focused C1 matrix + affected-crate checks + hygiene | `PASS` |

No gate may be marked PASS until its named evidence has actually run successfully.

---

# 7. Verification ledger

P1 implementation verification is complete at the focused-acceptance level. The
commands below are representative gate evidence; the full chronological test
transcript remains in the P1 walkthrough.

| Gate / evidence | Scope | Command / target | Result | Proves |
|---|---|---|---|---|
| planning repository inspection | remote `main` at P1 planning baseline | source/architecture inspection only | `INFORMATIONAL` | establishes planning assumptions; **not implementation verification** |
| P1 focused semantic foundations | `phalcom-semantic` semantic integration target | `foundations::type_members` (6), `foundations::type_annotations` (40), `foundations::record_row_inference` (3), `foundations::generic_application` (9) | `PASS` | structural operations, argument validation, type formation, rows, and generic application |
| P1 metadata and parser focus | `phalcom-semantic` / `phalcom-ast` | `integration::metadata` (14), parser kind/projection/intrinsic tests (1 + 1 + 1) | `PASS` | stable metadata and accepted source forms |
| P1 affected-crate and hygiene checks | `phalcom-ast`, `phalcom-type-meta`, `phalcom-semantic`, `phalcom-core` | `cargo check`, `cargo fmt --all -- --check`, `git diff --check` | `PASS` | affected crate compilation and patch hygiene |
| focused trait suite residuals | `phalcom-semantic` semantic integration target | 36 passed, 2 baseline failures reproduced at predecessor `5b63ade8` | `BASELINE_DEFERRED` | unrelated semantic dependency/evidence baseline; see §9 and P1 walkthrough |
| P2 AST integration focus | `phalcom-ast` integration test target | `parser::intrinsic_attribute_is_not_builtin`, `parser::parses_intrinsic_declarations`, `parser::parses_intrinsic_fields_and_methods` (3 tests) | `PASS` | `intrinsic` keyword parsing preserved; `@intrinsic` retired as builtin attribute |
| P2 native presentation focus | `phalcom-semantic` integration test target | `integration::native_conformance` (3 tests) | `PASS` | virtual source has no `@intrinsic`; internal `NativeIntrinsicId` structured |
| P2 type members regression | `phalcom-semantic` foundations test target | `foundations::type_members` (6 tests) | `PASS` | regression proof for all P1 type-member operations |
| P2 affected-crate compilation | `phalcom-ast`, `phalcom-semantic`, `phalcom-core` | `cargo check` for each crate | `PASS` | all affected crates compile cleanly |
| P2 specification & audit searches | `docs/specs`, `docs/decisions`, `phalcom-*` | G1, G2, G3, T5 audit queries and `git diff --check` | `PASS` | zero live `@intrinsic` in current authority, singular ownership, no diff warnings |
| P3 closure integration | `phalcom-semantic` integration target | `integration::type_member_closure` (9 tests) | `PASS` | alias identity, domain separation, trait compatibility, structural composition, zero products, metadata fingerprints, density |
| P3 incremental cold parity | `phalcom-semantic` incremental target | `incremental::type_members` (2 tests) | `PASS` | cold/incremental final-state parity for Tuple indexing and Map edits without global invalidation |
| P3 affected-crate & hygiene checks | `phalcom-ast`, `phalcom-type-meta`, `phalcom-semantic`, `phalcom-core` | `cargo check`, `cargo fmt --all -- --check`, `git diff --check` | `PASS` | affected crates check cleanly, formatting clean, zero diff warnings |

Current checkpoint verification:

```yaml
verification: FOCUSED_TESTED
```

P1's focused gate does not claim workspace-wide, clippy, corpus, runtime,
LSP, performance, or release certification.

---

# 8. Deferred verification

P1 intentionally targets focused acceptance, not release-wide certification.

Unless later required by evidence or user instruction, the following are deferred beyond BUILD/STABILIZE:

```text
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
full language corpus
unrelated VM/runtime suites
all LSP tests
broad performance suites
```

`RELEASE_COMPLETE` must not be used unless the named broad certification gates actually run successfully.

---

# 9. Deferred / baseline issue ledger

P1's focused execution observed two residual trait-suite failures. Both
reproduce at the clean predecessor revision and are deferred as unrelated
baseline work:

- `semantic::capabilities::traits::trait_default_replays_external_nominal_callable_dependencies`
- `semantic::capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities`

They do not block P1 or P2 because they are outside the changed type-member
path and reproduce before the P1 implementation.

### BASELINE-001

- **Observed:** focused trait test fails on the active tree because an
  unexpected Universe declaration-shell dependency is present.
- **Classification:** baseline / unrelated.
- **Why it does not block C1:** the assertion concerns trait dependency
  evidence, not type-member formation, projection, or structural normalization;
  the failure reproduces at the clean predecessor.
- **Reproduction:**
  `cargo test -p phalcom-semantic --test semantic capabilities::traits::trait_default_replays_external_nominal_callable_dependencies -- --nocapture`
- **When to revisit:** the owning semantic dependency/evidence plan.

### BASELINE-002

- **Observed:** focused trait test fails on the active tree because a Bool
  declaration-shell dependency accompanies the expected trait-surface evidence.
- **Classification:** baseline / unrelated.
- **Why it does not block C1:** the assertion concerns trait default dependency
  filtering, not type-member formation, projection, or structural normalization;
  the failure reproduces at the clean predecessor.
- **Reproduction:**
  `cargo test -p phalcom-semantic --test semantic capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities -- --nocapture`
- **When to revisit:** the owning semantic dependency/evidence plan.

Required entry shape:

```text
ID:
Observed:
Classification: unrelated | baseline | unclear-but-nonblocking
Why it does not block C1 acceptance:
Reproduction:
When to revisit:
```

Failure policy:

```text
A — definitely caused by C1 patch: fix
B — probably caused by C1 patch: investigate/fix
C — unclear: one bounded classification pass; defer if nonblocking
D — clearly unrelated/baseline: record and continue
```

Do not silently repair unrelated failures during P1.

---

# 10. Consultation / amendment ledger

## AMEND-001 — Lifecycle bookkeeping reinstated

**Date:** 2026-09-21

**Plan:** `LANG007.C1.P1`

**Trigger:** The initial P1 planning artifact carried an explicit temporary user instruction to skip `docs/implementation/**` updates during implementation. The user subsequently explicitly requested canonical `STATUS.md`, `PROGRAM.md`, checkpoint, and guidance records under the implementation lifecycle convention.

**Decision:** Canonical lifecycle bookkeeping is enabled for C1.

**Architectural consequence:** None to language semantics or compiler design.

**Execution consequence:** P1 must:

- read and update this checkpoint at plan start;
- update it at durable architecture/verification events;
- record consultations and deferred failures;
- update final P1 lifecycle state;
- create P1 walkthrough/handoff when applicable.

**Scope limit:** This does not authorize opportunistic normative specification changes. Specs remain read-only implementation authority unless separately requested.

## AMEND-002 — `intrinsic` keyword ratified and `@intrinsic` attribute retired

**Date:** 2026-09-22

**Plan:** `LANG007.C1.P2`

**Trigger:** Explicit user ruling establishing that `intrinsic` is a source keyword and declaration modifier mapped to `ImplementationKind::Intrinsic`, and that `@intrinsic` is retired as a source/builtin attribute.

**Decision:**
- Ratified and recorded in `docs/decisions/accepted/0089-intrinsic-is-a-first-class-implementation-keyword.md` (TDR-0089), amending TDR-0082 and TDR-0088.
- Synchronized accepted decision registry in `docs/decisions/README.md`.
- Removed `BuiltinAttr::Intrinsic` and compiler expander; updated virtual source rendering.
- Preserved internal `NativeIntrinsicId` metadata for compiler/runtime optimization.
- Converged normative specifications (`source-declarations.md`, `type-members.md`, `universe.md`, `type-level-domains.md`, `structural-products.md`, `generic-constraints.md`, `associated-types.md`, `objects/traits.md`, `objects/impl.md`).

**Architectural consequence:** Pure compiler/semantic normalization model with clean implementation-kind separation, zero-VM type-member execution, and single-source-of-truth type theory.

---

# 11. Current implementation state

```text
AST/parser implementation:             IMPLEMENTED
TypePack/LabeledTypePack kinds:        IMPLEMENTED
canonical pack forms:                  IMPLEMENTED
type-member registry/identity:         IMPLEMENTED
generalized projection:                IMPLEMENTED
typed const member arguments:          IMPLEMENTED
Tuple Positionals/Labeled/At:          IMPLEMENTED
Tuple/pack/labeled-pack Map:           IMPLEMENTED
Record/RecordRow Map:                  IMPLEMENTED
inference/materialization integration: IMPLEMENTED
stable metadata:                       IMPLEMENTED
source-index integration:              IMPLEMENTED
Universe source integration:           IMPLEMENTED
runtime negative gate:                 PASS
focused acceptance:                    PASS
release certification:                 DEFERRED
```

Planning/specification completion is not code completion.

---

# 12. Active plan and next action

Current:

```text
active plan: none
latest completed C1 plan: LANG007.C1.P3
next checkpoint: LANG007.C2
blockers: none recorded; two unrelated baseline trait failures deferred
```

Next action:

> `LANG007.C1` is complete and closed. Define and plan follow-on checkpoint
> `LANG007.C2` (variadic generic parameters and pack expansion syntax).

C1 closure state:

```yaml
checkpoint status: COMPLETE
checkpoint completion: IMPLEMENTED
checkpoint verification: FOCUSED_TESTED
P1 status: COMPLETE
P1 completion: IMPLEMENTED
P1 verification: FOCUSED_TESTED
P2 status: COMPLETE
P2 completion: IMPLEMENTED
P2 verification: FOCUSED_TESTED
P3 status: COMPLETE
P3 completion: IMPLEMENTED
P3 verification: FOCUSED_TESTED
```

---

# 13. Checkpoint acceptance rules

`LANG007.C1` may be marked `COMPLETE` only when:

1. the bounded C1 vertical is implemented;
2. P1 and any required C1 corrective plans are complete or explicitly superseded;
3. this checkpoint reflects the actual landed architecture;
4. all P1-required hostile cases have passing evidence;
5. existing associated-type semantics remain verified;
6. residual failures are classified;
7. no active consultation incident remains;
8. focused verification state is truthfully recorded;
9. the next LANG007 action is explicit.

Expected normal completion state for P1/C1 is at most:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

unless broader certification is explicitly run.

`FOCUSED_TESTED` does not imply `RELEASE_COMPLETE`.

---

# 14. Do-not-redesign list

Subsequent C1 plans/agents must not casually reopen:

- `TypePack` versus Tuple distinction;
- `LabeledTypePack` versus `RecordRow` distinction;
- `TypeId` as the canonical carrier of non-row kinded forms;
- `ProperTypeId` as the kind-`Type` boundary;
- one canonical subscript identity for bracket indexing and `At`;
- typed compile-time integer member arguments rather than an `Int` kind;
- use of ordinary `apply_type_form` for Map payload application;
- symbolic projection preservation;
- separate query-local versus canonical identities;
- no-VM type-level normalization;
- existing trait associated-type proof/identity authority.

If live implementation evidence invalidates one of these premises, STOP AND CONSULT and record the resulting amendment here before continuing.

---

# 15. Canonical record location

Expected:

```text
docs/implementation/LANG007/
    PROGRAM.md
    STATUS.md
    LANG007.C1/
        LANG007.C1-CHECKPOINT.md
        LANG007.C1-GUIDANCE.md
        LANG007.C1.P1-patch-grade-implementation-plan.md
```

At P1 completion add:

```text
LANG007.C1.P1-walkthrough.md
LANG007.C1.P1-handoff.md
```

unless an explicit later instruction makes one redundant.

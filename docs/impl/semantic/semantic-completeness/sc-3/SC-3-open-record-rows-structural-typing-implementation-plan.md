# SC-3 — Open Record Rows, Immutable Structural Records, Row Polymorphism, and Row Inference — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish Phalcom's immutable structural Record type domain so open row syntax is preserved, row-polymorphic callables infer and correlate record remainders, structural subtyping is sound and useful, row solver state remains query-local, and canonical rows can be published through snapshots and metadata without leaking solver variables.

**Architecture:** Keep the existing canonical `RecordRowData -> RecordRowId -> TypeData::Record` representation. Strengthen its construction invariants, replace the prototype row unifier with a normalized query-local row solver, add a row-specific inference domain coordinated with (not merged into) ordinary generic inference, and route all callable application through the existing canonical `apply_resolved_callable` funnel. Records remain immutable structural products with width + covariant-depth subtyping; Maps remain mutable dynamic-key collections and do not acquire row semantics.

**Tech Stack:** Rust; `phalcom-semantic`; `phalcom-ast`; `phalcom-type-meta`; existing semantic query budgets/cancellation; existing explanation/diagnostic infrastructure; current integration-test harness under `phalcom-semantic/tests/semantic`.

**Spec basis:**
- `docs/impl/semantic/semantic-completeness/part-5/05-advanced-kinds-constraints-effects-and-proofs-REVISED.md`
- `docs/impl/semantic/semantic-completeness/sc-1/SC-1-type-formation-kinds-generics-technical-spec.md`
- `docs/impl/semantic/semantic-completeness/sc-1/SC-1-type-formation-kinds-generics-implementation-plan.md`
- `docs/impl/semantic/semantic-completeness/sc-2/SC-2-generic-callable-application-receiver-specialization-technical-spec.md`
- `docs/impl/semantic/semantic-completeness/sc-2/SC-2-generic-callable-application-receiver-specialization-implementation-plan.md`

**Repository baseline:** `aureat/phalcom-lang` `main` at commit `abb2b5d80654e2525d68f4ea8ff9d32b810330b3`.

---

# Global constraints

1. `RecordRow` remains a semantic domain/kind distinct from ordinary `Type`.
2. A `RecordRow` parameter is represented durably by `TypeParameterId`, never by `TypeData::Parameter`.
3. `RecordRowVarId` is query-local solver state and must never enter `TypeStore`, snapshots, metadata, reflection, or runtime objects.
4. `TypeData::Record(RecordRowId)` remains the one canonical in-memory type representation for closed and open Records.
5. Records are immutable structural products. Ordinary Record subtyping is width + covariant depth.
6. Delete `RecordAccess::{ReadOnly, WriteOnly, ReadWrite}` from Record semantics. Do not add writable-record capability semantics.
7. `Map<K,V>` remains a mutable dynamic-key collection. A Map's runtime key set is not a `RecordRow`.
8. Open row syntax `#{ field: T, | R }` must never be lowered as a false closed Record.
9. Known fields and an open tail are disjoint. `#{ x: T, | R }` implies an internal `R lacks x` obligation.
10. Repeated uses of the same stable row parameter in one generic instantiation refer to the same inferred row.
11. Row inference must cooperate with ordinary type inference without sharing variable IDs.
12. Expected-result typing may constrain row variables under the same SC-2 policy as ordinary generic variables; it must not fabricate runtime evidence.
13. Underconstrained row variables remain underconstrained. Do not default them to the empty row.
14. Row solver budget exhaustion, cancellation, blocked state, contradiction, and internal failure remain distinct terminal states.
15. Canonical semantic truth must not depend on whether a remainder row happened to be interned earlier.
16. Nominal class layouts do not structurally satisfy Record types merely because field names match.
17. Arbitrary row-kinded nominal generic application and arbitrary row-kinded type-lambda application are outside SC-3 unless a separate multi-domain generic-argument representation is ratified.
18. Generic getter support remains SC-7. Do not broaden getter grammar in this work.
19. Metadata already owns `TypeNode::OpenRecord` / `ScopedTypeNode::OpenRecord`; reuse them instead of inventing a second durable row graph.
20. No task may add a second callable-application engine parallel to `apply_resolved_callable`.

---

# Current repository state

The baseline includes a WIP SC-1 implementation, so several findings from the earlier requirements analysis are now resolved. The remaining SC-3 gaps are narrower and more concrete.

| Area | Current symbol/path | Current state | SC-3 consequence |
|---|---|---|---|
| canonical rows | `phalcom-semantic/src/types/row.rs` | `RecordRowData`, `RecordRowTail::{Closed, Parameter}` already exist | preserve representation |
| row access policy | `types/row.rs::RecordAccess` | read/write capability model still present | delete for immutable Records |
| row store | `types/store.rs::{intern_record_row, record_row, find_record_row, record, record_type}` | open-capable arena exists; validation is permissive | add checked formation boundary |
| row binder safety | `types/store.rs::parameter_form` | explicitly rejects `RecordRow` parameter forms | preserve law |
| source resolver | `types/annotation.rs::{TypeLevelBinding,TypeResolver}` | domain-aware `RecordRow(TypeParameterId)` binding is now live | consume; do not recreate |
| formation outcomes | `types/annotation.rs::TypeFormationOutcome` | explicit ready/dynamic/missing/unresolved/invalid/terminal model is live | use for row formation |
| formation context | `types/annotation.rs::TypeFormationSite` | owner/side-aware `Self` formation is live | preserve |
| generic signature | `types/annotation.rs::resolve_generic_signature` | uses `type_level_binding_for_parameter`; row binder no longer panics | add SC-3 binder-site policy only |
| direct Record annotation | `types/annotation.rs::resolve_type_form` | still matches `tail: _` and publishes closed row | Task 4 correctness blocker |
| scoped Record annotation | `types/annotation.rs::lower_scoped_type_form` | explicitly returns `UnsupportedOpenRecordTail` | Task 10 handoff |
| scoped type lowering | `ScopedBinderStack` / `lower_scoped_type_form` | capture-safe SC-1 lowering is live | extend, do not replace |
| declaration generic scope | `checker/declaration_signature.rs::declaration_type_level_bindings_for_side` | side-aware domain-safe binding map is live | preserve |
| type environment | `types/environment.rs::TypeEnvironment` | `HashMap<TypeParameterId, TypeId>` only | add row binding map |
| substitution | `types/substitution.rs::TypeSubstitution` | Record tail copied unchanged | add checked domain-aware materialization |
| row solver | `types/row_solver.rs::RecordRowSolver` | prototype `Canonical/Var/Extend`; private step limit | replace/harden |
| solver history bug | `row_solver.rs` | subtraction uses `find_record_row` | remove pre-intern dependence |
| inference | `checker/inference.rs::InferenceSession` | `InferVarId -> TypeId`; no decomposable Record term | add Record term + companion row domain |
| call funnel | `checker/call.rs::apply_generic_callable_inner` | owns generic instantiation/constraints/solve/materialization | integrate here |
| Record relation | `types/relation.rs::check_record_row_subtype` | capability argument; ordinary relation passes `ReadOnly` | simplify to immutable relation |
| literal typing | `checker/expression.rs::synthesize_record_literal` | expected parameter is `_expected` | make bidirectional |
| Record projection | `checker/composition.rs::project_record_fields` | only succeeds for closed rows | split known-field vs complete projection |
| Record expansion | AST/parser/checker | `RecordLiteralEntry::Expansion` already live | preserve open Record tails |
| patterns | `checker/pattern.rs::resolve_record_pattern` | already distinguishes known/open/closed fields correctly | primarily lock in with source tests |
| metadata schema | `phalcom-type-meta/src/type_node.rs` | `TypeNode::OpenRecord` exists | reuse |
| metadata export | `semantic/src/metadata/export.rs::export_type_form` | all Records export as closed; `record_rows: false` | branch on tail and enable |
| tests | `tests/semantic/advanced/record_rows.rs` | prototype tests; subtraction pre-interns empty remainder | rewrite/expand |

---

# Target architecture

```text
source <R: RecordRow>
        |
        v
stable TypeParameterId(R)
        |
        +------ canonical signature ------+
        |                                  |
        v                                  |
RecordRowTail::Parameter(R)                |
        |                                  |
        v                                  |
call instantiation                         |
        |                                  |
        +-- Type/arrow binder -> InferVarId
        +-- RecordRow binder -> RecordRowVarId
                                           |
actual TypeData::Record(RecordRowId)        |
        |                                  |
        v                                  |
record constraint decomposition             |
        +-- field type constraints -> InferenceSession
        +-- label/remainder equations -> RecordRow solver
                                           |
        +----------------------------------+
        v
GenericInstantiation
  types: TypeParameterId -> TypeId
  rows:  TypeParameterId -> RecordRowId
        |
        v
checked materialization
        |
        v
canonical TypeData::Record
        |
        +-- snapshot/read model
        +-- TypeNode::{Record, OpenRecord}
```

The row solver owns label/tail algebra. The ordinary inference solver owns type relationships. Neither solver reimplements the other.

---

# Planned file map

## Create

- `phalcom-semantic/src/types/instantiation.rs`
- `phalcom-semantic/src/checker/row_inference.rs`
- `phalcom-semantic/tests/semantic/foundations/record_row_materialization.rs`
- `phalcom-semantic/tests/semantic/foundations/record_row_inference.rs`
- `phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs`
- `phalcom-semantic/tests/semantic/incremental/record_rows.rs`

## Modify

- `phalcom-semantic/src/types/mod.rs`
- `phalcom-semantic/src/types/row.rs`
- `phalcom-semantic/src/types/store.rs`
- `phalcom-semantic/src/types/environment.rs`
- `phalcom-semantic/src/types/substitution.rs`
- `phalcom-semantic/src/types/annotation.rs`
- `phalcom-semantic/src/types/relation.rs`
- `phalcom-semantic/src/types/row_solver.rs`
- `phalcom-semantic/src/types/type_lambda.rs`
- `phalcom-semantic/src/checker/mod.rs`
- `phalcom-semantic/src/checker/inference.rs`
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/composition.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/pattern.rs`
- `phalcom-semantic/src/diagnostic.rs`
- `phalcom-semantic/src/metadata/export.rs`
- `phalcom-semantic/tests/semantic/advanced/record_rows.rs`
- `phalcom-semantic/tests/semantic/advanced/integration_matrix.rs`
- `phalcom-semantic/tests/semantic/foundations/mod.rs`
- `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
- `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`
- `phalcom-semantic/tests/semantic/integration/mod.rs`
- `phalcom-semantic/tests/semantic/integration/metadata.rs`
- `phalcom-semantic/tests/semantic/incremental/mod.rs`

## Do not modify for SC-3 semantics unless a failing conformance test proves necessity

- `phalcom-core` runtime Record representation
- `phalcom-ast` Record type grammar
- `phalcom-ast` Record literal expansion grammar
- Map type semantics to encode key sets
- class/enum generic application representation to carry row-valued arguments
- generic getter grammar

---

# Task dependency graph

```text
Task 0  baseline/prerequisite gate
  |
  +--> Task 1 canonical row integrity
  |      +--> Task 2 domain-aware materialization
  |      +--> Task 3 row solver rewrite
  |
  +--> Task 4 source open-row lowering
             +--> Task 5 immutable Record relations
             +--> Task 6 Record-aware inference representation
                         |
Task 2 -----------------+
Task 3 -----------------+
                         v
                    Task 7 call-funnel integration
                         +--> Task 8 Record literal/expansion
                         +--> Task 9 pattern decomposition
                         +--> Task 10 scoped open Records
                                     |
                                     v
                                Task 11 diagnostics/explanations
                                     |
                                     v
                                Task 12 metadata/fingerprints
                                     |
                                     v
                                Task 13 incrementality
                                     |
                                     v
                                Task 14 certification
```

---

### Task 0: Re-pin the implementation branch and verify the now-landed SC-1 foundation seams

#### Why

The repository moved while this plan was being drafted. The current pinned baseline already contains a substantial WIP SC-1 implementation: domain-aware type-level bindings, explicit type-formation outcomes, `TypeFormationSite`, capture-safe scoped type lowering, alias publication, and side-aware declaration generic environments. SC-3 must consume those landed seams rather than reimplement the older SC-1 plan. Two row-specific handoff gaps still remain: direct Record lowering still discards the tail, and scoped Record lowering explicitly rejects open tails.

#### Architectural background

SC-3 starts from these now-existing SC-1 laws:

```text
TypeLevelBinding::TypeForm(TypeId)
TypeLevelBinding::RecordRow(TypeParameterId)

TypeFormationOutcome::{Ready, Dynamic, Missing, Unresolved, Invalid,
                      Blocked, Cancelled, BudgetExceeded, InternalFailure}

TypeFormationSite
capture-safe ScopedBinderStack / lower_scoped_type_form
```

The prerequisite gate is therefore no longer “add domain-aware lexical binders.” It is “verify those foundations are present and finish only the RecordRow consumer seam.”

The SC-2 call architecture is unchanged on this baseline: `apply_resolved_callable -> apply_generic_callable -> apply_generic_callable_inner` remains the one generic callable-application authority.

#### Current path through the code

At `main@abb2b5d80654e2525d68f4ea8ff9d32b810330b3`:

```text
types/annotation.rs
  TypeLevelBinding                         -> landed
  type_level_binding_for_parameter        -> landed
  TypeResolver::resolve_type_level_binding-> landed
  TypeFormationOutcome                    -> landed
  TypeFormationSite                       -> landed
  resolve_kind_syntax invalid != Type     -> landed
  resolve_generic_signature
      -> type_level_binding_for_parameter -> landed; row binder no longer parameter_form panic
  lower_scoped_type_form                  -> landed capture-safe lowering
  scoped Record with tail                 -> Invalid(UnsupportedOpenRecordTail)
  direct Record                           -> still matches tail: _ and publishes closed row

checker/declaration_signature.rs
  declaration_type_level_bindings_for_side -> landed
  callable generic resolver path            -> domain-aware

checker/call.rs
  apply_resolved_callable
    -> apply_generic_callable_inner
       -> InferenceSession                  -> still type-variable-only; SC-3 integration target
```

#### Exact files

- Inspect and pin:
  - `phalcom-semantic/src/types/annotation.rs`
  - `phalcom-semantic/src/types/parameter.rs`
  - `phalcom-semantic/src/types/store.rs`
  - `phalcom-semantic/src/checker/declaration_signature.rs`
  - `phalcom-semantic/src/checker/call.rs`
  - `phalcom-semantic/src/checker/inference.rs`
- Tests:
  - `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
  - `phalcom-semantic/tests/semantic/advanced/record_rows.rs`

#### Exact symbols

Verify, do not recreate:

- `TypeLevelBinding`
- `type_level_binding_for_parameter`
- `TypeFormationSite`
- `TypeFormationOutcome`
- `TypeFormationInvalid::UnsupportedOpenRecordTail`
- `ScopedBinderStack`
- `lower_scoped_type_form`
- `resolve_generic_signature`
- `GenericBinderSite`
- `declaration_type_level_bindings_for_side`
- `apply_resolved_callable`
- `apply_generic_callable_inner`
- `InferenceSession::instantiate_generic_signature`

#### Exact insert/replace locations

1. At implementation start, record:
   ```bash
   git rev-parse HEAD
   git status --short
   ```
2. Require the branch to contain `TypeLevelBinding` and `TypeFormationOutcome`; if a later SC-1 commit has renamed them, map this plan to the landed equivalents rather than restoring old APIs.
3. Confirm direct Record lowering still contains `TypeAnnotationExpr::Record { fields, tail: _, ... }`; this is Task 4's production seam.
4. Confirm scoped Record lowering still returns `TypeFormationInvalid::UnsupportedOpenRecordTail`; this is Task 10's production seam.
5. Confirm `resolve_generic_signature` uses `type_level_binding_for_parameter`; do not add another row-binder map.
6. Confirm `apply_resolved_callable` remains the call funnel. If SC-2 implementation lands before SC-3 starts, rebase Tasks 6–7 onto its final types while preserving the single-funnel law.
7. Record the implementation SHA in the in-repo copy of this plan before editing production code.

#### Paste-ready code where safe

Add/retain this SC-1/SC-3 handoff probe in `advanced/record_rows.rs` if no equivalent exact test exists:

```rust
#[test]
fn record_row_parameter_resolves_to_row_domain_binding() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(test_decl("Owner"));
    let parameter = store.intern_type_parameter(TypeParameterData::new(
        owner,
        0,
        "R",
        KindId::RECORD_ROW,
    ));

    assert_eq!(
        phalcom_semantic::types::annotation::type_level_binding_for_parameter(
            &mut store,
            parameter,
        ),
        phalcom_semantic::types::annotation::TypeLevelBinding::RecordRow(parameter),
    );
    assert!(!store.contains_parameter_type(parameter));
}
```

Do not call `parameter_form(parameter)` in this positive handoff test.

#### What not to change

- Do not recreate `TypeLevelBinding` or `TypeFormationOutcome` under SC-3-local names.
- Do not restore `current_declaration`-based `Self` formation; `TypeFormationSite` now owns that context.
- Do not recreate a scoped type lowerer; SC-1's `lower_scoped_type_form` exists.
- Do not copy `apply_generic_callable_inner` into a row-specific call engine.
- Do not merge `RecordRowVarId` into `InferVarId`.
- Do not implement generic getters.

#### Tests to add first

Before Task 1 production edits, run the existing SC-1 type-formation/generic tests and add the handoff probe above only if equivalent coverage is absent.

Also add one **red** SC-3 boundary test proving the remaining gap:

```text
callable generic R: RecordRow
parameter type #{ name: String, | R }
```

must not currently publish a falsely closed row. On this baseline it should fail in the predicted Record-tail lowering seam; Task 4 turns it green.

#### Tests to add afterward

No additional feature behavior belongs to Task 0. The task ends when the branch state is classified and later tasks are rebased onto the landed SC-1/SC-2 APIs.

#### Expected compiler errors

None should be introduced by the baseline probe.

If the repository advances again and symbols have moved, `E0432`/`E0599` from plan snippets are a rebase signal. Resolve them by following the newest canonical interfaces; do not restore older compatibility APIs merely to match this document.

#### Rust explanations

`TypeLevelBinding` is the crucial Rust-level sum type separating proper/type-constructor forms from row-domain binders. `TypeParameterId` is only stable binder identity; it is not itself a proper type. The new SC-1 `TypeFormationOutcome` likewise prevents invalid/blocked formation from being collapsed into ordinary value-type `Unknown`.

#### Verification commands

```bash
git rev-parse HEAD
git status --short

rg -n 'pub enum TypeLevelBinding|pub enum TypeFormationOutcome|pub struct TypeFormationSite' \
  phalcom-semantic/src/types/annotation.rs
rg -n 'UnsupportedOpenRecordTail|tail:\s*_' \
  phalcom-semantic/src/types/annotation.rs
rg -n 'type_level_binding_for_parameter' \
  phalcom-semantic/src/types/annotation.rs \
  phalcom-semantic/src/checker/declaration_signature.rs
rg -n 'fn apply_generic_callable_inner|fn apply_resolved_callable' \
  phalcom-semantic/src/checker/call.rs

cargo test -p phalcom-semantic type_annotations
cargo test -p phalcom-semantic record_row_parameter_resolves_to_row_domain_binding
```

#### Completion checklist

- [ ] Implementation HEAD/worktree recorded.
- [ ] Landed `TypeLevelBinding` seam verified.
- [ ] Landed explicit formation outcome seam verified.
- [ ] Landed capture-safe scoped lowerer verified.
- [ ] Generic row binder no longer goes through `parameter_form` during signature publication.
- [ ] Direct Record-tail discard identified as Task 4 seam.
- [ ] Scoped open-tail rejection identified as Task 10 seam.
- [ ] Canonical SC-2 call funnel identified/re-pinned.
- [ ] No duplicate SC-1 semantic infrastructure introduced.

---

### Task 1: Establish the checked canonical Record-row formation boundary

#### Why

`TypeStore::intern_record_row` accepts arbitrary `RecordRowData`, and `record_type` trusts the resulting ID. Once source open rows and solver-zonked rows become reachable, malformed tails or non-proper field types must not enter the canonical store. Ordinary semantic errors must not become panics.

#### Architectural background

```text
input fields/tail
  -> validate field kinds and tail kind
  -> sort field names
  -> reject duplicates
  -> intern RecordRowData
  -> intern TypeData::Record(row_id)
```

`RecordRowData` remains pure structural data. `TypeStore` owns validation needing kind/parameter metadata.

#### Current path through the code

Current closed construction sorts and `assert_ne!`s duplicates. Direct `intern_record_row` plus `record_type` can bypass kind validation.

#### Exact files

- `phalcom-semantic/src/types/row.rs`
- `phalcom-semantic/src/types/store.rs`
- `phalcom-semantic/src/types/mod.rs`
- `phalcom-semantic/tests/semantic/advanced/record_rows.rs`

#### Exact symbols

- `RecordRowData::{new_closed,new_with_tail}`
- `DuplicateFieldError`
- `RecordAccess`
- `TypeStore::{intern_record_row,record,record_type,type_parameter,is_proper_type}`
- add `TypeStore::try_type_parameter`
- add `TypeStore::record_row_checked`
- add `TypeStore::record_row_type_checked`

#### Exact insert/replace locations

1. Insert `RecordRowFormationError` after `DuplicateFieldError` in `types/row.rs`.
2. Remove `RecordAccess` from `types/row.rs`.
3. Insert `TypeStore::try_type_parameter` next to current `type_parameter`; this is the checked arena lookup used by row formation.
4. Insert `TypeStore::record_row_checked` immediately before the current raw row-interner methods. It validates and canonicalizes a row, returning `RecordRowId`.
5. Insert `TypeStore::record_row_type_checked` immediately before current `pub fn record`; it delegates to `record_row_checked` and wraps the row in `TypeData::Record`.
6. Replace `record` body with a trusted closed-row wrapper over the checked type method.
7. After caller migration, reduce `intern_record_row` and `record_type` visibility to `pub(crate)`.
8. Remove `RecordAccess` export from `types/mod.rs`; export `RecordRowFormationError`.

#### Paste-ready code where safe

```rust
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RecordRowFormationError {
    #[error("duplicate record field: {0}")]
    DuplicateField(Box<str>),

    #[error("record field `{field}` is not a proper type")]
    FieldNotProperType {
        field: Box<str>,
        ty: TypeId,
    },

    #[error("record row tail parameter is missing")]
    TailParameterMissing(TypeParameterId),

    #[error("record row tail parameter must have kind RecordRow")]
    TailParameterWrongKind {
        parameter: TypeParameterId,
        actual: KindId,
    },
}
```

Checked store lookup and constructor shape:

```rust
pub fn try_type_parameter(&self, id: TypeParameterId) -> Option<&TypeParameterData> {
    self.type_parameters.get(id.index())
}

pub fn record_row_checked(
    &mut self,
    fields: Vec<RecordRowField>,
    tail: RecordRowTail,
) -> Result<RecordRowId, RecordRowFormationError> {
    for field in &fields {
        if !self.is_proper_type(field.ty) {
            return Err(RecordRowFormationError::FieldNotProperType {
                field: field.name.clone(),
                ty: field.ty,
            });
        }
    }

    if let RecordRowTail::Parameter(parameter) = tail {
        let Some(data) = self.try_type_parameter(parameter) else {
            return Err(RecordRowFormationError::TailParameterMissing(parameter));
        };
        if data.kind != KindId::RECORD_ROW {
            return Err(RecordRowFormationError::TailParameterWrongKind {
                parameter,
                actual: data.kind,
            });
        }
    }

    let row = RecordRowData::new_with_tail(fields, tail)
        .map_err(|DuplicateFieldError(field)| RecordRowFormationError::DuplicateField(field))?;
    Ok(self.intern_record_row(row))
}

pub fn record_row_type_checked(
    &mut self,
    fields: Vec<RecordRowField>,
    tail: RecordRowTail,
) -> Result<TypeId, RecordRowFormationError> {
    let row = self.record_row_checked(fields, tail)?;
    Ok(self.record_type(row))
}
```

Keep trusted closed compatibility:

```rust
pub fn record(&mut self, fields: Box<[RecordRowField]>) -> TypeId {
    self.record_row_type_checked(fields.into_vec(), RecordRowTail::Closed)
        .expect("internal closed Record construction must satisfy canonical row invariants")
}
```

#### What not to change

- Do not split closed/open Records into separate `TypeData` variants.
- Do not add mutability to `RecordRowField`.
- Do not use last-write-wins duplicates.
- Do not expose solver variables in `RecordRowTail`.

#### Tests to add first

Add:
- checked wrong-kind tail rejection;
- checked duplicate rejection;
- closed/open same-prefix distinctness;
- field permutation canonicalization.

Representative wrong-kind test:

```rust
#[test]
fn checked_row_rejects_non_row_tail_parameter() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(test_decl("Owner"));
    let t = store.intern_type_parameter(TypeParameterData::new(
        owner,
        0,
        "T",
        KindId::TYPE,
    ));
    let int_ty = store.nominal(test_decl("Int"));

    let result = store.record_row_type_checked(
        vec![RecordRowField { name: "value".into(), ty: int_ty }],
        RecordRowTail::Parameter(t),
    );

    assert!(matches!(
        result,
        Err(RecordRowFormationError::TailParameterWrongKind {
            parameter,
            actual: KindId::TYPE,
        }) if parameter == t
    ));
}
```

#### Tests to add afterward

- same open row + same stable tail canonicalizes identically;
- different stable tail parameter produces a distinct canonical type;
- direct raw interning is no longer needed by integration tests.

#### Expected compiler errors

Removing `RecordAccess` intentionally produces unresolved-type/import errors in `relation.rs`, `types/mod.rs`, and tests. Do not reintroduce it; Task 5 completes migration. Reducing raw interner visibility may produce privacy errors in integration tests; migrate them to checked APIs.

#### Rust explanations

`Result` is appropriate at source/import/solver publication boundaries. Raw interners can remain crate-private so downstream code can rely on invariants. Sorting before interning makes semantic equality source-order independent.

#### Verification commands

```bash
cargo test -p phalcom-semantic record_rows
cargo test -p phalcom-semantic checked_row_rejects_non_row_tail_parameter
rg -n 'RecordAccess' phalcom-semantic/src/types/row.rs
```

#### Completion checklist

- [ ] Checked canonical row-ID constructor exists.
- [ ] Checked canonical Record-type wrapper exists.
- [ ] Field proper-type validation exists.
- [ ] Tail kind validation exists.
- [ ] Duplicate rejection is non-panicking at checked boundary.
- [ ] `RecordAccess` removed from `row.rs`.
- [ ] Closed/open share `TypeData::Record`.
- [ ] Canonical permutation behavior preserved.

---

### Task 2: Add domain-aware generic instantiation and checked Record-tail materialization

#### Why

Current `TypeEnvironment` and `TypeSubstitution` bind only `TypeParameterId -> TypeId`. They cannot represent `R -> RecordRowId`. Current Record materialization updates known field types but copies the tail unchanged.

#### Architectural background

Keep separate maps:

```text
TypeParameterId -> TypeId
TypeParameterId -> RecordRowId
```

A `GenericInstantiation` coordinates both. Materialization is fallible because row substitution can create duplicate labels, recurse, or remain unresolved where a complete solution is required.

#### Current path through the code

`TypeView::materialize -> materialize_view -> TypeData::Record` materializes known fields and reuses original tail. `TypeSubstitution::apply` does the same.

#### Exact files

- Create `phalcom-semantic/src/types/instantiation.rs`
- Create `phalcom-semantic/tests/semantic/foundations/record_row_materialization.rs`
- Modify `types/mod.rs`, `types/environment.rs`, `types/substitution.rs`, `tests/semantic/foundations/mod.rs`

#### Exact symbols

- Create `GenericInstantiation`
- Create `TypeMaterializationError`
- Create `RowMaterializationMode`
- Create `materialize_type`
- Modify `TypeEnvironment`, `TypeView::materialize`, private `materialize_view`, `TypeSubstitution::apply`

#### Exact insert/replace locations

1. Add `pub mod instantiation;` to `types/mod.rs` next to `environment`.
2. Export `GenericInstantiation`, `RowMaterializationMode`, `TypeMaterializationError`.
3. Replace `TypeEnvironment.bindings` with `type_bindings` and `row_bindings`.
4. Add `bind_row/get_row` while keeping `bind_param/get_param` for ordinary types.
5. Add `TypeView::materialize_checked` and migrate semantic publication callers to it.
6. Leave `TypeSubstitution` type-only; document that stable row-tail substitution uses `GenericInstantiation`.

#### Paste-ready code where safe

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GenericInstantiation {
    type_bindings: HashMap<TypeParameterId, TypeId>,
    row_bindings: HashMap<TypeParameterId, RecordRowId>,
}

impl GenericInstantiation {
    pub fn bind_type(&mut self, parameter: TypeParameterId, ty: TypeId) {
        self.type_bindings.insert(parameter, ty);
    }

    pub fn bind_row(&mut self, parameter: TypeParameterId, row: RecordRowId) {
        self.row_bindings.insert(parameter, row);
    }

    pub fn type_binding(&self, parameter: TypeParameterId) -> Option<TypeId> {
        self.type_bindings.get(&parameter).copied()
    }

    pub fn row_binding(&self, parameter: TypeParameterId) -> Option<RecordRowId> {
        self.row_bindings.get(&parameter).copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RowMaterializationMode {
    PreserveUnboundStableTail,
    RequireSolvedTail,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TypeMaterializationError {
    #[error("record row parameter has no solved row binding")]
    UnresolvedRowParameter(TypeParameterId),
    #[error("recursive record row substitution")]
    RecursiveRowSubstitution(TypeParameterId),
    #[error(transparent)]
    RecordRow(#[from] RecordRowFormationError),
    #[error("type application failed while materializing a generic type")]
    TypeApplication,
}
```

Materializing a Record with tail `Parameter(p)`:
- preserve `p` in declaration-view mode if no binding exists;
- require a binding in call-finalization mode;
- if bound, recursively materialize the bound row and merge via `record_row_type_checked`;
- track visited stable row parameters with `HashSet<TypeParameterId>`.

#### What not to change

- Do not make `TypeSubstitution.bind` accept rows.
- Do not encode a row binding as synthetic `TypeId`.
- Do not add `TypeData::Row`.
- Do not use `unwrap_or(original_type)` on failed row materialization.

#### Tests to add first

Create and register `record_row_materialization.rs` with:
- `row_binding_is_substituted_into_record_return`;
- duplicate merge rejection;
- preserve unbound stable tail in declaration view;
- require solved tail in call finalization;
- recursive stable row substitution rejection.

#### Tests to add afterward

- ordinary type substitution inside row fields;
- `Self` + row materialization;
- nested Record in tuple/callable/union;
- bound row that remains open preserves stable tail.

#### Expected compiler errors

Changing public `TypeEnvironment.bindings` produces `E0609` at direct field accesses. Migrate all to `type_bindings`/methods. Making checked materialization return `Result` produces `E0308`; explicitly propagate or use invariant `expect`, never fallback to original type.

#### Rust explanations

Separate maps give stronger API separation than a single untyped map. Recursive materialization needs a visited set. Clone/copy row fields/tails before recursively mutating `TypeStore` to satisfy borrow rules.

#### Verification commands

```bash
cargo test -p phalcom-semantic record_row_materialization
cargo test -p phalcom-semantic substitution
rg -n '\.bindings' phalcom-semantic/src
rg -n 'unwrap_or\(ty\)' phalcom-semantic/src/types/environment.rs phalcom-semantic/src/types/instantiation.rs
```

#### Completion checklist

- [ ] Separate type/row instantiation maps exist.
- [ ] `TypeEnvironment` can bind/query rows.
- [ ] Checked materialization exists.
- [ ] Row tails are substituted.
- [ ] Duplicate-after-substitution rejected.
- [ ] Recursive row substitution rejected.
- [ ] No row binding encoded as `TypeId`.

---

### Task 3: Replace the prototype row solver with normalized, budgeted, history-independent row algebra

#### Why

The current solver's subtraction depends on `TypeStore::find_record_row`, its test pre-interns the empty remainder, it owns a private step limit, and cancellation is not threaded. It is not ready for call inference.

#### Architectural background

Normalize every solver row as sorted known fields plus one tail:

```text
tail = Closed | stable Parameter | solver Var
```

Do not intern speculative remainders. Canonicalize only after successful solving/zonking.

#### Current path through the code

`RecordRowTerm::{Canonical,Var,Extend}` -> `unify` -> canonical/extension subtraction -> build remainder -> `store.find_record_row(remainder)`.

#### Exact files

- Replace/harden `phalcom-semantic/src/types/row_solver.rs`
- Modify `types/mod.rs`
- Rewrite/expand `tests/semantic/advanced/record_rows.rs`

#### Exact symbols

- `RecordRowTerm`
- `RecordRowLacks`
- `RecordRowSolution`
- `RecordRowFailure`
- `RecordRowBlockedReason`
- `RecordRowSolveResult`
- `RecordRowSolver::{new,fresh_var,add_lacks,occurs,normalize_term,unify,solve}`

#### Exact insert/replace locations

1. Replace `RecordRowTerm` enum with normalized struct and tail enum.
2. Delete solver-owned `RowBudgetReport`; use `types::outcome::BudgetReport`.
3. Replace `RecordRowSolver::new(step_limit)` with `new()`.
4. Thread `&mut QueryBudget` and `&CancellationToken` through `solve`/rewrite operations.
5. Remove every `find_record_row` from solver exploration.
6. Make `lacks` representative-aware and validate immediately when added to an already-solved variable.
7. Add explicit `Underconstrained` if required variables remain unresolved.

#### Paste-ready code where safe

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordRowTerm {
    pub fields: Box<[RecordRowField]>,
    pub tail: RecordRowTermTail,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RecordRowTermTail {
    Closed,
    Parameter(TypeParameterId),
    Var(RecordRowVarId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordRowUnderconstrained {
    pub variables: Box<[RecordRowVarId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowSolveResult {
    Solved(RecordRowSolution),
    Underconstrained(RecordRowUnderconstrained),
    Rejected(RecordRowFailure),
    Blocked(RecordRowBlockedReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(IncidentId),
}
```

Require `RecordRowSolution` to publish a normalized substitution for every solved allocated row variable, not only union/find representatives. Add:

```rust
impl RecordRowSolution {
    pub fn term_for(&self, variable: RecordRowVarId) -> Option<&RecordRowTerm> {
        self.substitutions.get(&variable)
    }
}
```

Add an explicit publication error and canonical-zonking entrypoint:

```rust
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RecordRowZonkError {
    #[error("record row variable is unsolved")]
    Unsolved(RecordRowVarId),
    #[error("recursive record row substitution during zonking")]
    Recursive(RecordRowVarId),
    #[error(transparent)]
    Formation(#[from] RecordRowFormationError),
}

impl RecordRowSolution {
    pub fn zonk_variable_to_canonical(
        &self,
        variable: RecordRowVarId,
        store: &mut TypeStore,
    ) -> Result<RecordRowId, RecordRowZonkError> {
        let term = self
            .term_for(variable)
            .ok_or(RecordRowZonkError::Unsolved(variable))?;
        self.zonk_term_to_canonical(term, store, &mut HashSet::new())
    }
}
```

Implement private `zonk_term_to_canonical` by recursively following only `RecordRowTermTail::Var` through the solution map, preserving `Closed` and stable `Parameter` tails, and finally calling Task 1's `TypeStore::record_row_checked`. It must not call raw `intern_record_row` directly.

This keeps call-site projection independent of the solver's private representative graph.

Use the existing `QueryBudget` consume/check API. Do not maintain a second independent `step_count` for semantic policy.

#### What not to change

- Do not intern speculative remainders.
- Do not call full subtype recursion from the row solver.
- Do not make stable row parameters solver variables.
- Do not map budget exhaustion to `Rejected` or `KindMismatch`.
- Do not default unsolved tails to `Closed`.

#### Tests to add first

Before production edits:
- remove empty-row pre-interning from `test_row_subtraction_solves`;
- add `remainder_solution_does_not_depend_on_store_history`;
- lacks survives variable alias;
- lacks added after solution checks immediately;
- direct/indirect occurs checks;
- cancellation;
- budget exhaustion;
- underconstraint;
- rigid stable tail equality/mismatch.

#### Tests to add afterward

- deterministic solution independent of equation insertion order;
- two-sided subtraction;
- rejected/blocked solving does not increase canonical row count;
- successful zonking interns only final canonical results.

#### Expected compiler errors

Replacing enum constructors causes `E0599`/`E0223` in old tests. Rewrite tests first. Changing constructor/solve signatures produces `E0061` until all callers pass shared budget/cancellation.

#### Rust explanations

Normalized structs simplify field partitioning and borrow checking. Solver exploration should be referentially transparent with respect to canonical interning. Cancellation and budget are distinct terminal states.

#### Verification commands

```bash
cargo test -p phalcom-semantic record_rows
cargo test -p phalcom-semantic remainder_solution_does_not_depend_on_store_history
rg -n 'find_record_row' phalcom-semantic/src/types/row_solver.rs
rg -n 'step_limit|RowBudgetReport' phalcom-semantic/src/types/row_solver.rs
```

#### Completion checklist

- [ ] Normalized solver terms.
- [ ] Stable parameter/closed/solver-var tails distinct.
- [ ] No speculative interning.
- [ ] No store-history dependence.
- [ ] Lacks follows aliases/substitutions.
- [ ] Direct/indirect occurs checks.
- [ ] Shared budget/cancellation used.
- [ ] Underconstraint explicit.
- [ ] Solved rows zonk through checked canonical construction.
- [ ] Zonk failure does not partially publish a row.
- [ ] Terminal outcomes preserved.

---

### Task 4: Finish the SC-1-to-SC-3 source boundary: lower direct open Records and enforce SC-3 row-binder-site legality

#### Why

The new baseline has already solved the dangerous `RecordRow -> TypeData::Parameter` problem: `resolve_generic_signature` publishes `TypeLevelBinding::RecordRow` safely. What remains is specifically SC-3-owned. Direct Record annotation lowering still ignores `tail`, while the generic grammar currently permits `RecordRow` binders at nominal declaration, alias, and type-lambda sites even though Phalcom has no multi-domain applied-type argument representation with which to instantiate those binders. SC-3 should enable callable row-tail polymorphism without claiming those broader applications work.

#### Architectural background

Use the existing SC-1 formation model:

```text
resolver lookup
  -> TypeLevelBinding::RecordRow(TypeParameterId)
  -> legal only in Record tail slot

TypeAnnotationExpr::Record
  -> known fields : proper TypeId
  -> tail         : Closed | stable row parameter
  -> checked canonical Record construction
```

Binder-site policy for this SC-3 delivery:

| Source binder site | `RecordRow` binder |
|---|---:|
| callable method/function generic | allowed |
| nominal class/enum generic | rejected |
| transparent type-alias generic | rejected |
| source type-lambda parameter | rejected |
| future generic getter | deferred to SC-7 |

The rejected sites require a future multi-domain generic argument model (`Type` arguments and row arguments) across `TypeData::Applied`, type-lambda beta reduction, metadata application nodes, and reflection. SC-3 does not sneak that model in.

#### Current path through the code

At the new pinned baseline:

```text
resolve_generic_signature
  -> resolve kind as TypeFormationOutcome
  -> intern TypeParameterData
  -> type_level_binding_for_parameter
       RecordRow -> TypeLevelBinding::RecordRow   // already correct

resolve_type_form
  -> Reference(R)
       RecordRow binding in ordinary type position -> explicit ExpectedProperType invalidity
  -> Record { fields, tail: _ }
       fields lowered correctly
       tail discarded
       store.record(closed fields)

lower_scoped_type_form
  -> Record { fields, tail }
       tail.is_some() -> Invalid(UnsupportedOpenRecordTail)

lower_scoped_type_lambda
  -> accepts any kind returned by resolve_kind_syntax, including RecordRow
```

#### Exact files

- Modify:
  - `phalcom-semantic/src/types/annotation.rs`
  - `phalcom-semantic/src/types/row.rs` imports/use only as needed
  - `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
  - `phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs` after Task 7 creates it
- Inspect only:
  - `phalcom-semantic/src/checker/declaration_signature.rs`
  - `phalcom-semantic/src/session.rs` nominal/alias generic publication
  - `phalcom-semantic/src/types/store.rs::parameter_form`

#### Exact symbols

Existing symbols to consume:

- `TypeLevelBinding`
- `type_level_binding_for_parameter`
- `TypeFormationOutcome`
- `TypeFormationInvalid`
- `TypeFormationUnresolved`
- `TypeFormationSite`
- `GenericBinderSite`
- `resolve_generic_signature`
- `resolve_type_form`
- `lower_scoped_type_lambda`
- `TypeAnnotationExpr::Record`
- `TypeResolver::resolve_type_level_binding`
- `TypeStore::record_row_type_checked` from Task 1

Add:

- `TypeFormationInvalid::RecordRowTailKindMismatch { actual: KindId }`
- `TypeFormationInvalid::UnsupportedRecordRowBinderSite`

#### Exact insert/replace locations

1. In `TypeFormationInvalid`, add the two row-specific formation variants next to `UnsupportedOpenRecordTail`.
2. In `resolve_type_form`, replace the direct `TypeAnnotationExpr::Record { fields, tail: _, ... }` arm's final construction path:
   - keep the current field lowering and proper-type checks;
   - resolve the AST tail after fields;
   - `None` -> semantic `RecordRowTail::Closed`;
   - `Some(tail)` -> `resolver.resolve_type_level_binding(&tail.name)`;
   - `TypeLevelBinding::RecordRow(parameter)` -> semantic `RecordRowTail::Parameter(parameter)`;
   - `TypeLevelBinding::TypeForm(form)` -> emit existing `KindExpectedType` diagnostic for now and return `RecordRowTailKindMismatch { actual: store.kind_of(form) }`;
   - no binding -> emit unresolved-tail diagnostic using existing `AnnotationUnresolved` until Task 11 specializes the code, return `TypeFormationOutcome::Unresolved(TypeFormationUnresolved::Name(...))`.
3. Replace `store.record(...)` in that arm with Task 1's `record_row_type_checked(fields, semantic_tail)` and map formation errors to structured type-formation invalidity/diagnostics.
4. In `resolve_generic_signature`, immediately after resolving each parameter kind and before interning `TypeParameterData`, reject `KindId::RECORD_ROW` unless `binder_site == GenericBinderSite::Callable`.
5. In `lower_scoped_type_lambda`, after collecting each parameter kind but before pushing the binder layer, reject `KindId::RECORD_ROW` with `UnsupportedRecordRowBinderSite`. This makes the current lack of row-valued beta arguments explicit rather than publishing an unusable public constructor.
6. Do **not** remove `TypeFormationInvalid::UnsupportedOpenRecordTail` yet. Task 10 removes the scoped open-Record restriction after the scoped representation can preserve a free stable tail.
7. Verify callable signature construction already builds a `ScopedTypeResolver` from the callable generic signature; do not add another resolver layer.

#### Paste-ready code where safe

Direct tail lowering inside the Record arm:

```rust
let semantic_tail = match tail {
    None => crate::types::row::RecordRowTail::Closed,
    Some(tail) => match resolver.resolve_type_level_binding(&tail.name) {
        Some(TypeLevelBinding::RecordRow(parameter)) => {
            crate::types::row::RecordRowTail::Parameter(parameter)
        }
        Some(TypeLevelBinding::TypeForm(form)) => {
            let actual = store.kind_of(form);
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::KindExpectedType,
                format!(
                    "record tail `{}` must have kind RecordRow, got {}",
                    tail.name,
                    store.format_kind(actual),
                ),
                tail.range,
            ));
            return TypeFormResolution::Invalid(
                TypeFormationInvalid::RecordRowTailKindMismatch { actual },
            );
        }
        None => {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("unresolved record row tail `{}`", tail.name),
                tail.range,
            ));
            return TypeFormResolution::Unresolved(
                TypeFormationUnresolved::Name(tail.name.clone().into()),
            );
        }
    },
};
```

Binder-site guard inside `resolve_generic_signature`:

```rust
if kind == KindId::RECORD_ROW && binder_site != GenericBinderSite::Callable {
    diagnostics.push(SemanticDiagnostic::error_in(
        current_module.clone(),
        DiagnosticCode::AnnotationUnsupported,
        "RecordRow generic parameters are currently supported only on callables",
        p.range,
    ));
    return TypeFormationOutcome::Invalid(
        TypeFormationInvalid::UnsupportedRecordRowBinderSite,
    );
}
```

Use the current AST's `RecordRowTail { name, range }` fields; do not parse the tail text again.

#### What not to change

- Do not recreate `TypeLevelBinding`.
- Do not downgrade back to the old `TypeFormResolution::{Known,Dynamic,Unknown}` model.
- Do not reintroduce declaration fallback `store.nominal_type(decl)`; SC-1 removed it.
- Do not silently close a failed tail.
- Do not permit nominal/alias/type-lambda row binders merely because their declaration can be interned.
- Do not generalize `TypeData::Applied` in this task.
- Do not remove the scoped `UnsupportedOpenRecordTail` guard until Task 10 can preserve the tail.

#### Tests to add first

In `foundations/type_annotations.rs`, add failing tests for:

1. `callable_record_row_binder_is_domain_safe` — preserve existing SC-1 success;
2. `open_record_annotation_preserves_tail` — currently fails because direct tail is discarded;
3. `record_tail_rejects_type_kind_binding`;
4. `record_tail_rejects_unresolved_binding`;
5. `record_row_binder_in_field_type_is_rejected` — preserve existing SC-1 behavior;
6. `nominal_record_row_binder_is_rejected_until_multidomain_application`;
7. `type_alias_record_row_binder_is_rejected_until_multidomain_application`;
8. `type_lambda_record_row_binder_is_rejected_until_multidomain_application`.

The direct open-row test must inspect the canonical semantic `RecordRowTail::Parameter` identity, not just the absence of diagnostics.

#### Tests to add afterward

- `<T, R: RecordRow>` callable signature preserves both stable parameter IDs/kinds;
- direct open Record field permutation canonicalizes identically;
- wrong tail kind and unresolved tail preserve source range;
- no `TypeData::Parameter` exists for the row binder (`store.contains_parameter_type(r) == false`);
- a class/alias/type-lambda row-binder rejection cannot leave a partially published usable generic signature/form.

#### Expected compiler errors

No trait migration is required on this baseline; `TypeResolver::resolve_type_level_binding` already exists.

Adding `TypeFormationInvalid` variants can trigger `E0004` in diagnostic/projection matches. Update all current formation-outcome projections explicitly.

Replacing `store.record` with a checked `Result` constructor may trigger `E0308`; map each `RecordRowFormationError` deliberately rather than falling back to a closed Record.

#### Rust explanations

The important Rust distinction is now already encoded: `TypeLevelBinding` carries either `TypeId` or `TypeParameterId`. SC-3 consumes that sum type at the Record tail slot. The binder-site guard is semantic feature gating, not a kind-system limitation: `RecordRow` is a valid kind, but some source declarations cannot yet be instantiated because the applied-type argument representation is type-only.

#### Verification commands

```bash
cargo test -p phalcom-semantic type_annotations
cargo test -p phalcom-semantic kinds
cargo test -p phalcom-semantic generics_core

rg -n 'TypeAnnotationExpr::Record\s*\{[^}]*tail:\s*_' \
  phalcom-semantic/src/types/annotation.rs
rg -n 'UnsupportedOpenRecordTail' phalcom-semantic/src/types/annotation.rs
rg -n 'type_level_binding_for_parameter|resolve_type_level_binding' \
  phalcom-semantic/src/types/annotation.rs \
  phalcom-semantic/src/checker/declaration_signature.rs
```

#### Completion checklist

- [ ] Direct open Record tail is preserved as `RecordRowTail::Parameter`.
- [ ] Wrong-kind tail returns structured invalidity.
- [ ] Unresolved tail returns structured unresolved formation.
- [ ] Callable row binder remains domain-safe.
- [ ] Nominal declaration row binder is explicitly rejected for SC-3 scope.
- [ ] Type-alias row binder is explicitly rejected for SC-3 scope.
- [ ] Type-lambda row binder is explicitly rejected for SC-3 scope.
- [ ] No row binder becomes `TypeData::Parameter`.
- [ ] No false closed Record is published.
- [ ] Scoped open Record remains intentionally deferred to Task 10, not silently dropped.


---

### Task 5: Simplify structural Record subtyping to immutable width + covariant depth

#### Why

The current relation helper carries `RecordAccess` and contains read/write branches inherited from an older mutable-structural-record design. Phalcom Records are immutable products. Keeping mutation capability in their subtype relation adds dead complexity and risks future callers selecting irrelevant or unsound branches.

#### Architectural background

For immutable Records:

```text
S <: T
iff
for every field required by T:
    S definitely has that field
    S.field <: T.field
```

Extra fields in `S` are allowed. An open source tail does not prevent proving a closed known-prefix requirement. An open target tail is a rigid requirement unless generic instantiation has explicitly turned its stable parameter into a solver variable.

#### Current path through the code

`check_subtype_impl` currently matches two `TypeData::Record` values and calls:

```rust
check_record_row_subtype(
    ...,
    RecordAccess::ReadOnly,
    ...,
)
```

The read-only branch first blocks when open tails differ, even when all target known fields are already established by the source prefix.

#### Exact files

- `phalcom-semantic/src/types/relation.rs`
- `phalcom-semantic/src/types/outcome.rs` if richer row failures are introduced
- `phalcom-semantic/src/types/mod.rs`
- `phalcom-semantic/tests/semantic/advanced/record_rows.rs`
- `phalcom-semantic/tests/semantic/advanced/integration_matrix.rs`

#### Exact symbols

- `check_record_row_subtype`
- `check_subtype_impl` Record/Record match arm
- `RelationFailure`
- `RefutationReason`
- all `RecordAccess` imports/usages

#### Exact insert/replace locations

1. Remove `access: RecordAccess` from `check_record_row_subtype`.
2. Delete the `match access` body and keep one immutable required-field relation.
3. Replace the Record/Record arm in `check_subtype_impl` so it passes no capability.
4. Check all required target fields first.
5. Only after known-prefix checks, evaluate whether a target open tail imposes an additional rigid-tail condition.
6. A target closed row imposes no exact-shape condition; width subtyping remains allowed.
7. A source open tail does not block a target closed known-prefix record.
8. Do not solve stable row parameters inside canonical subtype checking. Generic call inference owns instantiation.
9. If adding richer `RelationFailure` variants, update `relation_to_assignability` exhaustively in the same patch.

#### Paste-ready code where safe

Core relation shape:

```rust
pub fn check_record_row_subtype(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub_row_id: RecordRowId,
    sup_row_id: RecordRowId,
    sub_ty: TypeId,
    sup_ty: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
) -> RelationOutcome<()> {
    let sub_row = store.record_row(sub_row_id).clone();
    let sup_row = store.record_row(sup_row_id).clone();

    for required in sup_row.fields.iter() {
        let Some(actual) = sub_row.find_field(&required.name) else {
            return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                actual: sub_ty,
                expected: sup_ty,
            });
        };

        match check_subtype_impl(
            store,
            hierarchy,
            actual,
            required.ty,
            budget,
            cancellation,
            visited,
        ) {
            RelationOutcome::Proven { .. } => {}
            terminal => return terminal,
        }
    }

    match sup_row.tail {
        RecordRowTail::Closed => RelationOutcome::proven(()),
        RecordRowTail::Parameter(required_tail) => match sub_row.tail {
            RecordRowTail::Parameter(actual_tail) if actual_tail == required_tail => {
                RelationOutcome::proven(())
            }
            _ => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),
        },
    }
}
```

If Task 11 introduces a dedicated row-tail `BlockReason`, replace the final generic `RecursiveFixpoint` with that specific reason. Do not keep `RecursiveFixpoint` merely because it already exists.

#### What not to change

- Do not solve row parameters during canonical subtype queries.
- Do not add structural subtyping from nominal class instances to Records.
- Do not add Map-to-Record subtyping.
- Do not require exact field sets for immutable Records.
- Do not use invariant field types.
- Do not treat target open tail as an existential wildcard outside generic instantiation.

#### Tests to add first

Add failing tests before relation changes:

1. `immutable_record_width_subtyping`
2. `immutable_record_covariant_depth_subtyping`
3. width + depth combined
4. `open_source_satisfies_closed_known_prefix`
5. open source missing a required known field is not proven
6. same rigid tail succeeds
7. different rigid tails do not silently succeed
8. nominal class with same field name does not structurally subtype Record

Representative law:

```rust
#[test]
fn open_source_satisfies_closed_known_prefix() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let string_ty = store.nominal(test_decl("String"));
    let owner = TypeParameterOwner::Declaration(test_decl("RowOwner"));
    let r = store.intern_type_parameter(TypeParameterData::new(
        owner,
        0,
        "R",
        KindId::RECORD_ROW,
    ));

    let source = store.record_row_type_checked(
        vec![RecordRowField { name: "name".into(), ty: string_ty }],
        RecordRowTail::Parameter(r),
    ).unwrap();
    let target = store.record(Box::new([
        RecordRowField { name: "name".into(), ty: string_ty },
    ]));

    assert!(is_subtype(&mut store, &hierarchy, source, target));
}
```


#### Tests to add afterward

- cancellation/budget terminal outcomes survive nested field relation;
- nested Record depth relations;
- unions containing Records preserve current union semantics;
- closed source can satisfy a closed narrower target regardless of extra fields.

#### Expected compiler errors

Removing `RecordAccess` from the helper produces `E0061` at call sites until the Record arm is updated. Adding row-specific `RelationFailure` variants causes `E0004` in projection matches until all variants are handled. Do not use wildcard arms to hide new failures.

#### Rust explanations

Width subtyping is sound for immutable products because a consumer of the narrower type only reads guaranteed fields. Covariant depth is sound for the same reason. An open source tail means “there may be additional fields”; it does not weaken the guarantee of known prefix fields.

#### Verification commands

```bash
cargo test -p phalcom-semantic record_subtyping
cargo test -p phalcom-semantic integration_matrix
cargo test -p phalcom-semantic nested_structural_relation_preserves_terminal_outcomes
rg -n 'RecordAccess' phalcom-semantic/src phalcom-semantic/tests
rg -n 'check_record_row_subtype' phalcom-semantic/src/types/relation.rs
```

#### Completion checklist

- [ ] `RecordAccess` absent from semantic source/tests.
- [ ] Immutable width subtyping works.
- [ ] Covariant depth works.
- [ ] Open source satisfies closed known-prefix target.
- [ ] Unknown tail does not prove a missing field.
- [ ] Rigid-tail cases explicit.
- [ ] Nominal classes remain nominal.
- [ ] Relation terminal states preserved.

---

### Task 6: Add Record-aware inference terms and a dedicated row-inference coordinator

#### Why

SC-2's `InferenceSession` is correctly specialized to type variables whose solutions are `TypeId`. SC-3 needs Record terms that can contain ordinary type inference variables in fields and row inference variables in tails, without teaching `InferenceSession` that a row itself is a type.

#### Architectural background

Inside one Record inference form:

```text
field type unknown -> InferVarId
row remainder unknown -> RecordRowVarId
```

The row coordinator maps stable `RecordRow` parameters to row solver variables. Ordinary `InferenceSession` continues to own type parameter variables and type constraints.

#### Current path through the code

Current `InferenceTerm` has `Canonical`, `Var`, `Applied`, `ExactCase`, `Union`, `Tuple`, and `Callable`. `InferenceSession::instantiate_generic_signature` creates an ordinary inference variable for every generic parameter based on its kind, which is the wrong domain for `RecordRow`.

#### Exact files

- Create `phalcom-semantic/src/checker/row_inference.rs`
- Create `phalcom-semantic/tests/semantic/foundations/record_row_inference.rs`
- Modify `phalcom-semantic/src/checker/mod.rs`
- Modify `phalcom-semantic/src/checker/inference.rs`
- Modify `phalcom-semantic/tests/semantic/foundations/mod.rs`

#### Exact symbols

In `inference.rs`:
- `InferenceTerm`
- `InferenceSession::instantiate_generic_signature`
- `InferenceSession::type_id_to_inference`
- `InferenceSession::type_term_to_inference`
- `InferenceSession::term_variables`
- `InferenceSession::materialize`
- every unification/subtype traversal over `InferenceTerm`

Create in `row_inference.rs`:
- `InferenceRecord`
- `InferenceRecordField`
- `InferenceRecordTail`
- `GenericInferenceBinding`
- `GenericApplicationSession`
- `CombinedInferenceOutcome`
- `CombinedInferenceFailure`
- add `InferenceSession::solved_type_for`

#### Exact insert/replace locations

1. Add `pub mod row_inference;` in `checker/mod.rs` adjacent to `inference`.
2. Add `InferenceTerm::Record(super::row_inference::InferenceRecord)` next to Tuple/Callable.
3. Put Record-specific structures in `row_inference.rs` instead of expanding `inference.rs` further.
4. Move generic-signature partitioning into `GenericApplicationSession::instantiate_generic_signature`.
5. For each stable parameter:
   - `KindId::RECORD_ROW` -> fresh `RecordRowVarId`;
   - supported type/type-constructor kind -> ordinary `InferVarId`.
6. Extend `type_id_to_inference` so `TypeData::Record` recursively converts field types and maps a stable tail parameter to the row variable only when that parameter belongs to the current generic instantiation; otherwise keep it rigid.
7. Extend all `InferenceTerm` traversals for Record fields.
8. Never insert row variables into `InferenceSession.variables` or `InferenceSolution.substitutions`.

#### Paste-ready code where safe

`checker/row_inference.rs` public core:

```rust
use std::collections::HashMap;

use crate::identity::InferVarId;
use crate::types::id::TypeParameterId;
use crate::types::row_solver::RecordRowVarId;

use super::inference::InferenceTerm;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceRecordField {
    pub name: Box<str>,
    pub ty: InferenceTerm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InferenceRecordTail {
    Closed,
    Parameter(TypeParameterId),
    Var(RecordRowVarId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceRecord {
    pub fields: Box<[InferenceRecordField]>,
    pub tail: InferenceRecordTail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericInferenceBinding {
    Type(InferVarId),
    RecordRow(RecordRowVarId),
}

#[derive(Clone, Debug, Default)]
pub struct GenericApplicationSession {
    pub parameter_bindings: HashMap<TypeParameterId, GenericInferenceBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombinedInferenceFailure {
    UnderconstrainedType(TypeParameterId),
    UnderconstrainedRow(TypeParameterId),
    RowRejected(crate::types::row_solver::RecordRowFailure),
    RowZonk(crate::types::row_solver::RecordRowZonkError),
    Blocked(crate::types::outcome::BlockReason),
    Cancelled,
    BudgetExceeded(crate::types::outcome::BudgetReport),
    InternalFailure,
}
```

Add this narrow ordinary-solution projection helper to `InferenceSession` so aliases stay private to that solver:

```rust
pub fn solved_type_for(
    &self,
    solution: &InferenceSolution,
    variable: InferVarId,
) -> Option<TypeId> {
    let representative = self.find_var(variable);
    solution.substitutions.get(&representative).copied()
}
```

In `InferenceTerm`:

```rust
Record(super::row_inference::InferenceRecord),
```

Do not add a standalone `InferenceTerm::RecordRow(RecordRowVarId)` variant. A row variable is only valid in a Record-tail slot.

#### What not to change

- Do not change `InferVarState::Solved(TypeId)` to carry rows.
- Do not put `RecordRowId` in `InferenceSolution.substitutions`.
- Do not count row variables in `term_variables` as `InferVarId`.
- Do not make `RecordRow` first-class as a proper type term.
- Do not add nominal row-valued generic application.

#### Tests to add first

Create and register `record_row_inference.rs` with:

1. `<T: Type, R: RecordRow>` partitions into distinct variable domains.
2. open canonical Record converts to `InferenceTerm::Record`.
3. field `T` maps to ordinary inference var and tail `R` maps to row var.
4. stable row parameter not owned by current generic signature remains rigid.
5. ordinary `InferenceSession` never allocates an `InferVarId` for `RecordRow`.

#### Tests to add afterward

- nested Record terms inside tuple/callable;
- Record traversal sees type vars in fields;
- proof-state propagation through field type variables;
- rigid row tail survives inference-term conversion.

#### Expected compiler errors

Adding `InferenceTerm::Record` intentionally causes `E0004` at every exhaustive traversal in `inference.rs`. Use these failures as a migration checklist. Do not add wildcard arms. Moving instantiation ownership may cause `E0599` in `call.rs`; Task 7 updates it.

#### Rust explanations

Distinct ID newtypes are a compile-time barrier against cross-domain substitution. Recursive structural terms use boxes/slices to keep enum size finite. Exhaustive matching forces every solver operation to understand the new Record form.

#### Verification commands

```bash
cargo test -p phalcom-semantic record_row_inference
cargo test -p phalcom-semantic inference
cargo test -p phalcom-semantic generic_inference_proof_integrity
rg -n 'fresh_variable\(.*RECORD_ROW|KindId::RECORD_ROW.*fresh_variable' phalcom-semantic/src/checker
```

#### Completion checklist

- [ ] `InferenceTerm::Record` exists.
- [ ] Field vars and row-tail vars use distinct IDs.
- [ ] Generic signature partition is kind-aware.
- [ ] Stable external row tails remain rigid.
- [ ] Ordinary inference solution remains `InferVarId -> TypeId`.
- [ ] No row variable enters canonical `TypeStore`.

---

### Task 7: Integrate row-polymorphic inference into the canonical callable-application funnel

#### Why

This task delivers the user-visible feature: infer a Record remainder from arguments/expected result, correlate repeated row parameters, combine row inference with ordinary generic inference, and materialize the return. It must happen inside the existing SC-2 call funnel.

#### Architectural background

For:

```phalcom
f<T, R: RecordRow>(_ x: #{ item: T, | R }) -> #{ item: T, | R }
```

called with `#{item: 1, name: "x"}`, generate:

```text
ordinary type constraint: Int <: ?T
row equation:             ?R = #{name:String}
implicit lacks:           ?R lacks item
```

Solve both domains, build `GenericInstantiation`, checked-materialize the return.

#### Current path through the code

Current `apply_generic_callable_inner` already owns:
1. generic instantiation;
2. return inference term;
3. static argument binding;
4. where constraints;
5. argument analysis and constraints;
6. expected-result constraints;
7. solve;
8. return materialization.

#### Exact files

- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/inference.rs`
- `phalcom-semantic/src/checker/row_inference.rs`
- `phalcom-semantic/src/types/instantiation.rs`
- `phalcom-semantic/src/diagnostic.rs`
- `phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs`
- `phalcom-semantic/tests/semantic/integration/mod.rs`

#### Exact symbols

- `apply_generic_callable_inner`
- `apply_generic_callable`
- `apply_resolved_callable`
- `terminal_generic_return`
- `generic_conflict_message`
- `InferenceSession::solve`
- `GenericApplicationSession::{instantiate_generic_signature,constrain_argument,constrain_expected_result,solve,build_instantiation}`

#### Exact insert/replace locations

1. In `apply_generic_callable_inner`, replace the initial `InferenceSession::new` + `instantiate_generic_signature` pair with `GenericApplicationSession` setup.
2. Preserve current `call_id`, static shape binding, argument evaluation, explanation capture, and evidence authority logic.
3. When a parameter term is `InferenceTerm::Record` and actual argument has known Record type:
   - compare common known fields;
   - emit ordinary type constraints for field types;
   - subtract formal known prefix from actual row;
   - constrain the row variable to that remainder.
4. Add implicit lacks for each formal known prefix label against the formal tail variable.
5. Reuse the same row variable for repeated stable `R`.
6. Apply expected-result Record constraints under SC-2's contextual-inference policy.
7. Solve ordinary and row domains; if either is terminal, preserve its category.
8. Build one `GenericInstantiation` only after both domains solve.
9. Replace direct `session.materialize(return_term, ctx.store)` with checked combined materialization in `RequireSolvedTail` mode.
10. Preserve `InferenceProofState`; contextual row selection is not value evidence.

#### Paste-ready code where safe

Combined instantiation builder shape:

```rust
pub fn build_instantiation(
    &mut self,
    type_solution: &InferenceSolution,
    row_solution: &RecordRowSolution,
    store: &mut TypeStore,
) -> Result<GenericInstantiation, CombinedInferenceFailure> {
    let mut result = GenericInstantiation::default();

    for (&parameter, &binding) in &self.parameter_bindings {
        match binding {
            GenericInferenceBinding::Type(variable) => {
                let Some(ty) = self.types.solved_type_for(type_solution, variable) else {
                    return Err(CombinedInferenceFailure::UnderconstrainedType(parameter));
                };
                result.bind_type(parameter, ty);
            }
            GenericInferenceBinding::RecordRow(variable) => {
                if row_solution.term_for(variable).is_none() {
                    return Err(CombinedInferenceFailure::UnderconstrainedRow(parameter));
                }
                let row = row_solution
                    .zonk_variable_to_canonical(variable, store)
                    .map_err(CombinedInferenceFailure::RowZonk)?;
                result.bind_row(parameter, row);
            }
        }
    }

    Ok(result)
}
```

When applying a Record-shaped formal constraint, branch on the actual canonical type with an ordinary `match` and route a non-Record actual through the existing argument/type-mismatch path. Do not introduce a new `RowConstraintApplication` result solely for that branch. `Dynamic` and `Unknown` are not empty rows.

#### What not to change

- Do not bypass `bind_static_arguments`.
- Do not evaluate arguments twice.
- Do not weaken causal/explanation capture.
- Do not turn row failure into `UncheckedExpression`.
- Do not default `R` to empty row.
- Do not treat Map actuals as Records.
- Do not infer row parameters on first-class monomorphic callable values.

#### Tests to add first

Create source-level integration tests:

1. infer only row remainder and preserve it in return;
2. infer `T` and `R` simultaneously;
3. repeated `R` across two arguments succeeds with same remainder;
4. repeated `R` conflicts with different remainders;
5. row parameter only in return is underconstrained without expected context;
6. expected result selects row under SC-2 policy;
7. argument-derived row conflicts with expected-result row;
8. proven empty remainder works because subtraction proves it;
9. row call path records the resolved callable through `apply_resolved_callable`.

#### Tests to add afterward

- nested row-polymorphic call in expected context;
- where constraints plus row inference;
- unknown/dynamic argument epistemic preservation;
- row solver cancellation/budget propagates to `CallCheckResult`;
- fixed return remains available only when genuinely independent of failed row variables.

#### Expected compiler errors

Expect `E0308` where old `var_map` was `HashMap<TypeParameterId, InferenceTerm>`, `E0599` for moved instantiation helpers, and `E0004` for new combined outcomes. Do not resolve by converting rows to `InferenceTerm::Canonical`.

#### Rust explanations

One coordinator can own two solvers while retaining domain separation. Clone canonical row data before mutable solver/store calls to avoid overlapping borrows. Avoid holding `&RecordRowData` across a mutable `TypeStore` call.

#### Verification commands

```bash
cargo test -p phalcom-semantic record_row_polymorphism
cargo test -p phalcom-semantic bidirectional_calls
cargo test -p phalcom-semantic generic_inference_proof_integrity
cargo test -p phalcom-semantic canonical_call_application
rg -n 'apply_row|row_generic_callable' phalcom-semantic/src/checker
```

#### Completion checklist

- [ ] Row inference enters through `apply_resolved_callable`.
- [ ] Generic parameters partition by kind.
- [ ] Common field types constrain ordinary inference.
- [ ] Remainders constrain row inference.
- [ ] Lacks generated for formal prefix labels.
- [ ] Repeated `R` correlated.
- [ ] Expected result participates.
- [ ] Underconstraint explicit.
- [ ] No argument evaluated twice.
- [ ] Checked combined materialization returns final type.

---

### Task 8: Make Record literal typing bidirectional and preserve static open tails through `**Record`

#### Why

Current `synthesize_record_literal` ignores its expected type. Record expansion exists, but complete field projection only works for closed Records. SC-3 should use known structural expectations and preserve open Record tails without fabricating fields from Map expansions.

#### Architectural background

Separate:

```text
lookup known field by name     -> valid for open or closed row
enumerate complete field set   -> only valid when tail is fully known/closed
```

Expected type guides children. It does not replace the actual literal's synthesized type.

#### Current path through the code

`Expr::RecordLiteral -> synthesize_record_literal(ctx, rec, _expected)`. Expansion analyzes source then calls `composition::project_record_fields`, which succeeds only for closed rows.

#### Exact files

- `phalcom-semantic/src/checker/composition.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`
- `phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs`

#### Exact symbols

- `project_record_fields`
- add `lookup_record_field`
- add `project_complete_record_fields`
- optionally add `project_record_shape`
- `synthesize_record_literal`
- `RecordLiteralEntry::{Field,Expansion}`

#### Exact insert/replace locations

1. Rename old closed-only `project_record_fields` to `project_complete_record_fields`.
2. Add `lookup_record_field` beside it.
3. If more than one caller needs fields + tail, add `project_record_shape` returning owned known fields and `RecordRowTail`.
4. Rename `_expected` parameter of `synthesize_record_literal` to `expected`.
5. For each statically named explicit field, project expected Record's known field type and analyze the child under that expectation.
6. For `**Record`, merge known fields and preserve open tail.
7. For `**Map` or dynamic-shape sources, preserve runtime legality but do not fabricate static row fields.
8. For explicit fields added after open expansion, generate/validate tail-lacks obligations.
9. Finalize open results through `record_row_type_checked`.

#### Paste-ready code where safe

```rust
pub(crate) fn lookup_record_field(
    store: &TypeStore,
    knowledge: &TypeKnowledge,
    name: &str,
) -> Result<TypeKnowledge, TypeKnowledge> {
    match knowledge {
        TypeKnowledge::Known(_) => {
            let Some(source_ty) = knowledge.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(source_ty) {
                TypeData::Record(row_id) => {
                    let row = store.record_row(*row_id);
                    match row.find_field(name) {
                        Some(field_ty) => Ok(knowledge.derive_known_type(
                            field_ty,
                            EvidenceOrigin::PatternDecomposition,
                        )),
                        None => Err(TypeKnowledge::Unknown(
                            UnknownReason::UncheckedExpression,
                        )),
                    }
                }
                _ => Err(TypeKnowledge::Unknown(
                    UnknownReason::UncheckedExpression,
                )),
            }
        }
        TypeKnowledge::Unknown(reason) => Err(TypeKnowledge::Unknown(reason.clone())),
        TypeKnowledge::Dynamic(reason) => Err(TypeKnowledge::Dynamic(reason.clone())),
    }
}
```

Complete projection keeps the closed-tail guard. Do not return only known fields for an open row from a function named “complete”.

#### What not to change

- Do not alter runtime Record evaluation order.
- Do not infer Map key identities.
- Do not give arbitrary extra fields an expectation from unresolved open tail.
- Do not replace actual literal type with expected type.
- Do not change parser expansion operators.

#### Tests to add first

- known-field lookup works on open row;
- complete projection rejects open row;
- expected Record field guides nested empty literal/collection;
- expected open Record guides known prefix only;
- `**` from open Record preserves tail;
- explicit extension after open Record preserves tail and checks lacks;
- known duplicate from expansion rejected;
- Map expansion does not create finite static row.

#### Tests to add afterward

- multiple compatible open Record expansions;
- dynamic expansion preserves causal/dynamic status;
- empty Record/Unit semantic conformance;
- canonical field order after expansion.

#### Expected compiler errors

Renaming `project_record_fields` yields `E0425` at callers; migrate each call based on whether it needs complete enumeration or only lookup/shape. No signature break is needed for `synthesize_record_literal` because expected is already passed.

#### Rust explanations

Field lookup is sound on an open row because the prefix is guaranteed. Complete enumeration is not. Bidirectional typing is guidance, not coercion.

#### Verification commands

```bash
cargo test -p phalcom-semantic expression_composition
cargo test -p phalcom-semantic record_row_polymorphism
cargo test -p phalcom-semantic expression_analysis
rg -n 'project_record_fields' phalcom-semantic/src
rg -n 'fn synthesize_record_literal.*_expected' phalcom-semantic/src/checker/expression.rs
```

#### Completion checklist

- [ ] Known-field lookup works for open rows.
- [ ] Complete projection remains honest.
- [ ] Record literal uses expected known fields.
- [ ] Literal synthesizes actual row.
- [ ] Open `**Record` preserves tail.
- [ ] Map expansion does not fabricate key row.
- [ ] Duplicate/lacks rules enforced.
- [ ] Runtime evaluation order unchanged.

---

### Task 9: Lock in the already-correct open-Record pattern prefix semantics and share helpers only where behavior is unchanged

#### Why

The fresh repository audit found that `checker/pattern.rs::resolve_record_pattern` is already ahead of the earlier SC-3 requirements analysis. It clones the canonical row, uses exact known field types, marks a missing field impossible only when the row tail is `Closed`, and remains conservative when an open tail might contain an otherwise-unknown field. SC-3 should preserve and test this behavior rather than rewrite it unnecessarily.

#### Architectural background

The required law is already the implemented shape:

```text
known prefix field
  -> exact PatternDecomposition type

missing field + Closed tail
  -> statically impossible pattern

missing field + open tail
  -> not statically established
  -> conservative/refutable child typing
```

Pattern exactness remains syntax/pattern semantics; Record type closure does not redefine it.

#### Current path through the code

At the pinned baseline, `resolve_record_pattern` effectively performs:

```rust
match row.find_field(&entry.label) {
    Some(ty) => Some(ty),
    None if row.tail == RecordRowTail::Closed => {
        impossible = true;
        emit MatchPatternFieldMismatch;
        None
    }
    None => None, // open tail: possible, not proven
}
```

It then uses `conservative_pattern_type` for the non-proven case. This already matches SC-3's evidence discipline.

#### Exact files

- Add/modify tests:
  - `phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs`
  - `phalcom-semantic/tests/semantic/advanced/integration_matrix.rs` if its pattern matrix is the better established home
- Modify production code only if shared-helper refactoring is justified by Task 8:
  - `phalcom-semantic/src/checker/pattern.rs`
  - `phalcom-semantic/src/checker/composition.rs`

#### Exact symbols

- `resolve_record_pattern`
- `conservative_pattern_type`
- `RecordRowData::find_field`
- `RecordRowTail::Closed`
- `PatternResolution::Record`
- `PatternSpace::Record`
- Task 8's `lookup_record_field`, if sharing it does not alter closed/open missing-field behavior

#### Exact insert/replace locations

1. Add tests before touching `pattern.rs`.
2. If all new tests pass on the current production code, make **no semantic production change** in this task.
3. If Task 8 introduced a reusable field lookup helper and deduplication is worthwhile, replace only the `Some(row) -> row.find_field(...)` lookup portion while preserving the explicit closed-tail `impossible` branch.
4. Do not route the open-tail missing case through a helper that returns generic `Unknown` if doing so loses the distinction between “possible in tail” and “closed impossible.”
5. Preserve existing `MatchPatternFieldMismatch` diagnostic anchoring for closed missing fields.

#### Paste-ready code where safe

The production logic below is the behavior to retain; if refactoring, preserve it structurally:

```rust
let field_ty = match &known_row {
    Some(row) => match row.find_field(&entry.label) {
        Some(ty) => Some(ty),
        None if matches!(row.tail, RecordRowTail::Closed) => {
            impossible = true;
            // keep existing MatchPatternFieldMismatch diagnostic
            None
        }
        None => None,
    },
    None => None,
};
```

Do not replace the final `None` with a fabricated type from the open tail.

#### What not to change

- Do not add Record rest-pattern grammar.
- Do not change match exhaustiveness merely because open row types are now source-reachable.
- Do not treat a possible tail field as guaranteed.
- Do not turn an open missing field into immediate contradiction.
- Do not structurally inspect nominal class layout.
- Do not refactor working pattern logic solely to make SC-3 appear to have more production changes.

#### Tests to add first

Add source-level tests proving current behavior with **source-lowered** open rows:

1. known open-row field binds its exact declared type;
2. closed Record missing required pattern field is impossible and diagnoses `MatchPatternFieldMismatch`;
3. open Record missing-from-prefix field is not diagnosed as statically absent;
4. missing-from-open-prefix child does not acquire a fabricated precise type;
5. nested known Record prefix field decomposes recursively.

These tests are important because the current unit-level code is correct but open Record annotations are not yet source-reachable before Task 4.

#### Tests to add afterward

- or-pattern binding joins preserve a row-derived known field type;
- open-row pattern behavior remains stable after incremental edits;
- enum/GADT pattern/exhaustiveness suites remain green;
- if Task 8 helper sharing is performed, add a direct regression proving the refactor did not change the open-tail missing case.

#### Expected compiler errors

None are expected if this remains test-only.

If refactoring through `composition.rs`, borrow checker `E0502` may appear if a borrowed row survives recursive mutable pattern checking. Copy the `TypeId`/tail fact out before recursion; do not introduce `unsafe`.

#### Rust explanations

This task deliberately demonstrates an important planning principle: repository-grounded implementation does not require rewriting code that already satisfies the law. The compiler's existing `Option<TypeId>` lookup plus explicit `RecordRowTail::Closed` branch correctly distinguishes known proof from possible tail membership.

#### Verification commands

```bash
cargo test -p phalcom-semantic record_row_polymorphism
cargo test -p phalcom-semantic pattern
cargo test -p phalcom-semantic match_pattern
cargo test -p phalcom-semantic adts
```

#### Completion checklist

- [ ] Source-level open-row pattern tests exist.
- [ ] Known prefix fields decompose precisely.
- [ ] Closed missing fields remain impossible.
- [ ] Open missing fields remain possible-but-unproven.
- [ ] No fabricated field type appears.
- [ ] Existing pattern logic is left unchanged if tests already pass.
- [ ] Match/GADT regressions pass.


---

### Task 10: Remove the SC-1 scoped-open-Record gate and preserve free row tails in capture-safe scoped type structures

#### Why

The fresh baseline already contains SC-1's capture-safe `ScopedBinderStack` and `lower_scoped_type_form`; it does **not** use the old `ScopedTypeData::Free(body_ty)` shortcut anymore. Its deliberate SC-3 handoff is explicit: `TypeAnnotationExpr::Record { tail: Some(_) }` inside scoped lowering returns `TypeFormationInvalid::UnsupportedOpenRecordTail`. SC-3 must replace that temporary rejection with an honest scoped open-Record node while keeping the Task 4 scope decision that public row-kinded type-lambda parameters remain unsupported.

#### Architectural background

Two capabilities remain distinct:

```text
A. A scoped type body captures an already-stable RecordRow parameter
   and uses it as a Record tail.

B. A type lambda itself binds a RecordRow parameter and later receives
   a row-valued argument during beta reduction.
```

SC-3 implements A.

Task 4 explicitly rejects public B because the current application representation is still:

```rust
TypeData::Applied { arguments: Box<[TypeId]> }
TypeLambdaArena::beta_reduce(args: &[TypeId])
```

There is no row-valued application argument. The semantic scoped representation may reserve a `Bound` row-tail case to match the already-versioned metadata schema, but that case is not a public source capability in SC-3.

#### Current path through the code

At `main@abb2b5d80654e2525d68f4ea8ff9d32b810330b3`:

```text
lower_scoped_type_form
  -> Reference
       ScopedBinderStack lookup / free TypeLevelBinding lookup
  -> Tuple / Callable / Union / Applied
       capture-safe scoped nodes
  -> Record { fields, tail }
       if tail.is_some():
         diagnostic AnnotationUnsupported
         Invalid(UnsupportedOpenRecordTail)
       else:
         ScopedTypeData::Record(fields)

lower_scoped_type_lambda
  -> ScopedBinderStack push/pop
  -> alpha-normalized TypeLambdaArena

TypeLambdaArena
  -> ScopedTypeData::Record(fields)
  -> no semantic open-tail node

phalcom-type-meta
  -> ScopedRecordTailRef::{Bound, FreeParameter}
  -> ScopedTypeNode::OpenRecord already defined
```

#### Exact files

- Modify:
  - `phalcom-semantic/src/types/type_lambda.rs`
  - `phalcom-semantic/src/types/annotation.rs`
  - `phalcom-semantic/src/metadata/export.rs`
  - `phalcom-semantic/src/types/mod.rs` if re-exports are needed
  - `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
  - `phalcom-semantic/tests/semantic/integration/metadata.rs`
- Inspect, preserve schema:
  - `phalcom-type-meta/src/scoped_type.rs`
  - `phalcom-type-meta/src/type_node.rs`

#### Exact symbols

Existing:

- `ScopedTypeData`
- `ScopedRecordField`
- `ScopedBinderStack`
- `ScopedBinder`
- `lower_scoped_type_form`
- `lower_scoped_type_alias_form`
- `lower_scoped_type_lambda`
- `TypeFormationInvalid::UnsupportedOpenRecordTail`
- `TypeLambdaArena::has_free_bound`
- `TypeLambdaArena::collect_free_types`
- `TypeLambdaArena::subst_scoped_to_canonical`
- `TypeLambdaArena::subst_scoped_partial`
- `BetaReductionError`
- `MetadataExporter::export_scoped_type`
- `ScopedTypeNode::OpenRecord`
- `ScopedRecordTailRef`

Add:

- `ScopedRecordTail`
- `ScopedOpenRecord`
- `ScopedTypeData::OpenRecord`
- `BetaReductionError::UnsupportedRecordRowArgument` only if a bound row-tail node can reach canonical beta reduction internally

#### Exact insert/replace locations

1. In `types/type_lambda.rs`, immediately after `ScopedRecordField`, add `ScopedRecordTail` and `ScopedOpenRecord`.
2. In `ScopedTypeData`, keep `Record(Box<[ScopedRecordField]>)` for closed scoped Records and add `OpenRecord(ScopedOpenRecord)`.
3. Update every exhaustive `ScopedTypeData` traversal:
   - `has_free_bound`;
   - `collect_free_types`;
   - `subst_scoped_to_canonical`;
   - `subst_scoped_partial`;
   - any scoped structural fingerprint/equality traversal outside the derived `Hash/Eq` implementation.
4. In `lower_scoped_type_form`'s Record arm, remove the early `tail.is_some() -> UnsupportedOpenRecordTail` return.
5. Lower fields exactly as SC-1 currently does.
6. Tail lowering:
   - `None` -> keep `ScopedTypeData::Record(fields)`;
   - tail name resolving through the **lexical resolver** as `TypeLevelBinding::RecordRow(parameter)` -> `ScopedTypeData::OpenRecord { tail: FreeParameter(parameter) }`;
   - tail resolving as an ordinary type form -> structured row-tail kind mismatch from Task 4;
   - unresolved -> structured unresolved formation from Task 4.
7. If `ScopedBinderStack` can internally expose a bound `RecordRow` binder despite Task 4's public source gate, encode it as `ScopedRecordTail::Bound { depth, index }`; otherwise retain the enum case for metadata symmetry and test it at the arena level only.
8. In `subst_scoped_to_canonical`:
   - `FreeParameter(p)` -> construct canonical open Record with `RecordRowTail::Parameter(p)` through `record_row_type_checked`;
   - `Bound` row tail -> return `UnsupportedRecordRowArgument` because the `args: &[TypeId]` API cannot supply a row value.
9. In `subst_scoped_partial`, preserve/shift a bound row-tail depth exactly as bound proper-type nodes are shifted; free stable row parameters are unchanged.
10. In `metadata/export.rs::export_scoped_type`, map `ScopedTypeData::OpenRecord` directly to the already-defined `ScopedTypeNode::OpenRecord` and corresponding `ScopedRecordTailRef`.
11. After all scoped open-row tests pass, remove `TypeFormationInvalid::UnsupportedOpenRecordTail` if no other handoff path uses it.

#### Paste-ready code where safe

Add to `types/type_lambda.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ScopedRecordTail {
    Bound { depth: u32, index: u32 },
    FreeParameter(TypeParameterId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopedOpenRecord {
    pub fields: Box<[ScopedRecordField]>,
    pub tail: ScopedRecordTail,
}
```

Add the node:

```rust
pub enum ScopedTypeData {
    // existing variants ...
    Record(Box<[ScopedRecordField]>),
    OpenRecord(ScopedOpenRecord),
    Callable(ScopedCallableType),
    // existing variants ...
}
```

Free-type collection:

```rust
ScopedTypeData::OpenRecord(record) => {
    for field in record.fields.iter() {
        self.collect_free_types(field.ty, out);
    }
    // record.tail belongs to the RecordRow domain; it is never a TypeId.
}
```

Free stable tail canonicalization:

```rust
ScopedRecordTail::FreeParameter(parameter) => {
    store
        .record_row_type_checked(fields, RecordRowTail::Parameter(parameter))
        .map_err(|_| BetaReductionError::Application(
            TypeApplicationError::MalformedLambda,
        ))
}
```

If `TypeApplicationError::MalformedLambda` is too lossy after Task 1 introduces a precise row formation error, add a dedicated `BetaReductionError::RecordRowFormation(...)` instead. Do not convert the row parameter to a `TypeId`.

#### What not to change

- Do not restore the old `ScopedTypeData::Free(body_ty)` type-lambda lowering.
- Do not change `TypeData::Applied.arguments: Box<[TypeId]>`.
- Do not pass `RecordRowId` through `beta_reduce(&[TypeId])`.
- Do not invent a synthetic row `TypeId`.
- Do not re-enable public row-kinded type-lambda binders rejected by Task 4.
- Do not re-enable row-kinded generic aliases rejected by Task 4.
- Do not change the already-versioned `ScopedOpenRecord` metadata wire shape.

#### Tests to add first

Add failing tests for source-reachable SC-3 behavior:

1. a type lambda/scoped form nested inside a callable with stable `R: RecordRow` can capture `R` in `#{ field: T, | R }` without `UnsupportedOpenRecordTail`;
2. closed scoped Record still lowers to `ScopedTypeData::Record`;
3. free stable open tail lowers to `ScopedTypeData::OpenRecord::FreeParameter(R)`;
4. wrong-kind scoped tail is rejected with Task 4's structured invalidity;
5. unresolved scoped tail remains unresolved rather than becoming closed;
6. scoped metadata export chooses `ScopedTypeNode::OpenRecord`.

Add one arena-level test for a manually constructed `ScopedRecordTail::Bound` and one negative test proving public row-kinded type-lambda application remains rejected/unsupported.

#### Tests to add afterward

- `has_free_bound` handles a manually constructed bound row tail correctly;
- `collect_free_types` visits field types but never treats the row tail as `TypeId`;
- nested lambda depth shifting preserves a bound row tail if the internal node is used;
- partial ordinary type-argument reduction preserves an unrelated free stable row tail;
- alpha-equivalent open scoped Records with the same free stable tail canonicalize identically.

#### Expected compiler errors

Adding `ScopedTypeData::OpenRecord` should intentionally produce `E0004` in every scoped traversal and metadata exporter match. Use those compiler errors as a migration checklist; do not add wildcard arms.

Adding a `BetaReductionError` variant likewise requires exhaustive error mapping in `TypeStore::apply_type_form`; update it explicitly.

#### Rust explanations

The baseline now already demonstrates why this representation should be domain-specific: scoped proper types use `ScopedTypeId`, while row tails are binder references of a different kind. A dedicated tail enum prevents accidental kind confusion. A free stable row tail can be reified into canonical `RecordRowTail::Parameter` without solving anything; a bound row tail cannot be substituted until a future beta-reduction API accepts row-domain arguments.

#### Verification commands

```bash
cargo test -p phalcom-semantic type_annotations
cargo test -p phalcom-semantic metadata
cargo test -p phalcom-type-meta

rg -n 'UnsupportedOpenRecordTail' phalcom-semantic/src/types/annotation.rs
rg -n 'ScopedTypeData::' \
  phalcom-semantic/src/types/type_lambda.rs \
  phalcom-semantic/src/types/annotation.rs \
  phalcom-semantic/src/metadata/export.rs
```

#### Completion checklist

- [ ] SC-1 capture-safe lowerer is extended, not replaced.
- [ ] Closed scoped Record representation remains intact.
- [ ] Free stable row tail is preserved in scoped open Record.
- [ ] Optional internal bound row-tail representation is domain-specific.
- [ ] Every scoped traversal handles open Record.
- [ ] `UnsupportedOpenRecordTail` handoff is removed after support is real.
- [ ] Existing `ScopedTypeNode::OpenRecord` metadata schema is used.
- [ ] No row is encoded as `TypeId`.
- [ ] Public row-kinded type-lambda application remains outside SC-3.


---

### Task 11: Add row-specific diagnostics and explanation edges while preserving one diagnostic authority

#### Why

The current implementation can collapse row problems into generic `TypeMismatch`, `UncheckedExpression`, or `Blocked(RecursiveFixpoint)`. Once row syntax and row inference are source-reachable, developers need diagnostics that name the actual semantic failure: wrong tail kind, unresolved tail, lacks violation, recursive row, underconstrained row inference, or conflicting remainders.

#### Architectural background

The solver should return structured facts, not user-facing prose. `phalcom-semantic` remains the semantic/diagnostic authority:

```text
row/type formation or solver outcome
        -> stable structured failure
        -> checker/annotation boundary
        -> SemanticDiagnostic + explanation DAG
        -> CLI/LSP presentation
```

No LSP-specific row reasoning belongs here.

#### Current path through the code

At the pinned baseline:

```text
DiagnosticCode
  -> stable as_str() code

call.rs
  -> maps InferenceOutcome to diagnostics/status
  -> records GenericConstraint explanation steps

CheckingContext
  -> record_derivation
  -> attach_explanation_to_cause
  -> record_call_dependency

row_solver.rs
  -> structured RecordRowFailure
  -> no dedicated presentation layer
```

#### Exact files

- Modify:
  - `phalcom-semantic/src/diagnostic.rs`
  - `phalcom-semantic/src/checker/call.rs`
  - `phalcom-semantic/src/checker/row_inference.rs`
  - `phalcom-semantic/src/explain.rs` if a row-specific derivation step is required
  - any exhaustive presenter over `ExplanationStep`
  - `phalcom-semantic/tests/semantic/foundations/diagnostics.rs`
  - `phalcom-semantic/tests/semantic/foundations/diagnostic_presentation.rs`
  - `phalcom-semantic/tests/semantic/foundations/explanations.rs`
  - `phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs`

#### Exact symbols

- `DiagnosticCode`
- `DiagnosticCode::as_str`
- `SemanticDiagnostic`
- `GenericInferenceConflict`
- `GenericInferenceUnderconstrained`
- `generic_conflict_message`
- `terminal_generic_return`
- `RecordRowFailure`
- `RecordRowSolveResult`
- `ExplanationStep`
- `DerivationRule`
- `CheckingContext::record_derivation`

#### Exact insert/replace locations

1. In `DiagnosticCode`, insert Record/row codes alongside existing type diagnostics, before project/module diagnostics.
2. Add one exact `as_str()` arm per code.
3. `types/annotation.rs` uses tail formation codes for source lowering failures.
4. `checker/row_inference.rs` converts low-level row solver failures into stable call-level row failure data, retaining the stable `TypeParameterId` and source constraint origin.
5. `checker/call.rs` emits row inference diagnostics at the same application boundary that currently emits ordinary generic inference diagnostics.
6. A combined ordinary-type + row conflict gets one root call diagnostic. Attach additional row/type causes as labels/notes/explanation parents rather than duplicating root diagnostics.
7. If adding explanation nodes, record stable information only:
   - source call/argument expression IDs;
   - stable row parameter identity;
   - canonical actual/formal Record `TypeId`s or `RecordRowId`s for internal explanation storage;
   - canonical solved remainder after zonking.
8. Never put `RecordRowVarId` in a user-rendered message.

#### Paste-ready code where safe

Add these `DiagnosticCode` variants:

```rust
RecordDuplicateField,
RecordRowTailUnresolved,
RecordRowTailKindMismatch,
RecordRowLacksViolation,
RecordRowOccursCheck,
RecordRowInferenceUnderconstrained,
RecordRowInferenceConflict,
```

Add exact stable strings:

```rust
Self::RecordDuplicateField => "type.record.duplicate_field",
Self::RecordRowTailUnresolved => "type.record.row_tail_unresolved",
Self::RecordRowTailKindMismatch => "type.record.row_tail_kind_mismatch",
Self::RecordRowLacksViolation => "type.record.row_lacks_violation",
Self::RecordRowOccursCheck => "type.record.row_occurs_check",
Self::RecordRowInferenceUnderconstrained => "type.record.row_inference_underconstrained",
Self::RecordRowInferenceConflict => "type.record.row_inference_conflict",
```

A useful stable call-level failure representation in `row_inference.rs` is:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RowInferenceFailure {
    Underconstrained {
        parameter: TypeParameterId,
    },
    LacksViolation {
        parameter: TypeParameterId,
        field: Box<str>,
    },
    OccursCheck {
        parameter: TypeParameterId,
    },
    Conflict {
        parameter: TypeParameterId,
    },
}
```

The low-level solver may retain richer query-local data; convert to this stable form before presentation.

#### What not to change

- Do not emit `AnalysisInternalFailure` for ordinary row contradictions.
- Do not call a row-tail mismatch `RecursiveFixpoint` after a row-specific blocked/failure category exists.
- Do not emit one root diagnostic per field involved in the same inference conflict.
- Do not expose `RecordRowVarId` numbers.
- Do not add LSP-owned row diagnostic logic.
- Do not downgrade cancellation or budget exhaustion into stable source errors.

#### Tests to add first

Add exact-code tests for:

1. unresolved row tail;
2. wrong-kind row tail;
3. duplicate known Record field;
4. lacks violation;
5. recursive row equation;
6. underconstrained row-polymorphic call;
7. conflicting repeated row parameter.

Add a presentation assertion that rendered row inference messages do not contain strings such as `RecordRowVarId(` or solver-local numeric metavariable notation.

#### Tests to add afterward

- explanation chain links argument -> row constraint -> solved remainder -> return materialization;
- repeated-row conflict emits one primary root diagnostic;
- expected-result conflict carries the expected expression/context as a secondary cause;
- cancellation and budget paths produce terminal analysis status without false persistent diagnostics.

#### Expected compiler errors

Adding `DiagnosticCode` variants may produce `E0004` in exhaustive code mappings. Update all mappings explicitly.

Adding an `ExplanationStep` variant will similarly produce exhaustive-match errors in presenters/serializers. Complete each projection; do not add wildcard arms.

#### Rust explanations

Stable diagnostics should identify language-level entities such as the source row parameter, not transient solver representatives. Converting a query-local failure into a stable failure object is a useful ownership boundary: it prevents lifetime/identity leakage and makes diagnostics deterministic across cold and incremental runs.

#### Verification commands

```bash
cargo test -p phalcom-semantic diagnostics
cargo test -p phalcom-semantic diagnostic_presentation
cargo test -p phalcom-semantic explanations
cargo test -p phalcom-semantic record_row_polymorphism

rg -n 'RecordRowVarId' \
  phalcom-semantic/src/diagnostic.rs \
  phalcom-semantic/src/explain.rs
```

#### Completion checklist

- [ ] Stable row diagnostic codes exist.
- [ ] Tail resolution/kind errors are specific.
- [ ] Lacks and occurs-check failures are specific.
- [ ] Row underconstraint/conflict are specific.
- [ ] Combined conflicts have one root diagnostic.
- [ ] Solver-local row IDs do not appear in rendered diagnostics.
- [ ] Cancellation/budget remain terminal states, not source errors.

---

### Task 12: Publish open Records through the existing metadata schema and make fingerprints tail-sensitive

#### Why

The metadata schema already contains `TypeNode::OpenRecord`, but the current exporter ignores `RecordRowData.tail` and emits every `TypeData::Record` as closed `TypeNode::Record`. `MetadataFeatures.record_rows` is also currently `false`. Once SC-3 makes open rows publishable, the current exporter would silently lie about their type.

#### Architectural background

The projection is direct:

```text
semantic RecordRowTail::Closed
  -> TypeNode::Record(fields)

semantic RecordRowTail::Parameter(R)
  -> validate R.kind == RecordRow
  -> StableTypeParameterRef(R)
  -> TypeNode::OpenRecord { fields, tail: R }
```

A solver variable can never appear here because the canonical semantic row tail does not contain `RecordRowVarId`.

Fingerprints must encode denotation, not arena allocation order. For an open row, the stable tail **owner plus parameter index** must participate; the current `OpenRecord` fingerprint branch writes only `tail.index`, which is insufficient if two different owners both have parameter zero.

#### Current path through the code

At the pinned baseline:

```text
MetadataExporter::export_type_form
  -> TypeData::Record(row_id)
  -> export field refs
  -> TypeNode::Record(fields)             // tail ignored

fingerprint match
  -> TypeNode::OpenRecord(...) branch exists
  -> writes fields + tail.index

build_bundle
  -> MetadataFeatures { record_rows: false, ... }
```

`phalcom-type-meta/src/type_node.rs` already defines:

```text
OpenRecordTypeRef
TypeNode::OpenRecord
```

and `scoped_type.rs` already defines its scoped counterpart.

#### Exact files

- Modify:
  - `phalcom-semantic/src/metadata/export.rs`
  - `phalcom-semantic/tests/semantic/integration/metadata.rs`
- Inspect, normally no schema modification:
  - `phalcom-type-meta/src/type_node.rs`
  - `phalcom-type-meta/src/scoped_type.rs`
  - `phalcom-type-meta/src/generic.rs`
- Modify `phalcom-type-meta` only if a reusable stable-parameter fingerprint helper is required; do not change `OpenRecord` wire shape.

#### Exact symbols

- `MetadataExporter::export_type_form`
- `TypeData::Record(row_id)` arm
- `TypeNode::Record`
- `TypeNode::OpenRecord`
- `OpenRecordTypeRef`
- `MetadataExporter::export_type_parameter`
- structural fingerprint match for `TypeNode::OpenRecord`
- `MetadataFeatures.record_rows`
- `StableTypeParameterRef`

#### Exact insert/replace locations

1. In `export_type_form`, replace the entire `TypeData::Record(row_id)` arm.
2. Clone the semantic row before recursively exporting field types so no immutable store borrow survives a mutable exporter call.
3. Export known fields once.
4. Branch on `row.tail`:
   - `Closed` -> `TypeNode::Record`;
   - `Parameter(parameter)` -> verify canonical parameter kind is `KindId::RECORD_ROW`, export stable parameter ref, build `TypeNode::OpenRecord`.
5. In the fingerprint arm for `TypeNode::OpenRecord`, replace index-only tail hashing with the same stable owner+index identity components used by generic parameter identity elsewhere in metadata.
6. If there is no helper, add a private `write_stable_type_parameter_fingerprint` in `metadata/export.rs`; it must encode owner kind/path/callable identity plus index, not a runtime arena ID.
7. Change `record_rows: false` to `record_rows: true` only after both closed/open export tests pass.
8. Keep schema version unchanged unless the repository's metadata compatibility policy explicitly requires a version bump when activating an already-defined node. If it does, update version constants/tests in the same commit.

#### Paste-ready code where safe

The `TypeData::Record` arm should be shaped like this:

```rust
TypeData::Record(row_id) => {
    let row = self.store.record_row(row_id).clone();
    let mut field_refs = Vec::with_capacity(row.fields.len());

    for field in row.fields.iter() {
        field_refs.push(RecordFieldRef {
            name: field.name.clone(),
            ty: self.export_type_form(field.ty)?,
        });
    }

    match row.tail {
        RecordRowTail::Closed => {
            TypeNode::Record(field_refs.into_boxed_slice())
        }
        RecordRowTail::Parameter(parameter) => {
            if self.store.type_parameter(parameter).kind != KindId::RECORD_ROW {
                return Err(MetadataExportError::NonExportableForm(ty));
            }

            TypeNode::OpenRecord(OpenRecordTypeRef {
                fields: field_refs.into_boxed_slice(),
                tail: self.export_type_parameter(parameter),
            })
        }
    }
}
```

Add explicit imports rather than relying on broad glob imports:

```rust
use crate::types::id::KindId;
use crate::types::row::RecordRowTail;
use phalcom_type_meta::type_node::OpenRecordTypeRef;
```

Adjust the exact `phalcom_type_meta` module path to the current import style in `export.rs`.

#### What not to change

- Do not add another durable row-node representation.
- Do not export `RecordRowId` as identity.
- Do not export `RecordRowVarId`.
- Do not close an open row merely to make metadata export succeed.
- Do not enable `record_rows` before exporter tests prove support.
- Do not bump schema version reflexively if the existing schema already reserves the node.

#### Tests to add first

In `integration/metadata.rs`, add failing tests for:

1. closed Record -> `TypeNode::Record`;
2. open Record -> `TypeNode::OpenRecord`;
3. exported tail points to the correct stable parameter owner/index;
4. otherwise-identical closed and open Records have different structural fingerprints;
5. otherwise-identical open Records with tails belonging to different owners have different fingerprints;
6. bundle feature flag reports row support only after open export succeeds.

#### Tests to add afterward

- scoped open Record export uses `ScopedTypeNode::OpenRecord`;
- metadata bytes/fingerprints are deterministic across cold equivalent builds;
- retained snapshot export preserves the old tail identity after a new revision;
- malformed wrong-kind tail cannot be exported through checked canonical construction.

#### Expected compiler errors

Adding `KindId`/row/meta imports can produce duplicate-name `E0252` if equivalents are already imported. Consolidate imports rather than aliasing gratuitously.

Cloning the row before recursive exports avoids `E0502` mutable/immutable borrow conflicts.

If the metadata feature struct is constructed in tests with explicit fields, changing expected `record_rows` value produces assertion failures, not a type error; update only row-support expectations.

#### Rust explanations

Canonical arena indexes are implementation-local and can vary with allocation order. Stable metadata fingerprints must instead encode semantic identity. The tail's parameter index alone is not globally unique; its owner is part of the binder identity.

#### Verification commands

```bash
cargo test -p phalcom-semantic metadata
cargo test -p phalcom-type-meta

rg -n 'record_rows:\s*false' phalcom-semantic/src/metadata/export.rs
rg -n 'TypeData::Record' phalcom-semantic/src/metadata/export.rs
rg -n 'OpenRecord' phalcom-semantic/src/metadata/export.rs phalcom-type-meta/src
```

#### Completion checklist

- [ ] Closed Records export as closed nodes.
- [ ] Open Records export as `OpenRecord`.
- [ ] Tail parameter kind is validated.
- [ ] Tail owner+index participates in fingerprint.
- [ ] Raw row arena IDs are absent from metadata identity.
- [ ] Solver row IDs are absent from metadata.
- [ ] `record_rows` feature flag reflects real support.
- [ ] Existing metadata schema is reused.

---

### Task 13: Add snapshot, incremental, and semantic-read-model coverage for row-bearing products

#### Why

Rows will appear in callable signatures, canonical types, inferred call results, and metadata. A row-tail-only signature edit must invalidate dependent callers, while a body-only edit must not invalidate a stable signature. Retained snapshots must preserve their old row denotation. Solver-local state must never become a DB key or snapshot product.

#### Architectural background

Published row state is canonical:

```text
TypeParameterId
RecordRowData / RecordRowId
TypeData::Record
```

Query-local state is not published:

```text
RecordRowVarId
row substitution worklist
solver union/find aliases
pending lacks obligations
```

Incremental fingerprints therefore derive from stable/canonical semantic structure rather than solver allocation order or raw arena index.

#### Current path through the code

The current semantic DB already has:

```text
db/fingerprint.rs
  -> structural product fingerprints

db/query.rs
  -> declaration/callable/expression queries

session.rs
  -> incremental update + retained snapshots

tests/semantic/incremental
  -> callable dependencies
  -> checker dependencies
  -> fingerprints
  -> product stability
  -> type-store revisions
```

SC-3 should extend these products/tests, not add a separate row cache.

#### Exact files

- Create:
  - `phalcom-semantic/tests/semantic/incremental/record_rows.rs`
- Modify:
  - `phalcom-semantic/tests/semantic/incremental/mod.rs`
  - `phalcom-semantic/src/db/fingerprint.rs` only if tests prove row tail/field semantics are omitted
  - `phalcom-semantic/src/db/query.rs` only if dependency capture is missing
  - `phalcom-semantic/src/session.rs` only if canonical row-bearing products fail to publish/reuse
  - `phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs` only for shared fixture reuse
  - `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs` for retained-row arena semantics if appropriate

#### Exact symbols

Inspect before editing:

- declaration shell/signature fingerprint functions in `db/fingerprint.rs`
- callable signature fingerprint functions
- expression/call analysis product fingerprint functions
- `SemanticSnapshot`
- `SemanticSession::update_with_budget_and_cancel`
- existing incremental fixture helpers
- retained `TypeStore` snapshot/revision tests

Do not introduce `QueryKey::RecordRowSolve` or equivalent in SC-3.

#### Exact insert/replace locations

1. Register `mod record_rows;` in `tests/semantic/incremental/mod.rs` next to other semantic-domain incremental tests.
2. Add incremental tests **before** changing DB/fingerprint production code.
3. Revision A establishes a row-polymorphic callable and dependent caller.
4. Revision B changes only callable body text; assert signature product reuse/semantic equivalence according to existing fixture observability.
5. Revision C changes a known Record prefix field type; assert dependent caller recomputes.
6. Revision D changes row-tail binder kind `RecordRow -> Type`; assert stale valid open-row products are not reused.
7. Revision E changes only stable row-binder source location/name if names/provenance are non-semantic; assert behavior according to SC-1's binder identity/fingerprint policy.
8. If a row-tail semantic edit fails to invalidate, inspect the existing signature fingerprint and add stable row data there. Hash:
   - field names;
   - field type structural fingerprints;
   - tail closed/open tag;
   - stable tail parameter semantic identity/kind.
9. If body-only edit invalidates the signature, repair dependency ownership rather than adding a row-specific cache.
10. Add a cold-vs-incremental equivalence test that compares final semantic types/diagnostics, not raw arena IDs.
11. Search DB/snapshot structs for `RecordRowVarId`; no match is acceptable.

#### Paste-ready code where safe

Use this revision matrix in the new test module; adapt to the repository's existing fixture API rather than inventing a new harness:

```text
A:
  preserve<R: RecordRow>(
      _ x: #{ name: String, | R }
  ) -> #{ name: String, | R } { x }

B:
  same signature; semantically equivalent/body-only source edit

C:
  prefix changes to #{ name: Object, | R }

D:
  binder changes to <R: Type>
  while the Record tail still names R
```

Expected dependency behavior:

```text
A -> B : signature reusable; row denotation unchanged
B -> C : dependent callers invalidated/rechecked
C -> D : declaration/callable signature becomes invalid; no stale open row reused
```

#### What not to change

- Do not cache `RecordRowSolver` or `GenericApplicationSession` across queries.
- Do not use `RecordRowVarId` in any DB key/product.
- Do not fingerprint raw `RecordRowId.0` as durable semantic identity.
- Do not disable incremental reuse globally to make tests pass.
- Do not push row analysis into the LSP.
- Do not force whole-workspace invalidation for a local row signature edit.

#### Tests to add first

Create these tests first:

1. `row_body_only_edit_preserves_signature_semantics`;
2. `row_prefix_edit_invalidates_dependent_call`;
3. `row_tail_kind_edit_invalidates_open_record_signature`;
4. `cold_and_incremental_row_semantics_match`;
5. `retained_snapshot_preserves_previous_open_row_denotation`;
6. `solver_row_variables_never_appear_in_published_products` if product inspection APIs permit it.

#### Tests to add afterward

- independent module edit does not invalidate row-polymorphic caller;
- fixing a wrong-kind tail removes the diagnostic without stale invalid product reuse;
- metadata fingerprints from cold-equivalent sessions match;
- row solver allocation order has no effect on published output.

#### Expected compiler errors

No broad production compile errors are expected. Failing incremental assertions are the signal for targeted DB/fingerprint changes.

If adding fingerprint handling requires matching a new semantic enum variant introduced earlier, Rust may emit `E0004`; update the existing fingerprint function rather than adding a parallel row fingerprint function detached from the containing product.

#### Rust explanations

Retained snapshots rely on old immutable semantic arena entries retaining their denotation. New revisions may allocate new `TypeParameterId`/`RecordRowId` entries when semantic meaning changes; tests should compare semantic structure or stable projection, not assume raw IDs are identical across unrelated cold runs.

#### Verification commands

```bash
cargo test -p phalcom-semantic incremental
cargo test -p phalcom-semantic product_stability
cargo test -p phalcom-semantic type_store_revisions
cargo test -p phalcom-semantic callable_dependencies

rg -n 'RecordRowVarId' \
  phalcom-semantic/src/db \
  phalcom-semantic/src/session.rs
```

#### Completion checklist

- [ ] Row-tail semantic edits invalidate dependents.
- [ ] Known-prefix semantic edits invalidate dependents.
- [ ] Body-only edits do not unnecessarily invalidate stable signatures.
- [ ] Cold/incremental semantic results agree.
- [ ] Retained snapshots preserve old row denotation.
- [ ] No row solver session is cached/published.
- [ ] No solver row variable appears in DB/snapshot products.

---

### Task 14: Run full SC-3 certification, remove obsolete paths, verify performance boundaries, and mark delivery complete

#### Why

SC-3 crosses canonical type formation, subtyping, generic inference, literal composition, pattern decomposition, metadata, and incrementality. Passing isolated tests is insufficient if stale code can still silently close rows, route through `RecordAccess`, or make row solving dependent on interner history.

#### Architectural background

The final gate combines:

```text
behavioral tests
+ architectural deletion ledger
+ whole-workspace compilation/tests
+ deterministic metadata/incremental checks
+ performance sanity
```

The deletion ledger matters because several obsolete paths compile successfully while violating the intended semantics.

#### Current path through the code

The pinned baseline contains all of the following targets for removal/replacement:

```text
RecordAccess
TypeAnnotationExpr::Record { tail: _ }
RecordRowSolver::new(step_limit)
row_solver -> find_record_row(remainder)
metadata record_rows: false
metadata TypeData::Record -> always TypeNode::Record
closed-only project_record_fields naming
ordinary inference allocation for every generic kind
```

#### Exact files

- Modify only after all tests pass:
  - `docs/impl/semantic/semantic-completeness/README.md` — SC-3 status/links
- When committing this plan into the repository, create:
  - `docs/impl/semantic/semantic-completeness/sc-3/SC-3-open-record-rows-structural-typing-implementation-plan.md`
- No production file should be changed in Task 14 except to fix a failure uncovered by certification.

#### Exact symbols

Deletion/search ledger:

- `RecordAccess`
- discarded Record `tail: _`
- `RecordRowSolver::new(<integer>)`
- `TypeStore::find_record_row` calls from `row_solver.rs`
- `MetadataFeatures.record_rows: false`
- `TypeData::Record` exporter that ignores tail
- `project_record_fields` ambiguous closed-only helper
- row binder passed to `parameter_form`
- `InferenceSession::fresh_variable` for `KindId::RECORD_ROW`
- any `apply_row_*`/`row_generic_callable` parallel call engine
- `RecordRowVarId` in DB/metadata/type-meta

#### Exact insert/replace locations

1. Do not edit the README until all certification commands are green.
2. Build a requirement-to-test matrix from the 40 acceptance laws below.
3. Run focused suites first so failures are localizable.
4. Run workspace compile/test/lint gates.
5. Run deletion-ledger searches.
6. If a forbidden match remains:
   - determine whether it is a legitimate definition/API (`TypeStore::find_record_row` itself may remain) or stale usage;
   - remove stale usage;
   - add/regress a test proving the semantic law.
7. If ordinary closed Record-heavy checking regresses materially, profile before optimizing. The closed Record relation must not invoke the row solver merely because SC-3 exists.
8. Only after all gates pass, update SC-3 status in the semantic-completeness README.

#### Paste-ready code where safe

Deletion ledger:

```bash
rg -n 'RecordAccess' phalcom-semantic phalcom-type-meta
rg -n 'tail:\s*_' phalcom-semantic/src/types/annotation.rs
rg -n 'find_record_row' phalcom-semantic/src/types/row_solver.rs
rg -n 'RecordRowSolver::new\([0-9]' phalcom-semantic
rg -n 'record_rows:\s*false' phalcom-semantic/src/metadata
rg -n 'RecordRowVarId' \
  phalcom-semantic/src/db \
  phalcom-semantic/src/metadata \
  phalcom-type-meta
rg -n 'apply_row|row_generic_callable|row_callable_application' \
  phalcom-semantic/src/checker
rg -n 'KindId::RECORD_ROW.*fresh_variable|fresh_variable.*KindId::RECORD_ROW' \
  phalcom-semantic/src/checker
```

Expected deletion-ledger result:

```text
RecordAccess                                  -> no production matches
annotation Record tail discard               -> no matches
row_solver find_record_row                    -> no matches
integer-private row solver budget constructor -> no matches
record_rows: false                            -> no active exporter match
solver row IDs in DB/metadata                 -> no matches
parallel row call engines                     -> no matches
ordinary InferVar allocation for RecordRow    -> no matches
```

`TypeStore::find_record_row` may remain as a general canonical lookup API if another non-solver caller legitimately uses it; the forbidden case is row **solving** depending on it.

#### What not to change

- Do not mark SC-3 complete after focused tests only.
- Do not ignore cancellation/budget tests.
- Do not weaken tests to preserve stale behavior.
- Do not remove incremental reuse to force semantic consistency.
- Do not expand scope to effects, variant rows, mutable Records, general row application, or generic getters.
- Do not add performance optimizations without measurement if the semantic implementation is already within existing expectations.

#### Tests to add first

No new feature test should originate in this task. Before running certification, construct a checklist mapping every acceptance law below to an existing test introduced by Tasks 1–13. If any law lacks a test, return to its owning task and add the test there before proceeding.

Required named gates should include at least:

```text
formation
  checked_row_rejects_non_row_tail_parameter
  open_record_annotation_preserves_tail

solver
  remainder_solution_does_not_depend_on_store_history
  lacks_constraint_survives_variable_alias
  indirect_row_occurs_check_is_rejected
  row_solver_preserves_budget_and_cancellation

relation
  immutable_record_width_and_covariant_depth
  open_source_satisfies_closed_known_prefix

generic call
  row_polymorphic_call_preserves_remainder
  type_and_row_variables_infer_together
  repeated_row_parameter_conflict_is_rejected
  expected_result_can_select_row
  row_only_return_is_underconstrained_without_context

composition
  expected_record_guides_known_literal_fields
  open_record_expansion_preserves_tail
  map_expansion_does_not_fabricate_row

metadata
  open_record_exports_open_record_node
  row_tail_owner_changes_fingerprint

incremental
  row_prefix_edit_invalidates_dependent_call
  row_body_only_edit_preserves_signature_semantics
  cold_and_incremental_row_semantics_match
```

#### Tests to add afterward

Only add regression tests for concrete defects discovered by the full certification run. Name each new test after the semantic law it protects; do not create a generic `sc3_regressions` dumping ground.

#### Expected compiler errors

None are acceptable at completion.

Warnings introduced by SC-3 are failures when running the repository's `-D warnings` Clippy gate. Do not add blanket `#[allow(...)]` to silence architectural problems; use a narrowly justified allow only when existing repository policy supports it.

#### Rust explanations

Whole-workspace verification catches cross-crate enum exhaustiveness, metadata schema projections, and feature interactions that focused `phalcom-semantic` tests cannot. Search-based architecture gates are appropriate here because stale branches such as `RecordAccess::ReadWrite` can remain dead yet compile, undermining the declared model.

#### Verification commands

Run in this order:

```bash
cargo fmt --all -- --check

cargo check --workspace

cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-type-meta
cargo test -p phalcom-semantic
cargo test --workspace

cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Then run the deletion ledger exactly as shown above.

If repository benchmark/performance harnesses exist, compare at minimum:

```text
1. closed Record-heavy source before/after SC-3
2. row-polymorphic call-heavy source
3. cold analysis versus one-file incremental edit
```

Acceptance criterion: ordinary closed Record subtyping/literal synthesis must stay on the direct canonical path and must not instantiate `RecordRowSolver` unless a genuine row equation is present.

#### Completion checklist

- [ ] Every acceptance law below maps to at least one test.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo check --workspace` passes.
- [ ] Focused semantic test binary passes.
- [ ] `phalcom-type-meta` tests pass.
- [ ] `phalcom-semantic` crate tests pass.
- [ ] Full workspace tests pass.
- [ ] Clippy passes under repository policy.
- [ ] `RecordAccess` is gone.
- [ ] No Record annotation tail discard remains.
- [ ] Solver has no pre-interned-remainder dependency.
- [ ] Solver-local row IDs are absent from DB/metadata.
- [ ] Metadata exports open rows honestly.
- [ ] No parallel row-call engine exists.
- [ ] Maps remain outside row semantics.
- [ ] Nominal classes remain nominal.
- [ ] Underconstrained rows never default to empty.
- [ ] General row-valued generic application remains outside claimed support.
- [ ] SC-3 status documentation updated only after all gates pass.

---

# Suggested commit sequence

Use reviewer-sized commits. Each implementation commit should be preceded locally by its failing test and followed by focused verification.

1. `test(semantic): expose record row formation invariants`
2. `feat(semantic): add checked canonical record row construction`
3. `test(semantic): specify row materialization semantics`
4. `feat(semantic): add domain-aware generic instantiation`
5. `test(semantic): expose row solver history and lacks defects`
6. `refactor(semantic): normalize and bound record row solver`
7. `test(semantic): specify open record annotation lowering`
8. `feat(semantic): lower stable open record tails`
9. `test(semantic): specify immutable structural record relations`
10. `refactor(semantic): remove record access capability relation`
11. `test(semantic): specify record row inference domain`
12. `feat(semantic): add record-aware inference terms`
13. `test(semantic): specify row-polymorphic callable application`
14. `feat(semantic): integrate row inference into canonical calls`
15. `test(semantic): specify bidirectional record literal typing`
16. `feat(semantic): preserve open record shape through composition`
17. `test(semantic): specify open record pattern decomposition`
18. `feat(semantic): use known open-row fields in patterns`
19. `test(semantic): specify scoped open record preservation`
20. `feat(semantic): preserve scoped open records`
21. `feat(semantic): add record row diagnostics and explanations`
22. `test(metadata): specify open record publication`
23. `feat(metadata): publish canonical open records`
24. `test(semantic): specify incremental row stability`
25. `fix(semantic): complete row dependency fingerprints`
26. `docs(semantic): certify SC-3 completion`

Do not combine the solver rewrite, canonical call integration, and metadata publication into one commit. They are separate semantic/reviewer gates.

---

# Final semantic acceptance laws

SC-3 is complete only when all of these are true.

## Formation laws

1. `#{ a: A, b: B }` and `#{ b: B, a: A }` canonicalize equivalently.
2. Duplicate known fields are rejected.
3. Every Record field type is a proper `Type`.
4. An open tail parameter has kind `RecordRow`.
5. A `RecordRow` binder never becomes `TypeData::Parameter`.
6. Source open-row syntax is never silently closed.

## Structural relation laws

7. A wider immutable Record is a subtype of a narrower required-field Record.
8. Record field depth is covariant.
9. Width and depth compose.
10. An open source Record can satisfy a closed known-prefix target.
11. An unknown/open tail does not prove a field absent from the known prefix.
12. Different rigid row tails are not silently equated.
13. Nominal class layout is not a structural Record proof.
14. Map key sets are not Record rows.

## Row-solving laws

15. Solver results do not depend on prior row interning.
16. Lacks constraints follow variable aliases/substitutions.
17. Adding a lacks constraint after substitution validates immediately.
18. Direct recursive rows are rejected.
19. Indirect recursive rows are rejected.
20. Cancellation is not contradiction.
21. Budget exhaustion is not contradiction.
22. Underconstrained row variables remain underconstrained.
23. Solver variables never escape publication.
24. Successful solutions are deterministic modulo canonical identity.

## Generic-call laws

25. Type and row parameters can infer simultaneously.
26. Repeated occurrences of one row parameter correlate one remainder.
27. Incompatible repeated remainders conflict.
28. Argument and expected-result row constraints meet under the SC-2 policy.
29. Expected-result inference does not manufacture established value evidence.
30. A row parameter appearing only in the result remains underconstrained without context.
31. Empty remainder is inferred only when subtraction proves it.
32. Return materialization substitutes both type and row bindings.
33. Failed materialization is explicit; it never returns the unspecialized original by fallback.
34. No second callable-application authority exists.

## Composition/pattern laws

35. Expected Record known fields guide child literal checking.
36. Actual Record syntax still determines the synthesized literal shape.
37. `**Record` preserves a statically known open tail.
38. Extending an open Record enforces lacks/disjointness.
39. `**Map` does not fabricate static field identities.
40. Known open-row fields can be decomposed precisely by Record patterns.
41. A possible field in an unknown tail is not treated as guaranteed.

## Publication laws

42. Closed Records export as `TypeNode::Record`.
43. Open Records export as `TypeNode::OpenRecord`.
44. Stable row-tail owner/index participates in metadata fingerprints.
45. Raw `RecordRowId` indexes are not durable identity.
46. `RecordRowVarId` never enters DB, metadata, reflection, or snapshots.
47. Scoped open Records use the existing `ScopedTypeNode::OpenRecord` schema.

## Scope laws

48. Records remain immutable structural products.
49. `RecordAccess` is absent from Record semantics.
50. Maps remain mutable dynamic-key collections with no key-set row typing.
51. Nominal classes remain nominal.
52. General row-valued nominal generic application is not claimed complete.
53. General row-valued transparent-alias application is not claimed complete.
54. General row-valued type-lambda application is not claimed complete.
55. Generic getters remain owned by SC-7.
56. Effect rows and variant rows remain separate future domains.

---

# Rust implementation notes and common failure modes

## Borrow checker around `TypeStore`

The most common SC-3 borrow failure will be holding:

```rust
let row = store.record_row(row_id);
```

while recursively invoking something that needs `&mut TypeStore`.

Use an owned snapshot of the small semantic components:

```rust
let (fields, tail) = {
    let row = store.record_row(row_id);
    (row.fields.to_vec(), row.tail)
};
```

Then recurse. Do not use `unsafe` or interior mutability to evade this normal ownership boundary.

## Exhaustive enums are migration checklists

Adding:

```text
InferenceTerm::Record
ScopedTypeData::OpenRecord
new row outcomes
new DiagnosticCode variants
```

should create compile errors in incomplete traversals. Fix each match explicitly. A wildcard arm that converts an unknown structural term to `UncheckedExpression`, the original type, or `Blocked` defeats the migration.

## Canonical IDs versus solver-local IDs

Canonical/stable-within-store semantic IDs:

```text
TypeId
RecordRowId
TypeParameterId
```

Solver-local IDs:

```text
InferVarId
RecordRowVarId
```

Both are integer newtypes, but they belong to different lifetimes and semantic domains. Do not add conversions between them.

## `Result` versus invariant `expect`

Use `Result` at untrusted/fallible semantic boundaries:

```text
source annotation lowering
row solver zonking
combined generic materialization
metadata projection of potentially malformed canonical input
```

Use `expect` only after a preceding semantic proof makes failure an internal invariant breach, and name that invariant in the message.

Never write:

```rust
materialize(...).unwrap_or(original_type)
```

for row-aware specialization.

## Determinism

Sort or otherwise stabilize:

```text
Record known fields
reported underconstrained stable parameters
conflict contributor lists
explanation roots where existing infrastructure expects order
```

Never let `HashMap` iteration order reach diagnostics, fingerprints, or serialized metadata.

## Solver/store mutation discipline

A failed, blocked, cancelled, or budget-exhausted row solve should not intern speculative rows. Canonicalization occurs during successful zonking/publication. This keeps store growth and semantic identity independent of failed exploration history.

---

# Plan self-verification record

This plan was written against `aureat/phalcom-lang` `main` at:

```text
abb2b5d80654e2525d68f4ea8ff9d32b810330b3
```

The repository was inspected for these live symbols before the plan was generated:

```text
phalcom-semantic/src/types/row.rs
  RecordRowField
  RecordRowTail
  RecordRowData
  RecordAccess

phalcom-semantic/src/types/row_solver.rs
  RecordRowVarId
  RecordRowTerm
  RecordRowLacks
  RecordRowSolution
  RecordRowSolveResult
  RecordRowSolver

phalcom-semantic/src/types/store.rs
  parameter_form
  intern_record_row
  find_record_row
  record
  record_type

phalcom-semantic/src/types/environment.rs
  TypeEnvironment
  TypeView::materialize
  materialize_view

phalcom-semantic/src/types/substitution.rs
  TypeSubstitution::apply

phalcom-semantic/src/types/annotation.rs
  TypeLevelBinding
  type_level_binding_for_parameter
  TypeFormationOutcome
  TypeFormationSite
  GenericBinderSite
  ScopedBinderStack / lower_scoped_type_form
  TypeFormationInvalid::UnsupportedOpenRecordTail
  TypeResolver
  ScopedTypeResolver
  SimpleTypeResolver
  resolve_type_form
  resolve_generic_signature
  direct TypeAnnotationExpr::Record { tail: _ }

phalcom-semantic/src/types/relation.rs
  check_record_row_subtype
  TypeData::Record relation arm

phalcom-semantic/src/checker/inference.rs
  InferenceTerm
  InferenceSession
  instantiate_generic_signature

phalcom-semantic/src/checker/call.rs
  apply_generic_callable_inner
  apply_resolved_callable

phalcom-semantic/src/checker/composition.rs
  project_record_fields

phalcom-semantic/src/checker/expression.rs
  synthesize_record_literal
  RecordLiteralEntry::Expansion path

phalcom-semantic/src/checker/pattern.rs
  resolve_record_pattern

phalcom-semantic/src/metadata/export.rs
  export_type_form
  TypeData::Record arm
  TypeNode::OpenRecord fingerprint arm
  MetadataFeatures.record_rows

phalcom-type-meta/src/type_node.rs
  OpenRecordTypeRef
  TypeNode::OpenRecord

phalcom-type-meta/src/scoped_type.rs
  ScopedRecordTailRef
  ScopedOpenRecordTypeRef
  ScopedTypeNode::OpenRecord
```

Repository-specific corrections deliberately captured by this plan:

1. The row representation is already unified; SC-3 does not add a second open-Record `TypeData` variant.
2. The existing solver test pre-interns the empty remainder and therefore masks the `find_record_row` history-dependence defect.
3. Record literal `**` expansion is already represented in the AST/parser/checker; SC-3 changes static typing, not syntax.
4. `RecordAccess` is stale for immutable Records and is removed rather than generalized.
5. The metadata schema already has open Record nodes; only semantic export/preservation is incomplete.
6. The WIP SC-1 foundation is now partially implemented at this pinned baseline: domain-aware binders, explicit formation outcomes, aliases, side-aware generic scope, and capture-safe scoped lowering are live; direct/scoped open Record tails remain the SC-3 handoff gaps.
7. SC-2's canonical `apply_resolved_callable` funnel exists today and remains the only call-application authority.
8. General row-valued generic application remains outside SC-3 because current applied-type and beta-reduction argument representations are `TypeId`-only.


# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P2
- Task/Gate: T4 — mixed term/row inference in heterogeneous applications
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `1c9985b6d449d8680667a91651ea64b2808c44ca`
- Current revision: `1c9985b6d449d8680667a91651ea64b2808c44ca` (working tree changes)

## 2. Escalation trigger

Mandatory triggers 3 and 5: composing mixed inference requires changing the row
solver's owner and lifetime. T4's local STOP/CONSULT rule also applies.

## 3. Required invariant

- `InferVarId` and `RecordRowVarId` remain distinct identities and use distinct solvers.
- No row-solver identity enters a canonical `TypeId`, metadata, or published semantic
  product.
- A row variable referenced by a retained inference constraint remains owned until the
  query context that owns that constraint is dropped.
- A child frame may retain a relation to an outer row variable without treating that
  outer variable as the child's unresolved result or materializing it as a concrete row.

## 4. Expected architecture

T4 requires `InferenceStaticArgument::{Term, RecordRow}` and category-directed
unification. A query-context coordinator owns both the ordinary `InferenceSession` and
a separate `RecordRowSolver`; `InferVarId` and `RecordRowVarId` remain distinct.

## 5. Observed behavior

`type_id_to_inference_with_rows` converts `Applied` and `StaticData` children through
`StaticArgument::as_type_id()`. A `RecordRow` child makes the entire form fall back to
opaque `Canonical(TypeId)`, hiding ordinary terms in neighboring positions.

At incident discovery, `InferenceSession` was retained by `CheckingContext` while each
call created a stack-local row solver. This made row identities unsafe to retain in
nested inference terms and made whole-solver completion misclassify unrelated outer
variables. The live correction now gives the query context one coordinator with a
separate row solver, frame-owned row variables, and a lightweight per-call binding view.

## 6. Minimal reproduction

The confirmed original failure seam was structural: any `TypeData::Applied` or
`TypeData::StaticData` containing `StaticArgument::RecordRow` took the `Canonical(ty)`
fallback in `InferenceSession::type_id_to_inference_with_rows`. The live implementation
now preserves both row and ordinary children in the heterogeneous inference envelope.

## 7. Relevant code path

```text
CheckingContext inference-context map
  -> context-owned InferenceCoordinator
     -> ordinary InferenceSession
     -> distinct RecordRowSolver<InferenceTerm>
call::apply_generic_callable_in_context
  -> per-call GenericApplicationSession binding view
  -> type_id_to_inference_with_rows
  -> heterogeneous Applied / StaticData inference terms
  -> coordinator alternates row shape and ordinary field constraints
```

Relevant files: `phalcom-semantic/src/checker/{context.rs,inference.rs,call.rs,row_inference.rs}`
and `phalcom-semantic/src/types/row_solver.rs`.

## 8. Scoped implementation diff

T4 changes are present in `inference.rs`, `row_inference.rs`, `context.rs`, `call.rs`, and
`types/row_solver.rs`. The existing T7 work remains recorded under INC-C3-7. No test
source was changed.

## 9. Evidence

- `InferenceSession::close_frame` marks frames closed but retains their graph state.
- `CheckingContext` stores inference sessions in its context-indexed map.
- `call.rs` allocates `GenericApplicationSession` locally, adds ordinary constraints to
  the context session, and solves rows through the local wrapper.
- `RecordRowSolver::solve_many` checks all allocated variables for underconstraint.
- Read-only T4 architecture review and adviser consultation confirmed that an inference
  envelope alone is insufficient and identified query-context coordinator ownership as
  the safe boundary.
- The current query coordinator owns the row solver separately from `InferenceSession`;
  that session keeps only a non-owning handle for row-aware traversal/materialization.
- Row shape solving emits same-label `InferenceTerm` pairs back to ordinary inference;
  the coordinator shares budget/cancellation and repeats propagation to a fixed point.
- Frame row-parameter mappings are inherited by nested views. A child result that still
  depends on an unresolved ancestor row is retained as a same-context symbolic result.
- Cross-domain occurs traversal follows row payload terms and row substitutions before
  row-tail bindings are accepted.

## 10. Implementer's causal hypothesis

The defect is both representation opacity and solver-lifetime mismatch. The row child is
opaque because the inference representation assumes type-only applied arguments; once the
representation becomes heterogeneous, row equations must route to a solver whose lifetime
matches the retained inference constraints.

## 11. Alternative hypotheses

A wrapper could decompose every mixed equation before it enters the context-owned
`InferenceSession`, keeping row IDs local. It is safe only if nested expectations and
retained constraints prove that no row ID can escape the call. Current call paths do not
establish that property.

## 12. Attempts already made

The architecture was consulted before implementation. The first implementation draft
put the row solver directly inside `InferenceSession`; it was replaced with the accepted
coordinator ownership model before continuing. The current semantic library compiles.

## 13. Why local authority is insufficient

The fix changes solver ownership/lifetime and the meaning of frame-scoped completion.
Those are explicit T4 STOP/CONSULT concerns; an implementer cannot resolve them by
adding a variant to `InferenceTerm` or by converting a local solver into a shared solver
without changing completion semantics.

## 14. Decision and plan amendment

Keep separate ordinary and row identities. Introduce a query-context-owned inference
coordinator that owns both existing solvers and routes mixed equations by category.
Row variables live for the query-context lifetime even after a frame closes. Solve/check
only the row roots required by the completing frame's result; preserve unresolved
cross-frame equations for the parent, and never publish a child row that still depends on
an unresolved outer variable. Do not hold a `RefCell` borrow across argument analysis or
nested calls.

Generalize the existing row solver over field payload `V`, retaining `TypeId` as its
default/current payload. The row solver owns only label/tail shape and emits same-label
payload pairs to the coordinator instead of treating host-language equality as semantic
field equality. Instantiate it with `InferenceTerm` for mixed application inference:
canonical row fields become `InferenceTerm::Canonical(TypeId)`, while inferred fields
remain ordinary solver terms. The coordinator translates emitted pairs into ordinary
`Equivalent` constraints and alternates ordinary propagation, queued row equations,
row-shape solving, and field-pair processing to a fixed point under shared budget and
cancellation. Row materialization resolves field payloads through ordinary inference and
rejects unresolved/cyclic cross-domain dependencies before canonical `RecordRow` creation.
Cross-domain occurs traversal follows type terms in row fields and row substitutions.
This requires no fake `TypeId` and no merged identity space.

`GenericApplicationSession` is a lightweight per-call binding view over the shared
coordinator. A child whose row solution depends on an unresolved ancestor retains a
same-context symbolic result and skips canonical generic instantiation. Only root-context
completion diagnoses unresolved rows before state teardown.

Focused acceptance must cover mixed row/type terms inside `Applied` and `StaticData`,
domain/constructor mismatch, distinct variable identities, ordinary and cross-domain
occurs checking, retained underconstraint, child-to-parent row dependencies that later
resolve, inference through shared row labels, row-tail extension with inferred fields,
conflicting field values, and unchanged ordinary inference. This implementation detail
is part of the existing T4/checkpoint decision and does not require a new language ADR.

Verification: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib` passed
after the current T4 edits. No tests were added or run; behavior, nested-frame resolution,
and cold/incremental parity remain unverified.

Status: IMPLEMENTED / UNVERIFIED — the coordinator, mixed-term routing, cross-domain
occurs checks, and symbolic ancestor-row path are present; focused acceptance remains open.

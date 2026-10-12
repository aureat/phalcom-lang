# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4
- Plan: LANG007.C4.P7
- Task/Gate: T7 / G7
- Repository: /Users/altunhasanli/dev/phalcom/phalcom
- Branch: main
- Starting revision: 3f8cd71358b5c5155ac0a92c7ab232e5a86c4fb6
- Working-tree base revision: 3f8cd71358b5c5155ac0a92c7ab232e5a86c4fb6; task changes are pending delivery.

## 2. Escalation trigger

The active plan's residual identity and specialization semantics are
insufficient for direct child-residual escape. This triggers mandatory
consultation because canonical identity and the cross-layer semantic contract
must change (shared protocol RED triggers 4 and 8; repository escalation
conditions 3, 4, and 9).

## 3. Required invariant

For every supported residual family, cold exact evaluation must agree
semantically with open evaluation followed by specialization. A residual that
escapes from an active application must retain every exact argument binding
and canonical proof fact that can affect its eventual result. Normalization is
canonical, source-syntax-free, and fails closed on conflicting concrete
bindings.

## 4. Initial architecture before the first consultation

T7 specializes Parameter, Apply, Proof, and MemberProjection residuals and
re-interns a normalized result. Section 18.1 names exact kinded/value arguments
and canonical proof facts, while the current environment key contains only
type bindings and proof constraints. Proof and MemberProjection payloads do not
carry that environment.

## 5. Initial observed behavior before the partial T7 edits

`StaticResidualData::Proof` contains only its proof goal and result type;
`StaticResidualData::MemberProjection` contains only the canonical projection
and result domain. `StaticEvaluator::intern_proof_residual` and
`intern_member_projection_residual` intern those payloads without an
environment. The evaluator's active application key includes its exact
arguments and current environment, but `FinishStaticApplication` can return a
child residual directly, so the enclosing Apply is not guaranteed to survive.

`StaticSpecializationEnvironmentKey` contains canonical type-parameter
bindings and proof constraints but has no formal static parameter to exact
ordinary value argument binding. Active branch proof environments can also
change evaluation of nested goals/applications.

## 6. Minimal reproducer

The structural failure is direct: an open Proof or MemberProjection is created
while an application has exact arguments or branch proof facts; the returned
child residual contains neither that environment nor a guaranteed enclosing
Apply node. Specializing that residual alone cannot reconstruct the original
effective environment. T7 has not yet added source changes or acceptance tests
for this path at the time the incident was opened. The partial edits described
below have since added environment fields but have not closed the path.

## 7. Relevant code path

```text
active application / branch proof environment
  -> evaluate proof or member projection
  -> intern child residual without environment
  -> FinishStaticApplication may return child residual
  -> later specialization receives only the child payload
```

Relevant live definitions include:

- `phalcom-semantic/src/static_eval/residual.rs` — residual payloads and
  interning API;
- `phalcom-semantic/src/static_eval/application.rs` — current normalized
  environment key;
- `phalcom-semantic/src/static_eval/machine.rs` — residual creation and
  application completion;
- `docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-luna-patch-grade-implementation-plan.md`
  §§6.3, 18.1, and T7.

## 8. Scoped implementation diff

Before consultation, no T7 source edits were made. After the adviser decision,
the plan amendment and this packet were recorded. Partial, unverified source
work then added an exact formal-static-parameter binding carrier and
normalization/merge APIs in `static_eval/application.rs`, environment fields
and fingerprint participation for Proof and MemberProjection payloads in
`static_eval/residual.rs`, and an evaluator helper that gathers current
application arguments and branch facts in `static_eval/machine.rs`. The
specialization engine, supplied-environment composition, and focused T7 tests
are not implemented. The user requested review and consultation before source
work resumes; these partial changes are not accepted as a completed slice.
T6's checked projection bridge is unchanged.

## 9. Evidence

Initial baseline evidence: Proof and MemberProjection payloads did not carry an
environment, their interning helpers accepted no environment, and the
specialization key lacked exact ordinary value arguments. This motivated the
first amendment.

Follow-up review evidence against the partial implementation:

- `effective_specialization_environment` at
  `phalcom-semantic/src/static_eval/machine.rs:949-1012` currently merges every
  active frame strictly, so recursive re-entry with a changed value for the
  same formal is rejected.
- `active_generic_evidence` at
  `phalcom-semantic/src/static_eval/machine.rs:2956-2991` consumes canonical
  parameter bindings and equality/subtype facts; it ignores local bindings,
  local equalities, and `HasMember`.
- `resolve_type_term_bounded` at
  `phalcom-semantic/src/static_eval/machine.rs:2993-3025` consumes canonical
  parameter bindings and equalities; it ignores checker-local structures.
- `StaticSpecializationEnvironmentKey::new` at
  `phalcom-semantic/src/static_eval/application.rs:116-126` constructs a key
  without normalization. No repository call site was found.
- `StaticResidualStore::content_fingerprint` at
  `phalcom-semantic/src/static_eval/residual.rs:389-395` claims allocation
  order independence but hashes `TypeId` and `StaticValueRef` directly; its
  supported contract must be explicitly store-lineage-local.
- The plan requires cold exact/open-specialized semantic parity, idempotence,
  and no-new-facts identity for each supported residual family.

## 10. Implementer's causal hypothesis

Residual identity currently describes the local predicate/projection but not
the effective specialization environment that produced it. Direct child
escape therefore drops semantically relevant context, making T7's claimed
specialization parity impossible to guarantee.

## 11. Alternative hypotheses

- Retain an environment-bearing Apply wrapper around every child residual.
  This changes residual shape/identity and cannot be assumed because current
  completion can return a child directly.
- Keep the current payload and rely on its proof goal/projection alone. This
  loses exact ordinary value arguments and branch facts and does not satisfy
  cold/open parity.

## 12. Attempts already made

The initial partial T7 implementation added the environment carrier, residual
payload fields/fingerprints, and active-environment capture. A subsequent
read-only review identified a recursive-activation collision in the capture
helper and additional composition/identity contract gaps. No specializer or
T7 acceptance gate has been completed.

## 13. Why local authority is insufficient

The fix changes residual payload identity and the contract for composing
captured and supplied environments. Choosing whether to duplicate context in
the child payload or guarantee a retained wrapper is architectural; neither
can be selected as a mechanical local detail.

## 14. Decision requested

Should Proof and MemberProjection residuals capture the canonical effective
specialization environment at creation, including exact formal static
parameter-to-kinded/value bindings and canonical proof facts, so they can be
specialized after direct escape? Specify composition, normalization, conflict
handling, identity, and focused acceptance.

## Adopted adviser decision — 2026-10-12

The adviser ruled: **stop T7 pending a Level 2 plan amendment.** Proof and
MemberProjection payloads must capture a normalized effective environment at
creation. Extend the current environment contract with formal-parameter to
exact kinded/value argument bindings; the existing type bindings and
constraints alone do not identify exact value arguments. Compose supplied
substitutions through captured bindings, coalesce duplicate identical
bindings, fail closed on conflicting concrete bindings, canonically union
captured and supplied proof facts, check them through existing proof
authority, and re-intern with environment included in residual identity. Do
not rely on an enclosing Apply wrapper. Keep payloads free of source ranges,
source syntax, and query-local proof identity. T6 remains unchanged.

Required focused coverage: direct child-residual escape and later exact
specialization for both residual families; distinct environments yield
distinct identities; reordered/duplicate equivalent environments normalize
equally; cold exact equals open then specialize; idempotence; no-new-facts
identity; and conflicting bindings fail closed.

## Implementation resolution — 2026-10-12

The first Level 2 amendment is recorded in the active P7 plan §§6.3, 18.1, and
T7, and in the checkpoint consultation ledger. Partial environment-carrier,
payload, fingerprint, and capture-helper edits exist, but no residual
specializer or T7 acceptance tests exist. At that point, `git diff --check`
passed and no Cargo check or behavioral tests had been run after the partial
edits. The user then requested read-only review/consultation before
implementation continues. T7 remained paused and unverified.

## Follow-up read-only review — 2026-10-12

The review returned a blocking verdict: **amend again and keep T7 stopped until
the amendment is recorded**. It found that
`StaticEvaluator::effective_specialization_environment` currently flattens all
active application frames through strict environment merge. For recursive
`Grow<T> -> Grow<Next<T>>`, the same formal has different exact arguments in
the outer and inner activations; this is normal activation shadowing, not a
conflicting supplied specialization. The recommended capture rule is nearest
active frame per repeated formal, while preserving non-shadowed outer bindings
and canonical proof facts. Strict conflict handling remains required for
bindings supplied together at specialization time.

Additional findings: captured substitutions must be composed through bound
kinded types before conflict checks and residual re-evaluation; the public
`StaticSpecializationEnvironmentKey::new` constructor bypasses normalization;
and direct `TypeId`/`StaticValueRef` hashing does not support a cross-lineage
fingerprint claim. The current proof consumers do not read local branch
bindings/equalities or `HasMember`, so the amendment bounds residual proof
identity to canonical facts they actually consume. Raw checker-local identity
remains excluded.

The active plan records the follow-up amendment in T7 and clarifies §21.3's
fingerprint boundary. Source work remains paused at the consultation gate.
`RUSTFLAGS='' cargo check -p phalcom-semantic --tests` now passes after
mechanical import, iterator, and enum-pattern updates. No tests were executed.
T7/G7 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

## Updated consultation question

Confirm nearest-active-frame shadowing for recursive re-entry of the same
static definition, with strict conflict failure limited to conflicting
bindings supplied at one specialization layer after substitution. Confirm
that canonical-only proof projection is sufficient for T7 given the current
consumers above, and that P7 fingerprints may remain explicitly
store-lineage-local.

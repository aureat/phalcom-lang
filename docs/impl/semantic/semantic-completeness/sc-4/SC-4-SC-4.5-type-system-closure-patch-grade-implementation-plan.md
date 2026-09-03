# SC-4 / SC-4.5 — Type-System Closure Patch-Grade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended for a supervised continuous implementation session) or `superpowers:executing-plans` to execute this plan checkpoint-by-checkpoint. Do not reinterpret the semantic design silently when mechanics drift.

**Goal:** Complete Phalcom's currently ratified static type-system correctness and completeness closure in two integrated stages: SC-4 closes higher-kinded and nested executable generic inference; SC-4.5 proves that the rest of the language consumes the canonical type system consistently.

**Architecture:** Keep `phalcom-semantic` as the sole static semantic authority. Extend the existing `InferenceSession` / `apply_resolved_callable` architecture rather than introducing a second solver. Make nested inference ownership explicit through a query-local inference context and per-application frames; generalize constructor-kinded inference over the existing `KindId`/type-lambda kernel; implement generic getters as zero-value-argument applications of canonical callable declarations; then certify bidirectional expression typing, canonical type relations, flow/refinement, ADT/GADT elimination, source constraints/variance/aliases/Families, SC-3 row integration, epistemic state, and cold/incremental equivalence.

**Tech Stack:** Rust, `phalcom-ast`, `phalcom-semantic`, `phalcom-core`, existing `TypeStore`, `TypeLambdaArena`, `TypeEnvironment`/`TypeView`, `InferenceSession`, semantic snapshot/explanation products, Phalcom source fixtures, Cargo integration tests.

**Requirements analysis:** `SC-4-type-system-correctness-completeness-closure-requirements-analysis.md` produced for this program. Existing repository specifications remain authoritative where they describe ratified semantics, especially SC-1, SC-2, SC-3, the generic inference specification, associated-family specifications, and `phalcom-core/tests/core/monads/LAWS.md`.

**Prepared against remote repository:** `aureat/phalcom-lang`

**Prepared against branch:** `main`

**Pinned remote HEAD:** `2b6f28a943d9a76ca33f66763b6a1d391c623424`

**Pinned commit:** `fix(parser): parse compact generic type-lambda applications`

**Remote inspection limitation:** connected GitHub exposes the remote repository and branch state only. This plan does not claim knowledge of an implementing agent's local working tree, uncommitted changes, stashes, or local-only branches.

**CI observation at plan preparation:** GitHub exposed no combined status entries and no workflow runs for the pinned HEAD. This plan therefore treats local checkpoint execution as the source of executable evidence and makes no claim that the pinned remote HEAD is globally green.

**Intended repository destination if committed:**

```text
docs/impl/semantic/semantic-completeness/sc-4/
    SC-4-SC-4.5-type-system-closure-implementation-plan.md
    SC-4-SC-4.5-implementation-state.md
```

---

# 1. Global constraints

These constraints apply to every checkpoint.

1. `phalcom-semantic` remains the sole authority for static type semantics.
2. `InferVarId != TypeId` remains absolute. No inference metavariable is interned into `TypeStore`.
3. `RecordRow` remains a distinct SC-3 inference domain. SC-4 must not encode row variables as ordinary type inference variables.
4. The existing `TypeStore` / `TypeLambdaArena` are the canonical authorities for type constructor kind, alpha-stable lambda representation, and beta/application semantics.
5. `types/specialization.rs::specialize_receiver_to_owner` remains the one owner-relative receiver specialization authority.
6. `checker/call.rs::apply_resolved_callable` remains the canonical executable callable-application funnel.
7. The new union-receiver implementation at the pinned baseline is retained. SC-4 validates HKT/nested-inference behavior through it; it does not reimplement union dispatch.
8. Generic getters are applications of ordinary canonical callable declarations with zero value arguments. They do not get a private solver.
9. Generic getter selectors remain ordinary getter selectors. Generic instantiation must not alter runtime selector or `CallableId` identity.
10. Ordinary structural callable values remain monomorphic unless a retained semantic denotation identifies an actual generic declaration that may be reinstantiated.
11. Expected context is selection/control information. It does not fabricate stronger runtime value evidence.
12. A declaration restriction such as `where T <: Number` constrains a selected solution; it is not a default selecting candidate.
13. Caller-owned canonical generic parameters remain rigid inside nested callee inference.
14. `Self` means the semantic receiver in the appropriate role, not merely the lexical declaring owner.
15. Constructor reconstruction is driven by formal/actual type structure and kind. Do not assume the varying parameter is the final nominal type argument.
16. Unsupported/unavailable typing does not become `Object` or `Dynamic` as a recovery success.
17. Effects, raise sets, termination, `@total`, contract VC generation, SMT/proof backends, rank-N polymorphism, first-class `forall`, public kind polymorphism, generic setters, and generic indexers are not part of this program.
18. Until effects are authoritative, heap/alias-sensitive refinement across unknown calls is conservatively invalidated rather than assuming purity.
19. Tests are scheduled at semantic checkpoint boundaries. Do not add or run one behavioral test per mechanical edit.
20. If repository drift contradicts a semantic law in this plan, stop and escalate with repository evidence; do not silently redesign.

---

# 2. Repository architecture map

The implementing agent should understand these ownership boundaries before editing.

## 2.1 Syntax and type formation

```text
phalcom-ast/src/ast.rs
    source AST identities and declaration shapes
    current GetterDef has no callable-local generics

phalcom-ast/src/parser.rs
    source grammar
    current class/enum getter paths explicitly reject generic_parameters
    latest HEAD also contains compact <<... type-lambda application token fission

phalcom-semantic/src/types/annotation.rs
    source type/kind/generic-signature formation
    resolve_generic_signature
    TypeFormationOutcome
    TypeLevelBinding
    GenericBinderSite
```

## 2.2 Canonical type kernel

```text
phalcom-semantic/src/types/store.rs
    canonical TypeId / KindId store
    TypeData
    apply_kind
    apply_type_form
    parameter_form
    lambda/row/family arenas

phalcom-semantic/src/types/type_lambda.rs
    ScopedTypeData
    TypeLambdaData
    TypeLambdaArena
    capture-safe substitution
    alpha-stable scoped representation
    beta_reduce

phalcom-semantic/src/types/parameter.rs
    TypeParameterId
    TypeParameterOwner
    GenericSignature
    GenericConstraint
    SelfTypeTerm
```

## 2.3 Generic inference and application

```text
phalcom-semantic/src/checker/inference.rs
    InferenceTerm
    InferenceSession
    InferenceVariable
    ConstraintOrigin / InferenceConstraintRole
    solve / subtype / unify / materialization
    term_for_expected
    terminal outcome algebra

phalcom-semantic/src/checker/expected.rs
    ExpectedType
    ExpectationOrigin
    inference-shaped contextual type propagation

phalcom-semantic/src/checker/call.rs
    CallableApplicationTarget
    CallPremise
    ApplicationArgument
    PreAnalyzed
    apply_generic_callable_inner
    apply_resolved_callable
    apply_union_resolved_call

phalcom-semantic/src/checker/typed_expr.rs
    TypedExpression
    non-published checker-local expression result
    currently publishes only TypeKnowledge plus constraints/provenance,
    with no explicit solver-context/symbolic result field
```

## 2.4 Receiver and relation algebra

```text
phalcom-semantic/src/types/specialization.rs
    specialize_receiver_to_owner
    ReceiverSpecialization
    canonical owner-relative generic environment + Self binding

phalcom-semantic/src/types/relation.rs
    check_subtype_bounded
    nominal/generic-supertype relation
    variance
    callable relation
    tuple/union/exact-case/family relation
    current Record relation still delegates to the pre-SC-3 row access model
```

## 2.5 Source callable publication and getters

```text
phalcom-semantic/src/checker/declaration_signature.rs
    CallableSyntaxRef
    callable_id_for_syntax
    semantic_signature_for_syntax
    source-to-canonical CallableSemanticSignature boundary

phalcom-semantic/src/checker/expression.rs
    analyze_expression / analyze_expression_inner
    synthesize_get_property
    getter access already calls apply_resolved_callable,
    but currently passes ExpectedType::None
```

## 2.6 Associated/family generic application

```text
phalcom-semantic/src/checker/associated.rs
    SpecializedAssociatedMember
    BoundBehavioralMember
    FamilyApplicationResolution
    target retention
    shared specialize_receiver_to_owner consumption
```

Do not create a second family generic solver.

## 2.7 Flow and ADT/GADT elimination

```text
phalcom-semantic/src/checker/flow/state.rs
    FlowState

phalcom-semantic/src/checker/flow/transfer.rs
    flow predicate transfer

phalcom-semantic/src/checker/control.rs
    condition splits / branch integration

phalcom-semantic/src/checker/loop_analysis.rs
    LoopFixpoint / convergence

phalcom-semantic/src/checker/pattern.rs
    pattern typing and branch facts

phalcom-semantic/src/checker/gadt_proof.rs
    solve_gadt_branch_proof

phalcom-semantic/src/checker/exhaustiveness.rs
    pattern-space elimination, usefulness/exhaustiveness integration
```

## 2.8 Tests

```text
phalcom-semantic/tests/semantic.rs
phalcom-semantic/tests/semantic/mod.rs

phalcom-semantic/tests/semantic/foundations/
phalcom-semantic/tests/semantic/capabilities/
phalcom-semantic/tests/semantic/adts/
phalcom-semantic/tests/semantic/families/
phalcom-semantic/tests/semantic/incremental/
phalcom-semantic/tests/semantic/integration/
phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md

phalcom-core/tests/core/monads/
    53 MON-* conformance laws; preserve as a protected HKT integration suite

phalcom-core/tests/core/either/
    neighboring generic/ADT integration suite

phalcom-ast/tests/parser.rs
    parser-level type syntax and compact type-lambda tests
```

---

# 3. Source-of-truth register

| Concern | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Proper/canonical type | `TypeStore` / `TypeId` | snapshots, metadata, LSP, compiler | solver-local term |
| Kind | `TypeStore::kind_of`, `KindId`, `apply_kind` | formation, inference, diagnostics | ad hoc arity check |
| Type lambda | `TypeLambdaArena` + `TypeData::Lambda` | application, metadata, inference candidates | new HKT expression graph |
| Generic declaration binders | `GenericSignature` + `TypeParameterId` | call inference, reflection, metadata | parameter name string |
| Generic application constraint graph | existing `InferenceSession`, extended with explicit context/frame ownership | call result, proof/explanation | second HKT solver |
| Receiver specialization | `specialize_receiver_to_owner` | call, associated/family, Self specialization | feature-specific hierarchy walker |
| Runtime call target | canonical `CallableId` / retained `InvocationTargetId` | compiler/runtime lowering | type-directed selector mutation |
| Generic getter declaration | ordinary `CallableSemanticSignature` with getter selector | property access, tooling | getter-specific type descriptor |
| Published expression type | `TypeKnowledge` backed by canonical type | `ExpressionAnalysis`, snapshot/tooling | checker-local symbolic inference term |
| Record row | SC-3 canonical row representation and row solver | Record relation/materialization | ordinary `InferVarId` |
| Flow state | `FlowState` + canonical transfer/join owners | body analysis, summaries | expression-local parallel flow map |
| GADT branch reachability | `solve_gadt_branch_proof` + exact case products | pattern typing/exhaustiveness/refinement | pattern-local handwritten equality solver |

---

# 4. Tempting wrong fixes — explicitly prohibited

1. **Do not make `InferVarId` process-global using an atomic counter.** It would hide ownership rather than model it, introduce nondeterministic IDs, and still leave cross-frame solution ownership undefined.
2. **Do not intern unresolved inference variables as `TypeData`.** Solver state must remain query-local.
3. **Do not convert caller-owned `TypeData::Parameter` into fresh callee variables.** The recent inference regression proved that caller parameters must remain rigid.
4. **Do not “fix” nested inference by materializing expected terms earlier.** `term_for_expected` exists precisely because early materialization loses higher-order constraints.
5. **Do not increase solver iteration/budget limits to cure non-progress.** A repeated unchanged state is not progress.
6. **Do not special-case `Either` or `Box` by declaration name.** Constructor reconstruction must be structural and kind-directed.
7. **Do not assume a partial constructor always fixes a prefix and abstracts a suffix.** Arbitrary formal/actual correspondence may require a lambda with a middle or leading hole.
8. **Do not synthesize `<X> =>> Box<X>` when canonical `Box` is already the exact constructor solution.**
9. **Do not let `GenericWhere` upper bounds select an otherwise unconstrained variable.**
10. **Do not add generic type arguments to selector identity.** `#empty` remains the same getter selector for every static instantiation.
11. **Do not add a getter-only application engine.** Getter access already reaches `apply_resolved_callable`; extend that path.
12. **Do not create a new generic-family solver.** Retained invocation targets and ordinary call application are already the intended bridge.
13. **Do not reimplement union receiver calls.** The pinned baseline already analyzes all arms and arguments once.
14. **Do not repair SC-3 row gaps inside ordinary HKT inference.** Row variables remain a separate kind/domain.
15. **Do not weaken `Unknown`, conflict, kind mismatch, or underconstraint into `Dynamic` just to keep analysis moving.**
16. **Do not change runtime representation, selector identity, VM generic specialization, or class identity as part of this static closure program.**

---

# 5. Implementation-state file protocol

Create and maintain:

```text
docs/impl/semantic/semantic-completeness/sc-4/
    SC-4-SC-4.5-implementation-state.md
```

After each checkpoint, update only concise reviewable facts.

Required structure:

```md
# SC-4 / SC-4.5 Implementation State

Baseline:
- repository:
- branch:
- starting HEAD:
- current HEAD/worktree note:

## Established invariants
- I-001: ...

## Decisions
- D-001: ...

## Evidence ledger
| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Negative/deletion gates
| Checkpoint | Search/assertion | Result | Proves |
|---|---|---|---|

## Deferred gates
- command -> destination checkpoint

## Active incident
None.

## Next resume action
Begin C<N> Task <N>.
```

Do not store private reasoning or a raw implementation diary.

---

# 6. Checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–3 | Baseline and RED characterization are trustworthy | parser HKT test, semantic inference regressions, MON/Either baseline, local state file | workspace/clippy |
| C1 | 4–7 | Nested inference variables have explicit context/frame ownership and cannot collide/escape | direct nested-frame tests + higher-order contextual regressions | generalized HKT source suite |
| C2 | 8–11 | Constructor-kinded inference is kind-parametric and formal-shape-driven | multi-arity, higher-order, arbitrary-hole, multi-hole solver/source tests | getter and surface parity |
| C3 | 12–15 | Result-directed HKT inference and symbolic terms cross nested calls safely | expected-result HKT, outer→closure→nested call, no child escape | full constraints/variance |
| C4 | 16–19 | HKT constraints, variance, supertype projection, and `Self` agree with canonical relation/specialization | focused source + relation parity tests | getters |
| C5 | 20–23 | Generic getters are canonical zero-value-argument generic callables | parser, signature, contextual access, hostile underconstraint/constraint tests | whole workspace |
| C6 | 24–28 | Every ratified executable generic surface uses one inference mathematics; SC-4 closes | methods/constructors/variants/GADT/family/union parity + MON/Either + semantic crate | SC-4.5 and final workspace |
| C7 | 29–31 | SC-3 is either complete and consumable or recorded as an explicit blocker; relation matrix is frozen | SC-3 gates + relation matrix tests | flow/ADT coverage |
| C8 | 32–35 | Bidirectional expected typing covers all supported expression owners | expectation audit + focused expression modules | flow/GADT |
| C9 | 36–39 | Flow/refinement state, joins, loops, captures, and pre-effect invalidation are sound | branch/loop/capture hostile tests | ADT/GADT final |
| C10 | 40–43 | ADT/GADT elimination, branch refinement, exhaustiveness, and impossible-case reasoning close | targeted currently-gated ADT/GADT tests | source-debt certification |
| C11 | 44–47 | Source constraints, variance, nested `Self`, aliases, and Families are end-to-end certified | source semantic suites + regenerated ledger candidates | final publication gates |
| C12 | 48–52 | Epistemic/publication/incremental/deletion closure establishes SC-4.5 | cold/incremental parity, no solver escape, deletion searches, semantic crate + core closure suites | final workspace delivery gates |

---

# 7. Checkpoint C0 — Baseline, characterization, and execution state

Tasks:
- Task 1 — Pin local baseline and create implementation state.
- Task 2 — Run the protected current inference/HKT baseline.
- Task 3 — Add RED characterization fixtures for the genuinely missing SC-4 laws.

Why this is a checkpoint:

The repository is moving quickly and connected GitHub exposes no CI status for the pinned revision. Before changing inference, the implementing agent must know which tests are baseline-green locally and which new laws are genuinely RED. Otherwise later failures cannot be classified as PRODUCT versus BASELINE or PLAN DRIFT.

Entry conditions:
- checkout contains or descends from the pinned semantics;
- SC-1/SC-2 current production implementation is present;
- `phalcom-core/tests/core/monads` exists.

Working set:

Primary:
- `phalcom-semantic/src/checker/inference.rs`
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/expected.rs`
- `phalcom-semantic/tests/semantic/foundations/inference.rs`
- `phalcom-core/tests/core/monads/`
- `phalcom-core/tests/core/either/`
- `phalcom-ast/tests/parser.rs`

Secondary — inspect only if evidence requires it:
- `docs/work/logs/2026-09-02-generic-inference-bootstrap-regression.md`
- `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md`

Out of scope:
- changing production semantics;
- SC-3 implementation;
- effects/proofs;
- runtime representation.

Semantic contract established:
- the worker has an exact local baseline and can distinguish new SC-4 failures from pre-existing failures;
- current protected inference convergence/contextual-term laws are known-green or an INCIDENT is recorded before implementation;
- RED cases exist for missing nested-frame and generalized HKT capabilities.

Semantic risks:
- mistaking a pre-existing core failure for an SC-4 regression;
- writing fixtures that fail parser/type formation rather than the intended inference law;
- relying on old coverage-ledger statuses that MON has superseded.

Hostile cases:
- MON green but semantic direct regression red due local unrelated changes;
- a new multi-arity test fails because syntax is malformed rather than solver capability;
- local branch has uncommitted inference edits.

Required evidence:

1. `git status --short` and `git rev-parse HEAD` — records local execution baseline; do not require a clean tree, but record relevant changes.
2. `cargo test -p phalcom-ast --test parser parse_compact_type_lambda_as_generic_argument -- --nocapture` — proves current compact type-lambda syntax baseline.
3. `cargo test -p phalcom-semantic --test semantic semantic::foundations::inference -- --nocapture` — proves recent fixed-point/rigid-parameter/contextual-term regressions.
4. `cargo test -p phalcom-core --test core monads:: -- --nocapture` — proves the 53-law HKT integration baseline.
5. `cargo test -p phalcom-core --test core either:: -- --nocapture` — protects neighboring generic/ADT behavior.
6. Run newly added RED tests individually and record their exact failure categories.

Do not run yet:
- `cargo test --workspace --all-targets` — deferred to final delivery; no added evidence at C0.
- workspace clippy — deferred.

Escalate immediately if:
- the protected inference or MON baseline fails on a clean/pinned local baseline;
- latest local `main` has replaced `InferenceSession`, `ExpectedType::Inference`, or `apply_resolved_callable` ownership;
- a proposed RED test is already GREEN, which means the plan must reclassify that capability instead of implementing duplicate logic.

Checkpoint completion:
- [ ] all tasks implemented
- [ ] baseline evidence recorded
- [ ] RED cases fail for intended semantic reasons
- [ ] no parser-fixture false RED remains
- [ ] implementation state updated
- [ ] no active incident remains

Suggested commit:
```text
test(semantic): characterize SC-4 nested and higher-kinded inference gaps
```

## Task 1 — Pin local baseline and create implementation state

Purpose:
Create the execution record that prevents baseline drift and forgotten deferred gates.

Risk:
- Semantic: LOW
- Implementation fanout: local documentation

Owned files and symbols:
- Create `docs/impl/semantic/semantic-completeness/sc-4/SC-4-SC-4.5-implementation-state.md`

Inspect before editing:
- local `git status --short`
- local `git rev-parse HEAD`
- this plan's pinned HEAD

Do not inspect unless evidence forces expansion:
- compiler/VM implementation.

Dependencies:
- none.

Source of truth:
- local Git worktree state for execution;
- pinned remote SHA for plan provenance.

Implementation boundary:

Changes:
- create the state file with baseline, invariants, evidence ledger, deferred gates, active incident, next action.

Must not:
- claim local tree is clean if it is not;
- reset or discard user changes;
- copy private scratch reasoning into the state file.

Current implementation:
No program-specific SC-4 state file is guaranteed to exist.

Target implementation:
One concise state file is updated at every checkpoint.

Edit operations:
1. RUN `git status --short`.
2. RUN `git rev-parse HEAD`.
3. CREATE the state file.
4. RECORD pinned-plan SHA `2b6f28a943d9a76ca33f66763b6a1d391c623424` and actual local SHA.
5. RECORD any relevant local modifications without changing them.
6. SET next action to C0 Task 2.

Code instructions:

EXACT:

```md
# SC-4 / SC-4.5 Implementation State

## Baseline
- Plan baseline: `2b6f28a943d9a76ca33f66763b6a1d391c623424`
- Execution baseline: `<actual git rev-parse HEAD>`
- Working-tree note: `<concise factual status>`

## Established invariants

## Decisions

## Evidence ledger
| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Negative/deletion gates
| Checkpoint | Search/assertion | Result | Proves |
|---|---|---|---|

## Deferred gates
- workspace format/check/test/clippy -> Final Gate

## Active incident
None.

## Next resume action
C0 Task 2.
```

Testing classification:
- No standalone behavioral test.

Checkpoint state update:
Record exact local baseline.

## Task 2 — Run protected baseline suites

Purpose:
Establish current local behavior before RED additions.

Risk:
- Semantic: LOW
- Implementation fanout: no production edits

Owned files and symbols:
- no production edits.

Inspect before editing:
- `docs/work/logs/2026-09-02-generic-inference-bootstrap-regression.md`
- current MON law catalog.

Dependencies:
- Task 1 state file.

Source of truth:
- executable local test results.

Implementation boundary:

Changes:
- evidence only.

Must not:
- fix failures during this task;
- broaden into workspace debugging before classification.

Edit operations:
1. RUN the five required baseline commands listed at C0.
2. RECORD exact pass/fail in state.
3. IF any baseline fails, follow the failure protocol in §34 and mark C0 INCIDENT.
4. ONLY continue when baseline failure is classified and either resolved separately or explicitly accepted as unrelated with supervisor approval.

Testing classification:
- This task is itself baseline evidence.

## Task 3 — Add RED characterization for missing SC-4 laws

Purpose:
Create minimal source/solver tests proving the gaps identified by requirements analysis before architecture changes.

Risk:
- Semantic: HIGH
- Implementation fanout: tests only, multi-file

Owned files and symbols:
- `phalcom-semantic/tests/semantic/foundations/inference.rs` — low-level nested/context ownership characterization.
- Create `phalcom-semantic/tests/semantic/capabilities/higher_kinded_generics.rs` if no current module already owns these source laws.
- `phalcom-semantic/tests/semantic/capabilities/mod.rs` — register the new descriptive module.
- Optionally `phalcom-core/tests/core/monads/` only for one HKT integration characterization that directly extends a MON law; do not move the general closure suite there.

Inspect before editing:
- current inference foundation test helpers;
- `semantic/capabilities/generics.rs`;
- MON fixture helpers for exact HKT constructor assertions.

Do not inspect unless evidence forces expansion:
- runtime compiler;
- LSP.

Dependencies:
- protected baseline is green.

Source of truth:
- current semantic API and exact canonical `TypeId`/`TypeParameterId`/explanation products.

Implementation boundary:

Changes:
Add minimum RED cases for:
1. nested generic call receiving an inference-shaped ancestor expectation without variable collision;
2. binary constructor variable `F: Type -> Type -> Type`;
3. higher-order constructor variable `(Type -> Type) -> Type`;
4. leading/middle-hole constructor abstraction;
5. multi-hole abstraction;
6. expected-result HKT selection.

Must not:
- implement production helpers while writing characterization;
- weaken assertions to formatted type strings if canonical IDs/structure are inspectable;
- use declaration-name special cases.

Current implementation:
MON proves unary constructor inference and one partial `Either<E, _>` pattern; direct solver currently exposes session-local raw variable IDs and the unification path treats applied arity mismatch as structural mismatch unless another path pre-adapts it.

Target implementation:
RED tests identify exactly which generalized laws are absent at the execution baseline.

Edit operations:
1. OPEN `phalcom-semantic/tests/semantic/capabilities/mod.rs`.
2. SEARCH for an existing higher-kinded capability module.
3. IF absent, CREATE `higher_kinded_generics.rs` and register it.
4. REUSE the existing semantic `Fixture`.
5. ADD source declarations with explicit kind syntax; keep syntax as simple as possible.
6. ADD exact result/generic-solution/status assertions.
7. ADD low-level nested ownership test in foundations that deliberately creates parent/child variable-number reuse if current APIs allow it.
8. RUN each test individually.
9. RECORD whether each is RED or unexpectedly GREEN.
10. If a case is GREEN, remove it from the “missing implementation” ledger and retain it as a regression test.

Code instructions:

STRUCTURAL — example law shapes, not paste-ready syntax if the current parser requires fixture adaptation:

```text
Binary constructor:
    F : Type -> Type -> Type
    formal F<A, B>
    actual Pair<Int, String>
    expect F = Pair, A = Int, B = String

Higher-order:
    H : (Type -> Type) -> Type
    formal H<F>
    actual Wrap<List>
    expect H = Wrap, F = List

Leading hole:
    formal F<A>
    actual Either<Int, Error>
    expect F = <X> =>> Either<X, Error>, A = Int

Multi-hole:
    formal F<A, B>
    actual Triple<String, Int, Bool>
    expect F = <X, Y> =>> Triple<String, X, Y>
```

Testing classification:
- Focused RED characterization required now.

---

# 8. Checkpoint C1 — Scoped nested inference ownership

Tasks:
- Task 4 — Introduce query-local inference context and frame identities.
- Task 5 — Thread inference context identity through expected terms.
- Task 6 — Make nested generic calls join the owning constraint graph.
- Task 7 — Add escape/publication guards and nested-frame evidence.

Why this is a checkpoint:

These tasks are meaningful only together. A frame ID without context propagation does not solve nested inference; context propagation without shared graph ownership still leaves child calls unable to relate to ancestor variables. The checkpoint is complete only when nested calls can safely consume ancestor inference terms and no raw child variable can leak into public semantic state.

Entry conditions:
- C0 COMPLETE;
- current `InferenceSession` and `ExpectedType::Inference` architecture intact;
- protected `term_for_expected` behavior green.

Working set:

Primary:
- `phalcom-semantic/src/checker/inference.rs`
- `phalcom-semantic/src/checker/expected.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/typed_expr.rs`
- `phalcom-semantic/tests/semantic/foundations/inference.rs`
- new/selected higher-order source capability tests

Secondary:
- `phalcom-semantic/src/checker/analysis.rs` — inspect only if symbolic checker-local state threatens publication.
- `phalcom-semantic/src/explain.rs` / explanation types — inspect if stable frame-to-source translation is needed.
- `phalcom-semantic/src/lib.rs` — only if a newly public semantic type is genuinely required; prefer keeping context/frame IDs crate-private.

Out of scope:
- changing `TypeId`;
- making `InferVarId` process-global;
- row inference;
- HKT constructor abstraction algorithm;
- metadata/reflection.

Semantic contract:
- every active inference variable belongs to one explicit frame within one query-local inference context;
- nested calls that receive `ExpectedType::Inference` use the same owning inference context and allocate a child frame;
- variable IDs are unique inside that context and never accidentally alias by local numeric reuse;
- caller canonical type parameters stay rigid;
- symbolic inference state is checker-local and is eliminated/materialized before publication.

Semantic risks:
- re-entrant borrowing of inference graph from `CheckingContext`;
- child call terminalizing an outer frame prematurely;
- loss of existing per-call underconstraint/conflict provenance;
- leakage of solver-local context IDs into `ExpressionAnalysis`;
- changing deterministic result order.

Hostile cases:
- parent and child both would have allocated local `InferVarId(0)` in the old architecture;
- sibling nested calls use the same generic declaration and solve different types;
- child call is underconstrained while parent later provides a selecting constraint;
- child conflict does not poison an unrelated sibling frame;
- rigid caller `TypeData::Parameter` never becomes a child inference variable.

Required evidence:
1. direct inference-context/frame unit tests;
2. `semantic::foundations::inference` full module;
3. new nested generic source tests;
4. existing Either higher-order contextual tests;
5. negative search showing no atomic/global inference ID allocator was introduced;
6. publication assertion proving no context/frame-local symbolic term appears in snapshot products.

Do not run yet:
- MON full suite until C2/C3 unless C1 touches current HKT call path enough to justify it; run targeted MON higher-order cases only.
- workspace tests.

Escalate immediately if:
- implementing context ownership appears to require public `TypeId`/metadata changes;
- nested calls cannot share constraints without moving all checker state to interior mutability;
- a Rust borrowing workaround begins to create a second solver copy.

Checkpoint completion:
- [ ] all tasks implemented
- [ ] nested-frame tests pass
- [ ] protected inference regression module passes
- [ ] rigid parameter tests pass
- [ ] no solver-local state published
- [ ] state file updated
- [ ] no incident

Suggested commit group:
```text
refactor(semantic): scope nested inference variables by query context
test(semantic): enforce nested inference frame ownership
```

## Task 4 — Introduce query-local inference context and frame identities

Purpose:
Make ownership explicit without widening `InferVarId` into a process-global identity.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `checker/inference.rs` — `InferenceSession`, `InferenceVariable`, fresh-variable allocation.
- `checker/context.rs` — `CheckingContext` query-local inference-context storage/allocator.

Inspect before editing:
- every production `InferenceSession::new()` call;
- `InferenceSession::fresh_variable`;
- `instantiate_generic_signature`;
- all `InferVarId` production usages from `rg 'InferVarId' phalcom-semantic/src`.

Do not inspect unless evidence forces expansion:
- runtime/VM;
- metadata serializer.

Dependencies:
- C0 characterization.

Source of truth:
- one `InferenceSession` constraint graph per active nested inference context;
- frame metadata inside that graph.

Implementation boundary:

Changes:
- add crate-private `InferenceContextId` and `InferenceFrameId`;
- extend `InferenceVariable` with owner frame;
- add frame creation/closure bookkeeping to the existing `InferenceSession`;
- add a `CheckingContext`-owned table/arena of active inference contexts so recursive expression checking can recover a graph by ID in short mutable borrows.

Must not:
- replace `InferenceSession` with a competing solver;
- use atomic/global counters;
- place `InferenceContextId` in metadata/snapshots;
- change canonical `InferVarId != TypeId` law.

Current implementation:
`InferenceSession` owns a local `next_var_index`; every session begins its own numeric sequence.

Target implementation:
Within one `InferenceContextId`, all nested application frames allocate from one unique variable space; each variable records `InferenceFrameId`.

Edit operations:
1. OPEN `checker/inference.rs`.
2. FIND `InferenceSession` and `InferenceVariable`.
3. ADD crate-private ID wrappers near inference machinery.
4. ADD frame metadata to `InferenceVariable`.
5. EXTRACT variable allocation into `fresh_variable_in_frame(frame, kind)`; retain a test-only/root convenience only if useful.
6. OPEN `checker/context.rs`.
7. ADD active inference-context storage and a monotonic **query-local** context allocator.
8. ADD narrow methods:
   - create root inference context;
   - begin child frame in context;
   - short-lived mutable access to a context's `InferenceSession`;
   - finish/drop root context after materialization.
9. SEARCH for direct `InferenceSession::new()` production calls and migrate only call application owners.
10. Keep low-level unit tests able to instantiate a standalone root session.

Code instructions:

STRUCTURAL — exact field names may adapt to current `CheckingContext` layout:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct InferenceContextId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct InferenceFrameId(u32);

pub struct InferenceVariable {
    pub id: InferVarId,
    pub frame: InferenceFrameId,
    pub kind: KindId,
    // existing state/support/proof
}

pub struct InferenceSession {
    // existing graph state
    next_var_index: u32,
    next_frame_index: u32,
    // frame membership / ownership metadata
}
```

Preferred ownership model:

```text
CheckingContext
    -> active inference contexts by InferenceContextId
        -> one InferenceSession graph
            -> root/child InferenceFrameId
            -> globally unique InferVarId within that graph
```

Testing classification:
- No standalone behavioral evidence until C1 integration; compile after this task is useful.

Optional compile checkpoint:
```bash
cargo check -p phalcom-semantic
```

Proves:
- structural API fanout compiles;
- exhaustive constructors/callers are migrated.

Does not prove:
- nested inference semantics.

Checkpoint state update:
Record the final context/frame API and any changed constructor names.

## Task 5 — Carry owning context with `ExpectedType::Inference`

Purpose:
Make every solver-local expected term interpretable by the correct graph.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `checker/expected.rs` — `ExpectedType::Inference`.
- `checker/call.rs` — creation of inference-shaped argument/result expectations.
- expression helpers that construct `ExpectedType::Inference`.

Inspect before editing:
- all `ExpectedType::inference*` constructors/usages;
- `collection_element_type`, `map_key_val_types`, `callable_signature`;
- `call.rs` use of `term_for_expected`.

Dependencies:
- Task 4 context ID API.

Source of truth:
- `InferenceContextId` paired with every solver-local `InferenceTerm`.

Implementation boundary:

Changes:
- add `context: InferenceContextId` to the inference expected variant;
- update constructors/accessors;
- preserve the same context when structurally projecting collection/callable components;
- make context mismatch explicit rather than silently comparing raw variable IDs.

Must not:
- allow `ExpectedType::Inference` without an owning context;
- create a new context during a simple structural projection.

Current implementation:
`ExpectedType::Inference` stores only `InferenceTerm` and origin.

Target:
Every inference expectation is `(context, term, origin)`.

Edit operations:
1. OPEN `checker/expected.rs`.
2. CHANGE inference variant to include context ID.
3. CHANGE `ExpectedType::inference` and `inference_from` signatures accordingly; if a convenience constructor remains for tests, make ownership explicit.
4. UPDATE `collection_element_type`, `map_key_val_types`, `callable_signature` to propagate context unchanged.
5. UPDATE `call.rs` argument expectation construction.
6. `rg 'ExpectedType::Inference|ExpectedType::inference' phalcom-semantic/src phalcom-semantic/tests` and migrate every use.
7. ADD debug/assertion path for impossible cross-context term mixing.

Code instructions:

STRUCTURAL:

```rust
Inference {
    context: InferenceContextId,
    term: InferenceTerm,
    origin: ExpectationOrigin,
}
```

Testing classification:
- No standalone behavioral test; C1 nested tests prove it.

Optional compile:
`cargo check -p phalcom-semantic`

## Task 6 — Make nested generic applications join the owning graph

Purpose:
Use the context carried by an inference expectation to allocate a child application frame in the same constraint graph.

Risk:
- Semantic: HIGH
- Implementation fanout: local-to-multi-file

Owned files and symbols:
- `checker/call.rs` — `apply_generic_callable_inner`.
- `checker/inference.rs` — frame-local solution/outcome APIs.
- `checker/context.rs` — context access.

Inspect before editing:
- exact current `apply_generic_callable_inner` variable-map creation;
- solve calls after each argument;
- expected-result constraint insertion;
- terminal outcome/materialization logic.

Dependencies:
- Tasks 4–5.

Source of truth:
- the owning inference context from the incoming expected term, otherwise a newly created root context.

Implementation boundary:

Changes:
- root generic call creates context + root frame;
- nested generic call receiving inference expectation in context C joins C and starts child frame;
- local generic parameters are allocated in child frame;
- fixed receiver generics remain canonical/fixed bindings;
- solving must distinguish “propagate graph to fixed point” from “terminalize this frame as underconstrained.”

Must not:
- classify ancestor unresolved variables as missing child metadata;
- terminalize an ancestor frame while a child is still analyzing;
- erase existing per-constraint provenance.

Current implementation:
Every generic call creates `InferenceSession::new()` and owns all vars locally.

Target:
Nested application frames share the graph that owns the incoming inference expectation.

Edit operations:
1. OPEN `call.rs`.
2. FIND initial `InferenceSession::new()` in `apply_generic_callable_inner`.
3. REPLACE local-session creation with acquisition of a root-or-child application frame.
4. MOVE session mutations behind short-lived `CheckingContext` accessors; never hold a mutable graph borrow across recursive `analyze_expression`.
5. ADD a non-terminal propagation API if current `solve_with_control` always classifies underconstraint too early.
6. Preserve current argument order, `PreAnalyzed` behavior, explanation capture, and evidence authority.
7. Ensure a nested frame can leave a relation pending on ancestor vars without returning an incorrect terminal conflict.
8. Finish/drop only the root context when all nested frames are complete and its result has materialized.

Code instructions:

STRUCTURAL — the important distinction is propagation vs terminal classification:

```text
propagate(context)
    replay constraints to fixed point
    do not call unresolved ancestor variables terminal underconstraint

finish_frame(context, frame)
    classify variables owned/relevant to frame
    allow unresolved ancestor references to remain pending while ancestor frame is active

finish_root(context)
    require publishable canonical result or structured terminal outcome
    destroy query-local graph after publication
```

Testing classification:
- Focused C1 tests required after Task 7.

## Task 7 — Add symbolic escape guards and nested-frame hostile tests

Purpose:
Prove nested ownership works and no context-local term reaches public semantic products.

Risk:
- Semantic: HIGH
- Implementation fanout: tests + possible checker-local result plumbing

Owned files and symbols:
- `checker/typed_expr.rs` — only if a checker-local symbolic result handle is required.
- `checker/analysis.rs` — publication boundary.
- `semantic/foundations/inference.rs`
- higher-order capability tests.

Inspect before editing:
- `TypedExpression` conversion to/from `CallCheckResult`;
- `publish_expression_analysis`;
- whether current nested call can remain useful using only its expected term or needs a checker-local symbolic result field.

Dependencies:
- Task 6.

Source of truth:
- canonical `TypeKnowledge` for published facts;
- optional checker-local `(InferenceContextId, InferenceTerm)` only while analysis is active.

Implementation boundary:

Changes:
If nested call results can depend on unresolved ancestor vars, add a checker-local symbolic type fact to `TypedExpression`/`CallCheckResult` rather than publishing `Unknown` prematurely.

Must not:
- add this fact to durable `ExpressionAnalysis`;
- serialize it;
- expose it to LSP/metadata;
- let child-owned variables survive root completion.

Current implementation:
`TypedExpression` carries canonical `TypeKnowledge`, ordinary constraints, provenance, status, and denotation, but no solver-context symbolic result.

Target if required by RED test:
Internal expression analysis can carry a context-owned symbolic term until the root call materializes.

Edit operations:
1. RUN nested RED tests after Tasks 4–6.
2. IF they pass without symbolic output plumbing, do not add a new field.
3. IF failure is specifically “child result must remain symbolic until ancestor solves,” add a crate-private `InferenceFact`:
   - context ID;
   - inference term.
4. Wire it through `CallCheckResult -> TypedExpression` only.
5. Explicitly omit it from `ExpressionAnalysis` publication.
6. Add debug assertions at publication/root completion that no unresolved child-owned term is being converted to a known public type.
7. Add hostile tests:
   - parent/child same old local numeric index;
   - siblings;
   - child underconstraint then ancestor solution;
   - child conflict isolation;
   - rigid caller parameter.
8. Run C1 evidence.

Code instructions:

INVESTIGATE-BEFORE-EDIT for the symbolic field:
Do not add it pre-emptively. The exact RED path decides whether existing `ExpectedType`/constraint plumbing is sufficient.

If required, target conceptual shape:

```rust
pub(crate) struct InferenceFact {
    pub context: InferenceContextId,
    pub term: InferenceTerm,
}
```

Testing classification:
- Required checkpoint evidence.

Negative gate:

```bash
rg -n 'AtomicU|static .*Infer|GLOBAL.*Infer' phalcom-semantic/src
```

Expected:
- no new global/process-wide inference-ID allocation.

---

# 9. Checkpoint C2 — Generalized higher-kinded constructor inference

Tasks:
- Task 8 — Extract canonical constructor-view decomposition.
- Task 9 — Implement formal-shape-driven constructor abstraction.
- Task 10 — Generalize multi-arity and higher-order kind solving.
- Task 11 — Close generalized HKT conflict/kind/materialization evidence.

Why this is a checkpoint:

All four tasks establish one semantic claim: a constructor-valued variable can be solved from canonical type structure for every currently ratified explicit non-row arrow kind, without special-casing unary final-hole applications.

Entry conditions:
- C1 COMPLETE;
- canonical type-lambda application remains in `TypeStore`/`TypeLambdaArena`;
- no SC-3 row variable is routed through ordinary HKT inference.

Working set:

Primary:
- `checker/inference.rs`
- create `checker/type_constructor_inference.rs` **only if** extraction materially reduces `inference.rs`; otherwise keep helpers private in `inference.rs`.
- `types/store.rs`
- `types/type_lambda.rs`
- `types/annotation.rs` only for source fixture/kind formation issues
- higher-kinded capability tests
- MON inference/constructor-agreement tests

Secondary:
- `types/relation.rs` for relation parity only, not redesign.
- `metadata/export.rs` only to verify synthesized canonical lambda is already exportable.

Out of scope:
- new type lambda representation;
- eta-equivalence;
- kind polymorphism;
- row-valued generic args.

Semantic contract:
- exact nominal constructor is preferred when it satisfies the inferred kind;
- constructor abstraction can abstract leading, middle, trailing, or multiple positions according to the formal inference shape;
- free caller generics remain free; synthesized binders are capture-safe;
- all candidate constructors are kind-checked;
- no constructor is fabricated without selecting evidence.

Semantic risks:
- synthesizing structurally correct but wrong-kind lambdas;
- capturing caller generics;
- choosing lambda when nominal constructor should be used;
- assuming equal actual/formal application arity;
- treating canonical lambda identity by source binder names.

Hostile cases:
- `Either<Int, Error>` leading-hole abstraction;
- 3-parameter middle-hole;
- fixed-position mismatch between two arguments;
- binary variable with unary candidate;
- higher-order variable with proper type candidate;
- same actual type permitting multiple abstractions unless formal correspondence uniquely selects one.

Required evidence:
1. direct low-level abstraction tests inspecting `TypeData::Lambda` and `ScopedTypeData`;
2. source tests for binary and higher-order kinds;
3. MON protected constructor inference;
4. exact kind assertions;
5. negative conflicts/underconstraint.

Do not run yet:
- getter tests;
- workspace.

Escalate if:
- abstraction requires changing canonical `TypeData::Applied` to carry non-`TypeId` args;
- row kind appears in ordinary constructor abstraction;
- the only way to pass is declaration-name matching.

Checkpoint completion:
- [ ] tasks complete
- [ ] multi-arity tests green
- [ ] higher-order tests green
- [ ] arbitrary/multi-hole tests green
- [ ] MON HKT suite green
- [ ] hostile kind/conflict tests green
- [ ] state updated

Suggested commits:
```text
feat(semantic): generalize higher-kinded constructor inference
test(semantic): cover multi-arity and arbitrary constructor abstraction
```

## Task 8 — Extract a canonical constructor-view decomposition helper

Purpose:
Separate “what canonical constructor/application structure does this actual type expose?” from unification mechanics.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local

Owned files and symbols:
- `checker/inference.rs` current canonical/applied unification.
- optional new `checker/type_constructor_inference.rs`.

Inspect before editing:
- `InferenceSession::unify_terms`;
- `InferenceSession::subtype_terms`;
- `TypeStore::kind_of`;
- `TypeStore::applied_nominal_parts`;
- `TypeStore::apply_type_form`;
- `TypeLambdaArena::beta_reduce`.

Dependencies:
- C1.

Source of truth:
- canonical `TypeStore` forms and kinds.

Implementation boundary:

Changes:
Create one internal representation/helper that can expose:
- candidate origin constructor;
- canonical application arguments;
- origin kind;
- actual proper type;
without deciding which holes to abstract.

Must not:
- synthesize a lambda in this helper;
- normalize by formatted strings;
- expand aliases through a second path if `TypeStore` already resolved them.

Current implementation:
`unify_terms` directly matches `TypeData::Applied` and rejects argument-count mismatch in that branch.

Target:
Unification can ask a constructor-inference helper to align formal application structure to the canonical actual before falling back to structural mismatch.

Edit operations:
1. FIND the canonical-vs-`InferenceTerm::Applied` arms in `unify_terms` and `subtype_terms`.
2. EXTRACT read-only canonical application inspection.
3. RETURN a structured view or `None`; do not produce diagnostics here.
4. Keep exact-case handling separate unless the existing path already lowers exact case to its enum application safely.
5. Add unit tests for view extraction over nominal, applied, lambda-applied, and rigid parameter boundaries.

Code instructions:

STRUCTURAL:

```rust
struct CanonicalConstructorView {
    actual: TypeId,
    origin: TypeId,
    arguments: Box<[TypeId]>,
    origin_kind: KindId,
}
```

Use repository-native naming if a near-equivalent abstraction already exists.

Testing classification:
- unit-level helper test can be included, but checkpoint evidence remains C2.

## Task 9 — Implement formal-shape-driven constructor abstraction

Purpose:
Infer a constructor-valued variable when the actual proper type has more/fixed structure than the formal application.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file if lambda helper extracted

Owned files and symbols:
- inference constructor alignment helper;
- `types/type_lambda.rs` canonical scoped construction APIs;
- `types/store.rs::type_lambda`.

Inspect before editing:
- existing source type-lambda lowering in `types/annotation.rs` for correct `ScopedTypeData::Bound`/`Free` construction;
- `TypeLambdaArena::intern_lambda`;
- free-type collection/capture behavior.

Dependencies:
- Task 8.

Source of truth:
- formal inference term determines hole correspondence;
- `TypeLambdaArena` determines canonical lambda identity.

Implementation boundary:

Changes:
Given:
```text
formal F<X...>
actual C<A...>
```
derive:
- constraints solving formal argument variables from corresponding actual positions;
- a constructor candidate for `F` that abstracts exactly those positions and keeps remaining actual structure fixed.

Prefer nominal origin directly when every constructor parameter position maps one-to-one to formal arguments.

Must not:
- assume suffix holes;
- synthesize source-level names as semantic identity;
- capture free canonical caller parameters.

Current implementation:
Unary cases are covered by MON, but direct applied unification still has an equal-arity structural path and is not a general arbitrary-hole abstraction algebra.

Target examples:
```text
F<A> ~ Either<Int, Error>
=> A=Int
=> F=<X> =>> Either<X, Error>

F<A,B> ~ Triple<String,Int,Bool>
=> F=<X,Y> =>> Triple<String,X,Y>
```

Edit operations:
1. Define an alignment algorithm over formal argument terms and actual arguments.
2. Determine which actual positions correspond to formal variables/structures.
3. Reject non-unique or contradictory alignments as ambiguity/conflict, not arbitrary selection.
4. Build scoped lambda body using `ScopedTypeData::Free` for fixed/free canonical types and `Bound` for holes.
5. Use binder kinds from the formal constructor kind.
6. Intern through `TypeLambdaArena` and `TypeStore::type_lambda`.
7. Validate the synthesized lambda's kind before binding `F`.
8. Apply it back to solved formal arguments in tests to prove it reconstructs the actual type.
9. Preserve nominal constructor if no abstraction beyond the constructor's own full parameter list is needed.

Code instructions:

STRUCTURAL algorithm:

```text
align(formal_args, actual_constructor_args)
    collect bound-hole occurrences
    collect fixed actual substructure
    ensure every formal constructor parameter has a unique semantic position
    synthesize scoped body
    intern lambda
    verify lambda_kind == inference_variable.kind
```

Testing classification:
- C2 source + low-level tests.

## Task 10 — Generalize multi-arity and higher-order kind solving

Purpose:
Make constructor-variable inference use `KindId` recursively rather than unary arity assumptions.

Risk:
- Semantic: HIGH
- Implementation fanout: local-to-multi-file

Owned files and symbols:
- `checker/inference.rs`;
- constructor helper;
- `types/store.rs::apply_kind` as validation authority.

Inspect before editing:
- `InferenceVariable.kind`;
- `instantiate_generic_signature`;
- `kind_of`;
- any `arguments.len() == 1` or unary HKT assumptions in inference/call code.

Dependencies:
- Task 9.

Source of truth:
- canonical `KindId` and `TypeStore::apply_kind`.

Implementation boundary:

Changes:
Support:
```text
Type -> Type -> Type
(Type -> Type) -> Type
(Type -> Type) -> Type -> Type
```
to the extent represented by current kind grammar.

Must not:
- introduce kind variables;
- infer a constructor of wrong arity by eta-expanding blindly.

Edit operations:
1. `rg -n 'len\(\) == 1|len\(\) != 1|Type -> Type|KindId::TYPE' phalcom-semantic/src/checker`.
2. Classify each unary assumption.
3. Replace real HKT assumptions with kind-driven parameter decomposition.
4. Preserve ordinary proper-type variables as `KindId::TYPE`.
5. Use `apply_kind` for candidate application validation.
6. Add binary/higher-order source tests.

Testing classification:
- checkpoint evidence.

## Task 11 — Close HKT conflict, ambiguity, and materialization semantics

Purpose:
Ensure generalized constructor inference terminates in the correct structured outcome.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `InferenceOutcome`;
- conflict/ambiguity candidate collection;
- materialization functions;
- call diagnostics.

Inspect before editing:
- current ambiguity construction;
- `InferenceMaterializationFailure`;
- call diagnostic mapping.

Dependencies:
- Tasks 8–10.

Source of truth:
- existing inference outcome algebra.

Implementation boundary:

Changes:
- include constructor candidates in ambiguity/conflict provenance;
- retain fixed-position constraint origins;
- classify wrong kind as kind failure;
- classify truly no constructor evidence as underconstraint.

Must not:
- add a new HKT-specific terminal outcome unless existing algebra cannot express a real semantic distinction.

Edit operations:
1. Add hostile source/solver tests.
2. Run them before changing mapping.
3. Update candidate/conflict bookkeeping only where current outcome loses information.
4. Verify no outcome falls through to Dynamic/Object.
5. Run:
   - focused HKT module;
   - `cargo test -p phalcom-core --test core monads:: -- --nocapture`.

Testing classification:
- required C2 evidence.


---

# 10. Checkpoint C3 — Result-directed HKT inference and symbolic nested propagation

Tasks:
- Task 12 — Generalize expected-result constraint generation over HKT structure.
- Task 13 — Preserve partially solved HKT terms through contextual typing.
- Task 14 — Propagate nested-call symbolic results without public leakage.
- Task 15 — Prove context authority and conflict behavior.

Why this is a checkpoint:

Constructor reconstruction alone is insufficient. Real higher-order APIs require information to flow from the expected result back into constructor variables and then down into closures and nested generic calls. This checkpoint closes that bidirectional loop while preserving the existing rule that contextual selection is not runtime value evidence.

Entry conditions:
- C2 COMPLETE;
- C1 inference context/frame ownership active;
- `term_for_expected` protected behavior green.

Working set:

Primary:
- `phalcom-semantic/src/checker/call.rs`
- `phalcom-semantic/src/checker/inference.rs`
- `phalcom-semantic/src/checker/expected.rs`
- `phalcom-semantic/src/checker/typed_expr.rs` if C1 introduced `InferenceFact`
- `phalcom-semantic/src/checker/expression.rs`
- higher-kinded capability tests
- existing `semantic/capabilities/generics.rs`

Secondary:
- MON composition/bodies tests;
- explanation types only if provenance lacks contextual HKT detail.

Out of scope:
- getter syntax;
- flow branch joins;
- SC-3 rows.

Semantic contract:
- expected results can uniquely select proper-type and constructor-kinded variables;
- partial actual constructor structure can be reconstructed from expected result exactly as from value arguments;
- expected-result constraints may refine closure argument/result expectations;
- a nested generic call can remain symbolically connected to an outer frame until enough evidence exists;
- expected context cannot overwrite stronger intrinsic evidence;
- context-only selection remains `Assumed`/contextual according to existing evidence policy.

Semantic risks:
- sticky argument-phase underconstraint reappearing after context solves variables;
- child symbolic result being published as `Unknown` too early;
- contextual HKT selection marked Established;
- argument-derived constructor overwritten by expected result;
- outer and child terminalization order.

Hostile cases:
- no value arguments, expected `Either<String, Int>` selects both `F` and `A`;
- value argument selects `F=List`, expected result demands `Option`;
- callback return contains unsolved `F<B>` until another argument/result constraint solves it;
- child nested call initially underconstrained becomes solvable after ancestor context.

Required evidence:
1. ordinary expected-result tests remain green;
2. nominal HKT expected-result test;
3. partial-constructor HKT expected-result test;
4. outer result → closure expectation → nested generic call source test;
5. context-conflict hostile test retaining intrinsic knowledge;
6. explanation assertions distinguish ExpectedResult/ContextSelection from Argument/ValueSelection.

Do not run yet:
- getter suites;
- full semantic crate unless C3 touched shared solve logic enough to require it; normally focused modules + MON composition are sufficient.

Escalate if:
- expected-result inference requires treating context as established value evidence;
- a nested call can only be typed by publishing an unresolved solver term into `ExpressionAnalysis`;
- a child must finalize an active ancestor frame.

Checkpoint completion:
- [ ] tasks complete
- [ ] HKT expected-result cases pass
- [ ] nested contextual call case passes
- [ ] conflict authority passes
- [ ] explanation roles pass
- [ ] state updated
- [ ] no incident

Suggested commits:
```text
feat(semantic): propagate result-directed higher-kinded inference
test(semantic): cover nested contextual HKT solving
```

## Task 12 — Generalize expected-result constraint generation over HKT structure

Purpose:
Allow the expected result to constrain constructor variables using the same structural inference calculus as value arguments.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `checker/call.rs` — expected-result constraint insertion in `apply_generic_callable_inner`.
- `checker/inference.rs` — term lifting and constructor alignment.

Inspect before editing:
- current `ConstraintOrigin::ExpectedResult`;
- current `InferenceConstraintRole::ContextSelection`;
- current ordinary `make<T>() -> T` tests.

Dependencies:
- C2 generalized constructor solver.

Source of truth:
- canonical return type lifted through current generic var map;
- expected proper/inference term;
- existing context-selection role.

Implementation boundary:

Changes:
When the return formal contains constructor variables, structural comparison to the expected result must invoke the same generalized constructor alignment used by argument constraints.

Must not:
- add a separate “expected HKT solver”;
- treat a where-bound as context;
- force a result-only generic to concrete without an actual expected type.

Current implementation:
Ordinary expected-result selection is supported. HKT reconstruction from expected result is not yet certified.

Target:
```text
formal return: F<A>
expected: Either<String, Int>

=> F = <X> =>> Either<String, X>
=> A = Int
=> support role = ContextSelection
```

Edit operations:
1. FIND expected-result constraint construction.
2. Ensure formal return is converted to `InferenceTerm` using the active context/frame var map.
3. If expected is proper, lift it as canonical actual and let generalized unification/subtyping align constructors.
4. If expected is inference-shaped in same context, add the relation directly.
5. Do not copy expected proper type into substitutions.
6. Add result-only HKT tests.

Code instructions:

STRUCTURAL:

```text
return_term = type_id_to_inference(signature.return, var_map)
expected_term =
    Proper(T) -> Canonical(T)
    Inference(same_context, t) -> t

add Equivalent/Subtype(
    return_term,
    expected_term,
    origin=ExpectedResult,
    role=ContextSelection
)
```

Use the existing relation choice mandated by current call semantics; do not change equality/subtype policy merely for HKT.

Testing classification:
- C3 focused source tests.

## Task 13 — Preserve partially solved HKT terms through contextual typing

Purpose:
Make `term_for_expected` and related projection helpers recursively preserve solved and unresolved constructor structure.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `InferenceSession::term_for_expected`
- `ExpectedType::callable_signature`
- collection/product expected projections
- any helper that calls `materialize_for_expected`

Inspect before editing:
- recent regression tests listed in `2026-09-02-generic-inference-bootstrap-regression.md`;
- exact current recursive cases in `term_for_expected`.

Dependencies:
- Task 12.

Source of truth:
- active inference graph state.

Implementation boundary:

Changes:
Extend recursive contextual rewriting for any generalized HKT term shapes not already covered.

Must not:
- convert unresolved constructor vars to Unknown;
- recurse indefinitely on rigid canonical parameters;
- return `Changed` from fixed-point state when no persistent state changes.

Current implementation:
`term_for_expected` was added to preserve higher-order callable expectations after the Either regression.

Target:
It remains the one contextual-zonking helper and supports new constructor structures without duplicate recursive rewriting.

Edit operations:
1. Enumerate all `InferenceTerm` variants.
2. Confirm `term_for_expected` handles each structural variant recursively or intentionally leaves it unchanged.
3. Add any missing recursive handling introduced by C2/C1.
4. Preserve rigid canonical boundary logic.
5. Add test where only the inner type argument solves while constructor remains unresolved and vice versa.
6. Re-run recent fixed-point tests.

Testing classification:
- direct inference regression + C3 integration.

## Task 14 — Propagate nested-call symbolic results safely

Purpose:
Allow a nested generic call whose result is tied to an ancestor inference term to participate in enclosing closure/body inference before root materialization.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file if symbolic result plumbing was needed in C1

Owned files and symbols:
- `checker/call.rs::CallCheckResult`
- `checker/typed_expr.rs`
- `checker/expression.rs` closure analysis path
- inference context APIs

Inspect before editing:
- whether C1 added `InferenceFact`;
- closure parameter/body result constraint creation;
- `CallCheckResult -> TypedExpression`.

Dependencies:
- Tasks 12–13.

Source of truth:
- context-owned solver term while active;
- canonical `TypeKnowledge` once publishable.

Implementation boundary:

Changes:
- preserve symbolic term from child call as an internal result when canonical materialization is deferred;
- feed that term into enclosing closure result constraints;
- materialize before public analysis publication or retain public knowledge as explicitly unavailable while the enclosing root consumes the symbolic fact internally.

Must not:
- publish child `InferVarId`;
- convert symbolic equality into Dynamic;
- treat an unavailable canonical `TypeId` as type mismatch.

Current implementation:
`TypedExpression` has canonical knowledge and ordinary constraints only unless C1 already extended it.

Target:
The checker has exactly one internal symbolic bridge and exactly one public canonical publication boundary.

Edit operations:
1. Trace `apply_resolved_callable -> CallCheckResult -> TypedExpression -> closure body result`.
2. Reuse C1 `InferenceFact` if present.
3. When a call frame has a symbolic result term in an active context, attach that fact.
4. In closure result processing, if expected/result constraints share that context, consume the term directly.
5. On expression publication, never serialize the inference fact.
6. Add a debug assertion that `ExpressionAnalysis` construction has no inference-context field.
7. Add nested-call integration test.

Code instructions:

STRUCTURAL:
```text
child call:
    canonical result available -> normal TypeKnowledge
    result term depends on active ancestor vars -> internal InferenceFact
    terminal child failure -> normal structured failure

closure:
    internal result fact in same context -> add BlockResult constraint directly
```

Testing classification:
- focused nested HKT test.

## Task 15 — Prove contextual evidence and conflict authority

Purpose:
Protect Phalcom's epistemic rules while expanding contextual inference.

Risk:
- Semantic: HIGH
- Implementation fanout: tests + possible diagnostic mapping

Owned files and symbols:
- `semantic/capabilities/generics.rs`
- higher-kinded capabilities module
- explanation assertions
- call conflict mapping only if tests expose a defect

Inspect before editing:
- existing `expected_context_selects_but_does_not_establish_result_only_generic`;
- existing argument/context conflict tests.

Dependencies:
- Tasks 12–14.

Source of truth:
- inference constraint role + supporting evidence.

Implementation boundary:

Changes:
Add HKT analogues of existing ordinary generic evidence tests.

Must not:
- change evidence policy solely because HKT reconstruction produces a lambda.

Required cases:
1. context-only `F/A` solution is contextual/assumed;
2. argument selects `F`, context validates same result;
3. argument selects `F=List`, context requires incompatible constructor — invalid, argument fact retained;
4. unknown/dynamic required argument weakens result proof appropriately.

Testing classification:
- checkpoint evidence.

---

# 11. Checkpoint C4 — HKT constraints, variance, generic supertypes, and `Self`

Tasks:
- Task 16 — Close HKT generic constraint reinstantiation and F-bounds.
- Task 17 — Enforce variance/polarity parity between inference and canonical relations.
- Task 18 — Reuse canonical generic-supertype projection in HKT solving.
- Task 19 — Prove HKT + `Self` receiver specialization.

Why this is a checkpoint:

Once constructor selection works, it must be admissible under the same declaration constraints and subtype algebra as ordinary types. This checkpoint prevents a second, weaker HKT relation calculus from emerging.

Entry conditions:
- C3 COMPLETE;
- `types/relation.rs` current variance/callable relation is baseline authority;
- `types/specialization.rs` is active.

Working set:

Primary:
- `checker/inference.rs`
- `checker/call.rs`
- `types/relation.rs`
- `types/specialization.rs`
- `types/parameter.rs`
- source constraint/variance tests
- higher-kinded capability tests

Secondary:
- `types/annotation.rs` if source generic constraint formation cannot express a required ratified example.
- MON inheritance tests.

Out of scope:
- declaration-position variance language redesign;
- interface/typeclass search;
- effect covariance.

Semantic contract:
- HKT candidates obey `where` constraints without bounds acting as defaults;
- F-bounds are relations, not self-binding guesses;
- inference subtype decomposition matches declaration variance and callable polarity;
- generic-supertype projection is shared with canonical relation/specialization;
- nested `Self` uses the actual semantic receiver through HKT structures.

Semantic risks:
- constraint upper bound accidentally selects constructor;
- reversed contravariant relation;
- duplicated supertype walker;
- `Self` bound to declaring class instead of receiver;
- HKT candidate kind lost through superclass template.

Hostile cases:
- bound-only constructor variable remains underconstrained;
- selected candidate violates bound;
- contravariant nested callable reverses expected relation;
- transformed inheritance `Child<T> is Parent<List<T>>`;
- `F<Self>` inherited through multi-hop hierarchy.

Required evidence:
1. source `where` positive/negative HKT cases;
2. F-bound positive/negative;
3. canonical relation vs inference parity unit tests;
4. generic-supertype source inference;
5. HKT `Self` direct and multi-hop tests;
6. MON inheritance suite.

Do not run yet:
- generic getter parser;
- full workspace.

Escalate if:
- constraint formation uses string shapes instead of canonical `GenericConstraint`;
- a new hierarchy walker is proposed;
- `Self` fix requires changing runtime dispatch identity.

Checkpoint completion:
- [ ] constraint tests pass
- [ ] variance parity passes
- [ ] supertype projection tests pass
- [ ] HKT Self passes
- [ ] MON inheritance passes
- [ ] state updated

Suggested commits:
```text
fix(semantic): align HKT inference with constraints and variance
test(semantic): enforce HKT Self and supertype specialization
```

## Task 16 — Close HKT constraints and F-bounds

Purpose:
Reinstantiate canonical generic restrictions over HKT terms without allowing them to select candidates.

Risk:
- Semantic: HIGH
- Implementation fanout: local-to-multi-file

Owned files and symbols:
- `types/parameter.rs::GenericConstraint`
- `checker/call.rs` GenericWhere insertion
- `checker/inference.rs` constraint roles/term conversion

Inspect before editing:
- `resolve_generic_signature`;
- current GenericWhere term conversion;
- existing F-bound tests/specs.

Dependencies:
- C3.

Source of truth:
- canonical `GenericConstraint` attached to `GenericSignature`.

Implementation boundary:

Changes:
- ensure constraint terms containing applied constructor variables become inference terms using the active var map;
- preserve `DeclarationRestriction` role;
- validate after/during candidate selection;
- keep bound-only vars underconstrained.

Must not:
- bind a variable from its sole declaration upper bound;
- convert F-bound `T <: Comparable<T>` into `T = Comparable<T>`.

Edit operations:
1. Add RED source tests for ordinary bound-only and HKT bound-only behavior if not already active.
2. Trace GenericWhere conversion.
3. Ensure HKT applications recursively lift through `type_id_to_inference`.
4. Make restriction bounds ineligible as selecting evidence.
5. Add F-bound selected-candidate validation.
6. Verify diagnostic remains `GenericConstraintUnsatisfied` rather than generic conflict when declaration restriction owns the violation.

Testing classification:
- checkpoint evidence.

## Task 17 — Enforce inference/canonical variance parity

Purpose:
Make local inference relations observationally agree with `types/relation.rs`.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `InferenceSession::subtype_terms`
- `types/relation.rs::check_subtype_bounded`
- declaration variance lookup.

Inspect before editing:
- same-origin applied relation in both files;
- callable term relation;
- existing variance source tests.

Dependencies:
- Task 16.

Source of truth:
- declaration `Variance` metadata and canonical relation semantics.

Implementation boundary:

Changes:
- decompose applied terms according to declaration variance;
- apply callable parameter contravariance/result covariance;
- recursively compose polarity.

Must not:
- fall back to equality for covariant/contravariant structure;
- reverse relation due combined match arm.

Edit operations:
1. Build a parity table from `relation.rs`.
2. Add low-level inference cases mirroring canonical relation cases.
3. Modify only divergent local solver branches.
4. Keep directional canonical cases separate.
5. Run protected direction-preservation regression from recent fix.

Testing classification:
- focused parity tests.

## Task 18 — Reuse canonical generic-supertype projection

Purpose:
Allow HKT constraints to traverse declared generic inheritance without feature-specific hierarchy logic.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `types/relation.rs` generic supertype projection
- `types/specialization.rs`
- `checker/inference.rs` different-origin applied subtype path.

Inspect before editing:
- how `relation.rs` materializes `GenericSupertypeTemplate`;
- whether inference already calls the same `TypeHierarchy` methods.

Dependencies:
- Task 17.

Source of truth:
- `TypeHierarchy::supertype_template` and `TypeEnvironment`/`TypeView`.

Implementation boundary:

Changes:
Use the existing hierarchy template to transform one side before continuing inference.

Must not:
- copy `specialize_receiver_to_owner` loop into inference;
- use textual class names;
- silently ignore budget/cancellation.

Testing classification:
- transformed-inheritance HKT source test.

## Task 19 — Prove HKT + `Self`

Purpose:
Ensure owner generic specialization and receiver-relative Self substitution compose recursively.

Risk:
- Semantic: HIGH
- Implementation fanout: tests + possible specialization substitution fix

Owned files and symbols:
- `types/specialization.rs::specialize_receiver_to_owner`
- call signature specialization path
- `types/environment.rs` / `TypeView`
- source tests

Inspect before editing:
- current nested `Self` substitution behavior;
- `SelfTypeTerm` roles;
- MON receiver specialization helpers.

Dependencies:
- Task 18.

Source of truth:
- `ReceiverSpecialization.environment` binds declaration params and original receiver as Self.

Required source cases:
```text
Parent<F>.wrap() -> F<Self>
Child is Parent<List>
Child.wrap -> List<Child>

nested Box<F<Self>>
callable result () -> F<Self>
multi-hop transformed inheritance
```

Must not:
- bind `Self` to `Parent`;
- change selected `CallableId`.

Testing classification:
- checkpoint evidence.

---

# 12. Checkpoint C5 — Generic getters as canonical zero-argument applications

Tasks:
- Task 20 — Extend getter AST/parser with callable-local generics and where clauses.
- Task 21 — Publish generic getter signatures through `semantic_signature_for_syntax`.
- Task 22 — Forward expected context through getter property access.
- Task 23 — Add getter conformance, hostile cases, and source-index/tooling smoke checks.

Why this is a checkpoint:

Generic getters require syntax, canonical signature publication, and expected-result application to land together. Parser-only support is meaningless; semantic generics without expected propagation leave result-only getter parameters underconstrained even when context exists.

Entry conditions:
- C4 COMPLETE;
- ordinary generic application engine handles result-only contextual selection;
- getter access already calls `apply_resolved_callable`.

Working set:

Primary:
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-ast/tests/parser.rs`
- `phalcom-semantic/src/checker/declaration_signature.rs`
- `phalcom-semantic/src/checker/expression.rs`
- semantic capability tests

Secondary:
- enum behavior signature publication if getter AST is shared;
- source index/LSP only for smoke that existing `CallableId`/selector still resolves; no new LSP solver.

Out of scope:
- explicit type-argument syntax at getter access;
- generic setters/indexers;
- new selector kinds.

Semantic contract:
- a getter may declare callable-local generic binders and `where`;
- getter generic parameters have `TypeParameterOwner::Callable(getter_callable_id)`;
- selector/runtime identity remains ordinary getter;
- property access forwards expected result to canonical call application;
- no context leaves result-only generic underconstrained;
- constraints and evidence use ordinary generic mechanisms.

Semantic risks:
- parser ambiguity between getter generic header and method;
- accidentally permitting variance markers on callable binders if method policy rejects them;
- class-side declaration generics leaking into class getter scope;
- getter signature using declaration owner instead of callable owner;
- field-first lookup semantics changing.

Hostile cases:
- local field and getter same property behavior remains according to current field-first policy;
- generic getter no expected type;
- expected type violates where bound;
- inherited generic getter with transformed receiver;
- class-side getter cannot capture instance declaration generic if current SC-1 policy forbids class-side ambient generic params;
- enum behavior getter uses same semantics.

Required evidence:
1. parser positive + negative generic getter tests;
2. semantic signature owner/kind/constraint assertions;
3. property expected-result inference;
4. no-context underconstraint;
5. constraint failure;
6. inherited/Self getter;
7. existing ordinary getter tests.

Do not run yet:
- full workspace;
- generic setter/indexer tests.

Escalate if:
- getter generics appear to require selector grammar changes;
- implementation starts bypassing `semantic_signature_for_syntax`;
- LSP needs a new type checker rather than consuming updated semantic signature.

Checkpoint completion:
- [ ] AST/parser complete
- [ ] semantic signature publication complete
- [ ] expected context forwarded
- [ ] hostile tests pass
- [ ] ordinary getters unchanged
- [ ] state updated

Suggested commits:
```text
feat(ast): allow generic getter declarations
feat(semantic): apply generic getters through canonical call inference
test(semantic): cover contextual and constrained generic getters
```

## Task 20 — Extend `GetterDef` and parser

Purpose:
Represent the ratified generic getter declaration surface.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-ast/src/ast.rs::GetterDef`
- `phalcom-ast/src/parser.rs` class and enum behavior member parsing
- `phalcom-ast/tests/parser.rs`

Inspect before editing:
- `MethodDef` generic fields;
- parser variable `generic_parameters` and `where_clause` around member discrimination;
- latest `eat_less`/compact type-lambda logic only to avoid regressions.

Dependencies:
- C4.

Source of truth:
- source AST.

Implementation boundary:

Changes:
Add:
```text
generic_parameters
where_clause
```
to `GetterDef`, with the same syntax objects used by methods.

Remove the two explicit “generic parameters not permitted on getters” rejection branches.

Must not:
- add value parameters to getter;
- add generic args to selector;
- allow setter/index generics.

Edit operations:
1. OPEN `ast.rs`.
2. FIND `GetterDef`.
3. ADD fields matching method generic/where types.
4. OPEN parser class member path.
5. FIND exact rejection message.
6. REMOVE getter rejection and transfer already-parsed `generic_parameters`/`where_clause` into `GetterDef`.
7. Repeat enum behavior getter path.
8. `rg 'GetterDef \{'` across workspace and update constructors mechanically.
9. Add parser tests:
   - `value<T> -> T { ... }`;
   - `value<T> -> T where T <: Number { ... }`;
   - enum getter;
   - callable variance marker remains rejected according to existing callable binder rule.
10. Run `cargo test -p phalcom-ast --test parser <getter-pattern>`.

Code instructions:

EXACT field intent, reconcile actual AST types from `MethodDef`:

```rust
pub generic_parameters: Vec<GenericParameterSyntax>,
pub where_clause: Option<WhereClauseSyntax>,
```

Do not copy these exact type names if current `MethodDef` uses aliases/new wrappers; use the existing method fields verbatim.

Testing classification:
- parser evidence at checkpoint.

Optional compile:
`cargo check -p phalcom-ast`

## Task 21 — Publish getter `GenericSignature`

Purpose:
Make getter generics canonical declaration products.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `checker/declaration_signature.rs::semantic_signature_for_syntax`
- `CallableSyntaxRef::Getter`

Inspect before editing:
- method generic signature branch;
- declaration type-level bindings by side;
- generic resolver overlay;
- getter return formation.

Dependencies:
- Task 20.

Source of truth:
- `GenericSignature` owned by `TypeParameterOwner::Callable(callable.clone())`.

Implementation boundary:

Changes:
Factor/reuse the method path for callable-local generic signature formation so getter receives the same canonical machinery.

Must not:
- copy an independent constraint parser;
- set getter generics from class generic signature;
- change callable ID.

Current implementation:
Method branch resolves generics; getter branch publishes `generics: None`.

Target:
Getter branch resolves its own generics/where and uses a scoped resolver containing those callable-local binders before resolving return type.

Edit operations:
1. OPEN `declaration_signature.rs`.
2. FIND method generic publication block.
3. EXTRACT a private helper if doing so avoids duplicating the full outcome/diagnostic mapping.
4. Use helper for Method and Getter.
5. Ensure getter callable-local resolver overlays declaration resolver exactly as method does.
6. Preserve constructor special handling only for methods.
7. Add exact TypeParameterOwner/KindId/constraint tests.

Code instructions:

STRUCTURAL helper responsibility:

```text
resolve_callable_local_generics(
    callable,
    declaration_resolver,
    formation_site,
    generic_parameters,
    where_clause
) -> (Option<GenericSignature>, ScopedTypeResolver)
```

Return/API can differ; preserve current diagnostic handling.

Testing classification:
- semantic signature tests at C5.

## Task 22 — Forward `ExpectedType` through `synthesize_get_property`

Purpose:
Enable zero-argument result-directed getter inference through the existing call engine.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files and symbols:
- `checker/expression.rs::analyze_expression_inner`
- `checker/expression.rs::synthesize_get_property`

Inspect before editing:
- field-first branch;
- getter dispatch Found branch;
- current `apply_resolved_callable(..., &ExpectedType::None, ...)`.

Dependencies:
- Task 21.

Source of truth:
- existing canonical `apply_resolved_callable`.

Implementation boundary:

Changes:
- signature becomes `synthesize_get_property(ctx, get, expected)`;
- dispatch arm forwards `expected`;
- field access remains checked under existing field semantics rather than being turned into getter inference.

Must not:
- route field access into generic call engine;
- duplicate generic logic.

Edit operations:
1. CHANGE `Expr::GetProperty(get)` call to pass `expected`.
2. CHANGE helper signature.
3. In getter Found branch, replace `&ExpectedType::None` with `expected`.
4. Preserve receiver analysis with no result expectation.
5. Preserve field-first lookup and field causal invalidity.
6. Add contextual getter source test.

Code instructions:

EXACT mechanical target:

```rust
Expr::GetProperty(get) => synthesize_get_property(ctx, get, expected),
```

and getter application must pass the received `expected`.

Testing classification:
- focused C5 evidence.

## Task 23 — Generic getter hostile/conformance suite

Purpose:
Prove generic getter semantics are ordinary generic application semantics.

Risk:
- Semantic: HIGH
- Implementation fanout: tests

Owned files:
- create or extend descriptive semantic capability module for getters.
- existing ordinary getter/source signature tests.
- optional one integration source-index check.

Required cases:
1. expected-only success;
2. no-context underconstraint;
3. where bound success;
4. where bound failure;
5. inherited generic getter;
6. getter returning `F<Self>` if legal;
7. class-side scope hostility;
8. exact callable generic owner identity;
9. selector/CallableId unchanged by different contextual instantiations;
10. ordinary nongeneric getter behavior unchanged.

Cross-consumer consistency:
If semantic source occurrence/definition indexes expose getter declaration target, assert the generic getter still resolves to the same `CallableId`; do not add an LSP-only generic test unless adapter behavior changes.

Testing classification:
- C5 checkpoint evidence.

---

# 13. Checkpoint C6 — Executable generic surface parity and SC-4 certification

Tasks:
- Task 24 — Constructors and nullary/result-relevant owner generics.
- Task 25 — Enum/GADT constructor parity.
- Task 26 — Associated/family generic target parity.
- Task 27 — Union-receiver HKT/nested-inference parity.
- Task 28 — SC-4 deletion, incrementality, and package gate.

Why this is a checkpoint:

SC-4 is not complete when methods/getters work but another executable surface retains positional guessing, generic erasure, or a private solver. This checkpoint certifies one application calculus across all ratified surfaces and performs the first broad semantic package gate.

Entry conditions:
- C5 COMPLETE;
- union receiver call baseline implementation present;
- retained family targets present;
- ADT variant constructor products present.

Working set:

Primary:
- `checker/call.rs`
- `checker/expression.rs`
- `checker/associated.rs`
- enum/GADT constructor semantic products
- `types/specialization.rs`
- semantic ADT/family/incremental tests
- MON/Either core tests

Secondary:
- native surface importer;
- generated callable publication.

Out of scope:
- runtime monomorphization;
- new ADT syntax;
- SC-3 rows.

Semantic contract:
- all ratified executable generic targets share constraint generation, frame ownership, context selection, terminal outcome, and materialization semantics;
- no constructor positional generic guessing remains;
- variant residual generics do not become Object/Dynamic;
- family retained targets reinstate canonical declarations;
- union arms each specialize under the same inference context rules while source arguments are analyzed once;
- source/native/generated equivalent signatures behave equivalently.

Semantic risks:
- constructor special path bypassing C1 frames;
- variant/GADT constraints duplicated;
- family type structural shape losing target;
- union arm nested inference re-analyzing closure;
- broad incremental invalidation changes.

Hostile cases:
- constructor params reordered relative to class generic params;
- `Result::Ok(1)` leaves `E` underconstrained without context;
- `Option::None()` solved by expected `Option<Int>`;
- family capture stored in binding then invoked generically;
- union arm A/B require different HKT substitutions but common joined result;
- one union arm missing method remains invalid.

Required evidence:
1. constructor/variant/GADT/family focused modules;
2. union receiver generic source tests;
3. MON full;
4. Either full;
5. `cargo test -p phalcom-semantic --test semantic`;
6. focused incremental generic dependency tests;
7. negative searches for forbidden fallbacks.

Do not run yet:
- workspace all-target tests/clippy; final delivery.
- SC-4.5 flow/gadt broad work.

Escalate if:
- a surface cannot recover a canonical callable target and the temptation is to infer generics from structural callable shape alone;
- compiler/runtime selector changes appear necessary;
- a failing SC-3 row case is encountered; record as C7 dependency, do not absorb.

Checkpoint completion:
- [ ] surface parity complete
- [ ] semantic suite green or baseline incident classified
- [ ] MON/Either green
- [ ] negative gates pass
- [ ] incremental evidence passes
- [ ] SC-4 state marked COMPLETE
- [ ] no incident

Suggested commits:
```text
fix(semantic): converge executable generic surfaces on canonical inference
test(core): certify SC-4 generic application parity
docs(semantic): record SC-4 completion evidence
```

## Task 24 — Constructor generic application parity

Purpose:
Ensure class construction derives generics from constructor signature semantics only.

Risk:
- Semantic: HIGH
- Implementation fanout: local-to-multi-file

Owned files/symbols:
- `checker/expression.rs` unqualified/type-name construction path;
- `checker/call.rs` constructor target application;
- constructor semantic signature publication.

Inspect before editing:
- search `arguments.len()` near type-name construction;
- search for direct application of runtime arg types to declaration form;
- SC-2 positional-guessing deletion status.

Dependencies:
- C5.

Source of truth:
- canonical constructor `CallableSignature` and declaration owner.

Implementation boundary:
Delete any remaining path equivalent to:
```text
arg #0 -> generic #0
```
unless generated by actual signature constraints.

Required hostile fixture:
```text
class Pair<A,B> {
    @constructor
    new(_ second: B, _ first: A) { ... }
}
```
Call order must solve by formal signature.

Testing classification:
- focused constructor tests.

Negative search:
```bash
rg -n 'generic.*arguments|arguments.*generic|parameter.*index' phalcom-semantic/src/checker/expression.rs phalcom-semantic/src/checker/call.rs
```
Review every surviving hit; document intentional ones.

## Task 25 — Enum/GADT constructor parity

Purpose:
Use the same solver for variant owner generics and GADT case equations.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files/symbols:
- variant constructor target construction;
- enum semantic info/constructor signatures;
- `checker/call.rs` variant target application.

Inspect before editing:
- residual generic logic;
- GADT case environment insertion;
- nullary variant path.

Dependencies:
- Task 24.

Source of truth:
- enum `GenericSignature`;
- variant constructor signature;
- GADT case environment.

Required cases:
```text
Result::Ok(1) no expected result -> E underconstrained
let x: Result<Int,String> = Result::Ok(1) -> solved
let x: Option<Int> = Option::None() -> solved from context
GADT constructor exact result contributes equations
```

Must not:
- fallback unresolved payload type to Object;
- treat GADT equation as a parallel substitution map.

Testing classification:
- semantic ADT constructor/generics modules + Either.

## Task 26 — Associated/family generic parity

Purpose:
Ensure first-class family invocation with retained declaration target uses canonical generic application.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files/symbols:
- `checker/associated.rs::FamilyApplicationCandidate`
- `FamilyApplicationSelection`
- expression family invocation path
- `InvocationTargetId`

Inspect before editing:
- recent retained-target fixes;
- structural family invocation fallback.

Dependencies:
- Task 25.

Source of truth:
- retained `InvocationTargetId` / canonical callable declaration.

Implementation boundary:
When target exists, reconstruct `CallableApplicationTarget` from it and use ordinary call engine.

Must not:
- infer hidden polymorphism from `TypeData::Callable`;
- use structural Family type equality as declaration identity.

Hostile cases:
- captured family stored in a binding;
- same structural Family shape from two declarations;
- generic + Self family target;
- target intentionally absent -> remain monomorphic/dynamic per current family semantics.

Testing classification:
- family semantic tests.

## Task 27 — Union-receiver HKT and nested inference parity

Purpose:
Certify the new baseline union-call implementation remains correct with generalized HKT/nested contexts.

Risk:
- Semantic: HIGH
- Implementation fanout: tests, production only if defect

Owned files/symbols:
- `checker/call.rs::apply_union_resolved_call`
- `UnionCallArm`
- expression union receiver resolution
- union call tests.

Inspect before editing:
- baseline `PreAnalyzed` argument-once path;
- arm status/evidence join.

Dependencies:
- C1–C4.

Source of truth:
- existing union arm application implementation + canonical generic call engine.

Required hostile cases:
1. same generic method on A|B with same result;
2. per-arm HKT specializations join;
3. callback expression analyzed once;
4. one arm missing method invalid;
5. one arm dynamic/ambiguous handled per existing policy;
6. nested generic call inside callback does not get duplicate frames from arm replay.

Must not:
- re-analyze source arguments per arm.

Testing classification:
- focused union call tests.

## Task 28 — SC-4 negative, incremental, and broad semantic gate

Purpose:
Prove SC-4 is integrated and forbidden old mechanisms cannot silently run.

Risk:
- Semantic: HIGH
- Implementation fanout: verification + cleanup

Owned areas:
- all SC-4 touched files;
- incremental callable/checker dependency tests;
- MON/Either.

Required negative/deletion searches:

```bash
rg -n 'InferenceSession::new\(' phalcom-semantic/src/checker
```

Expected:
- only intentional root/test creation or the new context manager; no per-nested-call private session creation.

```bash
rg -n 'ExpectedType::Inference' phalcom-semantic/src
```

Expected:
- every constructor carries owning context ID after C1.

```bash
rg -n 'generic parameters not permitted on getters' phalcom-ast
```

Expected:
- zero production hits.

```bash
rg -n 'apply_resolved_callable\(.*ExpectedType::None' phalcom-semantic/src/checker
```

Expected:
- inspect every hit; getter contextual application must not remain hardcoded None.

```bash
rg -n 'Object.*fallback|fallback.*Object' phalcom-semantic/src/checker
```

Expected:
- no generic constructor/variant fallback introduced or retained in SC-4 paths.

Required tests:

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations::inference -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::capabilities -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::adts -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::families -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::incremental -- --nocapture
cargo test -p phalcom-core --test core monads:: -- --nocapture
cargo test -p phalcom-core --test core either:: -- --nocapture
cargo test -p phalcom-semantic --test semantic
```

What the final semantic command proves:
- crate integration across semantic modules after SC-4.

What it does not prove:
- workspace/compiler/LSP delivery readiness; deferred to Final Gate.

Checkpoint state:
Mark:
```text
SC-4 — COMPLETE
```
only if every C0–C6 checkpoint is COMPLETE.


---

# 14. SC-4.5 entry rule

SC-4.5 begins only after C6 marks SC-4 COMPLETE.

SC-4.5 does not reopen the generic inference architecture. Its job is to prove that:

```text
every supported expression
every canonical type relation
every flow/refinement path
every ADT/GADT elimination path
every source generic/alias/family surface
every durable/public consumer
```

uses the closed semantic model correctly.

If a SC-4.5 test exposes a real SC-4 solver bug, mark the current checkpoint INCIDENT and repair the owning SC-4 invariant explicitly. Do not work around it in expression/flow code.

---

# 15. Checkpoint C7 — SC-3 dependency gate and canonical relation matrix

Tasks:
- Task 29 — Reconcile actual SC-3 implementation status against its completion gates.
- Task 30 — Build the canonical relation coverage matrix from current `TypeData`.
- Task 31 — Close relation parity defects without duplicating SC-3 or inference logic.

Why this is a checkpoint:

Whole-language closure cannot be certified if Record rows are still transitional or if some `TypeData` form has undefined/inconsistent relation behavior. C7 is a dependency-and-authority gate, not permission to reimplement SC-3.

Entry conditions:
- C6 COMPLETE;
- current SC-3 plan/spec available;
- current `TypeStore::TypeData` enumerated;
- current relation implementation readable.

Working set:

Primary:
- `phalcom-semantic/src/types/store.rs`
- `phalcom-semantic/src/types/relation.rs`
- `phalcom-semantic/src/types/row.rs`
- `phalcom-semantic/src/types/row_solver.rs`
- SC-3 implementation/state documents
- relation/record tests
- `semantic/advanced/record_rows.rs`

Secondary:
- `types/family.rs`
- `types/type_lambda.rs`
- exact case/enum identity APIs

Out of scope:
- implementing missing SC-3 tasks unless the agent is explicitly executing SC-3 as a prerequisite;
- effects;
- reflection feature expansion.

Semantic contract:
- SC-4.5 knows whether SC-3 is COMPLETE or is blocked with an explicit incident;
- every current proper `TypeData` form has a defined equality/subtype role where meaningful;
- type constructors/lambdas are distinguished from proper runtime value types;
- relation and inference parity established in C4 remains intact;
- Record relation uses only the SC-3 canonical immutable structural model once SC-3 is complete.

Semantic risks:
- declaring whole-system closure while SC-3 is incomplete;
- retaining obsolete `RecordAccess` as a second language policy;
- accidentally treating `TypeData::Lambda` as ordinary value subtype;
- family structural equality fabricating declaration denotation.

Hostile cases:
- open source Record <: closed required-prefix Record;
- closed Record with missing field;
- family structural match but different retained declaration target;
- exact case vs owning enum;
- callable labels/rest mismatch.

Required evidence:
1. SC-3's own completion commands/evidence;
2. generated relation matrix checked against every current `TypeData` variant;
3. focused relation tests;
4. negative search for obsolete RecordAccess once SC-3 is complete.

Do not run yet:
- flow/ADT broad tests;
- workspace.

Escalate immediately if:
- SC-3 is not complete: mark C7 INCIDENT/BLOCKED; do not falsely complete C7;
- relation behavior depends on presentation strings;
- a new type form has no explicit relation policy.

Checkpoint completion:
- [ ] SC-3 prerequisite status is explicit and satisfied
- [ ] relation matrix complete
- [ ] relation parity tests pass
- [ ] obsolete Record relation policy removed where SC-3 requires it
- [ ] state updated
- [ ] no incident

Suggested commit:
```text
test(semantic): certify canonical type relation coverage
```
Production relation fixes should be separate focused commits if required.

## Task 29 — Reconcile actual SC-3 completion status

Purpose:
Make SC-3 an explicit dependency rather than silently assuming its plan has been executed.

Risk:
- Semantic: HIGH
- Implementation fanout: inspection/evidence

Owned files:
- SC-3 spec/plan/state if present;
- `types/row*`;
- row tests.

Inspect before editing:
- exact current `RecordRowData`/solver;
- open row source lowering;
- `GenericApplicationSession`/row inference bridge if implemented;
- row materialization;
- immutable Record relation;
- metadata/publication gates.

Dependencies:
- C6.

Source of truth:
- executable current SC-3 tests and current production code, not SC-3 planning text.

Implementation boundary:

Changes:
None if SC-3 is already complete; record evidence.
If incomplete, stop C7 and execute the separately approved SC-3 implementation program before returning.

Must not:
- cherry-pick only enough row behavior to make C7 tests green;
- encode row vars in ordinary C1 inference context.

Edit operations:
1. Compare current code to SC-3 completion ledger.
2. Run SC-3 focused tests/commands from its plan.
3. Record each required gate.
4. Mark C7 blocked if any core SC-3 gate is missing.

Testing classification:
- dependency gate.

## Task 30 — Build canonical relation matrix

Purpose:
Ensure no current proper type form lacks an explicit relation policy.

Risk:
- Semantic: MEDIUM
- Implementation fanout: tests/documentation

Owned files:
- `types/store.rs::TypeData`
- `types/relation.rs`
- create/update relation test module under semantic foundations/advanced.

Source of truth:
- current `TypeData` enum + canonical relation API.

Implementation boundary:

For each variant, classify:

| Form | Proper value type? | Kind | Equality | Subtyping | Notes |
|---|---|---|---|---|---|
| Never | yes | Type | canonical | bottom | |
| Unit | yes | Type | canonical | ordinary | |
| ClassObject | yes | Type | nominal | class-object rules | |
| Nominal | yes | Type | declaration | inheritance | |
| Applied | kind-dependent; proper when residual kind Type | residual | canonical | variance/inheritance | |
| ExactCase | yes | Type | case+enum | <: owner enum | |
| Union | yes | Type | canonical set | union algebra | |
| Tuple | yes | Type | structural | structural | |
| Record | yes | Type | canonical row | SC-3 structural | |
| Callable | yes | Type | structural | contra/co | |
| Family | yes as family value type | Type | structural | width/member | denotation separate |
| Parameter | proper only when parameter kind Type | declared | rigid identity | relation through bounds/environment | |
| Lambda | no ordinary runtime value type form | arrow kind | alpha/canonical | no ordinary value subtype | |
| SelfType | declaration term/form as represented | role-specific | specialization-required | do not compare as unspecialized runtime value | |

Reconcile exact current enum if it changed.

Edit operations:
1. Enumerate `TypeData`.
2. For every match arm in `relation.rs`, map supported forms.
3. Add explicit tests for missing matrix cells that should be meaningful.
4. Add tests proving intentionally non-applicable constructor relations fail/are blocked structurally rather than being treated as value subtype.

Testing classification:
- checkpoint evidence.

## Task 31 — Close relation parity defects

Purpose:
Fix only concrete matrix inconsistencies.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files:
- `types/relation.rs`;
- SC-3 Record relation owner if needed;
- relation tests.

Dependencies:
- Task 30.

Source of truth:
- matrix + ratified semantics.

Must not:
- change inference relation independently; if a relation fix changes C4 parity, update both at the owning shared abstraction or reopen C4 incident.
- infer family denotation from structure.

Required negative gate after SC-3:
```bash
rg -n 'RecordAccess' phalcom-semantic/src
```

Expected:
- zero production hits if SC-3 completion removed access modes as ratified.
- if SC-3 intentionally retains a compatibility symbol internally, document every hit and prove ordinary immutable Record relation does not branch on it.

Testing classification:
- focused relation module.

---

# 16. Checkpoint C8 — Bidirectional expression typing closure

Tasks:
- Task 32 — Inventory every `Expr` variant and expected-type owner.
- Task 33 — Close structural literal/product expectation gaps.
- Task 34 — Close call/property/index/match expectation forwarding gaps.
- Task 35 — Add expression-kind hostile coverage and remove unsupported fallthroughs that are actually implemented language constructs.

Why this is a checkpoint:

A closed solver is useless if expression dispatch silently drops contextual typing. C8 produces a complete expression ownership matrix and repairs only genuine expected-type propagation gaps.

Entry conditions:
- C7 COMPLETE;
- SC-4 generic getter path active;
- SC-3 Record expectation rules active.

Working set:

Primary:
- `phalcom-ast/src/ast.rs::Expr`
- `checker/expression.rs::analyze_expression_inner`
- `checker/expected.rs`
- relevant expression helper modules
- semantic expression-engine/capability tests

Secondary:
- match/control helpers;
- collection composition helpers.

Out of scope:
- inventing new semantics for parser-supported but unratified expressions;
- effects;
- flow joins (C9).

Semantic contract:
- every supported `Expr` variant has one explicit typing owner;
- expected type is consumed wherever bidirectional rules say it matters;
- expected context does not overwrite intrinsic knowledge;
- unsupported constructs remain explicitly unavailable rather than silently “successful Unknown.”

Semantic risks:
- forwarding expected type into an expression where it should not influence synthesis;
- treating collection expected origin as Established;
- duplicate analysis of subexpressions;
- changing diagnostic owner.

Hostile cases:
- empty collection under expected generic collection;
- tuple expected components with one mismatch;
- record expected known fields + SC-3 tail;
- property getter contextual selection;
- index result expected context if canonical index call supports it;
- incompatible expected union member set.

Required evidence:
1. expression coverage table;
2. focused expression engine tests;
3. collection/product tests;
4. generic getter tests retained;
5. no unexpected `Unknown(UncheckedExpression)` for supported cases.

Do not run yet:
- full ADT/GADT ignored set;
- workspace.

Escalate if:
- closing an expression variant requires inventing an unratified operator/type rule;
- the same expression is being typed independently in compiler/LSP.

Checkpoint completion:
- [ ] expression matrix complete
- [ ] expected propagation gaps fixed
- [ ] hostile tests pass
- [ ] unclassified supported fallthroughs removed
- [ ] state updated

Suggested commits:
```text
fix(semantic): complete bidirectional expression expectations
test(semantic): certify expression typing ownership
```

## Task 32 — Inventory `Expr` variants and typing owners

Purpose:
Create the definitive expression coverage map before patching.

Risk:
- Semantic: MEDIUM
- Implementation fanout: inspection/tests

Owned symbols:
- `phalcom_ast::ast::Expr`
- `analyze_expression_inner`.

Source of truth:
- AST enum and semantic dispatcher.

Edit operations:
1. Enumerate every `Expr` variant from current HEAD.
2. Map each to its semantic helper/match arm.
3. Record:
   - synthesis only;
   - uses expected proper type;
   - uses expected inference type;
   - expected type intentionally ignored;
   - unsupported/unratified.
4. Search for helper functions that recursively call `analyze_expression(... ExpectedType::None)` and classify whether that is semantically correct.
5. Convert the matrix into tests only for actual gaps.

Testing classification:
- no standalone behavior until Task 35.

## Task 33 — Structural literal/product expected typing

Purpose:
Close expected component propagation for lists/sets/maps/tuples/Records.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files:
- expression literal synthesis helpers;
- `expected.rs`;
- SC-3 Record projection helpers.

Source of truth:
- canonical expected proper/inference type structure.

Changes:
- lists/sets: element expectation when canonical container origin is appropriate;
- maps: key/value expectations;
- tuples: per-position/label expectations;
- Records: known field expectation + row-safe behavior;
- empty literals: contextual type can select shape without fabricating Established value evidence.

Must not:
- assume first applied argument is element type for arbitrary unknown container origins; use canonical known collection declarations/protocol rules where current code does.
- turn Map dynamic keys into Record rows.

Testing classification:
- focused structural/collection tests.

## Task 34 — Call/property/index/match expected forwarding

Purpose:
Ensure call-like or result-producing expression helpers do not hardcode `ExpectedType::None` where result context is semantically meaningful.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:
- `checker/expression.rs`;
- call/index/property helpers;
- match expression/control helper.

Inspect before editing:
```bash
rg -n 'ExpectedType::None' phalcom-semantic/src/checker
```

Classify every hit; do not mass-replace.

Source of truth:
- canonical expression rule per helper.

Changes:
- retain getter C5 behavior;
- forward expected to resolved method/unqualified/associated call paths where already intended;
- index getter/setter only if canonical callable application supports the result context;
- match arm expected context to reachable normal result arms where current semantics require it.

Must not:
- pass outer expression expectation to receiver operands;
- use expected type to change runtime selector resolution.

Testing classification:
- focused source tests.

## Task 35 — Remove supported-expression `UncheckedExpression` holes

Purpose:
Ensure remaining `Unknown(UncheckedExpression)` instances are intentional.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file as discovered

Owned files:
- all checker files returned by search.

Edit operations:
1. Run:
```bash
rg -n 'UnknownReason::UncheckedExpression' phalcom-semantic/src/checker
```
2. Classify each as:
   - intentional unratified/unsupported;
   - blocked by another explicit stage;
   - defect for a supported AST form.
3. Repair only supported defects.
4. Add a source test per distinct semantic risk, not per occurrence.
5. Record intentional remaining hits in state.

Negative gate:
No unclassified hit remains.

Testing classification:
- C8 evidence.

---

# 17. Checkpoint C9 — Flow/refinement, joins, loops, captures, and conservative call invalidation

Tasks:
- Task 36 — Audit persistent contract vs current knowledge transfer.
- Task 37 — Close branch/abrupt-flow join laws.
- Task 38 — Close loop fixed-point and capture laws.
- Task 39 — Introduce/verify conservative pre-effect call invalidation.

Why this is a checkpoint:

Flow correctness is one semantic system. Branch joins, loops, and captures must agree about what facts survive. Effects are deferred, so C9 establishes a sound conservative call boundary without pretending to know mutation effects.

Entry conditions:
- C8 COMPLETE;
- current `FlowState`/loop analysis active;
- no effects system required.

Working set:

Primary:
- `checker/flow/state.rs`
- `checker/flow/transfer.rs`
- `checker/control.rs`
- `checker/loop_analysis.rs`
- `checker/context.rs`
- `checker/body.rs`
- flow branch/loop/capture tests

Secondary:
- field lifecycle;
- call result integration for invalidation hook.

Out of scope:
- effect inference;
- alias analysis beyond what is necessary to conservatively drop facts;
- termination proof.

Semantic contract:
- declared/persistent constraints never become mutable flow facts;
- writes validate persistent constraints;
- refinements are path-local;
- joins include exactly reachable normal-flow states;
- loop analysis reaches an actual fixed point or explicit bounded failure;
- captured identities remain stable;
- calls drop facts that cannot be proven stable without effects.

Semantic risks:
- widening to contract after refutation;
- abrupt branch contributing to join;
- fixed-point “Changed” without persistent mutation;
- over-invalidating local scalar facts;
- under-invalidating fields/alias-sensitive facts.

Hostile cases:
- one branch returns/throws and should not widen join;
- conflicting branch writes produce union/current knowledge;
- loop executes zero times;
- break/continue;
- closure mutates captured binding;
- call between type test and field use invalidates field refinement unless stability proven.

Required evidence:
1. existing `flow_branches`/`flow_loops` modules;
2. deep regression provenance tests;
3. new call-invalidation hostile tests;
4. no effect metadata required.

Do not run yet:
- ADT/GADT full;
- workspace.

Escalate if:
- implementation begins creating an effect lattice in C9;
- local binding facts are being dropped merely because any call occurs, without distinguishing non-aliasable state.

Checkpoint completion:
- [ ] contracts/current knowledge audit passes
- [ ] branch joins pass
- [ ] loop fixed points pass
- [ ] captures pass
- [ ] conservative invalidation tests pass
- [ ] state updated

Suggested commits:
```text
fix(semantic): close flow refinement and join invariants
test(semantic): enforce conservative pre-effect call invalidation
```

## Task 36 — Audit contract/current flow separation

Purpose:
Ensure all binding/field transfer paths preserve the two-level semantic model.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:
- `flow/state.rs`;
- binding reconciliation;
- field lifecycle;
- assignment paths.

Source of truth:
- persistent contract stored in binding/field state;
- current `TypeKnowledge` as path fact.

Edit operations:
1. Inspect `BindingState`, `FieldState`.
2. Trace declaration initialization, assignment, refinement, join.
3. Search for assignments that overwrite persistent contract from current inferred type.
4. Add hostile test: broad declared contract + narrow current value + later reassignment.
5. Repair only concrete violations.

Must not:
- use declared type as a recovery current fact after actual current evidence was refuted.

Testing classification:
- flow-focused module at C9.

## Task 37 — Branch and abrupt-flow joins

Purpose:
Ensure joins include only reachable normal states.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files:
- `control.rs`;
- flow join helpers;
- body exit facts if needed.

Required cases:
- same-type branches;
- heterogeneous union;
- return branch excluded;
- throw branch excluded;
- nested branches;
- branch-local shadowing;
- diagnostic/causal status preserved.

Inspect known coverage-ledger note about `BodyExitFacts` fidelity. If publication is insufficient for a required proof, improve the product rather than asserting through an unrelated final type.

Testing classification:
- `flow_branches` + deep regressions.

## Task 38 — Loop and closure fixed-point laws

Purpose:
Protect convergence and captured identity.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files:
- `loop_analysis.rs`;
- `flow/state.rs`;
- closure/capture body handling.

Required cases:
- preheader included because loop may execute zero times;
- backedge changes converge;
- break states join at exit;
- continue states feed header;
- repeated unchanged state is not reported as progress;
- captured write observed outside according to current closure execution semantics;
- nested closure capture identity is stable.

Must not:
- increase iteration cap to hide non-progress.

Testing classification:
- loop/capture modules.

## Task 39 — Conservative call invalidation before effects

Purpose:
Keep flow sound while effects remain deferred.

Risk:
- Semantic: HIGH
- Implementation fanout: local-to-multi-file

Owned files:
- call completion boundary;
- flow fact invalidation APIs.

Inspect before editing:
- what current facts represent binding-local vs field/object facts;
- whether an existing invalidation helper already exists.

Source of truth:
- current FlowState fact ownership; absent effects means no proof of heap stability.

Implementation boundary:

Minimum rule:
```text
local immutable/value facts with no aliasable mutation path
    preserve

field/object/alias-sensitive refinement that an opaque call could invalidate
    drop unless current semantic model proves stability
```

Must not:
- add effect annotations;
- assume ordinary source call is pure;
- erase all local knowledge unconditionally.

Testing classification:
- focused hostile call/refinement tests.

---

# 18. Checkpoint C10 — ADT/GADT elimination and proof closure

Tasks:
- Task 40 — Activate/fix exact pattern payload specialization and ambiguity diagnostics.
- Task 41 — Close exhaustiveness/usefulness/witness gaps.
- Task 42 — Close GADT branch-local equality and impossible-case reasoning.
- Task 43 — Integrate match flow joins with C9 invariants.

Why this is a checkpoint:

Construction-time generic inference is already under the canonical application engine, but full ADT/GADT type-system closure requires the inverse operation: pattern elimination must establish exact local facts, prove residual value space, and join flow without leaking GADT equations.

Entry conditions:
- C9 COMPLETE;
- `solve_gadt_branch_proof` remains sole GADT proof authority;
- current exact case representation stable.

Working set:

Primary:
- `checker/pattern.rs`
- `checker/gadt_proof.rs`
- `checker/pattern_space.rs`
- `checker/exhaustiveness.rs`
- match flow/checker integration
- semantic ADT matching tests

Secondary:
- parser only for tests currently failing due fixture syntax; parser fixes are allowed only when syntax is already ratified and the failure is truly parser-owned.

Out of scope:
- changing runtime match defensive behavior;
- general theorem proving;
- new pattern syntax.

Semantic contract:
- exact variant pattern payloads use specialized types;
- ambiguity/visibility diagnostics do not choose arbitrary candidates;
- exhaustiveness/usefulness operate on sound pattern space;
- guards are conservative;
- GADT equations are branch-local;
- impossible cases require contradiction proof;
- match flow joins only reachable normal arms.

Semantic risks:
- using enum root type instead of exact case;
- branch proof leaks into outer flow;
- guarded arm counted as unconditional coverage;
- parser fixture bugs mistaken for semantic defects;
- abrupt arm included in result join.

Hostile cases:
- nested GADT proof;
- two contextual variant owners ambiguous;
- callable-family residual;
- multi-field witness;
- all arms abrupt => Never normal result;
- branch writes join after match.

Required evidence:
- currently gated/failing ADT tests reclassified and activated;
- exact diagnostic-code assertions;
- flow joins;
- no arbitrary ambiguity selection.

Do not run yet:
- workspace; run semantic ADT module.

Escalate if:
- GADT equality starts using ordinary generic call solver directly instead of its canonical branch proof environment;
- parser change would introduce new syntax not ratified.

Checkpoint completion:
- [ ] pattern specialization green
- [ ] exhaustiveness/usefulness green
- [ ] GADT branch-local tests green
- [ ] match flow green
- [ ] state updated

Suggested commits:
```text
fix(semantic): close GADT elimination and match proof gaps
test(semantic): activate ADT exhaustiveness and branch-refinement laws
```

## Task 40 — Pattern specialization and resolution diagnostics

Purpose:
Close exact payload typing and variant owner resolution defects.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files:
- `pattern.rs`;
- variant resolution helpers;
- ADT diagnostics tests.

Source of truth:
- canonical `VariantInfo`, exact case, `solve_gadt_branch_proof`.

Required hostile cases from known ledger:
- ambiguous contextual owner reports candidates/no arbitrary selection;
- inaccessible variant diagnostic points at explicit name;
- payload arity/type projections.

Testing classification:
- focused ADT matching diagnostics/resolution.

## Task 41 — Exhaustiveness/usefulness/witness closure

Purpose:
Make pattern-space elimination complete for currently ratified patterns.

Risk:
- Semantic: HIGH
- Implementation fanout: local-to-multi-file

Owned files:
- `pattern_space.rs`;
- `exhaustiveness.rs`.

Inspect before editing:
- current residual representation;
- known callable-family singleton residual defect;
- tuple/list product parser fixture statuses.

Source of truth:
- canonical scrutinee type + pattern-space algebra.

Required cases:
- exact enum variants;
- tuple product coverage;
- nested/multi-field witness;
- list partitions if syntax is ratified/current parser supports them;
- guarded arms;
- callable family residual classification.

Must not:
- mark a case covered merely because runtime would catch with MatchError.

Testing classification:
- ADT exhaustiveness module.

## Task 42 — GADT branch proof locality

Purpose:
Close nested equality propagation and impossible-case reasoning.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files:
- `gadt_proof.rs`;
- `pattern.rs`;
- branch refinement environment.

Source of truth:
- `GadtProofResult` and exact case environment.

Required cases:
- nested GADT proof is visible inside branch;
- not visible in sibling/after match;
- contradictory concrete scrutinee makes case impossible;
- symbolic scrutinee refines local generic;
- generic owner identity exact, no name matching.

Testing classification:
- GADT refinement module.

## Task 43 — Match flow integration

Purpose:
Apply C9 reachable-flow rules to match arms.

Risk:
- Semantic: HIGH
- Implementation fanout: local

Owned files:
- match checking flow integration;
- flow tests.

Required cases:
- abrupt arm excluded;
- all abrupt -> `Never` normal result;
- branch writes join;
- outer same-name binding restored after arm scope;
- impossible arm does not contribute flow.

Testing classification:
- match flow module.

---

# 19. Checkpoint C11 — Source constraints, variance, nested `Self`, aliases, and Families

Tasks:
- Task 44 — Activate full source generic constraint coverage.
- Task 45 — Activate source variance and nested `Self` coverage.
- Task 46 — Close transparent/generic alias source semantics.
- Task 47 — Close ratified Family capture/invocation source coverage and refresh the coverage ledger.

Why this is a checkpoint:

These are largely earlier-stage features whose core representations now exist, but a whole-system claim requires source-level proof. C11 converts historical “staged/gated” coverage into executable conformance or explicit remaining scope exclusions.

Entry conditions:
- C10 COMPLETE;
- SC-4 application semantics fixed;
- SC-3 rows complete.

Working set:

Primary:
- source semantic tests;
- `types/annotation.rs`
- declaration/session alias publication
- `types/relation.rs` only if tests expose a real source-to-canonical mismatch
- Family source/capture tests
- `COVERAGE_LEDGER.md`

Secondary:
- metadata/incremental alias tests.

Out of scope:
- first-class forall;
- typeclass search;
- new alias opacity feature;
- new Family syntax.

Semantic contract:
- source syntax reaches existing canonical constraint/variance/Self/alias/Family products without fallback;
- same laws hold across class/method owners;
- transparent alias semantics do not create nominal/runtime identity;
- Family structural type and declaration denotation remain distinct;
- coverage ledger reflects current evidence, not old implementation history.

Semantic risks:
- tests asserting old gated status despite implementation landing;
- source generic parameter owner confusion;
- alias cycle silently Unknown;
- Family target lost through binding;
- local presentation strings treated as semantic oracle.

Hostile cases:
- same-named generic parameters on class/method;
- contra/invariant relation;
- nested `Box<Self>`;
- alias of constructor-valued lambda;
- generic alias cycle;
- structurally equal Family values from different targets.

Required evidence:
1. constraints source module;
2. variance source module;
3. nested Self;
4. alias integration/incremental;
5. Family source invocation;
6. regenerated ledger with every type-system row READY, explicitly BLOCKED by SC stage, or intentionally out of scope.

Do not run yet:
- final workspace until C12.

Checkpoint completion:
- [ ] source constraints certified
- [ ] variance/Self certified
- [ ] aliases certified
- [ ] Families certified
- [ ] ledger refreshed
- [ ] state updated

Suggested commits:
```text
test(semantic): activate source generic and variance closure
fix(semantic): close alias and family source-semantic gaps
docs(semantic): refresh type-system coverage ledger
```

## Task 44 — Full source generic constraint coverage

Purpose:
Prove canonical constraint machinery is actually reachable from source owners.

Risk:
- Semantic: HIGH
- Implementation fanout: tests + formation if defect

Owned files:
- `types/annotation.rs::resolve_generic_signature`;
- source generic tests.

Required source matrix:
```text
class where T <: Number
method where T <: Number
T == U
Number <: T if ratified syntax
F-bound T <: Comparable<T>
class-owned T + method-owned U
generic superclass constraint substitution
distinct bound-violation diagnostic
```

Must not:
- invent defaults;
- conflate owner IDs.

Testing classification:
- focused source semantic module.

## Task 45 — Source variance and nested Self

Purpose:
Activate relation laws through real source declarations.

Risk:
- Semantic: HIGH
- Implementation fanout: tests + formation if defect

Owned files:
- generic binder formation;
- relation;
- receiver specialization.

Required cases:
- covariant declaration relation;
- contravariant;
- invariant;
- nested callable occurrence;
- transformed superclass variance;
- `Box<Self>`;
- `F<Self>`;
- class-side Self/ambient generic scope according to ratified SC-1 policy.

Testing classification:
- focused source module + C4 regressions.

## Task 46 — Transparent/generic alias closure

Purpose:
Make alias source paths observationally equivalent to canonical target forms while preserving alias declaration provenance.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file if defect

Owned files:
- alias declaration/session publication;
- resolver alias form;
- metadata/incremental alias tests.

Source of truth:
- alias declaration identity + canonical target type form.

Required cases:
- non-generic alias;
- generic alias;
- nested alias;
- constructor-valued alias/type lambda;
- alias in generic inference;
- cycle/malformed;
- cold/incremental;
- no new runtime nominal identity.

Testing classification:
- source/integration/incremental.

## Task 47 — Family source closure and coverage ledger refresh

Purpose:
Close ratified first-class Family semantics and make coverage status current.

Risk:
- Semantic: HIGH
- Implementation fanout: tests/documentation + production if real gap

Owned files:
- Family capture/resolution;
- Family tests;
- `COVERAGE_LEDGER.md`.

Required Family cases:
- exact getter/method family capture;
- pattern family;
- binding storage and invocation;
- class/instance distinction;
- hierarchy dependency;
- wrong-shape diagnostic;
- generic/Self specialization;
- two structural-same families retain distinct denotation/targets.

Ledger procedure:
1. Re-evaluate all old G/C/K/V/P/A/F slots against current tests.
2. Mark READY only with named source test and concrete oracle.
3. Do not mark MON-only tests as source-semantic coverage for unrelated declaration surfaces.
4. Remove stale “gated because type lambda syntax missing” entries where current source tests prove otherwise.
5. Record remaining non-type-system gates separately.

Testing classification:
- C11 evidence.

---

# 20. Checkpoint C12 — Epistemic, publication, incrementality, deletion, and SC-4.5 certification

Tasks:
- Task 48 — Audit Dynamic/Unknown/Invalid/Blocked/Cancelled/Budget state preservation.
- Task 49 — Prove no solver-local state crosses public semantic boundaries.
- Task 50 — Complete cold/incremental equivalence for new/closed type products.
- Task 51 — Run deletion/authority searches and remove obsolete compatibility mechanisms.
- Task 52 — Execute SC-4.5 semantic/core certification gate and close state.

Why this is a checkpoint:

The type system is not closed merely because individual features work. Final closure requires semantic states to remain distinct, public products to be canonical, incremental recomputation to agree with cold analysis, and obsolete alternate authorities to be unable to run.

Entry conditions:
- C11 COMPLETE;
- all prior incidents resolved;
- coverage ledger current.

Working set:

Primary:
- `checker/analysis.rs`
- `checker/typed_expr.rs`
- evidence/outcome types
- semantic snapshot/publication
- session/incremental database
- metadata exporter for type products
- incremental tests
- touched SC-4/C8-C11 areas

Secondary:
- LSP integration smoke only as consumer, not semantic owner;
- compiler metadata consumer only where public type product consistency is relevant.

Out of scope:
- new reflection features;
- effect/proof publication;
- performance optimization beyond correctness regressions.

Semantic contract:
- `Dynamic`, `Unknown`, invalidity, blocked, cancelled, budget, and internal failure remain distinct;
- public type facts contain only canonical/durable IDs;
- cold and incremental final source produce equivalent type facts/targets/diagnostics;
- no old generic fallback/parallel authority remains;
- every type-system coverage gap is classified.

Semantic risks:
- cancellation becomes Unknown;
- internal failure becomes source invalidity;
- checker-local inference fact accidentally cloned into snapshot;
- stale incremental cache uses old generic signature/constraint;
- compatibility fallback survives and masks a missing dependency.

Hostile cases:
- edit generic constraint only;
- edit variance only;
- edit superclass HKT template;
- edit generic getter return/where;
- edit alias target;
- edit Family target;
- incremental nested generic call must equal cold result;
- cancelled/budgeted solve publishes no Ready result.

Required evidence:
1. epistemic hostile tests;
2. public-boundary assertions;
3. cold/incremental comparisons;
4. negative searches;
5. full semantic integration binary;
6. full relevant core conformance modules.

Do not run yet:
- only final workspace delivery gates remain after C12.

Escalate if:
- public API currently serializes `InferVarId`;
- incremental equivalence requires disabling cache;
- deletion of fallback breaks behavior because a canonical dependency was never published: classify DEPENDENCY/PUBLICATION and fix owner, do not restore fallback.

Checkpoint completion:
- [ ] epistemic states certified
- [ ] no solver state leaks
- [ ] cold/incremental parity certified
- [ ] deletion gates pass
- [ ] semantic/core certification passes
- [ ] coverage ledger has no unclassified type-system gap
- [ ] state marks SC-4.5 COMPLETE
- [ ] no incident

Suggested commits:
```text
fix(semantic): close type-system publication and incremental invariants
test(semantic): certify cold incremental and epistemic type closure
docs(semantic): record SC-4.5 completion evidence
```

## Task 48 — Epistemic state audit

Purpose:
Prove type unavailability/failure categories are not collapsed.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file if defect

Owned types:
- `TypeKnowledge`;
- `AnalysisStatus`;
- `InferenceOutcome`;
- relation outcomes;
- call result mapping.

Required matrix:
```text
known established
known assumed
unknown reason
dynamic boundary
invalid cause
suppressed cause
blocked
cancelled
budget exceeded
internal failure
```

For each inference/call/publication path, record mapping.

Must not:
- map Cancelled/Budget to underconstrained;
- map internal failure to source conflict;
- use Dynamic for unsupported static analysis without a real dynamic boundary.

Testing classification:
- focused epistemic/authority tests.

## Task 49 — Public boundary no-leak audit

Purpose:
Enforce that inference context/frame/terms remain checker-local.

Risk:
- Semantic: HIGH
- Implementation fanout: local + metadata audit

Owned files:
- `checker/analysis.rs`;
- snapshot structures;
- export/metadata;
- `checker/typed_expr.rs`.

Search gates:
```bash
rg -n 'InferenceContextId|InferenceFrameId|InferenceFact|InferenceTerm|InferVarId' \
  phalcom-semantic/src/export.rs \
  phalcom-semantic/src/metadata \
  phalcom-type-meta
```

Expected:
- no new SC-4 context/frame/symbolic result in durable metadata;
- existing `InferVarId` references only where explicitly representing invalid/nonpublishable canonical signature state, never an emitted solved type.

Add debug/test assertions:
- root generic materialization must succeed or terminate structurally before publication;
- no public ExpressionAnalysis known type derived from unresolved inference variable.

Testing classification:
- focused publication/metadata tests.

## Task 50 — Cold/incremental equivalence

Purpose:
Prove new type semantics are query-path invariant.

Risk:
- Semantic: HIGH
- Implementation fanout: tests + dependency fixes

Owned files:
- semantic session/query dependencies;
- incremental test modules.

Required edit scenarios:
1. nested generic callback body changes;
2. generic parameter kind changes;
3. where constraint changes;
4. superclass type-lambda argument changes;
5. generic getter return/where changes;
6. transparent alias target changes;
7. Family retained target/signature changes;
8. SC-3 Record row signature changes if row query integration is current.

Compare actual semantic products:
```text
TypeId denotation / formatted canonical type as cross-store comparator
CallableId/target
TypeParameterId owner/index where stable within session model
diagnostic code
AnalysisStatus
evidence status/origin where published
```

Do not compare raw TypeId integers across independently constructed stores if IDs are arena-local; compare canonical semantic structure/format plus stable declaration identities according to existing incremental test conventions.

Testing classification:
- incremental checkpoint evidence.

## Task 51 — Deletion/authority audit

Purpose:
Prove obsolete mechanisms cannot silently continue as alternate semantics.

Risk:
- Semantic: MEDIUM
- Implementation fanout: cleanup

Required searches, adjusted to current tree:

```bash
rg -n 'LocalConstraintSolver' phalcom-semantic/src
```
Expected: zero production hits.

```bash
rg -n 'TypeData::Infer' phalcom-semantic/src
```
Expected: zero production ordinary-inference hits.

```bash
rg -n 'generic parameters not permitted on getters' phalcom-ast
```
Expected: zero.

```bash
rg -n 'InferenceSession::new\(' phalcom-semantic/src/checker
```
Expected: only context-owner/root/test paths documented after C1.

```bash
rg -n 'UnknownReason::UnannotatedDeclaration' phalcom-semantic/src/types/annotation.rs
```
Expected: no invalid type-formation recovery through this reason.

```bash
rg -n 'Object' phalcom-semantic/src/checker/call.rs phalcom-semantic/src/checker/associated.rs
```
Expected: every occurrence reviewed; no unresolved generic/variant parameter success fallback.

```bash
rg -n 'RecordAccess' phalcom-semantic/src
```
Expected: SC-3-compliant result documented at C7.

```bash
rg -n 'max_passes|max_scc_iterations|= 16' phalcom-semantic/src/checker/inference.rs
```
Expected: no hidden unnamed semantic convergence cutoff. Named/shared checker budget/fixed-point policy only.

If any compatibility occurrence remains intentionally, list exact path/symbol/reason in the state file.

Testing classification:
- negative/deletion gate.

## Task 52 — SC-4.5 certification gate

Purpose:
Establish the whole type-system closure claim before workspace delivery.

Risk:
- Semantic: HIGH
- Implementation fanout: verification

Required smallest-to-broad commands:

```bash
cargo test -p phalcom-semantic --test semantic semantic::foundations -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::capabilities -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::advanced -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::adts -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::families -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::incremental -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::integration -- --nocapture
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-core --test core monads:: -- --nocapture
cargo test -p phalcom-core --test core either:: -- --nocapture
```

Then mark SC-4.5 COMPLETE only if:
- all required checkpoint evidence is green;
- any baseline exclusions are recorded and demonstrably unrelated;
- no type-system ledger row remains unclassified;
- no INCIDENT remains.

---

# 21. Repository drift protocol

Before beginning each checkpoint:

1. RUN `git rev-parse HEAD`.
2. VERIFY every primary file still exists.
3. VERIFY primary symbols still own the responsibility described here.
4. REVIEW commits since the last checkpoint if they touched the working set.
5. SEARCH for newly added consumers when an API signature is changing.
6. Update mechanics in the state file if needed.

Do not redo a full repository audit unless:
- a primary symbol disappeared;
- source of truth moved;
- tests prove plan assumptions stale;
- a new parallel semantic subsystem appeared.

Allowed adaptation:
- helper naming;
- field placement;
- module extraction;
- Rust borrow-safe API shape.

Not allowed without escalation:
- changing `InferVarId != TypeId`;
- replacing canonical `TypeStore`/type-lambda authority;
- changing generic getter selector identity;
- creating a second solver;
- treating context as established evidence;
- collapsing RecordRow into type inference;
- restoring Object/Dynamic fallbacks.

---

# 22. Testing schedule summary

The implementation agent should not rerun the workspace repeatedly.

## C0
Run baseline parser + inference + MON + Either.

## C1
Run direct inference/nested contextual tests.

## C2
Run generalized HKT tests + MON.

## C3
Run expected-result/nested call tests + targeted MON composition.

## C4
Run constraints/variance/Self + MON inheritance.

## C5
Run parser getter tests + semantic getter tests.

## C6
Run affected semantic modules + MON/Either + full `phalcom-semantic --test semantic`.

## C7
Run SC-3 and relation focused tests.

## C8
Run expression/capability focused tests.

## C9
Run flow branch/loop/capture focused tests.

## C10
Run ADT/GADT focused tests.

## C11
Run source constraints/variance/alias/Family focused tests.

## C12
Run all semantic integration modules + MON/Either.

## Final Gate
Only then run workspace format/check/test/clippy and any final project-specific runtime/LSP consistency commands.

---

# 23. Verification commands and what they prove

## `cargo check -p phalcom-semantic`

Proves:
- Rust API fanout compiles;
- exhaustive matches/callers migrated.

Does not prove:
- inference semantics.

## `cargo test -p phalcom-semantic --test semantic semantic::foundations::inference -- --nocapture`

Proves:
- direct solver/fixed-point/rigid contextual laws in the semantic integration suite.

Does not prove:
- source HKT hierarchy/runtime.

## `cargo test -p phalcom-core --test core monads:: -- --nocapture`

Proves:
- end-to-end MON HKT/generic inheritance semantic and runtime conformance.

Does not prove:
- generalized multi-arity/higher-order laws unless added there.

## `cargo test -p phalcom-semantic --test semantic`

Proves:
- the canonical semantic integration binary is green for active tests.

Does not prove:
- ignored tests;
- workspace compiler/LSP compatibility.

## `cargo test --workspace --all-targets`

Proves:
- broad workspace target compatibility/test readiness.

Does not replace:
- checkpoint semantic evidence.

---

# 24. Failure protocol

If required evidence fails unexpectedly, stop scope expansion.

Record:

## Exact reproduction

```text
command:
test:
error/assertion:
checkpoint:
current HEAD:
```

## Direct path

Trace only the failing path, e.g.:

```text
source fixture
→ analyze_expression
→ apply_resolved_callable
→ active inference context/frame
→ constraint insertion
→ solve
→ failed assertion
```

## Passing comparator

Find one neighboring case that still works:

```text
unary HKT works, binary fails
expected proper T works, expected F<T> fails
cold works, incremental fails
direct getter works, inherited getter fails
```

## Classification

Use exactly one primary class:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## Narrow repair boundary

State the permitted files/symbols before editing.

## Rejected broad fixes

Always include relevant prohibitions:

```text
Do not:
- raise budgets to hide non-progress;
- turn failure into Dynamic/Object;
- convert caller generic to flexible variable;
- special-case a declaration by name;
- bypass canonical application;
- weaken the assertion;
- change parser syntax unless failure is parser-owned and syntax is ratified.
```

A checkpoint with failed required evidence is:

```text
C<N> — INCIDENT
```

not “mostly complete.”

---

# 25. Checkpoint supervisor report format

At each checkpoint, report:

```text
Checkpoint C<N> COMPLETE

Established:
    <one dominant semantic contract>

Changed:
    <path> — <symbols>
    ...

Evidence:
    <command> — PASS — proves ...
    ...

Hostile cases:
    <case> — PASS

Negative gates:
    <search> — expected result

Deferred:
    <command> -> <destination>

Unexpected findings:
    none | concise fact

Next:
    C<N+1> — <name>
```

If INCIDENT, replace COMPLETE with INCIDENT and include classification + narrow next diagnosis action.

---

# 26. Commit grouping

Suggested grouping; execution environment may combine commits when review policy prefers.

| Checkpoint | Suggested commits |
|---|---|
| C0 | `test(semantic): characterize SC-4 inference gaps` |
| C1 | `refactor(semantic): scope nested generic inference`; `test(semantic): enforce frame ownership` |
| C2 | `feat(semantic): generalize HKT constructor inference`; tests |
| C3 | `feat(semantic): propagate contextual HKT inference`; tests |
| C4 | `fix(semantic): align HKT constraints variance and Self`; tests |
| C5 | `feat(ast): allow generic getters`; `feat(semantic): infer generic getter access`; tests |
| C6 | `fix(semantic): converge generic executable surfaces`; certification tests |
| C7 | relation/SC-3 conformance fix only if necessary; relation tests |
| C8 | `fix(semantic): close bidirectional expression typing`; tests |
| C9 | `fix(semantic): close flow refinement joins`; tests |
| C10 | `fix(semantic): close GADT elimination`; tests |
| C11 | source closure fixes/tests + ledger |
| C12 | publication/incremental fixes + certification docs/tests |

Do not force one commit per Task.

---

# 27. Known scope exclusions

The following must not enter this program accidentally:

- effects/effect rows;
- raise-set inference;
- callable effect subtyping;
- `@pure`;
- termination and `@total`;
- contracts/VCs/SMT;
- rank-N/higher-rank polymorphism;
- first-class `forall`;
- impredicative polymorphism;
- public kind polymorphism;
- dependent types;
- intersection types;
- implicit typeclass search;
- runtime generic monomorphization;
- type-directed runtime overload selector changes;
- generic setter/indexer declarations;
- a new explicit generic call type-argument syntax;
- new Record mutation modes;
- general row-valued nominal generic application beyond SC-3;
- LSP-specific type inference;
- runtime class-layout changes.

If one becomes necessary for a required law, stop and escalate: either the law was mis-scoped or a ratified dependency is missing.

---

# 28. Final delivery gates

After C12 is COMPLETE, run these broad gates once.

## 28.1 Format

```bash
cargo fmt --all -- --check
```

Proves:
- repository Rust formatting is delivery-clean.

## 28.2 Workspace compile

```bash
cargo check --workspace --all-targets
```

Proves:
- all workspace targets compile against changed public/internal APIs.

Does not prove:
- semantic behavior.

## 28.3 Workspace tests

```bash
cargo test --workspace --all-targets
```

Proves:
- broad workspace regression compatibility.

If failures exist:
- classify using the failure protocol;
- do not claim release completion with unexplained failures.

## 28.4 Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Proves:
- no warning-level Rust issues remain under project clippy policy.

## 28.5 Project-specific protected core suites

Even if workspace test has run, preserve explicit evidence entries for:

```bash
cargo test -p phalcom-core --test core monads:: -- --nocapture
cargo test -p phalcom-core --test core either:: -- --nocapture
```

These are semantic conformance gates, not redundant ceremonial runs if their exact evidence was last collected before subsequent workspace-reaching changes.

If no SC-4 code changed after C12's successful exact runs, do not rerun them solely for ritual; reference the C12 evidence.

---

# 29. Final negative/deletion gates

Run after the final code shape is stable.

```bash
rg -n 'generic parameters not permitted on getters' phalcom-ast
```

Expected:
- zero hits.

```bash
rg -n 'InferenceSession::new\(' phalcom-semantic/src/checker
```

Expected:
- only documented root/context-owner construction; no nested private application solver.

```bash
rg -n 'AtomicU|static .*Infer|GLOBAL.*Infer' phalcom-semantic/src
```

Expected:
- no process-global inference identity allocator introduced.

```bash
rg -n 'LocalConstraintSolver|TypeData::Infer' phalcom-semantic/src
```

Expected:
- no production ordinary generic inference authority.

```bash
rg -n 'RecordAccess' phalcom-semantic/src
```

Expected:
- SC-3-authoritative result documented at C7.

```bash
rg -n 'UnknownReason::UncheckedExpression' phalcom-semantic/src/checker
```

Expected:
- every remaining occurrence listed in state as intentional unsupported/unratified boundary; no unclassified supported typing hole.

```bash
rg -n 'max_passes|max_scc_iterations' phalcom-semantic/src/checker/inference.rs
```

Expected:
- no unnamed magic success/failure cutoff; any remaining named convergence policy is documented and controlled by checker budget semantics.

```bash
rg -n 'fallback.*Object|Object.*fallback' phalcom-semantic/src/checker
```

Expected:
- no generic/variant inference success fallback; every remaining unrelated fallback is justified.

---

# 30. Deferred-evidence audit

Before release-complete status, inspect the state file.

No deferred command may remain unless it is:

1. executed successfully;
2. explicitly removed from scope with a semantic justification approved by the supervisor;
3. recorded as a release blocker, in which case this implementation is not release-complete.

Do not silently delete deferred entries.

---

# 31. Checkpoint evidence summary template

Maintain this table during execution.

| Checkpoint | Semantic contract | Required evidence | Status |
|---|---|---|---|
| C0 | trustworthy baseline/RED characterization | parser + inference + MON/Either + RED fixtures | PENDING |
| C1 | scoped nested inference ownership | frame/context + escape tests | PENDING |
| C2 | generalized constructor inference | multi-arity/higher-order/hole tests + MON | PENDING |
| C3 | result-directed/nested HKT | contextual HKT + nested call tests | PENDING |
| C4 | constraints/variance/Self parity | constraint + variance + hierarchy tests | PENDING |
| C5 | generic getter semantics | parser + signature + property application | PENDING |
| C6 | SC-4 executable surface closure | constructors/variants/GADT/family/union + semantic suite | PENDING |
| C7 | SC-3 dependency/relation closure | SC-3 + relation matrix | PENDING |
| C8 | expression typing closure | expression coverage/expectation suites | PENDING |
| C9 | flow/refinement closure | branch/loop/capture/call invalidation | PENDING |
| C10 | ADT/GADT elimination | matching/exhaustiveness/GADT flow | PENDING |
| C11 | source declaration closure | constraints/variance/Self/alias/Family + ledger | PENDING |
| C12 | epistemic/publication/incremental closure | parity + deletion + semantic/core certification | PENDING |

No row becomes COMPLETE from code inspection alone.

---

# 32. State-file completion requirements

At final delivery, the state file must contain:

- starting plan SHA;
- actual execution baseline;
- final HEAD;
- all established invariants;
- all deviations in mechanics;
- all semantic decisions retained;
- checkpoint evidence commands/results;
- negative/deletion gate results;
- cold/incremental evidence;
- zero active INCIDENT;
- zero forgotten deferred gates;
- current coverage-ledger classification;
- final next roadmap action.

Recommended final next action:

```text
Proceed to the effects/control/termination stage only after SC-4.5 is release-complete.
```

---

# 33. Release-complete criteria

The implementation program is complete only when:

- [ ] C0 through C12 are all COMPLETE.
- [ ] SC-4 is explicitly marked COMPLETE at C6.
- [ ] SC-4.5 is explicitly marked COMPLETE at C12.
- [ ] SC-3 dependency is satisfied.
- [ ] all checkpoint semantic evidence passes.
- [ ] all high-risk hostile cases pass.
- [ ] MON and Either protected suites pass.
- [ ] all required obsolete mechanisms are removed or each surviving compatibility occurrence is justified.
- [ ] no solver-local inference state leaks into public canonical products.
- [ ] cold/incremental type facts are equivalent for the required edit scenarios.
- [ ] no unclassified supported `Unknown(UncheckedExpression)` hole remains.
- [ ] coverage ledger contains no unclassified current type-system gap.
- [ ] final format gate passes.
- [ ] final workspace check passes.
- [ ] final workspace tests pass, or any genuinely pre-existing unrelated baseline failures are explicitly demonstrated and accepted by supervisor rather than hidden.
- [ ] final clippy gate passes.
- [ ] state file contains no active INCIDENT.
- [ ] no deferred evidence is forgotten.
- [ ] relevant semantic docs/comments are updated to remove statements made stale by generic getters, nested inference contexts, or completed source coverage.

---

# 34. Failure classification quick reference

| Class | Meaning | Typical repair owner |
|---|---|---|
| PRODUCT | semantic implementation wrong | owning production subsystem |
| FIXTURE | test does not establish intended preconditions | test/source fixture |
| DEPENDENCY/PUBLICATION | canonical product exists but consumer lacks/stales it | producer/query dependency |
| BACKEND/HARNESS | failure outside intended semantic layer | compiler/VM/test harness |
| BASELINE | predates checkpoint | record and separate before continuing |
| PLAN DRIFT | repository architecture changed | stop and reconcile plan mechanics/assumption |

---

# 35. Final implementation guidance

The program is intentionally ordered so the highest semantic leverage lands first.

```text
C1-C3
    make nested higher-kinded inference mathematically sound

C4
    align it with the existing type relation/specialization algebra

C5
    add the last clearly missing ratified executable generic source surface

C6
    certify one application model

C7-C12
    prove every other language/type consumer is actually closed over that model
```

Do not optimize implementation by skipping the ownership work and adding more local special cases.

The key invariant throughout the program is:

> Canonical types are durable semantic facts. Inference terms are scoped proof-search state. The checker may carry symbolic structure across nested analysis only while the owning inference context is active, and every public result must exit that context as a canonical type or an explicit structured non-success state.

That boundary is the foundation on which the remaining higher-kinded, getter, flow, GADT, incremental, and tooling correctness depends.

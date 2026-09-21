---
id: TEST002.C1.P1
category: TEST
program: TEST002
checkpoint: TEST002.C1
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG006.C1
  - LANG005.C6
follows: null
supersedes: null
---

# TEST002.C1.P1 — Post-Visibility Semantic/Core Baseline Failure Elimination

## 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High as the primary implementer under constrained architectural authority.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture, semantic ownership, and invariants below;
- must not improvise material architecture;
- must re-read every named live path before editing because the repository has just completed a visibility/access-control implementation closure;
- must not reinterpret a currently failing test as authoritative merely because it is red;
- must not weaken a correct test merely to obtain a green baseline;
- must not preserve a stale test merely because changing production code could also make it pass;
- must use the testing budget in this plan rather than testing reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when any global or task-local escalation trigger fires;
- must keep the shared `TEST002.C1` checkpoint record current;
- must produce the required walkthrough and handoff.

Testing is evidence gathering, not ritual. During BUILD MODE, run the smallest discriminator that can falsify the current hypothesis. Do not run the full semantic/core suites after each patch.

This is a **plan-only artifact**. Executing it authorizes the implementation work described below; producing or reading this plan does not itself authorize unrelated repository changes.

---

## 1. Goal

Eliminate the nineteen post-`LANG006.C1` semantic/core baseline failures by correcting two confirmed stale test authorities and repairing the owning semantic, incremental, compiler/runtime, conformance-transport, and typing defects without regressing the newly completed visibility/access architecture.

---

## 2. Checkpoint acceptance objective

`TEST002.C1` is a bounded **corrective stabilization checkpoint**.

The checkpoint advances from the current `LANG006.C1` state:

```text
phalcom-semantic --test semantic
    1,428 passed
       14 failed
       50 ignored

phalcom-core --test core
      607 passed
        5 failed
       28 ignored
```

to a state in which:

1. every one of the nineteen named residual failures is either:
   - passing under the accepted semantic oracle; or
   - migrated because the old test itself encoded retired behavior;
2. no valid assertion is weakened to achieve that result;
3. no cursor-era iteration authority, broad Universe dependency suppression, runtime trait proof, or consumer-local type reconstruction is reintroduced;
4. the newly completed `LANG006.C1` access/visibility model remains intact;
5. a final one-time broad semantic/core pass confirms that the original nineteen-failure baseline is gone;
6. any genuinely new unrelated failure introduced by concurrent repository movement is classified under the baseline policy rather than silently folded into this plan.

This P1 is intended to provide **full checkpoint closure** for `TEST002.C1`. If a mandatory STOP/CONSULT trigger shows that one residual requires a materially different architecture or a separate owner program, P1 may finish partially and a P2 corrective plan must remain in the same checkpoint.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   c3a53c3e364e588d46bd56ade246976e1996d744
revision message:
  lang006: complete C1 corrective closure and stage C2

relevant predecessor:
  LANG006.C1.P3 — compiler synthetic authority and certification baseline stabilization
predecessor broad baseline:
  semantic: 1,428 passed / 14 failed / 50 ignored
  core:       607 passed /  5 failed / 28 ignored
```

### 3.1 Repository facts verified during planning

The following facts were verified against the live repository at the revision above.

1. `LANG006.C1` is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`; its P3 residual ledger names exactly the nineteen failures owned by this plan.
2. The post-visibility commit modified code/tests inside this failure surface, including:
   - `phalcom-core/src/compiler/lib/expr.rs`;
   - `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`.
3. The new visibility architecture is already stable and is **not** part of this remediation:
   - normalized AST visibility is canonical;
   - absent visibility is `Public`;
   - `phalcom-semantic/src/access.rs` owns semantic access relations;
   - dispatch performs lookup then authorization;
   - exact module/package/project provenance is preserved;
   - compiler-internal constructor authority is call-site-local;
   - `ImplementationKind` and native `ImplementationProvenance` remain separate.
4. `FlowState::invalidate_opaque_calls` explicitly drops narrower instance-field current facts to `field_lifecycle::lifecycle_read_knowledge`; its unit coverage already expects a validated contract to remain readable as `EvidenceOrigin::FieldLifecycle`.
5. The integration test `opaque_call_drops_refined_field_current_without_rewriting_contract` still expects `Unknown(NoTypeEvidence)`, contradicting the current field-lifecycle model and its unit test.
6. `synthesize_list_literal` currently seeds an expansion projection as `Unknown(SuppressedByInvalidCause)` and only performs projection when the operand is `Known`, which can erase an existing `UnresolvedName` cause.
7. The four built-in subscript regressions still exist in `foundations/canonical_call_application.rs`.
8. `synthesize_index_expr` and `synthesize_set_index_expr` currently try ordinary dispatch before the existing transitional List/Map structural targets.
9. `structural_list_index_get_target`, `structural_map_index_get_target`, and `structural_list_index_set_target` already encode the intended generic scalar contracts:
   - `List<T>[Int] -> T`;
   - `Map<K,V>[K] -> V`;
   - `List<T>[Int] = T`, with assignment expression wrapping to `Unit`.
10. Source `List<T>` and `Map<K,V>` bracket members are deliberately broad/untyped enough to implement runtime behaviors that cannot by themselves express those scalar generic contracts.
11. `checker/context.rs` documents that only the aggregate Universe root is bootstrap-owned, but `is_bootstrap_declaration` currently returns true for every declaration whose project is `Universe`. This suppresses fine-grained dependencies for real Universe source modules and directly conflicts with the function's own documented invariant.
12. Current incremental tests distinguish computation revision from validation revision; unchanged products are expected to preserve:
    - their original `QueryState::revision`;
    - their product fingerprint;
    - their `Arc` identity;
    while `validated_revision` advances.
13. The current `dependency_edit_remove_readd_recomputes_affected_summary_deterministically` test no longer asserts a brittle hard-coded revision number. It now checks semantic recomputation/reuse, removal, re-addition, resulting type, and internal-incident freedom. If it remains red, it is implementation work, not a stale oracle.
14. Positional expansion lowering still contains:
    ```text
    semantic positional-expansion spec available
        -> invoke selected iterator/next requirements

    no semantic spec
        -> ordinary `.iterator` / `.next` sends
    ```
    after the Tuple/Unit product fast lane.
15. `RuntimeError::NonIterableStarOperand` still exists with the accepted structured boundary:
    ```text
    * expansion requires Tuple, Unit, or an iterable value; got ...
    ```
16. The current traceback spread fixture still implements retired `iterate(_)` / `iteratorValue(_)` cursor behavior and therefore is a confirmed stale fixture.
17. Call frames already carry `RuntimeConformanceEnvironmentId`, and method activation already transports it.
18. Generic call lowering currently emits `RuntimeConformanceSlotRecipe::Selection { callable, selector }` without a callee-specific nested conformance environment. Derived-conformance handling also explicitly omits one nested-slot construction branch when the selected callable is a trait default.
19. `invoke_trait_requirement` indexes the active frame environment by the requirement slot assigned to the **callee body**. Therefore carrying an environment with a different layout can produce:
    ```text
    no active conformance target for requirement ...
    ```
    even though an environment ID is present.
20. `traverse_body_resolves_symbolic_inherited_capabilities` remains a formal typing assertion that a symbolic `Monad<F>` can resolve inherited `Applicative`/`Functor` behavior with `Assumed` evidence. The test must not be made Dynamic or untyped.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

### 3.2 New TEST002 record location

The repository's current implementation convention requires new work to use category/program/checkpoint hierarchy. For this new program, the intended canonical location is:

```text
docs/implementation/test/TEST002-baseline-stabilization/
  PROGRAM.md
  STATUS.md                         # optional if consistent with live convention
  C1-semantic-core-baseline-elimination/
    CHECKPOINT.md
    TEST002.C1.P1-baseline-failure-elimination.md
    TEST002.C1.P1-walkthrough.md   # created at completion
    TEST002.C1.P1-handoff.md       # created at completion
```

The current `docs/implementation/INDEX.md` already uses `test/...` links for TEST-category programs even though older `TEST001` physical layout remains legacy. **Do not opportunistically move TEST001.**

If the live repository has normalized this layout again before implementation, adapt the directory mechanically while preserving the stable `TEST002`, `TEST002.C1`, and `TEST002.C1.P1` identities.

---

## 4. Required reads before implementation

Read these before the first production edit:

```text
Repository/workflow:
- AGENTS.md
- docs/implementation/README.md
- docs/workflow/implementation-record-lifecycle-convention.md
- docs/workflow/shared-consultation-escalation-protocol.md
- docs/workflow/luna-implementer-prompt.md          # if present under the live name
- docs/implementation/INDEX.md

Predecessor:
- docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md
- docs/implementation/LANG006/LANG006.C1/LANG006.C1.P3-walkthrough.md
- docs/implementation/LANG006/LANG006.C1/LANG006.C1.P3-handoff.md

Normative/current semantic authority:
- docs/specs/callables/arguments.md
- docs/specs/iterator.md
- docs/specs/objects/traits.md
- docs/specs/generic-constraints.md
- docs/specs/source-declarations.md
- the effective subscript/index specification or accepted decision named by the live source
- the effective semantic evidence/analysis-status specification used by SEMA
- accepted index/subscript and iteration decision records referenced by those specs

Current implementation:
- phalcom-semantic/src/checker/flow/state.rs
- phalcom-semantic/src/checker/field_lifecycle.rs
- phalcom-semantic/src/checker/expression.rs
- phalcom-semantic/src/checker/composition.rs
- phalcom-semantic/src/checker/call.rs
- phalcom-semantic/src/checker/context.rs
- phalcom-semantic/src/checker/iteration.rs
- phalcom-semantic/src/session.rs
- phalcom-semantic/src/db/mod.rs
- phalcom-semantic/src/db/query.rs
- phalcom-semantic/src/db/state.rs
- phalcom-semantic/src/db/fingerprint.rs
- phalcom-semantic/src/generic_evidence.rs
- phalcom-core/src/compiler/lib/expr.rs
- phalcom-core/src/compiler/lib/scope.rs
- phalcom-core/src/modules/compile.rs
- phalcom-core/src/modules/semantic_lowering.rs
- phalcom-core/src/typing/conformance_environment.rs
- phalcom-core/src/vm/dispatch.rs
- phalcom-core/src/vm/send.rs
- phalcom-core/src/frame.rs
- phalcom-core/src/error.rs
- phalcom-core/core/universe/src/collections/list.ph
- phalcom-core/core/universe/src/collections/map.ph
- phalcom-core/core/universe/src/collections/iterator_laws.ph

Test conventions:
- phalcom-semantic/tests/semantic/README.md
- phalcom-core/tests/README.md
```

Read the exact nineteen tests listed in §6 before changing them.

Avoid broad exploration unless new evidence makes it necessary. This plan already localizes the expected owners.

---

## 5. Normative authority

### 5.1 Accepted language behavior

The implementation must preserve these accepted rules.

#### Argument expansion

`docs/specs/callables/arguments.md` owns the `*` contract:

```text
Unit
    -> zero positional contributions

Tuple
    -> positional lane

canonical Iterable
    -> yielded items in iteration order

other
    -> invalid positional-expansion source
```

A real iterator's own errors/control effects propagate normally.

#### Iteration

`docs/specs/iterator.md` and accepted `LANG005.C6` state:

```text
semantic analysis
    proves Iterable / Iterator evidence
    normalizes Item / Iterator
    selects iterator / next requirements
        ↓
lowering
        projects selected execution facts
        ↓
compiler / VM
        execute those selected facts
```

A value does not become canonically iterable merely by exposing members named `iterator` and `next`.

#### Traits and conformances

`docs/specs/objects/traits.md` requires structured conformance evidence including:

```text
selected conformance
exact substitutions
associated bindings
requirement witnesses
selected defaults
nested conformance evidence
```

Runtime scanning, source order, reflection, or method coincidence cannot create or change conformance.

Trait requirements, defaults, witnesses, and conformances retain distinct semantic identities.

#### Visibility

`docs/specs/source-declarations.md` and completed `LANG006.C1` own visibility. TEST002 must preserve:

```text
default visibility = Public
exact semantic access sets
lookup then authorize
no `_` / `__` prefix inference
no blanket Universe/native access privilege
```

### 5.2 Current implementation authority

Current source is authoritative for implementation mechanics only where it does not conflict with accepted semantics.

Examples:

- the field-lifecycle implementation and its unit test establish the implemented opaque-call fallback;
- current DB `revision`/`validated_revision` APIs define the implemented product-stability mechanism;
- current conformance lowering and VM environment representations are the mechanisms to repair, not normative proof authorities.

### 5.3 Future/non-normative material

Historical cursor plans, superseded collection documents, earlier pre-LANG005 iteration notes, and old `Iterable` class inheritance designs are not authority.

Do not use future plans to:
- redesign subscript syntax;
- introduce runtime trait proof;
- add trait specialization;
- change visibility;
- redesign the semantic database;
- introduce a new dynamic iteration protocol.

---

## 6. Takeover state

### 6.1 Failure ledger at takeover

| ID | Current failing test | Owner | Planned disposition |
|---|---|---|---|
| BL-01 | `opaque_call_drops_refined_field_current_without_rewriting_contract` | SEMA flow | confirmed stale test oracle; migrate |
| BL-02 | `unknown_expansions_do_not_disappear` | SEMA composition | implementation repair |
| BL-03 | `assumed_list_receiver_caps_structural_index_result` | SEMA calls | implementation repair |
| BL-04 | `list_subscript_checks_index_contract` | SEMA calls | implementation repair |
| BL-05 | `list_subscript_set_checks_index_and_value_and_returns_unit` | SEMA calls | implementation repair |
| BL-06 | `map_subscript_checks_key_contract` | SEMA calls | implementation repair |
| BL-07 | `adding_impl_member_does_not_invalidate_another_target_or_its_caller` | SEMA incremental | implementation repair |
| BL-08 | `dependency_edit_remove_readd_recomputes_affected_summary_deterministically` | SEMA incremental | VERIFY-FIRST; current oracle is sound |
| BL-09 | `canonical_universe_reads_track_canonical_query_dependencies` | SEMA dependencies | implementation repair |
| BL-10 | `legacy_universe_core_path_is_not_a_dependency_sentinel` | SEMA dependencies | implementation repair |
| BL-11 | `signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies` | SEMA incremental | implementation repair |
| BL-12 | `superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies` | SEMA incremental | implementation repair |
| BL-13 | `a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products` | SEMA incremental | implementation repair |
| BL-14 | `unused_public_export_does_not_recompute_unrelated_consumer_body` | SEMA incremental | implementation repair |
| BL-15 | `positional_spread_of_non_iterable_has_structured_boundary` | compiler/runtime iteration | implementation repair |
| BL-16 | `test_list_spread_traceback_includes_spread_site` | core observability | confirmed legacy cursor fixture; migrate |
| BL-17 | `cv46_runtime_conditional_default_executes` | generic conformance runtime | implementation repair |
| BL-18 | `bounded_generic_function_dispatches_exact_witness_and_default` | generic conformance runtime | implementation repair |
| BL-19 | `traverse_body_resolves_symbolic_inherited_capabilities` | typing integration | implementation repair |

**Planning classification:** two confirmed stale tests + seventeen implementation failures, subject to the T0 live rebaseline. If a listed failure is already green at implementation start, do not edit code merely to match this historical ledger.

### 6.2 Stable interface map

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| access relation | `phalcom-semantic/src/access.rs` | SEMA | exact module/package/project identity |
| source visibility publication | `ClassMember::visibility()` -> semantic surface | AST/SEMA | default public; no prefix inference |
| opaque field fallback | `FlowState::invalidate_opaque_calls` | checker flow | discard narrow fact, retain valid lifecycle contract |
| field lifecycle read | `lifecycle_read_knowledge` | checker field lifecycle | validity/init/causality determine readable evidence |
| canonical call application | `apply_resolved_callable` | checker call | argument contracts checked once |
| built-in scalar subscript target | `structural_*_index_*_target` | checker call | semantic typing only; no final-result construction |
| expression subscript synthesis | `synthesize_index_expr`, `synthesize_set_index_expr` | checker expression | operation selection then canonical call application |
| semantic DB reuse | `SemanticDb::validate_reuse` and `QueryState` | semantic DB | computation revision distinct from validation revision |
| semantic dependency capture | `TrackingTypeResolver`, `TrackingTypeHierarchy` | checker/context | record exact consumed products |
| iteration proof | `resolve_positional_expansion` + analysis product | SEMA | semantic evidence authority |
| expansion lowering | `ModuleLoweringSemantics::positional_expansions` | core lowering | stable projection of semantic decision |
| Tuple/Unit expansion fast lane | `PackTryExpandTuplePositionals`, `ListTryExpandTuplePositionals` | VM | product-only optimization, not iteration proof |
| generic evidence layout | `GenericEvidenceLayout` | SEMA | deterministic semantic requirement slots |
| runtime conformance environment | `RuntimeConformanceEnvironment` | core typing/VM | executable handles only, no proof search |
| requirement execution | `InvokeTraitRequirement` | compiler/VM | execute preselected slot |
| frame evidence transport | `CallFrame::conformance_environment` | VM | environment travels with activation |
| symbolic generic evidence | checker inference/evidence | SEMA | symbolic type parameters may be `Assumed`; not no-evidence |
| compiler synthetic access | `InvocationAuthority::CompilerInternal` | compiler | generated allocator call only |

---

## 7. Architecture

TEST002 does not introduce a new cross-cutting architecture. It closes defects at their current canonical owners.

### 7.1 Semantic/evidence corrections

```text
source expression
    ↓
checker-owned TypeKnowledge + causal provenance
    ↓
composition
    preserves/combines that knowledge
    ↓
published expression product
```

Composition may add context. It may not replace a more precise upstream unknown/dynamic reason with a generic suppression reason simply because projection cannot proceed.

### 7.2 Built-in subscript typing

```text
known applied built-in receiver
    List<T> / Map<K,V>
        ↓
checker identifies canonical scalar subscript operation contract
        ↓
CallableApplicationTarget
        ↓
apply_resolved_callable
        ↓
type relation diagnostics + result knowledge

ordinary user-defined receiver
        ↓
normal exact selector dispatch
        ↓
same apply_resolved_callable engine
```

The special generic relationship for built-in scalar indexing remains semantic-layer knowledge. The compiler must not reconstruct it.

The runtime source member may stay broad enough to implement Range/dynamic behavior. A broad source member must not erase a stronger statically known scalar generic contract.

### 7.3 Incremental products

```text
changed source/input
    ↓
exact changed query roots
    ↓
reverse dependency candidates
    ↓
dependency products revalidated
    ↓
if direct input and dependency semantic products are unchanged:
        preserve old product/revision/Arc
        advance validated_revision only
    else:
        recompute exactly the affected product
```

Session orchestration must not preempt the DB's product-stability decision with aggregate or heuristic refresh work.

### 7.4 Positional expansion

```text
operand evaluated once
    ↓
Tuple/Unit product fast lane?
    ├─ yes -> append product positionals
    └─ no
         ↓
semantic positional-expansion evidence?
    ├─ yes -> execute preselected iterator + next requirements
    └─ no
         ↓
known invalid source / approved dynamic boundary
         -> structured invalid-expansion behavior
```

There is no fallback:

```text
missing semantic evidence
    -> ordinary `.iterator` lookup
```

because that would make structural member coincidence an iteration authority.

### 7.5 Conformance default execution

```text
semantic conformance proof
    ↓
selected witness/default + exact requirement identities
    ↓
callee evidence layout
    ↓
runtime environment recipe indexed by that callee layout
    ↓
CallFrame.conformance_environment
    ↓
InvokeTraitRequirement(slot from callee body)
    ↓
preselected executable requirement target
```

The environment used by a trait default must be compatible with the requirement-slot layout compiled into that default body. Merely forwarding "some conformance environment" is insufficient.

### 7.6 Symbolic inherited generic typing

```text
symbolic receiver Monad<F>
    ↓ inherited owner traversal
Monad -> Applicative -> Functor
    ↓
generic target specialization
    ↓
symbolic type parameters remain canonical symbolic terms
with Assumed evidence
    ↓
Ready call analysis + published specialization
```

No name-based Monad/Functor special case is permitted.

---

## 8. Ownership boundaries

### 8.1 Owns

TEST002.C1.P1 owns:

- correcting the two confirmed stale test authorities;
- preserving unknown/dynamic evidence through expression expansion;
- canonical List/Map scalar subscript checking;
- fine-grained Universe dependency capture;
- exact incremental product reuse for the listed scenarios;
- fail-closed positional expansion without structural iterator fallback;
- canonical spread traceback coverage;
- nested/callee-compatible conformance-environment transport for the two failing generic/default paths;
- preservation of symbolic assumed evidence across inherited generic capability resolution;
- broad semantic/core reclassification after those fixes.

### 8.2 Does not own

This plan does not own:

- new visibility syntax or visibility semantics;
- new parser modifier rules;
- `LANG006.C2` validated-data construction;
- new collection APIs or Range semantics;
- subscript syntax redesign;
- runtime trait solving;
- trait specialization;
- new dynamic trait objects;
- a new semantic DB architecture;
- new generic type syntax;
- iterator library redesign;
- cursor compatibility;
- workspace-wide formatting debt;
- unrelated language-corpus remediation under TEST001;
- LSP feature expansion;
- release certification.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| field readability after opaque call | field lifecycle + flow | expression reads | test-local invented rule |
| expression evidence provenance | semantic checker | diagnostics/lowering/editor | collection-specific replacement reason |
| built-in scalar subscript contract | semantic call application | diagnostics/lowering | compiler/runtime type inference |
| ordinary callable identity | semantic dispatch | call analysis/lowering | synthetic replacement identity for user members |
| query reuse validity | semantic DB query state/dependency index | session/snapshot | session-local parallel cache policy |
| Universe source declaration dependency | semantic dependency capture | incremental DB | project-wide bootstrap suppression |
| iterable proof | semantic conformance engine | lowering/compiler | VM structural lookup |
| trait witness/default selection | semantic conformance evidence | lowering/VM | runtime selector search as proof |
| requirement slot identity | `GenericEvidenceLayout` / lowering | runtime environment | compacted/ad-hoc runtime indexing |
| symbolic generic evidence | checker generic environment/inference | call application | `Dynamic`/`NoTypeEvidence` fallback |
| visibility/access | completed LANG006 semantic access model | all call paths | TEST002-local privilege workaround |

---

## 9. Global invariants

`INV-01 — accepted semantics outrank stale tests.`  
A test is changed only when current accepted semantics and live architecture prove its oracle obsolete. Correct tests are not weakened.

`INV-02 — opaque calls invalidate refinements, not valid declarations.`  
A definitely initialized, contract-valid field may fall back from a narrower write-derived fact to its validated/assumed lifecycle contract after an opaque call.

`INV-03 — evidence composition preserves specific uncertainty.`  
`Unknown(UnresolvedName(...))`, blocked states, and Dynamic reasons are not replaced by generic suppression merely because a containing expression cannot derive a more precise projected type.

`INV-04 — built-in scalar subscripts retain their generic contracts.`  
`List<T>[Int] -> T`, `Map<K,V>[K] -> V`, and List scalar assignment validates both index and value.

`INV-05 — assignment expression semantics remain Unit.`  
The semantic setter operation may have an implementation return, but `xs[i] = value` publishes `Unit` with the operation's status/causality.

`INV-06 — user-defined subscript identity remains exact.`  
Repairing built-in contracts must not rewrite or erase the canonical `CallableId` of an ordinary user-defined subscript.

`INV-07 — only the aggregate Universe root is bootstrap dependency input.`  
Concrete Universe source modules and synthetic Universe-looking module identities remain ordinary fine-grained dependency identities.

`INV-08 — recomputation and validation remain distinct.`  
An unchanged semantic product retains its computation revision/product identity/Arc; validation may advance independently.

`INV-09 — product stability stops propagation.`  
A recomputed intermediate whose semantic product fingerprint is unchanged must permit downstream reuse.

`INV-10 — incremental result equals cold final meaning.`  
Remove/re-add, signature changes, hierarchy changes, and exports must not leave stale products or produce a final meaning different from cold analysis.

`INV-11 — iteration proof is semantic.`  
An object is not iterable merely because ordinary lookup finds `iterator`/`next`.

`INV-12 — Tuple/Unit probing remains product-only.`  
`PackTryExpandTuplePositionals` / `ListTryExpandTuplePositionals` do not become conformance checks.

`INV-13 — invalid positional expansion has a structured boundary.`  
A known non-`Unit`/non-`Tuple`/non-Iterable source does not leak ordinary `doesNotUnderstand('iterator')`.

`INV-14 — real iterator errors propagate unchanged.`  
Once canonical iteration is selected, errors from its actual `iterator`/`next` behavior remain the user's error and retain the spread call site in traceback.

`INV-15 — conformance proof stays semantic; runtime execution stays preselected.`  
No VM method-table scan, reflection query, source order, or fallback selector search may create a conformance.

`INV-16 — runtime requirement slots are callee-layout compatible.`  
Every `InvokeTraitRequirement` indexes an environment whose layout was constructed for that compiled requirement site's semantic slot identity.

`INV-17 — nested evidence is not flattened by accident.`  
A selected default/witness that executes nested requirements receives the exact required nested environment rather than a coincidentally available caller environment.

`INV-18 — symbolic generic parameters are evidence-bearing terms.`  
A valid generic parameter under an active generic environment remains `Assumed` symbolic evidence; absence of a concrete type is not `NoTypeEvidence`.

`INV-19 — inherited capability traversal remains generic.`  
Monad/Applicative/Functor behavior is repaired through general inherited generic specialization, never by declaration-name special cases.

`INV-20 — completed visibility architecture remains untouched.`  
No TEST002 fix may reintroduce visibility prefixes, blanket internal privilege, access-before-lookup, or ambient compiler authority.

`INV-21 — one source of truth per fact.`  
No task may add a second type relation engine, iteration proof path, conformance proof path, incremental cache, or access model.

---

## 10. Non-goals

Do not:

- redesign `FieldLifecycleFact`;
- make every opaque call erase all fields to unknown;
- replace specific unknown reasons globally with a single category;
- encode collection type contracts in compiler bytecode;
- split List's runtime Range/scalar bracket implementation solely to satisfy the checker unless current normative collection design already requires that split;
- make selector dispatch choose overloads by runtime argument type;
- disable or bypass semantic DB reuse;
- force every changed module to recompute all callable bodies;
- restore `iterate(_)` / `iteratorValue(_)`;
- treat `.iterator` presence as conformance;
- catch arbitrary DNU raised *inside* a valid iterator and rewrite it as non-iterable;
- add a new proof opcode;
- perform trait proof in VM;
- compact requirement slots around absent entries;
- special-case `Printable`, `Monad`, `Functor`, or `Applicative` by name;
- change source visibility or package/project access rules;
- opportunistically move TEST001 or historical implementation records;
- run workspace tests or workspace Clippy during BUILD MODE.

---

## 11. Expected impact map

### 11.1 Expected source areas

- `phalcom-semantic/src/checker/expression.rs`
  - evidence-preserving expansion;
  - built-in subscript target ordering/selection if required.
- `phalcom-semantic/src/checker/composition.rs`
  - shared projection helper if that is the smallest non-duplicating location.
- `phalcom-semantic/src/checker/call.rs`
  - existing structural subscript target composition, only if required.
- `phalcom-semantic/src/checker/context.rs`
  - exact bootstrap dependency predicate.
- `phalcom-semantic/src/session.rs`
  - likely incremental work scheduling/reuse seam.
- `phalcom-semantic/src/db/{mod,query,state,fingerprint}.rs`
  - inspect; modify only if evidence proves DB/input fingerprint is the cause.
- `phalcom-core/src/compiler/lib/expr.rs`
  - positional expansion fallback removal/replacement.
- `phalcom-core/src/modules/compile.rs`
  - likely callee-layout runtime conformance recipe construction.
- `phalcom-core/src/modules/semantic_lowering.rs`
  - inspect for trait/default requirement slot projection.
- `phalcom-core/src/typing/conformance_environment.rs`
  - runtime recipe shape if nested selected environments require representation extension.
- `phalcom-core/src/vm/dispatch.rs`
  - materialization only; no proof.
- semantic generic call/inference paths in:
  - `phalcom-semantic/src/checker/call.rs`;
  - `phalcom-semantic/src/checker/context.rs`;
  - related generic specialization/normalization helpers discovered by T8.

### 11.2 Expected tests

- `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
- `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`
- `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`
- `phalcom-semantic/tests/semantic/impls/queries.rs`
- `phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs`
- `phalcom-semantic/tests/semantic/incremental/checker_dependencies.rs`
- `phalcom-semantic/tests/semantic/incremental/product_stability.rs`
- `phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs`
- `phalcom-core/tests/core/compiler/declaration_dispatch.rs`
- `phalcom-core/tests/core/observability/traceback.rs`
- `phalcom-core/tests/core/language/conditional_conformance.rs`
- `phalcom-core/tests/core/language/generic_conformance.rs`
- `phalcom-core/tests/core/typing_integration/monads/bodies.rs`

Use existing nearby tests as discriminators before adding redundant new cases.

### 11.3 Expected documentation/state

At implementation start/create:

- `docs/implementation/test/TEST002-baseline-stabilization/PROGRAM.md`
- `docs/implementation/test/TEST002-baseline-stabilization/C1-semantic-core-baseline-elimination/CHECKPOINT.md`
- update `docs/implementation/INDEX.md`
- place this plan in the checkpoint under its canonical filename.

At completion:

- `TEST002.C1.P1-walkthrough.md`
- `TEST002.C1.P1-handoff.md`
- checkpoint plan/verification/deferred ledgers.

### 11.4 Unexpected-touch rule

Touching adjacent private helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem not anticipated here if that entry changes:
- semantic authority;
- identity;
- representation;
- runtime evidence ABI;
- visibility/access;
- module topology;
- the public language contract.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. the twenty-one global invariants above;
2. the two confirmed stale-test classifications:
   - opaque field current fact;
   - cursor-era spread traceback fixture;
3. the current sound oracle of `dependency_edit_remove_readd_recomputes_affected_summary_deterministically`;
4. semantic ownership of iteration/conformance/type proof;
5. the DB computation-revision vs validation-revision distinction;
6. the exact-root-only bootstrap dependency rule;
7. user-defined subscript identity;
8. `Unit` result semantics for subscript assignment;
9. the accepted `*` source categories;
10. the new LANG006 visibility/access architecture;
11. the rule that a selected trait default's nested requirement execution must use a compatible callee evidence layout.

### 12.2 MECHANICALLY FLEXIBLE

Luna may adapt without consultation:

- helper names;
- private decomposition;
- test fixture class names;
- exact import paths;
- equivalent match/helper APIs;
- local Rust ownership/borrowing implementation;
- whether evidence-preserving expansion uses a shared helper or compact local match, provided all four expansion categories share the same law;
- whether the runtime conformance recipe extends `Selection` or adds an equivalent private variant, provided the semantic/runtime authority split is unchanged;
- exact debug instrumentation used temporarily in incremental/conformance diagnosis;
- exact canonical TEST002 directory suffix if repository conventions have mechanically changed.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| all nineteen failures still reproduce | T0 exact filters + current LANG006 ledger | drop already-green entries; do not invent work |
| opaque field fallback is still the implemented law | `flow/state.rs`, `field_lifecycle.rs`, unit test, effective SEMA spec | if accepted semantics changed materially, STOP AND CONSULT |
| no current semantic source surface already publishes scalar List/Map contracts | `canonical_call_application`, source surfaces, dispatch target debug | if complete canonical contract exists, fix target resolution mechanically; do not add second contract |
| `is_bootstrap_declaration` still suppresses all Universe project modules | `checker/context.rs` | if already fixed, mark BL-09/10 independently resolved |
| `SemanticDb::validate_reuse` still embodies stable-product reuse | `db/mod.rs`, `db/state.rs`, incremental unit tests | if DB contract changed, STOP AND CONSULT |
| session orchestration is responsible for remaining over-recompute | T5 instrumentation | if fingerprint/query ownership is cause, repair owner instead; architectural ambiguity => consult |
| positional expansion still ordinary-sends on absent semantic spec | `compiler/lib/expr.rs` | if absent, locate current fallback mechanically |
| semantic checker distinguishes known invalid from approved dynamic positional expansion | checker iteration/analysis | if current representation cannot encode required dynamic boundary, STOP AND CONSULT |
| generic default failure is an environment-layout mismatch | lowering recipe + runtime slot tracing | if method selection itself is wrong, diagnose semantic selection first |
| exact/derived selected defaults have enough semantic requirement data to build callee environment | conformance evidence + trait surfaces | if not, STOP AND CONSULT before adding runtime proof |
| monad failure loses symbolic evidence after target resolution | T8 evidence trace | if target resolution itself is wrong, repair inherited specialization at its owner |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing the affected issue and build a consultation packet if any of the following occurs:

1. the planned architecture appears impossible or materially wrong;
2. a core plan premise is false;
3. ownership/identity/lifetime/representation/semantic-authority boundary must change;
4. a downstream contract relied on by later tasks must materially change;
5. a semantic expected behavior/test oracle other than the two explicitly migrated tests appears wrong;
6. the same nontrivial semantic failure persists after one serious correction;
7. two plausible fixes have materially different architecture;
8. an unexpected subsystem is required for architectural rather than mechanical reasons;
9. a workaround/special-case architecture is emerging;
10. duplicate canonical reasoning/state would be introduced;
11. incremental correctness, generic reification, runtime evidence layout, or VM control semantics is uncertain/nonlocal;
12. spec/code/test/plan authorities materially conflict;
13. an invariant must be weakened to pass tests;
14. scope becomes materially more cross-cutting than planned;
15. fixing built-in subscript typing appears to require runtime value-type dispatch;
16. correct dynamic positional expansion would require VM trait proof;
17. conformance default execution appears to require method-table search rather than semantic evidence;
18. the incremental fix would require disabling validation/reuse rather than making dependencies precise;
19. the symbolic typing fix appears to require declaration-name special cases.

A triggered consultation cannot be self-waived.

### Consultation packet minimum

```text
Plan/task:
Invariant:
Observed:
Expected:
Minimal reproducer:
Relevant call path:
Scoped diff:
Evidence gathered:
Primary hypothesis:
Alternative hypothesis:
Serious corrective attempt already made:
Exact RED trigger:
Decision requested:
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
- moved helper;
- missing import;
- Rust borrow mismatch;
- test filter spelling;
- enum variant shape changed mechanically.

### 14.2 Semantic failures

Before changing production code, record locally:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow **one serious corrective attempt per underlying semantic failure family**, not one per test name.

Failure families in this plan:

```text
F1 evidence composition
F2 built-in subscript contracts
F3 Universe dependency ownership
F4 incremental product reuse
F5 positional expansion boundary
F6 conformance default evidence environment
F7 symbolic inherited generic evidence
```

If substantially the same family failure remains after its serious correction: STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

### 14.4 No unchanged reruns

Never rerun an unchanged failing test unless:
- relevant code changed;
- relevant fixture/oracle changed;
- a dependency/predecessor task changed the same mechanism.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | stale-oracle migration | INV-01,02 | opaque call drops narrow field fact to validated `Object` lifecycle evidence |
| CV-02 | unknown provenance | INV-03 | List expansion preserves `UnresolvedName("missing")` |
| CV-03 | composition breadth | INV-03 | Tuple expansion preserves same unknown |
| CV-04 | composition breadth | INV-03 | Map and Record expansion preserve same unknown |
| CV-05 | built-in success | INV-04 | assumed `List<Int>` scalar index returns `Int` at Assumed strength |
| CV-06 | negative contract | INV-04 | List non-Int scalar index emits argument mismatch |
| CV-07 | setter | INV-04,05 | List setter checks index + value and publishes Unit |
| CV-08 | Map contract | INV-04 | `Map<String,Int>` rejects Int key |
| CV-09 | identity | INV-06 | user-defined subscript setter keeps exact `CallableId` |
| CV-10 | dependency identity | INV-07 | canonical Universe `Int` read records exact declaration dependency |
| CV-11 | negative sentinel | INV-07 | synthetic `universe.core` path remains an ordinary identity |
| CV-12 | impl isolation | INV-08,09 | adding Beta impl member reuses Alpha caller |
| CV-13 | body reuse | INV-08,09 | body-only provider edit reuses unchanged caller/unrelated products |
| CV-14 | contract invalidation | INV-08,09 | signature edit recomputes exact caller only |
| CV-15 | hierarchy invalidation | INV-08,09 | superclass edit recomputes hierarchy consumers, not unrelated body |
| CV-16 | fanout | INV-08,09 | high-fanout unrelated structural edit retains existing products |
| CV-17 | export precision | INV-08,09 | unused export does not recompute consumer |
| CV-18 | lifecycle | INV-08,10 | edit/remove/re-add retires stale products and returns cold-equivalent meaning |
| CV-19 | invalid spread boundary | INV-11,12,13 | `*1` produces structured non-Iterable boundary, no DNU |
| CV-20 | canonical spread success | INV-11,14 | explicit Iterable/Iterator source expands in yielded order |
| CV-21 | traceback | INV-11,14 | actual `next` error and spread source site appear in traceback |
| CV-22 | exact generic default | INV-15,16,17 | bounded generic exact witness + default (`label -> self.print`) executes |
| CV-23 | conditional default | INV-15,16,17 | derived/conditional Box default executes sibling requirement |
| CV-24 | nested/forwarded evidence | INV-15,16,17 | nearby cv47/cv48 forwarding remains green |
| CV-25 | symbolic inherited capability | INV-18,19 | `Monad<F>` resolves inherited `Functor.map` at Assumed strength |
| CV-26 | cold/incremental differential | INV-08,10 | repaired incremental scenarios agree with cold final analysis |
| CV-27 | visibility non-regression | INV-20 | only if shared dispatch/access code is touched: focused LANG006 access-control regression remains green |
| CV-28 | broad baseline | all | final semantic and core suites no longer contain BL-01..BL-19 |

Not every task needs new tests. Existing regressions already cover most of this surface; strengthen only where the repaired mechanism would otherwise remain under-specified.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Default mode.

- exact failing filter or smallest grouped module;
- compile check only when it answers a current question;
- no full semantic/core suites;
- no workspace gates;
- no broad language corpus.

#### STABILIZE MODE

Enter at G2/G3/G4.

- directly affected semantic/core groups;
- adjacent mechanism tests named by the gate;
- no automatic workspace broadening.

#### CERTIFY MODE

Enter only at G5.

- exactly one final broad semantic suite;
- exactly one final broad core suite;
- `git diff --check`;
- no workspace test/clippy unless separately requested or a new TEST002 follow-up requires release certification.

### 16.2 Verification ladder

```text
T0 — exact reproducer/current baseline filter
T1 — directly affected feature tests
T2 — owning semantic/core subsystem group
T3 — adjacent cross-layer integration
T4 — broad phalcom-semantic/core test target
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

- exact target for the active failure family;
- test selection must select at least one test;
- `cargo check` only when compiler feedback is useful for a coherent slice.

### 16.4 Mandatory at named gates

- G1: two migrated stale-oracle tests;
- G2: evidence-composition + canonical subscript group;
- G3: incremental/dependency group;
- G4: expansion + conformance + monad typing group;
- G5: full `phalcom-semantic --test semantic` and `phalcom-core --test core`.

### 16.5 Only if evidence demands

- source/native surface generation;
- LSP tests;
- language corpus;
- module linker suites;
- full `cargo test -p phalcom-semantic`;
- full `cargo test -p phalcom-core` beyond the named integration targets.

### 16.6 Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
full language-corpus
full LSP suite
release tooling
repository-wide benchmark suites
```

Reason: C1 owns the nineteen semantic/core baseline failures, not release certification. Broad BUILD-mode runs would waste implementation time and obscure causality.

Use the pinned toolchain and prefer:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo ...
```

Run Cargo commands serially.

---

## 17. Baseline / unrelated failure policy

Classify any unexpected failure:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

Policy:

- **A/B:** active responsibility;
- **C:** one bounded classification pass; if nonblocking and still unclear, record and continue;
- **D:** record and continue immediately;
- do not consume adviser compute on D unless it blocks acceptance;
- do not weaken/skip assertions to make a broad gate green.

### 17.1 Starting baseline authority

The durable starting baseline is the `LANG006.C1.P3` recorded result plus T0's live targeted revalidation.

If current HEAD has moved and a listed test is already green:

```text
classification: resolved-before-TEST002 implementation
action: record; do not touch
```

If a new failure appears during final broad G5 but was absent from T0/current predecessor evidence:

- compare with a clean starting revision non-destructively when necessary;
- classify under A/B/C/D;
- do not silently expand P1.

---

# 18. Tasks

## T0 — Establish TEST002 records and re-pin the live post-visibility baseline

### Purpose

Create durable TEST002 execution state and prove which of the nineteen predecessor failures still reproduce on the live revision before any production correction.

### Preconditions

- no production edits for TEST002;
- read all files in §4;
- inspect `git status --short --branch`;
- preserve unrelated work.

### Consumes

- `LANG006.C1.P3` residual ledger;
- current `main` revision;
- repository workflow convention.

### Produces

- `TEST002` program record;
- `TEST002.C1` checkpoint record;
- this P1 installed at the canonical repository path;
- current live failure ledger with PASS/FAIL classification;
- starting revision/worktree state in checkpoint.

### Files and symbols

**Read:**
- `AGENTS.md`
- workflow files
- LANG006 C1 checkpoint/P3 records
- all nineteen test files.

**Modify/Create:**
- `docs/implementation/INDEX.md`
- `docs/implementation/test/TEST002-baseline-stabilization/PROGRAM.md`
- `.../C1-semantic-core-baseline-elimination/CHECKPOINT.md`
- `.../TEST002.C1.P1-baseline-failure-elimination.md`

**Tests:**
- only the nineteen named regressions, grouped where a single module filter is economical.

### Required implementation shape

Create a checkpoint whose objective is the one in §2. Record:

```text
starting revision
predecessor semantic/core counts
nineteen named baseline IDs
active plan TEST002.C1.P1
verification UNVERIFIED
```

Run targeted filters, not broad suites.

For Cargo's single positional test filter, use exact names or narrow module filters. Verify from output that tests were actually selected.

A recommended targeted sweep:

```sh
# Semantic stale/flow/composition
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  opaque_call_drops_refined_field_current_without_rewriting_contract -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  unknown_expansions_do_not_disappear -- --nocapture

# Canonical call application group
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  canonical_call_application -- --nocapture

# Incremental groups
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  adding_impl_member_does_not_invalidate_another_target_or_its_caller -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  dependency_edit_remove_readd_recomputes_affected_summary_deterministically -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  canonical_universe_reads_track_canonical_query_dependencies -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  legacy_universe_core_path_is_not_a_dependency_sentinel -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  unused_public_export_does_not_recompute_unrelated_consumer_body -- --nocapture

# Core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  positional_spread_of_non_iterable_has_structured_boundary -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  test_list_spread_traceback_includes_spread_site -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  cv46_runtime_conditional_default_executes -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  bounded_generic_function_dispatches_exact_witness_and_default -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  traverse_body_resolves_symbolic_inherited_capabilities -- --nocapture
```

If the `canonical_call_application` filter selects too much unrelated coverage, use the four exact names serially instead.

### Forbidden approaches

- no broad semantic/core run yet;
- no production edits before current classification;
- no rewriting an already-green test;
- no moving TEST001.

### Test changes required

None.

### Tests to run now

The targeted sweep above.

### Tests explicitly deferred

All G2-G5 suites.

### Acceptance

- every predecessor failure classified as current FAIL or already PASS;
- checkpoint records the exact current list;
- zero production behavior changed.

### Local STOP / CONSULT triggers

- a predecessor test's current oracle has materially changed and its semantics are unclear;
- broad repository drift has changed more than mechanics for three or more failure families.

### Checkpoint update

Yes. T0 establishes the authoritative starting ledger.

---

## T1 — Migrate the two confirmed stale test authorities

### Purpose

Remove false-red test expectations without changing accepted production behavior.

### Preconditions

- T0 confirms BL-01 and/or BL-16 still fail for the expected stale reason.

### Consumes

- `INV-01`, `INV-02`, `INV-11`, `INV-14`;
- current flow lifecycle behavior;
- canonical Iterator/Iterable architecture.

### Produces

- field-flow integration test aligned with the lifecycle contract;
- traceback fixture aligned with canonical stateful iteration.

### Files and symbols

**Read:**
- `phalcom-semantic/src/checker/flow/state.rs::invalidate_opaque_calls`
- `phalcom-semantic/src/checker/field_lifecycle.rs::lifecycle_read_knowledge`
- nearby flow unit tests
- canonical iterator source/tests.

**Modify:**
- `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
- `phalcom-core/tests/core/observability/traceback.rs`

**Tests:**
- BL-01
- BL-16

### Required implementation shape

#### T1.A — opaque field test

Rename for clarity if useful:

```text
opaque_call_drops_narrow_field_refinement_to_validated_contract
```

The accepted assertion is:

```text
before opaque call:
    field contract = Object
    current refined value = Int

after opaque call:
    narrow Int refinement is gone
    field remains definitely initialized
    contract remains Object
    current readable type = Object
    evidence origin = FieldLifecycle
    status follows lifecycle validity (Established for Validated)
```

Do not change `invalidate_opaque_calls` merely to make the old test pass.

#### T1.B — spread traceback fixture

Replace:

```phalcom
iterate(_ cursor)
iteratorValue(_ cursor)
```

with a real canonical Iterator/Iterable fixture.

Conceptual fixture:

```phalcom
class BrokenIterator {
  next -> Option<Int> {
    throw Error.new("iterator value boom")
  }
}
impl Iterator for BrokenIterator {
  type Item = Int
}

class Broken {
  iterator -> BrokenIterator { BrokenIterator.new() }
}
impl Iterable for Broken {
  type Item = Int
  type Iterator = BrokenIterator
}

[*Broken.new()]
```

Adapt exact imports/prelude names to current Universe.

Assert:

```text
"iterator value boom" is present
the canonical iterator method frame (`next`) is present
the List-spread caller source line is present
no cursor-era frame expectation remains
```

Prefer computing/maintaining source-line expectation robustly if nearby test conventions provide a helper.

### Forbidden approaches

- no production semantic changes in this task;
- no cursor compatibility shim;
- no simply deleting traceback coverage;
- no weakening traceback to only check the error string.

### Test changes required

- CV-01
- CV-21

### Tests to run now

Run exactly the two migrated tests.

### Tests explicitly deferred

- full flow module;
- all observability tests;
- broad core suite.

### Acceptance

Both tests pass using current production behavior and accepted semantics.

### Local STOP / CONSULT triggers

- current normative SEMA authority says an opaque call must erase even a validated field contract;
- a valid canonical Iterable spread cannot reach the iterator error without production changes unrelated to BL-15.

### Checkpoint update

Record both test-authority migrations as durable baseline decisions.

---

## T2 — Preserve precise unknown/dynamic provenance through collection/product expansion

### Purpose

Repair the shared expression-composition law that currently allows an outer expansion to replace a precise operand uncertainty with `SuppressedByInvalidCause`.

### Preconditions

- BL-02 reproduces.

### Consumes

- operand `TypedExpression`;
- `TypeKnowledge`;
- existing composition/projection helpers;
- semantic positional-expansion resolution for known operands.

### Produces

- one evidence-preserving expansion projection rule used consistently by List, Tuple, Map, and Record paths.

### Files and symbols

**Read:**
- `phalcom-semantic/src/checker/expression.rs`
  - List expansion
  - Tuple expansion
  - Map expansion
  - Record expansion
- `phalcom-semantic/src/checker/composition.rs`
- `compose_required_knowledge`
- `project_applied_argument`

**Modify:**
- `expression.rs`
- optionally `composition.rs` if a shared helper removes duplication cleanly.

**Tests:**
- `unknown_expansions_do_not_disappear`
- nearby unknown/dynamic composition tests
- `for_loop_preserves_iterable_unknown_reason` as a non-regression if relevant.

### Required implementation shape

Before expansion-specific projection, preserve epistemic non-known states:

```rust
match &typed.knowledge {
    TypeKnowledge::Unknown(reason) => {
        // contribute exactly this reason
    }
    TypeKnowledge::Dynamic(reason) => {
        // preserve exactly this dynamic boundary
    }
    TypeKnowledge::Known(evidence) => {
        // existing expansion-specific projection/resolution
    }
}
```

For known positional expansions:

1. attempt canonical semantic iteration resolution where applicable;
2. if a known structural product projection is valid, use it;
3. if projection genuinely fails without a more specific operand cause, use the existing appropriate unknown category.

Do not create four subtly different uncertainty policies for List/Tuple/Map/Record.

Preserve dependency/causal propagation from the operand even when its type cannot be projected.

### Forbidden approaches

- no special case for the identifier `"missing"`;
- no string-based diagnostic propagation;
- no changing all `SuppressedByInvalidCause` globally;
- no converting unknown to Dynamic.

### Test changes required

Strengthen existing BL-02 only if needed to distinguish:
- List;
- Tuple;
- Map;
- Record.

Avoid four new redundant tests if the existing loop already asserts all four.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  unknown_expansions_do_not_disappear -- --nocapture
```

If helper behavior is shared with for-loops:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  expression_composition -- --nocapture
```

only after the exact test passes.

### Tests explicitly deferred

Canonical call application; incremental; core.

### Acceptance

CV-02 through CV-04 pass and no existing composition test indicates lost causal status.

### Local STOP / CONSULT triggers

- preserving operand cause conflicts with an effective semantic evidence spec;
- one expansion category requires materially different evidence semantics rather than a mechanical projection distinction.

### Checkpoint update

No durable interface change unless a new shared composition helper becomes a canonical owner.

---

## T3 — Restore canonical scalar List/Map subscript application

### Purpose

Make built-in generic scalar subscript semantics reach the canonical argument-checking engine without disrupting broad runtime bracket behavior or user-defined subscript identity.

### Preconditions

- BL-03..BL-06 reproduce;
- inspect the actual resolved target/signature for each before editing.

### Consumes

- applied receiver type;
- existing `structural_*_index_*_target` helpers;
- `apply_resolved_callable`;
- ordinary selector dispatch for non-built-in/user-defined receivers.

### Produces

- correct scalar built-in contracts;
- unchanged ordinary user-defined subscript identity;
- `Unit` assignment expression result.

### Files and symbols

**Read:**
- `phalcom-semantic/src/checker/expression.rs`
  - `synthesize_index_expr`
  - `synthesize_set_index_expr`
- `phalcom-semantic/src/checker/call.rs`
  - structural List/Map targets
  - `apply_resolved_callable`
- List/Map Universe sources
- relevant subscript/index decision/spec.

**Modify:**
- primarily `expression.rs`;
- `call.rs` only for canonical helper consolidation if needed;
- do not edit Universe List/Map source unless verification proves their formal surface is the intended owner.

**Tests:**
- BL-03..BL-06
- `user_defined_subscript_set_keeps_exact_callable_identity`
- nearby generic index tests.

### Required implementation shape

First collect one debug fact for each failing built-in call:

```text
receiver type
selector
ordinary resolved target, if any
ordinary target parameter knowledge
ordinary return knowledge
structural target, if any
chosen target
```

If, as expected, broad source bracket dispatch is found first and lacks the scalar generic relation, repair the semantic selection order for **recognized built-in applied scalar subscripts**.

The accepted pattern is:

```text
if receiver is exactly canonical List<T> or Map<K,V>
and operation is the supported scalar selector shape
and existing canonical structural semantic target can be formed:
    apply that canonical semantic operation target
else:
    use ordinary dispatch behavior
```

For setters:

```text
operation target:
    checks Int index
    checks T value
    carries operation status/causality

assignment wrapper:
    publishes Unit
```

Do not bypass `apply_resolved_callable`.

Do not apply this override to:
- arbitrary subclasses unless the established type model says the canonical applied receiver relation applies;
- user-defined bracket members;
- Range-specific runtime behavior;
- multi-argument bracket forms.

Keep the existing structural helpers transitional and clearly owned. If the live semantic surface infrastructure now supports publishing a stronger canonical scalar operation *without lying about the broad runtime member*, use that existing mechanism instead. Do not invent a parallel registry.

### Forbidden approaches

- no compiler-side List/Map type checks;
- no VM type validation for static contracts;
- no runtime type-directed overload selection;
- no annotating a broad Range-or-Int method as Int-only if that is false;
- no loss of user-defined callable identity;
- no direct diagnostic emission that bypasses call application.

### Test changes required

Existing tests already cover CV-05..CV-09. Add only a narrowly missing discriminator if the fix could accidentally apply to user-defined subscripts.

### Tests to run now

Run the four failing exact tests, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  user_defined_subscript_set_keeps_exact_callable_identity -- --nocapture
```

Optionally run `canonical_call_application` as G2, not repeatedly here.

### Tests explicitly deferred

Incremental and core.

### Acceptance

- List assumed receiver produces Assumed `Int`;
- bad List index produces one mismatch;
- bad List setter produces two mismatches and Unit expression type;
- bad Map key produces one mismatch and Int result contract;
- user-defined setter keeps exact `CallableId`.

### Local STOP / CONSULT triggers

- correct built-in typing requires argument-value-based selector overload resolution;
- source/runtime Range behavior cannot coexist with semantic scalar specialization under current operation architecture;
- the fix would require changing public subscript semantics.

### Checkpoint update

Record the canonical built-in subscript selection rule only if it becomes a durable new semantic interface.

---

## G2 — Semantic expression/call coherence gate

### Purpose

Prove that T2/T3 repair formal expression semantics before entering incremental work.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  expression_composition -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  canonical_call_application -- --nocapture
```

If `canonical_call_application` is not the live module filter, inspect `-- --list` and use the actual path.

### Expected

```text
BL-02..BL-06 PASS
user-defined subscript identity PASS
no new formal-expression failure in these groups
```

Do not broaden after PASS.

On failure, classify before editing.

---

## T4 — Correct Universe bootstrap dependency ownership

### Purpose

Fix the direct contradiction in `is_bootstrap_declaration` so canonical Universe source declarations participate in exact semantic dependency tracking.

### Preconditions

- BL-09/BL-10 reproduce;
- confirm current `ModuleId::universe_root()` identity.

### Consumes

- declaration module identity;
- semantic dependency recorder.

### Produces

- root-only bootstrap exclusion;
- exact dependency edges for concrete/synthetic Universe module identities.

### Files and symbols

**Read:**
- `phalcom-semantic/src/checker/context.rs`
- `ModuleId` constructors/equality
- `core_surface::universe_declaration`.

**Modify:**
- `checker/context.rs`.

**Tests:**
- BL-09
- BL-10
- nearby built-in type-test dependency exception.

### Required implementation shape

Make code match the existing documented invariant:

```text
bootstrap declaration
    iff
declaration belongs to the aggregate Universe root bootstrap input
```

Conceptual correction:

```rust
fn is_bootstrap_declaration(declaration: &DeclarationId) -> bool {
    declaration.module == ModuleId::universe_root()
}
```

Use the canonical equivalent if ModuleId exposes a dedicated predicate.

Retain the separate, narrowly documented built-in type-test callable exception if it remains necessary.

Do not conflate:

```text
ProjectIdentity::Universe
```

with:

```text
aggregate bootstrap root
```

### Forbidden approaches

- no path-string sentinel;
- no `"core"` magic component;
- no suppress-all-Universe dependency policy;
- no removing dependency recording to pass tests.

### Test changes required

None unless a nearby test needs a stronger explicit real-Universe/synthetic-Universe distinction.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  canonical_universe_reads_track_canonical_query_dependencies -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  legacy_universe_core_path_is_not_a_dependency_sentinel -- --nocapture
```

Also run the nearby built-in type-test dependency test if its exception shares this helper.

### Tests explicitly deferred

Other incremental groups until T5.

### Acceptance

CV-10/CV-11 pass, with no aggregate root impossible-query dependency introduced.

### Local STOP / CONSULT triggers

- canonical Universe source declarations truly have no DB product to depend on under current session architecture;
- root identity cannot be distinguished from concrete source modules without changing module identity semantics.

### Checkpoint update

Record the root-only bootstrap dependency invariant.

---

## T5 — Restore exact incremental product reuse and propagation stopping

### Purpose

Eliminate the remaining unrelated-recompute failures without weakening the semantic DB reuse contract.

### Preconditions

- T4 complete;
- re-run the incremental failures after T4 because precise Universe edges may remove secondary symptoms;
- work only on those still failing.

### Consumes

- `QueryState::{revision,validated_revision}`;
- input/product fingerprints;
- dependency index;
- session contribution/work sets;
- `SemanticDb::validate_reuse`.

### Produces

- unaffected product reuse;
- exact affected recomputation;
- propagation stopping on stable semantic products;
- remove/re-add stale-product retirement;
- cold/incremental equivalence.

### Files and symbols

**Read:**
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/db/mod.rs`
- `db/query.rs`
- `db/state.rs`
- `db/fingerprint.rs`
- `semantic_shard.rs` if reached by the current work-set code.

**Modify:**
- the first canonical owner proven by instrumentation.
- Expected primary owner: `session.rs`.
- Modify DB/fingerprint code only if evidence proves the DB/input product itself is wrong.

**Tests:**
- BL-07
- BL-08
- BL-11
- BL-12
- BL-13
- BL-14
- nearby stable-intermediate/cold-equivalence tests.

### Required implementation shape

#### T5.A — instrument before correction

For each still-failing unrelated body/product, collect:

```text
QueryKey
old computation revision
old validated revision
old input fingerprint
current input fingerprint
old product fingerprint
candidate invalidation/work-set seed
dependency edge(s) causing candidate consideration
dependency old/new product fingerprint
SemanticDb::validate_reuse result
whether session discarded/forced recompute before reuse
final product fingerprint
```

Use temporary test/debug instrumentation; remove it unless it becomes genuinely useful low-cost diagnostics.

#### T5.B — classify the cause

Case A:

```text
DB says reusable
but session recomputes anyway
```

Repair session scheduling/orchestration. The session must allow the DB to own reuse.

Case B:

```text
DB says not reusable
because an unrelated dependency/input fingerprint changes
```

Repair dependency/fingerprint ownership at the producer. Do not force reuse around a genuinely changing fingerprint.

Case C:

```text
changed intermediate recomputes
but republishes identical semantic product
downstream still recomputes
```

Repair propagation so stable product fingerprint terminates reverse invalidation and downstream validation advances without recomputation.

#### T5.C — preserve exact invalidation

The same repair must still recompute:

- exact callers after callable contract change;
- exact hierarchy consumers after superclass change;
- consumer alias/type products when the consumed provider shell actually changes.

#### T5.D — remove/re-add lifecycle

Treat current `dependency_edit_remove_readd...` as a valid oracle.

It must prove:

```text
body-only edit:
    provider body recomputed
    unchanged caller/unrelated product reused

signature edit:
    exact caller recomputed
    result type updates

remove:
    old declaration/callable products retired

re-add:
    new product visible
    caller meaning correct
    no internal incidents
```

Do not restore a numeric revision assertion from an older test version.

### Forbidden approaches

- no blanket "reuse if source text unchanged";
- no test-specific query keys;
- no disabling reverse dependencies;
- no treating `validated_revision` as computation revision;
- no copying an old `Arc` when semantic input actually changed;
- no aggregate module invalidation as a shortcut;
- no per-test special cases.

### Test changes required

Prefer none. Existing tests are the contract.

If instrumentation reveals one test accidentally asserts an implementation detail beyond the DB's documented contract, STOP AND CONSULT rather than editing it.

### Tests to run now

After one coherent correction, run only the exact still-failing cases.

Then G3 runs the grouped incremental surface.

### Tests explicitly deferred

Core and broad semantic.

### Acceptance

CV-12..CV-18 pass.

For reuse cases:

```text
state.revision == original computation revision
state.validated_revision == current revision
semantic product fingerprint unchanged
Arc identity retained where the snapshot contract promises it
```

For changed cases, exact affected products recompute.

### Local STOP / CONSULT triggers

- DB and session both appear internally consistent yet valid tests conflict;
- fixing one reuse case breaks exact invalidation;
- semantic product fingerprint includes presentation/source-location data that requires a representation split beyond current architecture;
- a new cache layer appears necessary.

### Checkpoint update

Record the diagnosed canonical root cause and the repaired reuse invariant. Do not log every test run.

---

## G3 — Incremental correctness gate

### Purpose

Prove dependency ownership, product reuse, exact invalidation, lifecycle recovery, and cold-equivalent meaning as one coherent subsystem.

### Run

Use the current module filters, for example:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  incremental::checker_dependencies -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  incremental::callable_dependencies -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  incremental::product_stability -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  incremental::recomputation_metrics -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  impls::queries -- --nocapture
```

If any filter is stale, inspect `-- --list` and adapt mechanically.

### Expected

```text
BL-07..BL-14 PASS
no loss of exact invalidation
no cold/incremental divergence in adjacent tests
```

Do not run full semantic suite yet.

---

## T6 — Make positional expansion evidence-driven and fail closed

### Purpose

Remove the remaining ordinary `iterator`/`next` fallback from canonical positional expansion and surface the structured non-Iterable boundary while preserving Tuple/Unit fast lanes and real iterator errors.

### Preconditions

- T1 migrated the traceback fixture;
- BL-15 still reproduces;
- inspect semantic analysis for `*1` and for a valid canonical Iterable spread.

### Consumes

- `CallableAnalysis::positional_expansions`;
- `ModuleLoweringSemantics::positional_expansions`;
- Tuple/Unit product fast lane;
- `RuntimeError::NonIterableStarOperand`;
- canonical iterator requirement invocation.

### Produces

- no structural ordinary-send iteration proof;
- structured invalid spread boundary;
- valid Iterable execution from selected semantic facts.

### Files and symbols

**Read:**
- `phalcom-semantic/src/checker/iteration.rs`
- expansion publication in checker expression/call code
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/expr.rs::compile_positional_expansion`
- `phalcom-core/src/vm/dispatch.rs` product fast-lane bytecodes
- `phalcom-core/src/error.rs`
- current LANG005.C6 P5 audit/closure records if implementation intent is ambiguous.

**Modify:**
- semantic expansion publication if known invalid/dynamic state needs explicit representation;
- `semantic_lowering.rs` if a stable boundary fact must be projected;
- `compiler/lib/expr.rs`;
- VM only if an already-approved dynamic runtime guard has to surface the existing error.

**Tests:**
- BL-15
- migrated BL-16
- existing outgoing/list positional expansion success tests.

### Required implementation shape

#### T6.A — distinguish absence meanings

Determine why `positional_expansions` is absent for the failing non-Iterable operand.

Do not treat all absence as equivalent.

Required semantic categories are conceptually:

```text
Tuple/Unit product lane
canonical Iterable evidence selected
known invalid positional source
approved dynamic/unknown boundary
```

If current semantic analysis only omits the lowering fact for known invalid sources, extend the existing analysis/lowering product minimally enough that compiler lowering can fail closed without rediscovering type/conformance facts.

#### T6.B — delete structural proof fallback

The compiler must no longer do:

```rust
None => emit_getter_send("iterator")
...
None => emit_getter_send("next")
```

as a canonical eligibility path.

For a valid evidence-backed source, continue to use:

```text
emit_for_iteration_requirement(iterator)
emit_for_iteration_requirement(next)
```

#### T6.C — structured boundary

Known invalid `*` source must produce the established structured error/diagnostic:

```text
* expansion requires Tuple, Unit, or an iterable value; got <kind>
```

Use the current accepted compiler/runtime boundary; do not invent a second error category unless the effective spec requires one.

#### T6.D — preserve real iterator failures

Once `iterator`/`next` is selected from semantic evidence:
- do not catch/relabel its ordinary error;
- traceback must show the actual iterator behavior and spread caller site.

#### T6.E — dynamic boundary

If genuinely Dynamic operands are permitted by the current argument specification but no semantic evidence can preselect iteration, inspect the accepted LANG005/COLL005 decision.

If supporting them would require runtime conformance proof or structural probing: **STOP AND CONSULT.**

### Forbidden approaches

- no cursor selectors;
- no `has_iteration_protocol`;
- no `.iterator`/`.next` structural probe as proof;
- no runtime trait solver;
- no DNU-catching heuristic;
- no converting known invalid to arbitrary Dynamic;
- no moving Tuple/Unit fast-lane semantics into trait proof.

### Test changes required

BL-16 already migrated in T1.

Add/strengthen only if needed:
- canonical Iterable spread success;
- object with `iterator` method but no Iterable evidence fails closed;
- real `next` error propagates.

Prefer existing LANG005/COLL005 coverage where it already proves these.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  positional_spread_of_non_iterable_has_structured_boundary -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  test_list_spread_traceback_includes_spread_site -- --nocapture
```

Then the narrow outgoing positional expansion group that proves valid Iterable behavior.

### Tests explicitly deferred

Full core.

### Acceptance

CV-19..CV-21 pass and disassembly/source evidence shows no generic structural iterator fallback on the repaired canonical path.

### Local STOP / CONSULT triggers

- approved Dynamic iterable expansion lacks a representation that can execute without runtime proof;
- fixing BL-15 would require changing the `*` language contract;
- valid generic Iterable execution has no semantic lowering fact due to a deeper LANG005 proof defect.

### Checkpoint update

Record retirement of ordinary-send fallback if this is a durable implementation closure.

---

## T7 — Build callee-compatible conformance environments for selected defaults

### Purpose

Fix generic/conditional trait default execution so nested `InvokeTraitRequirement` uses a runtime environment laid out for the compiled callee body's requirement slots.

### Preconditions

- BL-17/BL-18 reproduce with:
  ```text
  no active conformance target for requirement Printable.print
  ```
  or equivalent current error;
- capture current semantic selection and runtime slot layouts before editing.

### Consumes

- semantic `ConformanceEvidence` / `GenericConformanceEvidence`;
- selected `RequirementSelectionTemplate`;
- `GenericEvidenceLayout`;
- call specialization;
- `RuntimeConformanceSlotRecipe`;
- `RuntimeConformanceEnvironment`;
- default callable's compiled requirement invocations.

### Produces

- deterministic callee-layout nested environment for selected default/witness behavior that invokes requirements;
- unchanged semantic selection;
- VM remains proof-free.

### Files and symbols

**Read:**
- `phalcom-semantic/src/generic_evidence.rs`
- `phalcom-semantic/src/checker/analysis.rs`
- generic call conformance binding construction in `checker/call.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/typing/conformance_environment.rs`
- `phalcom-core/src/compiler/lib/scope.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/vm/send.rs`
- `phalcom-core/src/frame.rs`

**Modify:**
- expected primary: `modules/compile.rs` + runtime recipe representation/materialization;
- semantic code only if required evidence is genuinely missing from the published product.

**Tests:**
- BL-17
- BL-18
- cv47/cv48 or closest nested/forwarded conformance tests
- repeated environment reuse if representation changes.

### Required implementation shape

#### T7.A — prove the mismatch

For `Printable.label` calling `self.print`, record:

```text
caller's GenericEvidenceLayout
callee default callable identity
callee body's TraitRequirementInvocationSpec
callee requirement slot for `print`
runtime environment id entering `label`
runtime environment slots actually present
selected `print` requirement target
```

The expected defect is:

```text
default body asks for slot Sdefault
environment belongs to a different layout
slot Sdefault absent
```

Do not patch until this is verified.

#### T7.B — construct environment from semantic evidence

For a selected trait default, construct the nested executable environment from the conformance proof's selected requirement targets **using the callee/default body's layout**.

Conceptually:

```text
selected default callable
    +
conformance evidence for exact target/trait ref
    +
default callable GenericEvidenceLayout
        ↓
for each callee requirement slot:
    select already-proven requirement target from evidence
    encode Selection / Forwarded / Derived recipe
        ↓
nested RuntimeConformanceEnvironment
        ↓
store nested env on selected default runtime slot
```

Do not assume the caller's slot number equals the callee's.

#### T7.C — representation

It is acceptable to mechanically generalize:

```rust
RuntimeConformanceSlotRecipe::Selection {
    callable,
    selector,
}
```

to carry a nested recipe/environment descriptor, or to add an equivalent private recipe variant.

The fixed rule is:

> the selected executable method and the environment required by that method form one runtime activation recipe.

Do not add semantic proof data to the VM beyond stable executable recipes.

#### T7.D — exact and derived evidence

Cover both:
- exact `User |= Printable` default;
- conditional/derived `Box<User> |= Printable` default.

Do not only fix the derived branch.

The current `if !is_trait_default` nested-slot exclusion is a prime audit site, not an instruction to delete blindly: sibling default requirements and antecedent nested conformances are different evidence relations. Build the environment from the correct requirement map/layout.

#### T7.E — preserve forwarding

Existing forwarded generic calls must continue mapping:

```text
callee slot <- caller slot
```

through semantic `GenericEvidenceLayout` identities.

No slot compaction.

### Forbidden approaches

- no runtime conformance search;
- no fallback `receiver.lookup_method("print")` as proof;
- no hard-coded `Printable` or slot zero;
- no copying caller environment wholesale without layout proof;
- no flattening nested evidence to first requirement;
- no source-order-based slot mapping;
- no disabling the missing-slot error.

### Test changes required

Existing BL-17/18 are sufficient primary regressions.

Add one low-level recipe/layout test if current tests cannot prove:
- same trait, sibling requirement, default body;
- environment slot mapping independent of caller ordering.

### Tests to run now

Run BL-18 first as smallest exact case, then BL-17.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  cv47_nested_runtime_evidence_invokes_nested_behavior -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  cv48_generic_assumed_nested_evidence -- --nocapture
```

Adapt names mechanically.

### Tests explicitly deferred

Full generic-conformance/conditional-conformance modules until G4.

### Acceptance

CV-22..CV-24 pass; VM trace/debug evidence confirms `InvokeTraitRequirement` resolves the preselected target from a callee-compatible environment.

### Local STOP / CONSULT triggers

- semantic evidence does not identify the sibling requirement target needed by the default;
- default callable lacks a stable evidence layout and adding one changes public metadata architecture;
- exact and derived cases require fundamentally different runtime proof mechanisms;
- two candidate environment representations have materially different lifetime/rooting semantics.

### Checkpoint update

Record the runtime activation/environment invariant if representation changes.

---

## T8 — Preserve symbolic assumed evidence through inherited generic capability specialization

### Purpose

Repair the `Monad<F>` inherited capability path so `Functor.map` remains analyzable and publishable while type parameters are symbolic.

### Preconditions

- BL-19 reproduces;
- `pure` and `flatMap` behavior in the same fixture establishes comparison evidence.

### Consumes

- symbolic `TypeParameterId` / canonical parameter form;
- `ExpectedType`;
- inherited receiver specialization;
- generic callable instantiation;
- call relation/status aggregation;
- call specialization publication.

### Produces

- Ready inherited `Functor.map` call;
- `Assumed` symbolic generic solutions;
- unchanged exact callable/owner traversal.

### Files and symbols

**Read:**
- `phalcom-core/tests/core/typing_integration/monads/bodies.rs`
- monad source fixture/support
- `phalcom-semantic/src/checker/context.rs`
  - dispatch target specialization
- `phalcom-semantic/src/checker/call.rs`
  - generic application/inference
- `types/instantiation.rs`
- `types/rigid.rs`
- generic evidence/specialization helpers reached by trace.

**Modify:**
- only the first general owner where `Known/Assumed(symbolic)` becomes `Unknown(NoTypeEvidence)`.

**Tests:**
- BL-19
- nearby `kleisli_body_proves_symbolic_callable_composition`
- other inherited generic capability tests.

### Required implementation shape

#### T8.A — trace first evidence loss

At the failing `monad.map(...)` expression, capture in order:

```text
receiver knowledge/status
receiver canonical type
resolved owner path:
    Monad -> Applicative -> Functor
resolved CallableId
receiver specialization
fixed generics
callable local generic parameters
argument knowledge
generic constraints/solutions
return knowledge before status cap
return knowledge after status cap
AnalysisStatus
call_specialization publication
trait/capability selection publication
```

Find the first transition:

```text
Known(symbolic type, Assumed)
    ->
Unknown(NoTypeEvidence)
```

#### T8.B — fix the general rule

A type parameter in the active generic environment is not an absence of type evidence.

Correct behavior:

```text
canonical symbolic TypeParameter form
+
active generic assumption
    ->
TypeKnowledge::Known(..., EvidenceStatus::Assumed)
```

Use the existing canonical representation; do not invent a new symbolic type kind.

#### T8.C — inherited owner specialization

Ensure substitutions compose across:

```text
Monad<F>
    -> Applicative<F>
    -> Functor<F>
```

without requiring `F` to be concretized.

The final `map` callable identity remains `Functor.map`; receiver traversal is recorded as existing fixture expectations require.

### Forbidden approaches

- no `Monad`/`Functor` name matching;
- no marking the call Dynamic;
- no making test assertion accept Blocked;
- no assigning `Object`/`Dynamic` to `F`;
- no bypassing generic inference;
- no weakening call-specialization publication.

### Test changes required

Prefer none.

Add a smaller semantic test only if the core typing-integration fixture is too broad to isolate the corrected evidence transition.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  traverse_body_resolves_symbolic_inherited_capabilities -- --nocapture
```

Then nearby monad body tests.

### Tests explicitly deferred

Full core typing integration until G4.

### Acceptance

CV-25 passes:
- `map` expression is Ready;
- callable identity is `Functor.map`;
- result is the expected symbolic state type;
- owner traversal remains `Monad -> Applicative -> Functor`;
- generic evidence remains Assumed, not Dynamic or NoTypeEvidence.

### Local STOP / CONSULT triggers

- preserving symbolic evidence requires a new published type representation;
- inherited generic composition conflicts with accepted variance/constraint semantics;
- the same loss occurs in unrelated HKT operations and implies a larger type-system redesign.

### Checkpoint update

Record only a durable generic-evidence invariant, not fixture details.

---

## G4 — Core cross-layer stabilization gate

### Purpose

Prove positional expansion, conformance transport, and symbolic typing after their owners are corrected.

### Run

Use focused modules:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  compiler_declaration_dispatch -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  observability::traceback -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  language::generic_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  language::conditional_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  typing_integration::monads::bodies -- --nocapture
```

Adapt exact live filters with `-- --list` if required.

If T7 changes runtime environment interning/materialization, also run the existing focused conformance-environment unit tests.

### Expected

```text
BL-15..BL-19 PASS
canonical iterator success remains PASS
nested/forwarded generic evidence remains PASS
no structural cursor fallback reappears
```

Do not broaden to the full core suite until G5.

---

## T9 — Focused closure, record cleanup, and broad baseline certification

### Purpose

Complete TEST002.C1 by proving all coverage obligations and running the broad semantic/core gates exactly once.

### Preconditions

- G1/G2/G3/G4 green;
- no unresolved A/B failure;
- consultations, if any, recorded.

### Consumes

- all implemented corrections;
- checkpoint ledger;
- final worktree diff.

### Produces

- final broad evidence;
- accurate completion/verification state;
- walkthrough;
- handoff;
- no temporary instrumentation.

### Files and symbols

**Read:**
- all diff;
- checkpoint record;
- plan coverage ledger.

**Modify/Create:**
- `CHECKPOINT.md`
- `TEST002.C1.P1-walkthrough.md`
- `TEST002.C1.P1-handoff.md`
- plan metadata/status only if repository workflow updates it on completion.

### Required implementation shape

1. Remove temporary debug-only instrumentation.
2. Inspect scoped diff for:
   - duplicate authorities;
   - stale comments;
   - accidental visibility/access changes;
   - test weakening;
   - cursor references in active code introduced by the patch.
3. Run `git diff --check`.
4. Run the final broad gates once.
5. If a broad failure is new, classify A/B/C/D before editing.
6. Map all CV obligations to actual PASS evidence.
7. Update checkpoint and create walkthrough/handoff.

### Forbidden approaches

- no last-minute unrelated cleanup;
- no workspace test just because broad crate targets are green;
- no claiming RELEASE_COMPLETE without release gates;
- no deleting a newly failing test to finish the plan.

### Tests to run now

See G5.

### Tests explicitly deferred

```text
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
full release/packaging certification
```

unless the user separately requests them.

### Acceptance

All named baseline failures eliminated; G5 classification complete; records truthful.

### Local STOP / CONSULT triggers

- a named baseline remains after one serious correction family attempt;
- broad G5 reveals a new A/B cross-layer regression not explained by current task architecture.

### Checkpoint update

Mandatory final update.

---

# 19. Verification gates

## G1 — Test-authority alignment gate

Purpose: prove the two confirmed stale tests now express accepted current semantics without production workaround.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  opaque_call_drops_narrow_field_refinement_to_validated_contract -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  test_list_spread_traceback_includes_spread_site -- --nocapture
```

Adapt the renamed first test if the implementer keeps the old name.

Expected:

```text
field fallback test PASS
canonical iterator traceback fixture PASS
```

Do not broaden after PASS.

---

## G2 — Semantic expression/call gate

Defined after T3.

Required evidence:
- CV-02..CV-09.

---

## G3 — Incremental ownership/reuse gate

Defined after T5.

Required evidence:
- CV-10..CV-18;
- cold/incremental equivalence remains intact.

---

## G4 — Core cross-layer gate

Defined after T8.

Required evidence:
- CV-19..CV-25.

---

## G5 — Final post-baseline certification gate

### Purpose

Prove that the exact broad targets which exposed the predecessor baseline no longer contain those failures.

### Run serially

```sh
git diff --check

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
```

### Expected

At minimum:

```text
none of BL-01..BL-19 fail
```

Preferred checkpoint result:

```text
phalcom-semantic --test semantic: 0 failed
phalcom-core --test core:         0 failed
```

Ignored-test counts need not change.

If new unrelated failures exist due repository movement, classify them under §17. They do not authorize weakening TEST002 fixes.

### Optional adjacent visibility check

Only if TEST002 touched shared dispatch/access selection beyond the local planned seams:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  integration::access_control -- --nocapture
```

Do not run this reflexively if access code is untouched.

---

## 20. Final focused acceptance

At plan completion, produce this filled mapping in the walkthrough:

| Coverage ID | Concrete test/evidence | Result |
|---|---|---|
| CV-01 | opaque-call field lifecycle test | PASS |
| CV-02 | List unknown expansion | PASS |
| CV-03 | Tuple unknown expansion | PASS |
| CV-04 | Map/Record unknown expansion | PASS |
| CV-05 | assumed List scalar subscript | PASS |
| CV-06 | List index mismatch | PASS |
| CV-07 | List setter index/value + Unit | PASS |
| CV-08 | Map key mismatch | PASS |
| CV-09 | user subscript identity | PASS |
| CV-10 | canonical Universe dependency | PASS |
| CV-11 | legacy-looking Universe path ordinary dependency | PASS |
| CV-12 | impl target isolation | PASS |
| CV-13 | body-only reuse | PASS |
| CV-14 | exact signature invalidation | PASS |
| CV-15 | exact hierarchy invalidation | PASS |
| CV-16 | high-fanout product retention | PASS |
| CV-17 | unused export precision | PASS |
| CV-18 | edit/remove/re-add lifecycle | PASS |
| CV-19 | non-Iterable spread boundary | PASS |
| CV-20 | canonical Iterable spread success | PASS |
| CV-21 | iterator-error traceback | PASS |
| CV-22 | exact generic default | PASS |
| CV-23 | conditional generic default | PASS |
| CV-24 | nested/forwarded evidence | PASS |
| CV-25 | symbolic inherited capability | PASS |
| CV-26 | cold/incremental differential | PASS |
| CV-27 | visibility/access non-regression if applicable | PASS / N-A |
| CV-28 | broad semantic/core baseline | PASS / classified |

Run only the plan's stabilization/certification gates.

Do not automatically run workspace certification.

---

## 21. Performance/resource evidence

This plan has no new performance target.

However, incremental correctness fixes must not achieve green tests by intentionally widening recomputation.

For T5, existing recomputation metrics are semantic correctness/performance guards. Preserve their bounded-work expectations.

For runtime conformance environment changes:
- reuse existing environment interning;
- do not introduce per-opcode proof/allocation work;
- if a representation change unexpectedly creates per-call unbounded environment construction, classify as an architectural risk and consult.

No new benchmark ceremony is required.

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

- create/confirm TEST002 program and C1 checkpoint records;
- register TEST002 in `docs/implementation/INDEX.md`;
- mark P1 active;
- record starting revision;
- record current T0 PASS/FAIL ledger.

### 22.2 During plan

Update checkpoint only for:

- stale-oracle migration decisions;
- durable new invariant/interface;
- consultation/amendment;
- deferred failure;
- G1/G2/G3/G4 coherent verification evidence.

Do not write a chronological test diary.

### 22.3 At plan completion

- set P1 status/completion honestly;
- record G5 evidence;
- record final invariants;
- record residual/deferred failures;
- record consultations;
- set C1 next action:
  - checkpoint closure if accepted;
  - P2 if a bounded remaining corrective plan is required.

### 22.4 Suggested checkpoint metadata at creation

```yaml
---
id: TEST002.C1
category: TEST
program: TEST002
kind: checkpoint-record
status: IN_PROGRESS
completion: NOT_STARTED
verification: UNVERIFIED
requires:
  - LANG006.C1
---
```

Do not mark C1 `COMPLETE` merely because all intended edits exist.

---

## 23. Walkthrough deliverable

Create:

```text
TEST002.C1.P1-walkthrough.md
```

It must contain:

1. final result;
2. starting and ending repository revisions/state;
3. which of BL-01..BL-19 reproduced at T0;
4. the two test migrations actually performed;
5. architecture implemented for each repaired failure family;
6. important source changes;
7. any deviations from this plan;
8. consultation decisions;
9. tests added/changed;
10. tests actually run;
11. tests deliberately deferred;
12. G1-G5 results;
13. completed CV mapping;
14. residual risks/failures;
15. truthful verification classification.

Never claim a broad gate passed if it was not run to completion.

---

## 24. Handoff deliverable

Create:

```text
TEST002.C1.P1-handoff.md
```

It must contain:

- current repository revision/state anchor;
- checkpoint status;
- stable interfaces established by TEST002;
- inherited LANG006 visibility invariants;
- all TEST002 global invariants that remain relevant;
- deferred failures;
- next objective, if any;
- must-read files;
- first recommended commands;
- explicit things not to redesign/re-explore:
  - visibility/access;
  - cursor iteration;
  - runtime trait proof;
  - subscript syntax;
  - semantic DB revision semantics.

If C1 closes fully, the handoff may state that no TEST002.C1.P2 is required and point the next owner to release/workspace certification if desired.

---

## 25. Completion truth table

Do not conflate:

```text
source written
    !=
tests migrated
    !=
targeted regressions passed
    !=
affected subsystem gates passed
    !=
broad semantic/core baseline passed
    !=
checkpoint accepted
    !=
release certified
```

Recommended truth states:

### During BUILD

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

### After implementation + G1-G4

```yaml
status: IN_PROGRESS
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

### After G5 with all predecessor baseline failures eliminated

If broad semantic/core targets are green:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Use `RELEASE_COMPLETE` only if separately required release/workspace gates were actually run.

If G5 is blocked solely by proven unrelated new baseline failures:

```yaml
status: COMPLETE          # only if C1 acceptance is otherwise met
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
```

and record those failures explicitly.

---

## 26. Plan self-review

The principal architect has checked this plan against the Luna patch-grade schema.

### Architecture

- [x] Every important fact has one canonical owner.
- [x] Semantic facts remain in `phalcom-semantic`.
- [x] Compiler/VM consume preselected iteration/conformance facts rather than reconstructing proof.
- [x] Incremental reuse remains DB-owned.
- [x] Visibility/access architecture is preserved.
- [x] Runtime environment slot identity is explicit.
- [x] Symbolic generic evidence semantics are explicit.

### Luna executability

- [x] FIXED, MECHANICALLY FLEXIBLE, and VERIFY-FIRST areas are separated.
- [x] Each semantic family has one-serious-attempt debugging budget.
- [x] STOP/CONSULT triggers are objective.
- [x] Tasks are coherent rather than edit-sized microsteps.
- [x] Current revision and changed predecessor surfaces are named.
- [x] A live rebaseline prevents stale-plan coding.

### Testing

- [x] Every original baseline failure maps to a task/coverage obligation.
- [x] Two confirmed stale tests are corrected rather than production-patched.
- [x] Current remove/re-add incremental test is treated as a valid oracle.
- [x] Broad suite execution is delayed to G5.
- [x] Workspace/release tests are explicitly deferred.
- [x] Cold/incremental and nested-evidence discriminators are included.

### Documentation lifecycle

- [x] TEST002 program/checkpoint creation is planned.
- [x] CHECKPOINT is the durable shared state.
- [x] Walkthrough and handoff are mandatory.
- [x] No competing implementation-state document is introduced.
- [x] TEST001 is not opportunistically migrated.

### Premium-compute efficiency

- [x] Likely architectural forks have been pre-decided.
- [x] Incremental and conformance tasks specify cheap evidence to collect before escalation.
- [x] Adviser escalation is decision-oriented.
- [x] No premium review is required for ordinary mechanical drift.

### Final implementation sequence

```text
T0  current baseline + TEST002 records
 ↓
T1  stale test-authority migrations
 ↓ G1
T2  expansion evidence preservation
 ↓
T3  canonical built-in subscript contracts
 ↓ G2
T4  Universe dependency root ownership
 ↓
T5  incremental product reuse/propagation
 ↓ G3
T6  fail-closed semantic positional expansion
 ↓
T7  callee-compatible conformance environments
 ↓
T8  symbolic inherited generic evidence
 ↓ G4
T9  cleanup + one-time broad certification
 ↓ G5
TEST002.C1 acceptance
```

No placeholders remain. If live repository evidence invalidates a fixed architectural premise, do not silently rewrite this plan during implementation; use the shared consultation/amendment protocol.

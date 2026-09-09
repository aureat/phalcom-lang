# Phalcom semantic correctness audit — 2026-09-09

## Executive Summary

The current analyzer is not safe to treat as an unconditional proof source for runtime types. Two accepted programs with no Dynamic escape return String through Int contracts. Seven confirmed findings are retained below. Existing tests passed before the adversarial probes: 1,138 semantic integration tests, 82 semantic unit tests, and 101 core typing integration tests.

This is a retained investigation checkpoint, stopped at the user's request to conserve the usage window. It is not completion of every area of the supplied prompt. No implementation fixes, commits or release gates were performed. The [original prompt](evidence/original-audit-prompt.md) is preserved. Audited HEAD: `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`. Existing edits in `docs/spec/reflection/Untitled.md` and `examples/type-related.ph` are untouched.

| ID | Severity | Finding |
| --- | --- | --- |
| [001](AUD-SEM-001-incompatible-overrides.md) | Critical | Incompatible overrides break nominal return contracts; runtime prints String. |
| [002](AUD-SEM-002-captured-write-stale-fact.md) | Critical | Invoked closure leaves a stale Int fact; runtime returns String. |
| [003](AUD-SEM-003-applied-type-lowering.md) | High | Applied type-form lowering substitutes None for Box<Int>. |
| [004](AUD-SEM-004-union-cycle-state.md) | Medium | Repeated union obligations falsely trigger cycle detection. |
| [005](AUD-SEM-005-applied-nominal-supertype.md) | Medium | Applied subclasses lose nongeneric ancestor assignability. |
| [006](AUD-SEM-006-unit-normalization.md) | Medium | Empty tuple literal and Unit annotations disagree. |
| [007](AUD-SEM-007-relation-budgets.md) | Medium | Recursive relation work bypasses pair/depth accounting. |

## Semantic Model Reconstructed

The canonical `TypeStore` interns `(TypeData, KindId)`. Its variants are Never, Unit, ClassObject, Nominal, Applied, ExactCase, Union, Tuple, Record, Callable, Family, Parameter, Lambda and SelfType. TypeId is store-local; equal integers from different stores are not comparable semantic identities. Public snapshot references pair a store identity with a type ID. Applied forms validate kinds and flatten partial applications; unions flatten, deduplicate and sort IDs; rows sort labels and reject duplicates. The canonicalization control for repeated/distinct applications, partial saturation, distinct binder owners and reordered fields passed.

`Dynamic` and `Unknown` are **knowledge states**, not ordinary TypeData nodes. Known evidence distinguishes Established from Assumed. Assignability yields explicit proof, refutation, dynamic obligation, blocked, cancelled, budget-exceeded or internal-failure outcomes. Unknown generally blocks; Dynamic produces a boundary obligation, not a subtype proof. Joining reachable knowledge absorbs Unknown before Dynamic. This separation is one of the strongest architectural choices inspected.

Nominal class subtyping follows declaration hierarchy; records and families support structural width, tuples compare labels/components, callables use contravariant parameters and covariant results, and applied arguments use declared variance with invariant default. ExactCase refines an enum and projects to its enclosing enum. Generic parameters use owner plus index, with callable and declaration owners distinguished. Inference variables live in a separate inference session; branch-local existential rigids use separate local representations. Type lambdas have scoped forms. Self carries owner, dispatch side and role, then specializes through receiver environments.

The relevant relationships are:

```text
annotation/type formation -> kinded canonical TypeStore
                                  |
                                  v
                      canonical subtype relation
                                  |
           knowledge assignability + explicit terminal outcomes
                                  |
          inference terms / generic constraints / row constraints
                                  |
             dispatch selection + receiver specialization
                                  |
                 flow knowledge / branch-local rigids
                                  |
              coverage usefulness + productive inhabitation
                                  |
               compiler executable semantic projection
```

This is layered, not one universal compatibility function. Concrete inference delegates to `is_subtype`, which collapses non-proof outcomes to false; row equality is stricter than record subtyping; GADT local proof uses separate rigid equalities; member specialization has its own hierarchy projection. The applied-ancestor defect demonstrates actual divergence between those paths.

Literal unit is a confirmed exception to intended canonicalization: the expression constructs Tuple([]), while Unit annotations resolve to Unit. Reified applied expressions are another confirmed boundary gap: the semantic applied constructor succeeds while executable lowering produces None. Ordinary List storage is a Vec<Value>; native set/push inspected in `primitive/list.rs:112,140` do not themselves enforce generic arguments. A full typed/untyped mutation proof was **not completed**, so no additional generic soundness finding is claimed.

## Core Invariants

1. Canonical constructors must give semantically identical proper types one identity, within a store.
2. Generic and rigid variables must preserve owner/scope and never escape into unrelated published products.
3. Nominal subtyping must justify every dynamically reachable member contract.
4. Current flow facts must be invalidated by writes through captures/aliases.
5. Dynamic and Unknown must not become unconditional proofs.
6. Relation recursion state must be path-local; candidate allocation order must not affect acceptance.
7. Specialized ancestors must agree between assignability and member resolution.
8. Exhaustiveness may be proven only after productive domain/refinement obligations succeed.
9. Budget exhaustion/cancellation must remain distinct from refutation and completion.
10. Lowering must preserve the semantic denotation it consumes.

001, 002, 003, 004, 005, 006 and 007 respectively demonstrate violations of these boundaries; unlisted invariants are not globally certified.

## Critical Findings

- [AUD-SEM-001](AUD-SEM-001-incompatible-overrides.md): incompatible overriding return accepted; runtime returns String through Int.
- [AUD-SEM-002](AUD-SEM-002-captured-write-stale-fact.md): legal union-typed captured write leaves a false established Int return.

## High-Severity Findings

- [AUD-SEM-003](AUD-SEM-003-applied-type-lowering.md): an accepted applied constructor's receiver lowers to None.

## Medium-Severity Findings

- [AUD-SEM-004](AUD-SEM-004-union-cycle-state.md): repeated union obligations leak recursion state. The allocation permutation executed as **bad-first=false, good-first=true**.
- [AUD-SEM-005](AUD-SEM-005-applied-nominal-supertype.md): applied subclass rejected as nongeneric base.
- [AUD-SEM-006](AUD-SEM-006-unit-normalization.md): literal () rejected against both Unit spellings.
- [AUD-SEM-007](AUD-SEM-007-relation-budgets.md): nested pairs bypass pair accounting; depth concern remains source-led.

## Low-Severity Findings

No separate confirmed low-severity defect was promoted. Test and architecture limitations below must not be counted as additional demonstrated bugs.

## Soundness Proof-of-Concepts

The exact source files are [override-return.ph](override-return.ph) and [captured-write.ph](captured-write.ph). Both passed semantic acceptance probes; the freshly Cargo-checked CLI executed them and printed `wrong` and `changed` respectively. Their declared outer return type is Int. Neither uses Dynamic. Their normative rejection assertions fail in the retained final probe, corroborating acceptance.

The closure case needs an `Int | String` contract: assigning String to an inferred Int local was correctly rejected in an earlier control. That failed candidate is not a finding.

## False-Rejection Cases

```phalcom
class Probe {
  @class run() {
    let a: Unit = ()
    let b: () = ()
    let value: (Int | String, Int | String) = (1, 2)
  }
}
```

Each form has an isolated retained source probe. The unit and tuple-union cases emit BindingInitializerMismatch. The generic-inheritance source probe emits GenericInferenceConflict plus ArgumentMismatch when passing Child<Int> to Base.

## Termination and Complexity Risks

The pair-accounting defect is measured, not a demonstrated nontermination or stack overflow. `check_subtype_impl` charges steps but not nested pairs/depth. Type substitution recursively revisits children without a per-query memo in `types/substitution.rs:37`; repeated shared DAG traversal is a performance lead, not a benchmarked defect. Inference clones constraints during fixed-point replay (`checker/inference.rs:1338`) and recursively solves structural terms. Further work should measure growth and thread control into nested work.

Current coverage code **does** have shared control, cancellation, blocked-result propagation, metrics and a productive inhabitation worklist. Historical notes saying these are absent were not reused as findings. `coverage/inhabitation.rs:48` explores canonical TypeIds and projects subjects to canonical form; expanding generic arguments and loss of local-rigid precision still require adversarial growth/inhabitation probes. No new false-exhaustiveness result was established here.

## Architectural Assessment

A useful semantic kernel exists: separate knowledge/evidence states, kinded interning, inference-local variables, owner-aware specialization and local rigid opening. The two executable soundness failures show that these strengths do not yet establish whole-program contracts. Prioritize override admission and captured-cell effect invalidation before compiler optimizations rely on types.

`MemberSurface` stores one signature per Selector (`surface.rs:16,75`), and dispatch returns the first matching owner (`dispatch.rs:295`). This implements selector-based override dispatch. Pattern-sensitive same-selector overloads would need a candidate-set identity and ambiguity/selection proof shared with runtime; simply adding patterns to the current maps would overwrite candidates. This is a future feature constraint, not a claim that current syntax promises pattern overloads.

Class-object identity and applied specialization are deliberately separate in the checker. Compiler type-form handling violates that distinction. Align executable denotation with canonical products rather than reconstructing it from syntax.

## Test-Suite Assessment

Completed existing suites:

| Suite | Result |
| --- | --- |
| semantic integration, before audit registration | 1,138 passed, 42 ignored |
| semantic unit tests | 82 passed |
| core typing integration | 101 passed, 384 filtered |

These are focused/current evidence, not workspace certification. The ignored existing tests were not changed. Existing coverage includes owner-scoped generics, inference occurs checks, record rows, recursive/GADT matches, Self specialization and incremental identity, but passing totals do not prove every interaction.

The new 13-test diagnostic run produced **3 passed, 10 failed**. Nine failures are intended invariant counterexamples covering the findings; one additional failure is an **invalid control fixture**, not a product defect: the extended relation test constructs a synthetic Universe-root `Object`, which is not the actual canonical core Object. It fails at `T <: Object`, so later record/callable/enum matrix assertions in that test did not execute. The seven-row matrix has the same synthetic Object limitation. Do not publish its Object column as the language's top-type relation. Fix that control to use canonical core identity before continuing matrix work.

The initial 9-test run was 1 passed / 8 failed. The added allocation permutation established a ninth invariant failure. Positive controls establish applied static result shape and several interning/binder/row-order properties; the logging-only matrix is not a property proof. Existing fixture parsing asserts parse errors separately, preventing malformed syntax from masquerading as the confirmed semantic failures.

## Proposed Regression Tests

Exact Rust assertions are retained in [semantic_audit_probe.rs](evidence/semantic_audit_probe.rs), with reproduction instructions in the evidence index. They were temporarily registered in the existing semantic test tree and then removed, following the prior runtime audit's evidence workflow. They remain permanent audit artifacts, not newly enabled failing suite tests. No failing assertion was weakened to make a gate pass.

During remediation, integrate valid invariant probes into `tests/semantic/foundations/` and source cases into the relevant capabilities modules. Assert exact binding/current evidence, selected callable and declaration invalidity in addition to diagnostic presence. Add a production-compiler test for applied construction under `phalcom-core/tests/core/typing_integration/`, using the existing full-Universe helper and semantic preflight. Runtime value-type assertions should accompany override and capture tests after static rejection/validation policy is implemented.

Additional required matrix work: canonical Object/Any; Unknown versus Dynamic at each boundary; record and callable variance; exact enum/refined cases; recursive applied arguments; declared generic constraints; row lacks contradictions; shadowed callable versus declaration binders; Self override variance; imported-declaration and label-preserving permutations. Do not assume reordered selector labels are equivalent unless selector semantics establish that transformation.

## Prioritized Remediation Plan

- **P0:** validate override compatibility and invalidate captured-cell current facts at invocation. Require static plus executable regression evidence.
- **P1:** repair applied type-form lowering, Unit canonicalization, relation cycle cleanup and applied ancestor projection.
- **P2:** re-evaluate inference diagnostics after canonical relation fixes; preserve terminal outcomes instead of treating every false bool as contradiction.
- **P3:** enforce real pair/depth accounting; measure expanding recursive generics, coverage state growth and cancellation behavior.
- **P4:** align shared specialization/proof boundaries and define candidate-set authority before pattern-sensitive dispatch.
- **P5:** benchmark substitution DAGs, constraint replay and diagnostic quality only after correctness gates hold.

## Confidence and Remaining Unknowns

High confidence in the seven bounded findings; insufficient confidence in global soundness. The most dangerous demonstrated seams are override admission and post-call flow facts. The strongest inspected area is explicit epistemic/evidence separation with owner-scoped canonical and local representations.

The audit stopped by explicit user request. Exhaustive per-feature A–H tests, the complete relation matrix, systematic generic permutations, recursive-growth stress, every GADT/row escape path, runtime generic mutation/reification, all getter/setter/indexer override cases, multithread determinism and release gates remain unfinished. No universal proof or complete audit is claimed. This report retains exact next work so continuation need not repeat the passing baseline or the confirmed counterexamples.

---
name: static-prover-theory-and-development
description: Use when designing, specifying, implementing, reviewing, testing, or optimizing Phalcom static proving, contract verification, verification-condition generation, symbolic execution, proof IR, SMT/decision-procedure integration, loop or recursion reasoning, heap/frame reasoning, proof diagnostics, proof caching, trusted native summaries, or any feature that must distinguish proven, disproven, and unknown program properties.
compatibility: Designed for Phalcom's Rust compiler/checker/semantic stack and future static-prover integration; assumes shared semantic identities, CFG/IR facts, optional correctness-participating typing, contracts, runtime metadata, diagnostics, modules, and native/FFI boundaries.
---

# Static Prover Theory and Development for Phalcom

## Overview

This skill equips an implementation agent to move from a Phalcom semantic property to a defensible proof result and back to a source-level explanation. Its central subject is not “using an SMT solver”; it is the construction of a sound verification pipeline whose assumptions correspond to actual Phalcom semantics.

A static prover is one semantic consumer among many. It must consume the same semantic identities, control-flow model, type facts, effect summaries, dispatch rules, module revisions, and source provenance that support the compiler, checker, LSP, diagnostics, refactoring, and optimizer. Do not create a prover-only shadow language unless it is an explicitly verified lowering with a documented correspondence to Phalcom.

The essential pipeline is:

```text
Phalcom source/specification
        ↓
shared parse + semantic identities
        ↓
semantic IR / CFG + control outcomes
        ↓
type/effect/alias/call summaries
        ↓
contract lowering + proof IR
        ↓
verification obligations / symbolic states
        ↓
sound simplification + abstract facts
        ↓
decision procedures / SMT backend
        ↓
Proven | Disproven(witness) | Unknown(reason)
        ↓
diagnostics + cache + runtime/checker policy
```

The prover's guarantee is only as strong as the correspondence between every arrow in that diagram.

## Required background and neighboring skills

Load neighboring skills rather than duplicating their intellectual ownership:

- `programming-language-semantics` owns operational/denotational semantic technique and language-level control/evaluation laws.
- `type-theory` owns general type relations, substitution, kinds, polymorphism, variance, recursive types, gradual typing theory, and formal typing machinery.
- `static-analysis-and-abstract-interpretation` owns lattices, monotone frameworks, abstract domains, widening, interprocedural fixed points, and sound approximation.
- `phalcom-semantic-model` owns Phalcom semantic identity, scopes, bindings, shared facts, provenance, and semantic-query architecture.
- `semantic-analysis-theory-and-development` owns reusable CFG/dataflow/call-summary construction rather than prover-specific reimplementation.
- Future `type-checker-development` or equivalent owns the concrete checker algorithm; this skill consumes checker facts and verifies propositions over executions.
- Rust/compiler-engineering skills own general arena, interning, ownership, and performance patterns. This skill specifies prover-specific invariants for those representations.

This skill owns the bridge from those domains into proof obligations, proof trust, solver models, proof diagnostics, and proof-sensitive optimization/runtime policy.

## Current Phalcom boundary

Repository inspection is mandatory before repository-specific work. As of the repository state inspected while this skill was deepened, Phalcom has implemented contract attributes such as `@requires` and `@ensures` as compiler AST-to-AST weaves with runtime checks. That is CURRENT behavior. A full static prover is FUTURE/PLANNED unless repository evidence encountered by the agent says otherwise.

Important current observations to re-check rather than memorize blindly:

- `docs/spec/current/decorators/requires.md` records `@requires` as implemented and woven to an entry predicate check.
- `docs/spec/current/decorators/ensures.md` records `@ensures` as implemented, including `old(...)` hoisting and normal-return rewriting.
- Current runtime-weave purity validation is a conservative syntactic floor, not a proof of semantic purity.
- Current contract metadata plumbing and future prover/checker integration may differ from older design documents.

Never convert these observations into eternal truths. Inspect current specs, ADRs/PDRs, Rust code, and tests before modifying them.

## When to use this skill

Use this skill when the task includes any of the following:

- specifying the meaning of `@requires`, `@ensures`, class invariants, assertions, `old`, frame/effect contracts, or proof-only assumptions;
- deciding whether a property is statically provable, runtime-checked, advisory, or uncheckable;
- designing verification conditions, weakest preconditions, symbolic execution, or proof IR;
- implementing proof handling for loops, recursion, exceptions, non-local return, dynamic sends, reflection, fibers, modules, native code, or FFI;
- mapping Phalcom values/types/heaps to solver sorts and theories;
- integrating abstract interpretation with theorem proving;
- producing counterexamples and proof diagnostics;
- designing proof cache keys, incremental invalidation, solver budgets, or deterministic parallel proving;
- deciding whether a runtime check or optimization may be justified by proof;
- reviewing a change for soundness, vacuity, stale assumptions, or semantic mismatch.

Do not invoke this skill merely because a feature has a type annotation. Many typing questions belong to type-system skills without requiring proof.

## Core doctrine

1. **Proof result is not Boolean.** At minimum represent `Proven`, `Disproven(witness)`, and `Unknown(reason)`. Solver timeout, unsupported theory, dynamic boundary, missing loop invariant, budget exhaustion, unavailable dependency, and untrusted native behavior are different unknown reasons.

2. **Proof is relative to assumptions.** Record which contracts, type facts, closed-world assumptions, native summaries, module revisions, and runtime-mode guarantees were used. A cached `Proven` without dependency provenance is unsafe.

3. **Do not prove a different language.** A source method send can execute user code, mutate heap, throw, trigger non-local control, or yield. Replacing it with an uninterpreted pure mathematical function is sound only under a trusted/verified summary that justifies those properties.

4. **Types are not propositions.** `x : Int` constrains the domain of `x`; `x > 0` constrains a particular state/value. A `ValueShape` or IDE approximation is neither automatically a language type nor proof evidence.

5. **Open-world behavior matters.** Dynamic dispatch, reflective method installation, mutable classes, native callbacks, or package loading can invalidate target assumptions. A proof requiring a closed world must state and version that assumption.

6. **Finite exploration is not universal proof.** Loop unrolling, bounded symbolic execution, testing, fuzzing, and concolic execution can disprove or find witnesses; they prove only bounded properties unless a separate argument closes the remaining cases.

7. **Heap proof requires a frame story.** If a call may mutate state reachable from receiver/arguments/globals, facts about those locations cannot survive unless the callee summary, ownership model, or frame rule preserves them.

8. **Runtime-check elimination is a semantic optimization.** Eliminate checks only from sound proof under assumptions guaranteed by the execution mode. LSP heuristics, successful tests, or solver `unknown` never justify removal.

9. **Diagnostics are part of architecture.** Preserve semantic IDs, source ranges, constraint origins, path decisions, and assumption provenance while generating obligations. Do not reconstruct explanations from opaque solver names after the fact.

10. **SMT is a backend, not the semantic model.** Build a solver-independent logic/proof IR. Choose theories deliberately and keep exact `Int`, IEEE `Float`, object identity, class identity, ADTs, strings, and heap state at the correct semantic layer.

## Workflow for a prover task

### 1. Classify the request and status

Write down whether the requested fact is CURRENT behavior, RATIFIED/NORMATIVE design, PROPOSED, EXPERIMENTAL, FUTURE/PLANNED, or a recommendation. Inspect repository evidence for current claims.

### 2. State the observable property

Examples:

```text
For every normal return from Account#deposit(amount),
if the caller established amount > 0,
then the resulting balance equals the entry balance plus amount.
```

Or:

```text
At this indexed collection access, index is in [0, length).
```

Do not start with “encode this in Z3.”

### 3. Select the correctness model

Distinguish partial from total correctness; normal from exceptional outcomes; safety from liveness; single-thread/cooperative assumptions from concurrent interference. Explicitly define what counts as termination and which abrupt outcomes are in scope.

### 4. Inventory trusted assumptions and boundaries

Identify:

- checker/type relation facts;
- callee contracts and whether verified or trusted;
- native/FFI summaries;
- module initialization guarantees;
- dispatch closure assumptions;
- reflection/mutation assumptions;
- scheduler/yield assumptions;
- solver theory approximation.

If any assumption is heuristic rather than sound, it must not be silently promoted into proof evidence.

### 5. Use shared semantic representations

Prefer stable semantic IDs and shared CFG/IR. The prover may add a proof IR, but it should lower from the semantic representation rather than re-resolve names, dispatch, or control flow itself.

### 6. Define control outcomes explicitly

A useful proof translation has separate continuations/outcomes:

```text
Normal(value, heap)
Return(value, heap)
Throw(error, heap)
NonLocalReturn(home, value, heap)
Break(loop, heap)
Continue(loop, heap)
Suspend(effect, continuation, heap)
```

The exact representation can differ, but implicit “fall-through-only” semantics are insufficient for Phalcom.

### 7. Generate obligations compositionally

Use Hoare/WP rules, symbolic transfer, or a verified combination. Every lowering rule must have an invariant explaining why it preserves the source property. Calls consume summaries; loops consume invariants; writes update heap versions; assertions generate obligations; assumptions constrain execution.

### 8. Simplify soundly before expensive solving

Use constants, checker facts, sound abstract interpretation, interval/presence facts, dead-path elimination, and normalized SSA equations. Keep proof-strengthening facts separate from heuristic IDE facts.

### 9. Choose a solver fragment consciously

Map each language domain to an appropriate theory. Exact language-level integers naturally map to mathematical integer arithmetic; runtime representation verification may instead need bit-vectors/bignums. IEEE floating-point claims need floating-point semantics. Strings/Unicode require care. Quantifiers and nonlinear arithmetic need explicit budget/unknown policy.

### 10. Interpret results conservatively

For obligation `VC`, common SMT use asks about `¬VC`:

```text
unsat    => Proven, assuming encoding and solver are trusted
sat      => Disproven only if the model corresponds to a valid Phalcom state
unknown  => Unknown(reason)
error    => Unknown/backend failure, never compiler success
```

### 11. Reconstruct user-facing evidence

Map logical variables back to semantic entities and source spans. Explain the branch path, contract source, and value witness. Hide heap-version plumbing unless it helps diagnosis.

### 12. Record dependencies and cache safely

A proof key needs more than the caller body hash. Include relevant contract/type/effect/native/module/trust/solver-model revisions. Invalidation must follow semantic dependencies.

### 13. Verify with adversarial tests

Add tests designed to break tempting unsound shortcuts: missing frame summary, reflective dispatch mutation, solver timeout, vacuous precondition, fake float-as-real proof, insufficient loop invariant, stale callee contract, non-local return through a block, suspension across shared state, and malformed source that must not produce a false proof.

## Quick-reference mental models

### Contracts

```text
callee.requires   -> obligation at call site
callee.ensures    -> assumption after verified/trusted normal return
callee.effects    -> heap locations that may change
callee.throws     -> exceptional successor summary
```

When verifying a method itself:

```text
assume its requires
DO NOT assume its own ensures for the current invocation
prove ensures on each relevant normal exit
prove invariant/frame/effect obligations according to policy
```

Recursive calls may use the candidate recursive contract as the induction hypothesis, but that does not mean the current invocation may assume its postcondition at entry.

### Proof status

```text
Proven(evidence, assumptions)
Disproven(witness, path)
Unknown(reason, residual_obligations)
```

Typical reasons include:

```text
UnsupportedTheory
SolverTimeout
BudgetExceeded
DynamicBoundary
OpenWorldDispatch
MissingLoopInvariant
InsufficientSummary
UntrustedNative
MissingDependency
AnalysisIncomplete
MalformedSourceBoundary
```

### Heap

```text
H : (ObjectId, FieldId) -> Value
read  = select(H, o, f)
write = H' = store(H, o, f, v)
```

A frame summary constrains which `(o,f)` locations may differ between entry and exit. Collection contents and native buffers are heap state even when Rust stores them outside a generic object-field table.

### Loop

```text
initiation:     Pre => I
preservation:   I ∧ guard ∧ body => I'
exit:           I ∧ ¬guard => Post
```

For total correctness, also prove a well-founded decreasing variant.

### Dynamic dispatch

```text
receiver fact -> possible targets -> contract guaranteed by all possible targets
```

Do not select one “likely” implementation for proof. A protocol/interface contract or sealed target set can supply a universal summary; otherwise return `Unknown(OpenWorldDispatch)` or retain a runtime check.

### Incremental proof

```text
proof result validity = body semantics
                      + contracts
                      + type relations
                      + effects/dispatch closure
                      + native specs
                      + module init/import facts
                      + solver/encoding version
                      + trust policy
```

A source-position-only cache key is not sufficient.

## Common failure modes

- Treating current runtime contract weaving as if it were already formal proof.
- Re-parsing or re-resolving semantics in the prover instead of consuming shared semantic facts.
- Generating solver calls directly from AST and losing semantic/source provenance.
- Treating method sends in contract predicates as pure mathematical expressions because they “look expression-like.”
- Forgetting exceptional/non-local/suspension exits when proving `@ensures` or invariants.
- Preserving heap facts across unknown calls or FFI.
- Assuming a dynamically dispatched target from an LSP guess.
- Unrolling loops/recursion to a fixed depth and reporting `Proven`.
- Modeling `Float` as mathematical real for exact runtime claims.
- Collapsing timeout, ambiguity, unsupported theory, dynamic boundary, and missing invariant into one undifferentiated `Unknown`.
- Reporting raw solver models that violate Phalcom runtime/class invariants.
- Caching proof solely by source text/body hash.
- Letting third-party packages forge “trusted native” summaries.
- Removing runtime checks based on type inference that is advisory or analysis-budget-limited.
- Failing to diagnose vacuous proofs caused by unsatisfiable preconditions.

## Reference map

Load only the references needed for the task, but load the intellectual engine before changing prover architecture.

- [proof-results-and-trust.md](references/proof-results-and-trust.md) — proof-status algebra, evidence, assumptions, TCB, vacuity, trust policies.
- [hoare-logic-and-contracts.md](references/hoare-logic-and-contracts.md) — Hoare reasoning, contracts, object invariants, modular call rules.
- [weakest-preconditions-and-vc-generation.md](references/weakest-preconditions-and-vc-generation.md) — compositional WP/VC algorithms, SSA, continuation/outcome handling, implementation invariants.
- [proof-ir-and-control-outcomes.md](references/proof-ir-and-control-outcomes.md) — solver-independent proof IR, control outcomes, lowering boundaries, Rust representations.
- [symbolic-execution-and-path-conditions.md](references/symbolic-execution-and-path-conditions.md) — symbolic state, branching/merging, feasibility, bounded versus proof-grade execution.
- [smt-sat-and-decision-procedures.md](references/smt-sat-and-decision-procedures.md) — theory selection, decidable fragments, exact-vs-approximate modeling.
- [solver-encoding-and-modeling.md](references/solver-encoding-and-modeling.md) — sorts, heap/object models, canonicalization, solver interfaces, model validity.
- [loops-and-invariants.md](references/loops-and-invariants.md) — inductive invariants, havoc/assume encoding, break/continue, invariant inference, termination.
- [recursion-and-procedure-summaries.md](references/recursion-and-procedure-summaries.md) — modular recursion, SCCs, contracts, termination and recursive data.
- [heap-aliasing-and-frame-conditions.md](references/heap-aliasing-and-frame-conditions.md) — heap SSA, aliasing, framing, old-state snapshots, collections, ownership/separation boundaries.
- [exceptions-nonlocal-control-and-effects.md](references/exceptions-nonlocal-control-and-effects.md) — abrupt control, cleanup, callbacks, effect summaries.
- [dynamic-dispatch-reflection-and-open-world-proof.md](references/dynamic-dispatch-reflection-and-open-world-proof.md) — Phalcom dispatch, reflection, class mutation, protocol contracts, closed/open-world proof.
- [typing-gradual-boundaries-and-proofs.md](references/typing-gradual-boundaries-and-proofs.md) — checker facts, refinements, Dynamic/unknown distinctions, runtime type validation.
- [modules-native-and-ffi-trust.md](references/modules-native-and-ffi-trust.md) — module initialization, native contracts, FFI effects, trust tiers and versioning.
- [fibers-concurrency-and-proof-boundaries.md](references/fibers-concurrency-and-proof-boundaries.md) — suspension/interference, cooperative scheduling assumptions, future parallelism.
- [counterexamples-and-diagnostics.md](references/counterexamples-and-diagnostics.md) — model reconstruction, path explanations, unsat cores, diagnostic stability.
- [prover-architecture-and-incrementality.md](references/prover-architecture-and-incrementality.md) — query architecture, dependencies, snapshots, cancellation/budgets, proof caching.
- [proof-engineering-in-rust.md](references/proof-engineering-in-rust.md) — Rust IDs/arenas/terms, solver isolation, memory/performance, deterministic concurrency.
- [testing-and-soundness.md](references/testing-and-soundness.md) — layered tests, negative/mutation/differential testing, fuzzing, incremental equivalence.
- [phalcom-current-contracts-and-proof-integration.md](references/phalcom-current-contracts-and-proof-integration.md) — repository-grounded boundary between CURRENT runtime contract weaving and FUTURE static proof integration.
- [phalcom-prover-roadmap.md](references/phalcom-prover-roadmap.md) — staged capability plan and prerequisites.
- [comparative-provers-and-reading.md](references/comparative-provers-and-reading.md) — precedent matrix and what to study without cargo-culting.
- [review-and-validation-scenarios.md](references/review-and-validation-scenarios.md) — pressure tests for agent/reviewer competence.

## Verification and review expectations

Before claiming a prover change correct, require evidence at the right levels:

1. The source semantic rule is identified and current/normative status is clear.
2. The proof lowering rule is stated or derivable and handles all relevant control outcomes.
3. The rule's assumptions are explicit and versioned where mutable.
4. Solver theory matches source semantics or any approximation is prohibited from producing `Proven` for affected claims.
5. Negative tests fail if a soundness-critical premise is removed.
6. `Unknown` pathways are tested, not only success/failure.
7. Counterexamples are validated against source/runtime domain constraints.
8. Incremental result after edits equals clean full recomputation.
9. Trusted native contracts have executable conformance tests or another justified validation mechanism.
10. Runtime-check removal and optimizer uses are tested under the exact execution mode whose assumptions were proved.

A proof feature is incomplete if it only “gets unsat from the solver.” The implementation must establish that the generated formula means the intended Phalcom property, that its assumptions are justified, and that the result remains valid as the program and semantic environment evolve.

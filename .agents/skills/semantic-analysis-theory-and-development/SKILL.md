---
name: semantic-analysis-theory-and-development
description: Use when designing, implementing, reviewing, debugging, or evolving Phalcom semantic analysis: semantic identities, binding and scopes, source surfaces, semantic lowering/HIR, CFG and dataflow, abstract interpretation, runtime-shape facts, dispatch approximation, interprocedural summaries, modules, incrementality, snapshots, provenance, recovery, and the shared semantic query layer consumed by the compiler, LSP, checker, lints, refactoring, optimizer, and future prover.
compatibility: Phalcom Rust compiler/runtime/LSP architecture; inspect the current repository before making implementation-specific claims.
---

# Semantic Analysis Theory and Development

Semantic analysis is the layer that turns syntax into durable program meaning without pretending that static knowledge is the runtime itself. In Phalcom, this skill owns the architecture and algorithms for identities, resolution, normalized semantic representations, flow facts, summaries, dependency tracking, snapshots, and protocol-neutral queries. Its central obligation is to preserve **one coherent semantic truth with multiple abstractions for multiple consumers**.

## When to use this skill

Load this skill before work that changes or depends on any of the following:

- declaration identity, lexical binding, scopes, shadowing, definition/use relations, or semantic occurrences;
- source-surface extraction, semantic lowering, HIR, program points, CFGs, control-flow joins, or dataflow;
- `ValueShape`-like runtime-shape inference, constant/refinement facts, field/parameter evidence, callable summaries, or effect summaries;
- static approximation of Phalcom message dispatch, inheritance, class/metaclass side, visibility, `super`, method families, reflection-sensitive lookup, or dynamic sends;
- module/import graphs, cross-file identity, initialization-aware analysis, dependency edges, or package/project analysis;
- incremental invalidation, generation-coherent snapshots, cancellation, memoized semantic queries, or editor latency;
- checker, typed-runner, prover, optimizer, linter, refactoring, documentation, or LSP features that need semantic facts;
- a proposal to introduce a new semantic IR or to move semantic logic out of an LSP handler/compiler pass;
- semantic correctness reviews, unsound shortcuts, stale caches, imprecise diagnostics, or analysis performance regressions.

## Boundaries and neighboring skills

This skill is deliberately not a catch-all compiler textbook.

- `programming-language-semantics` owns formal definitions of Phalcom's dynamic language meaning. This skill must model those rules; it must not invent replacements for them.
- `parser-development` owns grammar, precedence, parser machinery, and recovery algorithms. This skill owns the semantic contract over parsed/recovered structures and the source-identity consequences of recovery.
- `type-theory` owns general mathematical type machinery. `phalcom-typed-language` owns normative Phalcom typing semantics. `type-checker-development` owns checker-specific constraint solving and diagnostics. This skill owns shared bindings/CFG/facts/provenance and explicit bridges into those domains.
- `static-analysis-and-abstract-interpretation` may provide deeper general theory. This skill applies lattices, transfer functions, widening, summaries, and soundness specifically to Phalcom's semantic engine.
- `static-prover-development` owns verification-condition generation, solver encodings, heap logics, and proof policy. This skill owns the semantic/effect/refinement facts that a prover is allowed to trust.
- `lsp-development` owns LSP protocol behavior and UX. This skill owns the semantic query substrate; handlers should adapt queries rather than implement independent inference.
- `rust-compiler-engineering` owns broad Rust implementation practice. This skill focuses on semantic IDs, arenas, immutable generations, dependency storage, cancellation, and query/invalidation structures.
- compiler/VM/object-model/dispatch skills own executable runtime behavior. Semantic analysis must be checked against them.

When a task crosses a boundary, load the neighboring skill instead of copying its theory here.

## Core doctrine

### One truth, several domains

Never collapse these categories:

```text
dynamic Phalcom semantics
    != runtime object/value representation
    != semantic-analysis approximation
    != Phalcom language type
    != proof fact
    != optimization fact
```

A useful bridge is explicit and directional. For example, a runtime-shape approximation may help synthesize a candidate receiver set for completion; it does not thereby become a language type. A proved type fact may authorize an optimization only if reflection/mutation assumptions required by that optimization are also proved or guarded.

### Status every repository-specific statement

Before editing architecture or writing a specification, classify claims as:

- **CURRENT** — observed in the inspected repository revision;
- **RATIFIED / NORMATIVE** — established by a governing specification or decision;
- **PROPOSED** — concrete design under consideration;
- **EXPERIMENTAL** — implemented or specified without compatibility commitment;
- **FUTURE / PLANNED** — desired direction without current semantics;
- **RECOMMENDATION** — advice from the agent, not Phalcom doctrine.

Do not upgrade a draft architecture analysis into normative semantics. Re-inspect source before asserting CURRENT behavior.

### Current repository anchor

At the repository state inspected for this skill (main around commit `61dae3400ba810d8f709725974e3c51838762905`, 2026-08-15), **CURRENT** `phalcom-lsp/src/semantic/` contains a VM-free semantic database with module-qualified `ModuleId`, `ClassId`, `CallableId`, `FieldId`; lexical `BindingId`/scope machinery; source surfaces and occurrences; `ValueShape` and `InferredValue`; structured statement flow; callable summaries; module/dependency graphs; contribution-indexed parameter evidence; a mutable worker engine; and immutable published snapshots. `ValueShape` is explicitly documented as advisory runtime value knowledge, deliberately not a language type. The engine uses generations, candidate-state publication, cancellation checks, dependency frontiers, and copy-on-write `Arc` state. Treat these as strong existing infrastructure, not as a finished formal type system or proof engine.

The draft `docs/spec/typing/optional-typing-architecture-analysis.md` explicitly labels itself a draft architecture analysis rather than an implementation commitment. It also documents that the current compiler remains dynamic-first and that future formal typing should be a separate domain sharing semantic infrastructure. Preserve that status distinction.

## Operational workflow

For any semantic-analysis task, execute this sequence before touching Rust:

1. **State the semantic question.** What observable language distinction must the analysis represent? Separate dynamic semantics from the static approximation being requested.
2. **Inspect authority.** Read current code, tests, specifications, and decision records. Record CURRENT versus desired behavior.
3. **Identify entities and identity.** Decide which constructs have semantic identity and its lifetime. Never use source text or byte offsets as durable identity without proving that is the desired identity relation.
4. **Choose representation.** Decide whether source AST + side tables is sufficient, whether a normalized HIR is justified, or whether explicit CFG/program points are required.
5. **Define the domain.** Give the abstract state, order, join/meet where relevant, bottom/top/unknown meanings, precision bound, and provenance model.
6. **Define transfer and queries.** State how each relevant operation transforms facts and what protocol-neutral queries consumers receive.
7. **Handle control and calls.** Specify abrupt completion, loops, closures/non-local returns, dispatch candidate sets, effects, recursion, SCCs, and dynamic boundaries.
8. **Define dependencies.** For every derived fact/cache, name its key, inputs, dependency set, validity condition, invalidation events, revision/generation semantics, concurrency policy, and memory bound.
9. **Design incomplete-source behavior.** Separate parser recovery artifacts, unresolved dependencies, actual semantic errors, ambiguity, budget exhaustion, and unreachable code.
10. **Preserve explanation.** Keep provenance/evidence sufficient for future diagnostics; do not flatten facts and plan to reconstruct reasons later.
11. **Test semantic distinctions first.** Add unit, negative, fixed-point, incremental/full-equivalence, recovery, cross-module, and runtime-correspondence tests as appropriate.
12. **Measure.** Check changed-frontier size, allocations, cloning, hash/intern costs, worklist rounds, query fan-out, lock time, snapshot publication, and editor latency.

## Quick-reference mental models

### Identity

```text
source location answers: where is it now?
semantic identity answers: which declaration/entity is this?
revision identity answers: in which source generation is this fact valid?
runtime identity answers: which object exists during execution?
```

These relations can coincide for simple declarations and diverge under edits, imports, generated/core declarations, specialization, reflection, or runtime mutation.

### Dataflow

For a forward analysis over basic block `B`:

```text
IN[B]  = join(OUT[P] for P in predecessors(B))
OUT[B] = transfer_B(IN[B])
```

The domain must have a partial order and finite-height bound or an intentional widening strategy. Transfer must be monotone for standard worklist convergence arguments. Re-processing stops only when abstract states stabilize—not merely after an arbitrary number of rounds.

### Abstract interpretation

```text
Concrete states C  --alpha-->  Abstract states A
       ^                           |
       |--------- gamma ----------|

soundness goal: concrete behavior represented by input a
                remains represented after abstract transfer F#(a)
```

An advisory editor heuristic may intentionally lack this soundness guarantee, but then it must be labeled and prevented from justifying checker rejection, proof, or unsafe optimization.

### Interprocedural solving

```text
callable body -> summary
summary dependencies -> call graph edges
recursive region -> SCC
SCC -> iterate summaries to a fixed point / intentional widening
changed summary -> invalidate semantic dependents, not the whole workspace
```

A summary is an abstraction boundary, not a cached AST traversal result. State its semantic inputs and whether it is context-insensitive, receiver-sensitive, argument-shape-sensitive, effect-sensitive, or otherwise parameterized.

### Incrementality

```text
source revision
  -> recovered parse/source surface
  -> identities/scopes
  -> module + declaration dependencies
  -> local facts/summaries
  -> immutable published semantic generation
  -> cheap read-only consumer queries
```

A stale result that looks plausible is incorrect. Publication must never mix facts derived from incompatible generations.

## Non-negotiable invariants

- Bind before infer whenever name identity affects fact ownership.
- Selector identity is defined by Phalcom semantics, not by types guessed by analysis.
- `super` changes lookup start/dispatch semantics; it does not mean “receiver has superclass type.”
- Runtime class membership, language subtyping, assignability, and analysis receiver candidates are different relations.
- `Unknown`, `Dynamic`, bottom/unreachable, ambiguity, blocked analysis, inconsistency, and budget exhaustion must remain distinguishable when the distinction affects downstream behavior.
- Dynamic reflection/method mutation can invalidate dispatch facts and optimization assumptions even when lexical source did not change.
- Constructing a block is not equivalent to executing it. Captures, non-local returns, effects, and escape/suspension must be represented deliberately.
- Module discovery/resolution and runtime module initialization are related but not identical analyses.
- In an editor, malformed source may yield partial facts; recovery must not manufacture normative semantics for an invalid complete program.
- Native/core/FFI behavior requires explicit semantic contracts or conservative boundaries; Rust implementation code is not automatically visible to the analyzer.
- Every optimization-strength fact must identify the assumptions under which it remains valid.

## Common failure modes

Reject implementations that do any of the following without an explicit, justified policy:

- infer a “type” by mapping AST nodes to class-name strings;
- use byte offsets as long-lived binding/class/callable identity;
- let each hover/completion/refactoring handler re-run local inference;
- make a cache whose invalidation condition is “when something relevant changes”;
- collapse all hard cases to `Unknown`, erasing whether the cause was dynamic semantics, missing dependencies, ambiguity, inconsistency, or budget exhaustion;
- truncate a growing union silently and then use the widened result as proof of absence;
- iterate recursive summaries N times and call the result a fixed point without a convergence/widening argument;
- assume a dynamic send has no effects because the target is unresolved;
- execute a literal closure during analysis merely because the closure is syntactically visible;
- duplicate dispatch rules between compiler, LSP, checker, and optimizer;
- publish mutable semantic storage behind long-lived borrowed references;
- rebuild all modules after a body-only edit when dependency ownership can identify a smaller frontier;
- retain a derived fact without provenance/dependency metadata when diagnostics or invalidation need it;
- treat successful tests, finite symbolic unrolling, or solver timeout as proof.

## Reference map

Load references selectively. The first group is the intellectual engine; the second group covers bridges and engineering.

| Question | Reference |
|---|---|
| What belongs in semantic analysis and how should phases compose? | [semantic-analysis-role-and-pipeline.md](references/semantic-analysis-role-and-pipeline.md) |
| What exactly is a semantic identity and how should it be stored? | [semantic-identities-arenas-and-interning.md](references/semantic-identities-arenas-and-interning.md) |
| How do scopes, shadowing, imports, captures, and definition/use resolution work? | [binding-and-name-resolution.md](references/binding-and-name-resolution.md) |
| When is HIR justified and what must lowering preserve? | [hir-and-semantic-lowering.md](references/hir-and-semantic-lowering.md) |
| How do CFG, lattices, transfer functions, fixed points, abrupt control and loops work? | [control-flow-and-semantic-ir.md](references/control-flow-and-semantic-ir.md) |
| How should abstract domains, flow facts, calls, summaries, recursion and SCCs work? | [flow-and-interprocedural-semantics.md](references/flow-and-interprocedural-semantics.md) |
| How are uncertainty, confidence, provenance and explanatory evidence represented? | [facts-provenance-and-uncertainty.md](references/facts-provenance-and-uncertainty.md) |
| How should semantic analysis approximate Phalcom dispatch without changing it? | [dynamic-dispatch-and-member-resolution.md](references/dynamic-dispatch-and-member-resolution.md) |
| How do modules/imports/packages/cycles and initialization interact with semantics? | [modules-packages-and-project-semantics.md](references/modules-packages-and-project-semantics.md) |
| How do generations, dependency frontiers, cancellation, snapshots and caches stay coherent? | [incrementality-snapshots-and-queries.md](references/incrementality-snapshots-and-queries.md) |
| How should analysis survive malformed/incomplete editor source? | [recovery-and-incomplete-programs.md](references/recovery-and-incomplete-programs.md) |
| When should inherited/synthesized attributes or query systems be used? | [attribute-grammars-and-query-systems.md](references/attribute-grammars-and-query-systems.md) |
| How does semantic infrastructure feed the future checker without becoming the type system? | [type-checker-integration.md](references/type-checker-integration.md) |
| How does it feed effects/refinement/proving without claiming proof? | [prover-effects-and-refinement-integration.md](references/prover-effects-and-refinement-integration.md) |
| How should compiler and runtime behavior constrain static facts? | [compiler-runtime-conformance.md](references/compiler-runtime-conformance.md) |
| How should LSP/lints/refactoring consume the shared model? | [lsp-lints-and-refactoring-consumers.md](references/lsp-lints-and-refactoring-consumers.md) |
| Which Rust representations are safe and efficient for this architecture? | [rust-implementation-patterns.md](references/rust-implementation-patterns.md) |
| How should Phalcom migrate from current LSP semantics to shared infrastructure? | [architecture-migration-strategy.md](references/architecture-migration-strategy.md) |
| How should correctness, fuzzing, incrementality and performance be tested? | [testing-performance-and-observability.md](references/testing-performance-and-observability.md) |
| What can Phalcom learn from rust-analyzer, Roslyn, TypeScript, Julia, etc.? | [comparative-semantic-architectures.md](references/comparative-semantic-architectures.md) |
| How do we pressure-test an agent or a design review? | [review-and-validation-scenarios.md](references/review-and-validation-scenarios.md) |

## Verification expectations

Before claiming a semantic change complete, verify at least four dimensions:

1. **Local semantic correctness:** the changed resolver/domain/transfer/query behaves on positive and negative cases.
2. **Cross-layer correspondence:** the fact agrees with Phalcom runtime/compiler semantics where it claims to model them.
3. **Incremental equivalence:** applying edits incrementally reaches the same published facts as a clean full rebuild for the same final source.
4. **Consumer coherence:** LSP/checker/lint/refactoring consumers observe the same semantic identities/facts and do not reconstruct competing truth.

For performance-sensitive changes, also record a baseline and verify that any reduced work preserves the same semantic result. A smaller invalidation frontier is only an optimization if it is complete.

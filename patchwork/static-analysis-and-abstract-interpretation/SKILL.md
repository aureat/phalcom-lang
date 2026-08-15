---
name: static-analysis-and-abstract-interpretation
description: Use when designing, reviewing, implementing, debugging, or optimizing Phalcom control/data-flow analyses, abstract domains, fixed-point solvers, flow-sensitive refinement, reachability, callable summaries, effects, heap/alias approximations, semantic lints, checker-support facts, optimizer preconditions, incremental invalidation, or any analysis that must make explicit soundness, precision, termination, provenance, and cost trade-offs.
compatibility: Designed for Phalcom semantic-engine, checker, typed-runner, static-prover, lint, optimizer, compiler, and LSP work implemented primarily in Rust.
---

# Static Analysis and Abstract Interpretation for Phalcom

## Overview

This skill owns the engineering and mathematical discipline for approximating Phalcom program behavior without enumerating every concrete execution. Its purpose is not to turn every semantic question into abstract interpretation. Its purpose is to make analysis work explicit: what concrete behavior is being approximated, what information the analysis retains, which order means “more precise,” how branch and loop information merges, why the solver terminates, what dynamic behavior can invalidate a fact, how facts are incrementally maintained, and which consumers are allowed to trust them.

Phalcom has one dynamic language semantics and many semantic consumers. Static analysis must describe that semantics conservatively or intentionally heuristically; it must not silently create a second language. A runtime class observation, an LSP `ValueShape`, a declared language type, a path predicate, an effect summary, a proof fact, and an optimizer guard are different semantic objects even when bridges exist between them.

The central operating rule is:

```text
concrete semantics
      ↓ abstraction / evidence extraction
abstract semantic facts
      ↓ explicit bridges and trust policy
checker | LSP | lint | prover | optimizer | diagnostics
```

Never replace this with:

```text
AST node -> guessed type/class -> everybody trusts it
```

## When to use this skill

Use this skill when a task involves one or more of the following:

- local or global flow analysis;
- reachability, definite assignment, liveness, constant propagation, presence/tag refinement, or range reasoning;
- abstract domains such as runtime shapes, constants, intervals, effects, points-to sets, or path facts;
- loop or recursive convergence;
- interprocedural callable summaries and call-graph propagation;
- higher-order blocks, captured writes, non-local returns, fibers, yields, or unknown/native effects;
- reflection, dynamic sends, dynamic selector families, method mutation, or `doesNotUnderstand`-style uncertainty;
- alias, escape, field, or heap reasoning;
- static-analysis facts used by the future checker, typed runner, prover, optimizer, lints, or LSP;
- incremental analysis, invalidation, snapshots, cancellation, dependency tracking, or editor latency;
- review of an analysis implementation that appears precise but may be unsound or non-terminating;
- performance work where reducing semantic recomputation matters.

Do not reach for this skill merely because a feature has types or syntax. If the main problem is parser grammar, formal language typing, proof obligation generation, VM dispatch implementation, or LSP presentation, use the neighboring skill that owns that domain and load this skill only for the analysis component.

## Intellectual ownership and neighboring skills

This skill owns abstract semantic approximation and solver mechanics. Boundaries matter:

- `programming-language-semantics` should own Phalcom's concrete dynamic semantics and operational rules. This skill assumes those rules or derives an analysis contract from them; it does not redefine them.
- `phalcom-semantic-model` should own semantic identities, declaration surfaces, scope/name-resolution facts, selector identity, and shared semantic representation. This skill consumes stable IDs and semantic surfaces.
- `type-theory` should own general mathematical type relations, kinds, substitution, variance, polymorphism, and subtyping theory. This skill may use a type domain but does not redefine the formal type system.
- `phalcom-typed-language` should own concrete Phalcom typing semantics: which annotations mean what, how `Dynamic` behaves, assignability, generic semantics, and type reflection.
- `type-checker-development` should own checker architecture, constraint generation/solving, diagnostic policy, and enforcement. It may consume flow refinements and effects from this skill.
- `static-prover-development` should own Hoare logic, VC generation, symbolic execution, SMT encoding, invariants, and proof trust. Abstract interpretation may provide invariants but is not itself a proof result unless the proof contract says so.
- `semantic-analysis-development` should own repository integration, APIs, query surfaces, and concrete implementation boundaries. This skill supplies the theory and analysis-engine design constraints.
- `lsp-development` should own request handling, incomplete-source UX, cancellation protocol, and rendering of semantic facts. LSP handlers should query facts rather than implement independent inference.
- `rust-compiler-engineering` should own general Rust architecture, ownership patterns, arenas, interners, and performance idioms. This skill states analysis-specific requirements for those structures.
- optimizer/VM skills should own transformations and runtime execution. Optimizers may consume only facts whose trust contract is strong enough for the transformation.

When a task crosses these boundaries, keep one semantic truth and define explicit bridges rather than duplicating models.

## Status discipline: CURRENT is not PROPOSED

Every repository-specific analysis must label claims with one of these statuses:

```text
CURRENT       observed in current repository source/tests
RATIFIED      normative decision/spec explicitly accepted by project
PROPOSED      written design/spec not yet normative or implemented
EXPERIMENTAL  implementation or design intentionally provisional
FUTURE        stated direction without complete semantics/implementation
RECOMMENDATION analysis you are proposing now
```

Before changing code, inspect the current repository. Do not rely on the repository snapshot described in these references as permanent truth.

At the inspection baseline used to deepen this skill—`aureat/phalcom-lang` `main`, commit `b5477b74dfa6f79a4b4487896a1d63699d98685e`, 2026-08-14—the LSP semantic engine already contains substantial advisory static-analysis machinery: `ValueShape`, confidence/provenance, structured flow, bounded loop iteration/widening, callable summaries and worklists, contribution-indexed parameter facts, dependency-directed invalidation, cooperative cancellation, and immutable publication snapshots. The optional-typing architecture document inspected on 2026-08-13 explicitly identifies itself as draft architecture analysis and says `ValueShape` is not the formal type system. Treat those facts as a dated orientation point, not a permanent contract. See [Phalcom analysis domains](references/phalcom-analysis-domains.md).

## Required analysis contract

Before implementing or reviewing an analysis, write the contract. If you cannot fill this in, the design is not ready:

```text
Analysis name:
Concrete question/property:
Consumer(s):
Program points / identities indexed:
May or must:
Sound, conditionally sound, or advisory:
Concrete state components modeled:
Abstract domain A:
Precision/order relation ⊑:
Bottom ⊥:
Top ⊤ or uncertainty states:
Join ⊔ and meet ⊓ if needed:
Transfer function(s):
Edge refinements:
Call/dispatch model:
Heap/alias model:
Unknown/reflection/FFI/fiber policy:
Loop/recursion fixed-point strategy:
Widening/narrowing policy:
Budget and conservative fallback:
Provenance retained:
Dependencies / invalidation keys:
Publication/snapshot consistency:
Performance target:
Tests and metamorphic properties:
```

The contract distinguishes semantic design from implementation accident. A `BTreeMap<BindingId, InferredValue>` may implement part of an abstract state; it is not the definition of the domain.

## Core doctrine

### 1. Define the concrete question first

“Make inference smarter” is not a semantic question. “For every reachable program point, over-approximate the runtime classes that the lexical binding may contain” is. “Determine whether every reachable path initializes field `_x` before the first read” is a different must-analysis. “Suggest likely members while the user is typing” may be advisory and allowed to use heuristics.

The intended consumer changes the correctness contract. A completion hint can tolerate a false positive. An optimizer eliminating a dynamic send cannot use “probably String.” A checker may allow explicit `Dynamic` but must not treat analysis timeout as `Dynamic` unless the language definition says that timeout changes program meaning—which it should not.

### 2. Keep bottom, top, dynamic, ambiguity, and failure distinct

Do not collapse all missing precision into `Unknown`. At minimum, reason separately about:

```text
⊥ / unreachable          no concrete executions reach here
exact                     exact fact under modeled semantics
sound approximation       all concrete possibilities covered
bounded union             several known possibilities, still sound if complete
Dynamic                   language-level permission/escape, if defined
not inferred yet          computation has not produced an answer
blocked/missing dependency analysis unavailable for external reason
ambiguous                 multiple semantic interpretations/targets remain
inconsistent              constraints/facts contradict
budget exhausted          solver stopped before desired precision/convergence
unsupported               analysis intentionally does not model construct
⊤ / any possible value    maximum uncertainty within this abstract domain
```

These states can share representations only if a separate reason/tag preserves their semantic distinction for consumers and diagnostics.

### 3. Join represents control-flow alternatives, not traversal order

For forward may-analysis:

```text
IN[B]  = ⊔ { OUT[P] | P ∈ pred(B) }
OUT[B] = F_B(IN[B])
```

A branch result must not depend on which AST arm the visitor happened to traverse last. For must-properties, the algebra is different: facts surviving a join usually need to hold on every reachable predecessor. Learn the domain, not a memorized “union everything” rule.

### 4. Loops and recursion are equations

A loop header state is generally a fixed point:

```text
H = Entry ⊔ BackEdge(H)
```

Recursive summaries similarly satisfy mutually recursive equations. One pass is not a fixed point. A raw iteration cap is only safe when the fallback is explicitly conservative for the analysis contract. Infinite-height domains need widening or another convergence argument.

### 5. Transfer functions must model abrupt and user-code effects

A transfer function is not merely `State -> State`. Phalcom operations can return, throw, break, continue, non-locally return from blocks, invoke user code, yield, mutate fields, perform dynamic dispatch, or cross native boundaries. Use a result structure that can carry the relevant successor classes/events:

```text
TransferResult {
    normal,
    returns,
    throws,
    breaks,
    continues,
    nonlocal_returns,
    effects,
}
```

Only model categories the analysis needs, but never erase an effect that matters to the claimed property.

### 6. Dynamic behavior requires conservative boundaries

When target lookup, selector shape, reflection, native mutation, or shared-state interference is unknown, decide what concrete behavior remains possible. Then `havoc` only the affected abstract components. “Unknown call means no effect” is unsound for any analysis whose facts can be invalidated by the call.

### 7. Interprocedural reasoning uses summaries and dependencies

Avoid recursive AST descent into callees. Define summaries, dependency edges, and a worklist/SCC strategy. A summary is an abstraction of externally relevant callable behavior, not a cached copy of every internal fact. Separate summary semantic equality from provenance/publication revision; otherwise edits can cause needless propagation even when semantics are unchanged.

### 8. Precision is budgeted, not worshipped

Sensitivity dimensions multiply cost:

```text
path × context × heap × field × flow × relational precision
```

Choose the weakest abstraction that answers the concrete question. Add path partitions, context sensitivity, points-to precision, reduced products, or SMT only when evidence demonstrates a precision failure worth the cost.

### 9. Incrementality is part of semantic correctness

A cached fact is correct only while all semantic dependencies remain valid. Every cache needs:

```text
key
value
semantic dependencies
validity condition
invalidation event(s)
publication generation / coherence rule
concurrency or cancellation policy
memory bound / eviction policy where applicable
```

A stale fact that “looks plausible” is wrong. Compare incremental results against clean full recomputation.

### 10. Provenance is a first-class analysis product

If a future checker or LSP must explain why a value was inferred, retain causal structure early enough to support the explanation. Avoid storing only the final joined class/type. Provenance may be bounded, but the bound and truncation reason must be explicit.

### 11. Analysis facts have trust levels

Use explicit trust/evidence classes. A useful conceptual scale is:

```text
runtime/language invariant
exact syntax fact
sound abstract fact
ratified declared contract
trusted native summary
interprocedural sound summary
heuristic/editor inference
```

Consumers choose a minimum acceptable trust level. Never let an optimizer or prover accidentally consume heuristic facts because they share the same enum.

## Task workflow

### Phase 1 — establish reality

1. Read repository guidance (`AGENTS.md`, top-level project map, relevant specs/ADRs/PDRs).
2. Inspect current source and tests for the subsystem.
3. Identify current semantic IDs, facts, snapshots, effects, worklists, and invalidation rules.
4. Separate CURRENT from RATIFIED/PROPOSED/FUTURE material.
5. Find the concrete dynamic semantics governing the feature.

### Phase 2 — formulate the analysis

1. Name the concrete property.
2. Name consumer and trust requirement.
3. Decide may/must and forward/backward.
4. Define abstract state and program points.
5. Define order, `⊥`, `⊤`, join, and normalization.
6. Derive transfer functions and edge refinements.
7. Define call/heap/dynamic-boundary semantics.
8. Prove or argue monotonicity and convergence.
9. Decide precision limits and widening.
10. Define provenance and uncertainty reasons.

### Phase 3 — design the repository representation

1. Reuse stable semantic IDs; do not key durable facts by names or byte offsets alone.
2. Decide whether structured source flow remains sufficient or a reusable CFG/HIR is now justified.
3. Prefer immutable published products and worker-local mutable solver state.
4. Separate source facts, derived facts, and summaries.
5. Record dependency ownership and reverse edges.
6. Define canonical semantic equality for fixed-point and invalidation checks.
7. Choose deterministic collections/order where reproducibility matters.
8. Bound memory growth from unions, provenance, contexts, paths, and caches.

### Phase 4 — implement and verify

1. Unit-test algebraic domain laws.
2. Test transfer functions with hand-built states.
3. Test diamonds, terminating branches, loops, abrupt exits, closures, dynamic calls, and recursion.
4. Test unknown/native/reflection/fiber boundaries.
5. Test incremental edit sequences against clean recomputation.
6. Test cancellation does not publish partial generations.
7. Add fuzz/property/metamorphic tests where the state space is combinatorial.
8. Measure iteration counts, allocations, latency, and invalidation frontier.
9. Verify consumers use the fact at the intended trust level.
10. Re-read the concrete semantics after implementation and look for omitted observable effects.

## Quick-reference mental models

### Lattice mental model

For a may-analysis, think “set of possible concrete states.” More abstract means the represented set grows:

```text
{Int} ⊑ {Int, String} ⊑ AnyPossible
```

`⊥` represents no states; `⊤` represents all states modeled by the domain.

### Flow mental model

```text
source program
   ↓ semantic lowering / structured flow
program points + edges
   ↓ transfer
abstract states
   ↓ worklist + join
fixed point
   ↓
queryable facts + provenance
```

### Interprocedural mental model

```text
call-site evidence ──→ parameter contribution
       │                    │
       │                    ▼
       └──────────────→ callable summary
                            │
                            ▼
                      dependent callers
```

Think dependency propagation, not recursive expansion.

### Dynamic boundary mental model

```text
known operation      -> precise transfer
bounded target set   -> analyze each + join
unknown target       -> conservative summary/havoc
trusted native       -> use validated contract
untrusted native     -> boundary uncertainty
```

### Incremental mental model

```text
source delta
  -> semantic surface delta
  -> dirty IDs
  -> affected summaries/facts
  -> worklist to semantic stability
  -> atomic immutable publication
```

## When a CFG or IR is justified

Do not introduce an IR merely because compilers have one. Structured recursive flow is often simpler for source-oriented analysis and malformed-editor states. Introduce a reusable semantic CFG/HIR when multiple analyses independently reimplement the same control semantics—especially reachability, loop exits, exceptional edges, non-local control, dominance, liveness, or proof/optimization program points.

Warning signs:

- each analysis has its own interpretation of `if`, loops, `return`, `throw`, block invocation, or non-local return;
- program-point identity differs between checker, linter, prover, and optimizer;
- dominance/post-dominance or SSA/value-version reasoning becomes necessary;
- exceptional/control edges cannot be represented cleanly by structured recursion;
- performance requires sparse propagation over def-use rather than repeated AST walks.

A new IR must preserve semantic source identity and provenance. Normalizing syntax must not make diagnostics or refactoring unable to recover the originating construct.

## Common failure modes

### “Unknown means safe”

Wrong. Maximum uncertainty normally means more possible behavior, not no behavior.

### “One loop pass is conservative”

Usually wrong. It can miss values/effects produced only after repeated iterations. Solve a fixed point or use a conservative loop summary.

### “Iteration cap equals convergence”

Wrong. A cap is a resource policy. If reached, widen/fallback and record the reason.

### “The LSP already inferred a class, so the checker has a type”

Wrong. Runtime-shape evidence and language typing are separate domains with explicit bridges.

### “A dynamic call can be ignored because we cannot resolve it”

Wrong for any fact the dynamic call can invalidate. Model effects/havoc.

### “A cache keyed by `ClassId` is enough”

Wrong if the class surface or dependencies mutate while the ID stays stable. Include semantic revisions/dependency tracking.

### “Semantic equality means `Arc::ptr_eq`”

Wrong. Pointer reuse is an optimization signal. Fixed-point/invalidation equality must compare semantic content, excluding irrelevant provenance such as publication generation where appropriate.

### “More path sensitivity is always better”

Wrong. Precision may grow exponentially. Choose predicates and partitions deliberately.

### “Runtime tests proved the analysis sound”

Wrong. Differential runtime testing finds counterexamples but finite testing cannot establish universal soundness.

### “Solver timeout means property false/true”

Wrong. Timeout is an analysis result category, not a semantic truth.

### “Static analysis can assume no reflection because most code does not use it”

Wrong for a sound whole-language analysis unless the program/profile explicitly closes that capability.

## Reference map

Load references by question rather than reading everything indiscriminately:

- Mathematical order, joins, monotonicity, Knaster–Tarski, Kleene iteration, worklist termination: [orders-lattices-and-fixed-points.md](references/orders-lattices-and-fixed-points.md)
- Concrete/abstract semantics, Galois connections, sound transformers, abstraction design: [abstract-interpretation-foundations.md](references/abstract-interpretation-foundations.md)
- Forward/backward may/must analyses, worklists, distributivity, sparse frameworks: [dataflow-analysis-frameworks.md](references/dataflow-analysis-frameworks.md)
- CFG construction, dominance, post-dominance, SSA, MemorySSA trade-offs: [cfg-dominators-and-ssa.md](references/cfg-dominators-and-ssa.md)
- Statement/expression transfer, abrupt completion, strong/weak updates, havoc, provenance: [transfer-functions-and-state-modeling.md](references/transfer-functions-and-state-modeling.md)
- Infinite chains, widenings, narrowing, thresholds, recursive convergence, budget fallback: [widening-narrowing-and-termination.md](references/widening-narrowing-and-termination.md)
- Flow-sensitive narrowing, path predicates, trace partitioning, contradictions: [path-sensitivity-and-refinement.md](references/path-sensitivity-and-refinement.md)
- Call graphs, summaries, dynamic dispatch, higher-order calls, SCC/worklist solving: [interprocedural-analysis-and-call-graphs.md](references/interprocedural-analysis-and-call-graphs.md)
- Points-to, abstract allocation, strong/weak heap updates, escape and closure capture: [heap-alias-and-escape-analysis.md](references/heap-alias-and-escape-analysis.md)
- Effect domains, blocks, non-local returns, fibers, yields, shared-state interference: [effects-closures-and-concurrency-analysis.md](references/effects-closures-and-concurrency-analysis.md)
- Concrete examples of constant/shape/interval/presence/collection/product domains: [domain-design-examples.md](references/domain-design-examples.md)
- Dynamic sends, selectors, reflection, method mutation, `doesNotUnderstand`, FFI boundaries: [dynamic-language-and-reflection-analysis.md](references/dynamic-language-and-reflection-analysis.md)
- Dependency graphs, invalidation, immutable snapshots, cancellation, demand-driven queries: [incremental-and-demand-driven-analysis.md](references/incremental-and-demand-driven-analysis.md)
- Soundness contracts, false positives, trust tiers, resource budgets, precision cliffs: [soundness-precision-and-cost.md](references/soundness-precision-and-cost.md)
- Domain-law tests, flow fixtures, differential/metamorphic/fuzz/incremental testing: [testing-static-analyses.md](references/testing-static-analyses.md)
- CURRENT Phalcom semantic-engine map and bridges to future type/proof/effect domains: [phalcom-analysis-domains.md](references/phalcom-analysis-domains.md)
- Comparative systems and primary literature: [comparative-analysis-and-reading.md](references/comparative-analysis-and-reading.md)
- Review checklist and pressure-test scenarios: [review-and-validation-scenarios.md](references/review-and-validation-scenarios.md)

## Verification and review expectations

A serious change in this domain is incomplete until the review can answer all of the following:

```text
What concrete behavior can occur that the abstraction represents?
What does the order mean?
What are ⊥ and ⊤?
Are joins canonical and deterministic?
Why are transfer functions monotone, or why is non-monotonicity controlled?
Where is each fixed point?
What guarantees termination?
What is widened and why?
What happens at an unknown call or reflective/native boundary?
Which facts survive a yield or shared-state interference point?
What is may versus must?
Which semantic IDs own the facts?
Which changes invalidate them?
Can cancellation publish a half-solved state?
Can provenance explain the result?
Which consumers may trust the result?
Does incremental recomputation equal clean recomputation?
What is the measured hot path and memory bound?
```

If the implementation cannot answer these questions, it may still be useful exploratory code, but it is not yet a dependable semantic subsystem.

## Skill self-test

After reading this skill, an agent should reject these temptations immediately:

1. Using LSP `ValueShape::Unknown` as the checker's `Dynamic` type.
2. Keeping a field fact across an unknown reflective call without a mutation contract.
3. Analyzing a loop body once and treating that state as the loop exit.
4. Recursively descending through callees instead of solving recursive summaries.
5. Using an editor-confidence fact to devirtualize a VM send without a guard/proof.
6. Treating a solver budget exhaustion as proof of safety or type correctness.
7. Keying semantic caches by source offsets and assuming identities survive reparsing.
8. Publishing an incremental snapshot whose summaries and parameter facts come from different solver generations.
9. Adding unlimited path partitions or union alternatives because it improves one example.
10. Treating a draft typing architecture as CURRENT language behavior.

The expected response to each is not merely “don't.” The agent should be able to name the violated invariant, choose a sound/advisory alternative, identify affected consumers, and specify tests that distinguish the two designs.

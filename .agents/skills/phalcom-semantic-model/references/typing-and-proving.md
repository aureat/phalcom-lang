# Optional Typing, Checker, Typed Runner, and Static Proving Integration

This is deliberately a **bridge reference**, not the type-theory or prover textbook. Use
`type-theory` for formal type relations/inference and `static-prover-development` for Hoare
logic, VC generation, symbolic execution and SMT engineering. This file owns how those
domains connect to Phalcom's shared semantic identities, flow, dispatch, provenance and
incremental snapshots.

Treat typing/proving status explicitly. Detailed documents under `docs/spec/typing/` can be
normative design, proposed design, or future work without being a claim of current compiler/VM
support. Re-read their status before saying "Phalcom currently checks ...".

## Architectural goal

Phalcom's future typing should *reuse* the semantic machinery already built for the LSP:

- module identities/graph;
- scope and binding identities;
- class/member surfaces;
- selector/dispatch semantics;
- control-flow structure;
- callable dependencies/summaries;
- provenance/invalidation;
- immutable query snapshots.

It should not simply rename `ValueShape` to `Type`.

## Three semantic modes

Keep these conceptually separate even if they share code:

### Advisory semantic inference

Goal: useful editor information under incomplete dynamic code.

May tolerate:

- unknowns;
- heuristics;
- bounded widening;
- incomplete project state.

Must not claim correctness proof from heuristics.

### Static checker

Goal: decide obligations created by declared/inferred language types.

Needs:

- canonical type representation;
- substitution;
- subtype/assignability/consistency relations;
- generic constraints;
- bidirectional check/synthesize modes;
- flow refinement;
- diagnostic evidence;
- explicit handling of dynamic/unknown boundaries.

### Typed runner/runtime contract mode

Goal: validate contracts that cannot be statically proved or that the mode promises to
check dynamically.

Needs:

- reified metadata/check plan;
- exact boundary insertion semantics;
- error reporting with source/type provenance;
- rules for FFI/native calls;
- performance policy.

Do not silently enable runtime checks in ordinary execution unless specified.

## Type relations are not one operator

The checker must distinguish at least conceptually:

```text
semantic type equality
subtyping
assignability
constraint satisfaction
protocol conformance
gradual consistency
runtime instance/conformance test
type-expression isomorphism/normalization
```

Do not implement all of these as `is_subtype`.

## Bidirectional typing

For a dynamic OO language with optional annotations and local generic inference, a
bidirectional architecture is useful:

```text
synthesize(expr) -> Type
check(expr, expected Type) -> obligations/result
```

Expected types can flow inward to literals, blocks, and generic arguments without global
HM inference.

The exact Phalcom typing spec is authoritative; this is an implementation tool, not a
license to change inference policy.

## Constraint generation and solving

Separate:

```text
semantic traversal -> constraints/obligations
solver -> substitutions/judgments
```

Useful obligation kinds may include:

- `A <: B`;
- `A assignable-to B`;
- `T` satisfies bound/protocol;
- selector exists on type/receiver;
- argument labels/arity compatible;
- return type compatible;
- exhaustiveness/variant obligation.

Keep solver metavariables distinct from canonical user-visible types.

## Flow refinement

Reuse semantic control flow. Do not create a second branch walker inside the type checker.

Examples of possible refinements, only when specified:

```text
x: T | None; condition x != None -> true branch x: T
result: Option<T>; pattern Some(value) -> value: T
sealed variant test -> eliminate impossible variants
```

A refinement has:

- trusted predicate semantics;
- true-state transform;
- false-state transform;
- merge rule;
- provenance.

## Static proving

Contracts and invariants should become proof obligations over program points.

Suggested staged architecture:

1. constant folding / exact syntactic facts;
2. CFG/dataflow facts;
3. type/variant/Option refinements;
4. interval/range analysis where useful;
5. interprocedural pre/post summaries;
6. symbolic/SMT discharge for remaining supported obligations.

Most everyday proof wins should not require SMT.

## Proof status

Use a three-way or richer result:

```text
Proved
Refuted(counterevidence)
Unknown(reason)
```

Do not diagnose `Unknown` as a violated contract unless a mode explicitly requires proof.

## Contracts

For `@requires`, `@ensures`, `@invariant`, distinguish:

- source declaration/metadata;
- static obligation;
- proven fact;
- runtime check plan;
- observed runtime failure.

The same contract may be statically proved at one call site and dynamically checked at
another.

## Type metadata does not change ordinary dispatch

Ordinary selector identity and lookup must remain independent of type metadata unless a
ratified typed-dispatch feature explicitly changes dynamic semantics. Do not infer current
runtime type-directed dispatch from the existence of typing design documents.

A checker can report "receiver type lacks selector" based on the dynamic dispatch model;
it should not choose another implementation based on parameter annotations unless an
explicit typed-dispatch feature is being analyzed.

## Runtime shape assists typing

Advisory shape facts can accelerate/suggest typing but require a soundness gate.

Good uses:

- exact literal shape -> synthesized primitive/nominal type;
- exact constructor result -> class instance type;
- exact callable declaration -> method signature lookup;
- known tuple structure -> seed product type.

Questionable uses requiring spec:

- observed call sites determine parameter contract;
- observed field writes determine declared field type;
- absence of dynamic send proves closed-world behavior.

## Typing assists LSP

Once checker types exist, LSP can become more precise:

- completion over declared/proven receiver type;
- hover with declared + inferred type and provenance;
- signature help with substitutions;
- inlay type hints;
- diagnostics with constraint traces;
- dead/invalid member access highlighting.

But LSP should still work on untyped/incomplete code using shape inference.

## Avoiding the type-contract wall

Semantic tooling should be able to explain where annotations add information.
A future "annotation value" analysis can classify positions:

```text
exactly inferred -> annotation optional/documentational
inferred but widened -> annotation improves precision
recursive/public boundary -> annotation strongly recommended
FFI/exported API -> annotation recommended/required by policy
ambiguous/under-constrained -> annotation required
```

This turns typing guidance into semantic diagnostics/inlay hints instead of demanding
annotations everywhere.

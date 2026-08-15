---
name: type-theory
description: Use when designing, reviewing, formalizing, or implementing Phalcom type relations, type expressions, generics, substitution, variance, inference, callable types, unions/intersections, ADTs, protocols, gradual typing, recursive/self/metatypes, runtime type reification, or checker rules where mathematical type-system distinctions determine correctness.
compatibility: Designed for agents working on Phalcom language design, typing specifications, compiler/checker implementation, LSP semantics, runtime reflection, and static-proof integration.
---

# Type Theory for Phalcom

## Overview

This skill is the mathematical and semantic foundation for Phalcom typing work. It exists to let an implementation agent move deliberately between:

```text
mathematical relation
    ↕
language semantic rule
    ↕
canonical type representation
    ↕
constraint/relation algorithm
    ↕
Rust implementation boundary
    ↕
diagnostics and conformance tests
```

It is not a syntax catalog, a generic type-checker tutorial, or permission to replace Phalcom's dynamic object model with a textbook calculus. Phalcom typing must describe actual Phalcom execution and reflection.

Before repository-specific work, inspect current repository guidance and the current typing specifications. In `phalcom-lang`, read `AGENTS.md`, then the relevant files under `docs/spec/typing/` and any newer design/decision documents. Repository status beats this skill when they disagree.

## Intellectual ownership

This skill owns questions such as:

- What relation is type equality, equivalence, subtyping, consistency, acceptance, or conformance?
- What does a type variable bind, and how is capture-avoiding substitution defined?
- What are the valid variance rules and why?
- How do generic bounds and finite constraints differ?
- What kind of entity is a type constructor? What does kinding buy us?
- What are sound declarative and algorithmic rules for callables, unions, intersections, products, sums, recursive types, protocols, and gradual boundaries?
- When does inference have a principal answer, several admissible answers, or no answer?
- How should `Self`, class objects, metaclasses, and reflective type descriptors be modeled without collapsing runtime identity into type identity?
- Which facts are canonical types, which are flow refinements, and which are proof facts?

This skill does **not** own the complete repository architecture of a future checker, parser implementation, LSP handler design, abstract interpretation engine, or SMT prover. It supplies the type-theoretic obligations those skills must implement.

## Neighboring skills and boundaries

- `programming-language-semantics`: owns general operational/denotational semantics and semantic-equivalence methods. Use it when proving that a typing rule corresponds to dynamic execution.
- `phalcom-semantic-model`: owns shared semantic identities, scopes, definitions/uses, callable/member facts, and one-semantic-truth architecture.
- `semantic-analysis-development`: owns semantic-engine integration, flow facts, summaries, provenance, and incremental semantic queries.
- `static-analysis-and-abstract-interpretation`: owns abstract domains, lattices for program approximation, fixed-point iteration, widening, and analysis soundness. Its precision order is not automatically the subtype order.
- `type-checker-development` (when present): owns checker pass/query structure, repository data structures, incremental orchestration, diagnostics plumbing, and implementation sequencing.
- `static-prover-theory-and-development`: owns Hoare logic, VCs, SMT encoding, proof status, contracts, and counterexamples. A proof fact is not a canonical type.
- `rust-compiler-engineering`: owns Rust arenas, interning, snapshots, ownership, allocation discipline, profiling, and safe native boundaries.
- `lsp-development`: owns editor protocol behavior. LSP consumers query typing/semantic facts; they do not invent a second type system.

If a task belongs to several skills, use this one for the mathematical relation and the neighboring skill for subsystem mechanics.

## Status discipline for Phalcom typing

Never say "Phalcom does X" merely because a design document proposes X. Label claims as one of:

```text
CURRENT IMPLEMENTATION
NORMATIVE / RATIFIED DESIGN
PROPOSED
EXPERIMENTAL
FUTURE / PLANNED
RECOMMENDATION
```

At the repository checkpoint inspected while this skill was deepened (2026-08-14):

- `docs/spec/typing/01-core-type-lattice-and-unit.md` is marked a normative design specification and fixes major laws for `Never`, `()`, `Option`, `Any`, `Dynamic`, and ordinary return behavior.
- `docs/spec/typing/02-type-expression-foundation.md` and `03-type-parameters-and-generic-signatures.md` explicitly say "Proposed normative design; not a claim of current compiler or VM support" even though the status series records Documents 01–03 as completed design work.
- Later documents for applied types, substitution, complete subtype/consistency/acceptance relations, variance validation, inference, structural conformance, callables, aliases/intersections, checker modes, and tooling are planned/deferred in that series.

Treat these as a **status-reading example**, not a timeless snapshot. Re-read the repository before implementation.

## Core doctrine

1. **Name the relation before implementing it.** `equivalent`, `<:`, `accepts`, `conforms`, `consistent`, `runtime_is_a`, and `same_layout` are different relations.
2. **Typing describes dynamic Phalcom.** Type metadata does not silently become selector identity, overload resolution, allocation policy, or inline-cache identity.
3. **Keep semantic domains separate.** Runtime `ClassId`, LSP `ValueShape`, canonical `TypeId`, inference variable, proof proposition, and runtime validation obligation are distinct.
4. **Missing information is not one value.** Distinguish absent annotation, explicit `Dynamic`, safe top `Any`, unresolved inference variable, blocked analysis, ambiguity, inconsistency, recovery error, and unreachable bottom.
5. **Binders have semantic identity.** A type parameter is not its spelling. Substitution and shadowing operate on owner-qualified binder identity.
6. **Declarative relation first; algorithm second.** State what is true independently from the terminating procedure used to decide it.
7. **Termination is part of correctness.** Recursive types, F-bounds, structural conformance, unions/intersections, and generic constraints need explicit cycle/measure/SCC policy.
8. **Precision shortcuts are consumer-scoped.** An LSP union cap or shape widening must not silently become checker acceptance.
9. **Reification is not execution semantics.** Retaining `List<Int>` metadata does not require type-directed dispatch or distinct runtime classes.
10. **Provenance survives.** Constraints and relation failures need origins sufficient for explanatory diagnostics.
11. **Open-world mutation invalidates assumptions.** Reflection, class-surface mutation, module changes, and native boundaries must be represented in validity/trust conditions.
12. **Test negative space.** Every new relation rule needs a counterexample that would become incorrectly accepted under the tempting unsound shortcut.

## Minimum notation

Use notation only after defining it.

```text
Γ ⊢ e : T                 e has type T
Γ ⊢ e ⇒ T                 e synthesizes T
Γ ⊢ e ⇐ T                 e checks against expected T
Δ ⊢ T : κ                 type expression T has kind κ
Γ ⊢ A <: B                A is a subtype of B
Γ ⊢ A ≡ B                 semantic type equivalence
Γ ⊢ A ~ B                 gradual consistency, if defined
Γ ⊢ A ⊣ B                 assignment/acceptance relation, if defined
Γ ⊢ C : P                 candidate C conforms to protocol P
T[U/X]                     capture-avoiding substitution
FV(T)                      free type variables/parameters
A ⊔ B / A ⊓ B             join / meet in a named order
⊥ / ⊤                      bottom / top of a named domain
∀α. T / ∃α. T              universal / existential quantification
μX. T                      recursive type fixed point
```

Do not use the same `⊔` for a subtype lattice and an abstract-analysis precision lattice unless the document explicitly proves they are the same order.

## Workflow for a type-theory task

### 1. Identify the semantic question

Rewrite the request as one or more relations or formation problems.

Examples:

```text
"Can List<Cat> be passed here?"       -> subtyping or acceptance?
"Are these annotations the same?"    -> equivalence or source equality?
"What does Box<T> mean?"             -> type formation + binding + application?
"What type should None infer?"        -> synthesis + expected-type adaptation?
"Does class object satisfy P?"        -> metatype/class-side conformance?
```

If you cannot name the relation, do not code it yet.

### 2. Inspect governing Phalcom semantics

Read current dynamic behavior and current design status. Determine which of these are observable:

- selector identity and dispatch start point;
- class/metaclass identity;
- reflection metadata;
- type annotation presence/absence;
- runtime contract checks;
- module/open-world mutation;
- FFI/native behavior.

### 3. State formation rules

Define which type expressions are well formed:

```text
Δ ⊢ T type
Δ ⊢ C<T1,...,Tn> type
```

Include arity, kind, ownership, bounds, recursion, and trust restrictions.

### 4. State declarative relations

Write the intended laws before an algorithm. Include reflexivity/transitivity only if the relation is meant to have them. For gradual consistency, explicitly note non-transitivity where applicable.

### 5. Derive algorithmic rules

Choose a terminating decision/constraint procedure:

- structural recursion with memoized pairs;
- worklist of obligations;
- union-find/unification;
- lower/upper-bound constraint solving;
- SCC/fixed-point iteration;
- bidirectional synthesis/checking.

State soundness and completeness goals of the algorithm relative to the declarative relation.

### 6. Define representation

Map theory to semantic IDs and data structures without making representation equality the semantics.

Conceptual split:

```text
TypeId             canonical finalized type expression
TypeParamId        binder identity
InferenceVarId     solver-local metavariable
ConstraintId       relation obligation + provenance
ProofFactId        prover fact, not a TypeId
RuntimeTypeObject  reflected object/descriptor identity
```

### 7. Define uncertainty and failure

A solver must not return one vague `Unknown` for all cases. Distinguish at least:

```text
Solved(T)
Underconstrained(vars)
Ambiguous(candidates)
Inconsistent(conflict)
Blocked(dependency)
BudgetExceeded
Recovery(error-id)
```

Language-level `Dynamic` is not one of those implementation failures.

### 8. Check interactions

Review:

```text
dispatch × typing
reflection × canonicalization
mutability × variance
recursion × termination
open world × conformance cache
Dynamic × proof assumptions
FFI × runtime contracts
blocks × control effects
metaclasses × Self
unions × member lookup
```

### 9. Design diagnostics from provenance

Retain enough origin information to explain constraints, expected types, candidate relations, generic bindings, and dynamic boundaries.

### 10. Test the theorem and the algorithm

For each rule, test:

- positive derivation;
- minimally changed negative case;
- recursive/cycle case;
- malformed metadata/source case;
- generic substitution case;
- reflection/identity observation when relevant;
- incremental/full-analysis equivalence when implementation caches results.

## Quick mental models

### Types versus runtime classes

```text
runtime class identity  --describes--> runtime lookup/layout behavior
        │
        └── may be referenced by
              nominal type expression

canonical TypeId        --denotes--> static contract / set-like capability
ValueShape              --approximates--> possible runtime values for analysis
```

A nominal instance type may contain a `ClassId`; `Type = ClassId` is still too small for unions, protocols, callables, type parameters, `Self`, applied types, and special types.

### Subtyping versus consistency

```text
A <: B     substitutability; usually reflexive + transitive
A ~ B      gradual compatibility; often symmetric + non-transitive
```

Never close `~` transitively.

### Generic application

```text
origin + arguments + binder-aware substitution

Box<T>.value : T
Box<Int>.value : Int
```

Application identity, member substitution, runtime class identity, code specialization, and class-side state are independent design dimensions.

### Variance polarity

```text
positive position      covariant
negative position      contravariant
both / mutable         invariant

function parameter flips polarity
function result preserves polarity
```

### Inference

```text
syntax/expected type
      ↓
constraints + provenance
      ↓
solve obligations/bounds
      ↓
validate + substitute
      ↓
canonical type or explicit failure state
```

Inference is not `AST -> guessed class name`.

### Recursive relation

```text
relate(A,B):
  if memo[A,B] == Proven      return true
  if memo[A,B] == Disproven   return false
  if memo[A,B] == InProgress  apply relation's coinductive/guarded rule
  mark InProgress
  prove children
  finalize Proven/Disproven
```

Do not use an arbitrary recursion-depth limit as semantics.

## Common failure modes

- Implementing all relation queries through one `is_compatible` function.
- Treating source spelling as type-parameter identity.
- Interning inference variables into the permanent type arena.
- Treating `Dynamic` as `Any`, or `Any` as analyzer unknown.
- Using subtype transitivity to justify gradual consistency.
- Assuming every subtype lattice has a unique LUB/GLB without defining unions/intersections.
- Inferring covariance for mutable storage.
- Requiring type annotations to participate in selector lookup because checker implementation is easier that way.
- Concluding `Result<(), Never> == ()` from isomorphism alone.
- Treating a protocol's current observed implementors as a closed sum.
- Using bounded recursion/unrolling as proof of recursive type compatibility.
- Equating runtime generic reification with monomorphized runtime classes.
- Letting a type normalizer invoke arbitrary user code.
- Caching conformance without member-surface/version dependencies.
- Hiding an underconstrained generic inference behind `Dynamic`.
- Reconstructing diagnostic causality after constraint origins were discarded.

## Reference map

Load only what the task needs.

- Formal judgments, contexts, declarative/algorithmic correspondence, scoped soundness: [references/judgments-contexts-and-metatheory.md](references/judgments-contexts-and-metatheory.md)
- Equality, equivalence, subtyping, acceptance, consistency, decision procedures: [references/equality-equivalence-and-subtyping.md](references/equality-equivalence-and-subtyping.md)
- Products, sums, tuple/record relations, unit, bottom, Option/Result: [references/products-sums-unit-and-bottom.md](references/products-sums-unit-and-bottom.md)
- Union/intersection rules, normalization, joins/meets, explosion control: [references/unions-intersections-and-type-lattices.md](references/unions-intersections-and-type-lattices.md)
- Function/block/method types, callable compatibility, variance/polarity: [references/functions-callables-and-variance.md](references/functions-callables-and-variance.md)
- Parametric polymorphism, binders, substitution, bounds, existentials: [references/polymorphism-generics-and-substitution.md](references/polymorphism-generics-and-substitution.md)
- Unification, constraints, local inference, bidirectionality, ambiguity: [references/inference-constraints-and-bidirectionality.md](references/inference-constraints-and-bidirectionality.md)
- Kinds, constructors, arity, higher-kinded boundaries: [references/kinds-and-type-constructors.md](references/kinds-and-type-constructors.md)
- Recursive/equi/iso-recursive types, coinduction, SCCs: [references/recursive-types-and-fixed-points.md](references/recursive-types-and-fixed-points.md)
- Protocol/structural conformance and recursive requirement solving: [references/protocols-and-structural-typing.md](references/protocols-and-structural-typing.md)
- Gradual typing, precision, consistency, casts/contracts, blame: [references/gradual-typing-and-dynamic-boundaries.md](references/gradual-typing-and-dynamic-boundaries.md)
- Refinements, propositions, occurrence typing, proof boundaries: [references/refinements-propositions-and-static-proofs.md](references/refinements-propositions-and-static-proofs.md)
- ADTs, patterns, exhaustiveness/usefulness algorithms: [references/adts-patterns-and-exhaustiveness.md](references/adts-patterns-and-exhaustiveness.md)
- Static/reified/erased runtime type metadata and reflection: [references/reification-erasure-and-runtime-types.md](references/reification-erasure-and-runtime-types.md)
- `Self`, class objects, metaclasses, instance/class-side typing: [references/metatypes-self-and-class-objects.md](references/metatypes-self-and-class-objects.md)
- Canonical semantic type representation, interning, relation caches: [references/type-representation-and-canonicalization.md](references/type-representation-and-canonicalization.md)
- Exceptions, non-local return, yielding, effects versus value types: [references/effects-control-and-computation-types.md](references/effects-control-and-computation-types.md)
- Phalcom-specific status/doctrine and subsystem bridges: [references/phalcom-typing-doctrine.md](references/phalcom-typing-doctrine.md)
- Comparative language precedents and reading discipline: [references/comparative-language-notes-and-reading.md](references/comparative-language-notes-and-reading.md)
- Pressure tests and review scenarios: [references/review-and-validation-scenarios.md](references/review-and-validation-scenarios.md)

## Verification and review expectations

Before approving a type-theory-driven change, require answers to these questions:

1. Which judgment/relation is implemented?
2. What are its mathematical laws?
3. What is normative versus implementation recovery?
4. What is the termination argument?
5. Which semantic IDs are binders/origins?
6. Which canonicalization laws are permitted?
7. What dynamic Phalcom behavior does the rule describe?
8. Does the rule change reflection or selector identity? If yes, is that explicitly ratified?
9. What happens at `Dynamic`, native, reflection, and malformed-metadata boundaries?
10. Can the algorithm distinguish ambiguity, absence of information, inconsistency, and bottom?
11. What provenance is retained?
12. What counterexample demonstrates the unsound shortcut?
13. What conformance fixtures lock the intended behavior?

A solution that cannot answer these is not ready to implement, even if its Rust code looks clean.

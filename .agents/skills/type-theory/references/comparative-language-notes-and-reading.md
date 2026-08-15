# Comparative Language Notes and Reading Map

## Purpose

Use other type systems as experiments with explicit assumptions, not as feature catalogs. A precedent is useful only after answering:

```text
What problem did it solve?
What assumptions made its solution work?
Which assumptions does Phalcom share?
Which does Phalcom reject?
What semantic/implementation costs came with it?
What exactly would we borrow?
```

"Rust/TypeScript/Swift does X" is not a design argument.

## 1. Smalltalk and Self — dynamic object model baseline

Study for:

- message sends and selector identity;
- classes/metaclasses;
- reflection;
- method lookup and inline caches;
- open-world mutation.

Phalcom lesson: typing must describe receiver/selector dispatch rather than silently replace it with static overload resolution. Any optimization based on type information must remain valid under actual dynamic lookup/reflection rules.

Do not import: absence of static typing as a requirement. Phalcom can add checker contracts over same dynamic semantics.

## 2. Python + mypy/Pyright — optional typing over dynamic semantics

Study:

- annotations separate from runtime object model;
- `Any` as permissive escape;
- protocols/structural typing;
- generics and variance;
- flow-sensitive narrowing;
- differences between runtime annotations and checker interpretation.

Lessons:

- gradual adoption is practical;
- `Any` can become viral and hide errors;
- checker disagreement becomes ecosystem cost when semantics underspecified;
- runtime reflection of annotations needs source/normalized distinction.

Phalcom should define its relations centrally rather than allow each tool to invent them.

## 3. TypeScript — unions, intersections, control-flow narrowing, deliberate unsoundness

Study:

- `any` versus `unknown`;
- union/intersection normalization;
- structural compatibility;
- discriminated unions;
- control-flow analysis;
- contextual typing;
- performance engineering of a large practical checker.

Lessons:

- expressive set-like types are useful for dynamic code;
- structural systems can become complex and intentionally unsound at edges;
- aggressive union/intersection features create solver/performance cliffs;
- `any` and `unknown` demonstrate why permissive dynamic escape and safe unknown/top-like concepts must differ.

Do not assume TypeScript's deliberate unsoundness fits Phalcom's correctness-participating checker goals.

## 4. Kotlin — local inference, null elimination, smart casts, sealed hierarchies

Study:

- bidirectional/local type inference;
- smart casts and stability conditions;
- declaration/use-site variance;
- sealed class exhaustiveness;
- explicit nullability.

Phalcom lessons:

- flow narrowing must account for mutation/alias stability;
- closed hierarchies can make exhaustiveness provable;
- local inference reduces annotation burden.

Do not import nullable types automatically; Phalcom's current direction uses `Option` as explicit absence semantics.

## 5. Swift — rich generics and solver complexity

Study:

- protocols and associated types;
- existential `any` versus opaque `some`;
- generic constraints;
- conditional conformance;
- variance/subtyping choices;
- constraint solver diagnostics and compile-time complexity.

Lessons:

- expressive generic constraint systems can make inference search expensive;
- existential/opaque distinctions are semantically valuable;
- diagnostic quality requires retaining solver provenance.

Use Swift as a warning that "the solver can search" is not a termination/performance design.

## 6. Rust / rustc — explicit obligations and semantic IR boundaries

Study:

- trait obligations and canonical queries;
- generic substitution;
- associated types;
- variance inference;
- HIR/MIR separation;
- query dependency/incremental compilation;
- diagnostic obligation chains;
- explicit `unsafe` trust boundaries.

Phalcom lessons:

- represent binders/IDs semantically;
- separate declarative relation from obligation-solving algorithm;
- canonical query inputs matter for caching;
- a shared semantic IR can serve several analyses.

Do not import Rust ownership/lifetimes/trait coherence unless Phalcom explicitly designs corresponding semantics.

## 7. rust-analyzer — IDE semantic database

Study:

- immutable/query-oriented semantic facts;
- syntax/semantic identity separation;
- incremental invalidation;
- incomplete-source tolerance;
- IDE features as semantic queries.

Phalcom lesson: LSP should query shared type/semantic engine, not own separate inference.

## 8. OCaml / Standard ML — principal inference under controlled assumptions

Study:

- Hindley-Milner inference;
- unification and occurs check;
- generalization/instantiation;
- value restriction;
- algebraic data types and pattern exhaustiveness;
- modules.

Lessons:

- principal types are possible because the system deliberately limits subtyping/overloading/effects;
- mutation makes unrestricted generalization unsafe;
- ADT exhaustiveness can be algorithmic and precise.

Do not claim HM principal inference transfers to Phalcom's nominal/structural/gradual subtype system.

## 9. Haskell / GHC — advanced type theory and complexity cliffs

Study:

- System F/System Fω intuition;
- type classes/constraints;
- higher-rank polymorphism;
- GADTs and equality constraints;
- type families;
- Core;
- role/representation distinctions.

Phalcom lessons:

- GADT patterns require branch equality constraints;
- higher-rank inference generally needs annotations;
- type-level computation can threaten termination/coherence;
- semantic/representation roles must be distinguished.

Borrow only when concrete Phalcom use cases justify complexity.

## 10. Scala 3 — unions/intersections and advanced subtyping

Study:

- union/intersection types;
- path-dependent types;
- match types;
- contextual abstractions;
- sophisticated subtype normalization.

Lesson: powerful subtyping can make normalization, inference, and error explanation difficult. Use as evidence for complexity budgeting, not as a target feature checklist.

## 11. C# / CLR — reified generics and `dynamic`

Study:

- declaration-site variance;
- runtime generic metadata;
- reflection;
- `dynamic` operations;
- class/object/type descriptor distinctions.

Phalcom lessons:

- reification does not require source-level type-directed ordinary dispatch;
- runtime generic metadata has ABI/storage costs;
- dynamic operations can be localized rather than turning every type into top.

Do not assume CLR-style specialization/layout fits Phalcom VM.

## 12. Julia — dynamic multiple dispatch and inference for optimization

Study:

- multiple dispatch;
- runtime type lattice;
- specialization;
- inference used for optimization;
- union splitting.

Phalcom lesson: type-directed dispatch is a coherent language design when it is **explicitly the runtime semantics**. That is different from silently adding typed overload selection to a Smalltalk-style selector system.

Julia also demonstrates that optimizer inference may use approximations different from correctness typing.

## 13. Roslyn — immutable syntax/semantic models and tooling

Study:

- syntax trees versus semantic models;
- symbols/identities;
- binding/type info queries;
- diagnostics/refactoring sharing semantic facts.

Phalcom lesson: semantic identities and immutable snapshot-like results scale across IDE consumers.

## 14. Theory reading map

Foundations:

- Benjamin C. Pierce, *Types and Programming Languages*.
- Robert Harper, *Practical Foundations for Programming Languages*.
- Pierce, ed., *Advanced Topics in Types and Programming Languages*.
- Cardelli and Wegner, "On Understanding Types, Data Abstraction, and Polymorphism".

Inference:

- Hindley, Milner, Damas on principal type inference.
- Pierce and Turner on local type inference.
- bidirectional typing literature (Dunfield/Krishnaswami surveys are useful entry points).

Subtyping/recursive types:

- TAPL chapters on subtyping, recursive types, bounded quantification.
- Amadio/Cardelli and related recursive-type/subtyping work for advanced structural recursion.

Gradual typing:

- Siek and Taha foundational gradual typing.
- Wadler and Findler on blame/casts.
- gradual guarantee literature when Phalcom chooses a precise theorem.

Set-theoretic types:

- Castagna and collaborators for semantic subtyping/unions/intersections if Phalcom moves toward a richer set-theoretic algebra.

Patterns:

- Luc Maranget, "Warnings for pattern matching" and related usefulness-matrix work.

Parametricity/existentials:

- Reynolds on parametricity;
- System F/existential treatments in TAPL/PFPL.

Use primary papers/texts for formal claims. Use compiler implementations for engineering tradeoffs.

## 15. Comparison template for a Phalcom design document

When citing precedent, write:

```text
Problem:
  Generic call inference is ambiguous without expected context.

Precedent:
  Swift uses a global-ish constraint solver with contextual constraints.

Assumptions/cost:
  Rich overload/generic system; known compile-time complexity and diagnostic challenges.

Phalcom overlap:
  Contextual generic inference may be useful.

Phalcom difference:
  Ordinary dispatch must remain selector/receiver based; no typed overload selection.

Borrow:
  Constraint provenance and expected-type flow.

Reject/defer:
  Broad overload search and solver search space.
```

This format prevents cargo-culting.

## 16. Failure modes

- "Python does it" as sole justification.
- Importing syntax while ignoring semantic assumptions.
- Importing a solver without its termination/complexity tradeoffs.
- Copying TypeScript `any` while claiming strong correctness guarantees.
- Copying Julia multiple dispatch into checker without changing runtime semantics.
- Copying Rust trait/ownership concepts because implementation language is Rust.

## 17. Competency questions

1. Why is HM inference precedent limited for Phalcom?
2. What does TypeScript teach about `any` versus `unknown`?
3. Why is Julia relevant specifically as a contrast for explicit type-directed dispatch?
4. What can Phalcom borrow from rustc without adopting Rust ownership?
5. Which language is a useful precedent for smart-cast stability under mutation?

# Comparative Semantic Architectures: Lessons Without Cargo Culting

## 1. Comparison method

Other language implementations are case studies, not templates. For every borrowed mechanism ask:

```text
What problem does it solve?
What language/runtime assumptions make it correct?
Do Phalcom's assumptions match?
What semantic/API commitments would adoption create?
What does it make easier?
What future design does it constrain?
```

The useful unit of comparison is usually an invariant or dependency model, not a directory layout.

## 2. rust-analyzer: incremental semantic queries

### Problem solved

rust-analyzer serves IDE queries over a large statically typed language while source changes continuously. It separates syntax from semantic identities and uses query-driven incremental computation.

### Relevant lesson

Phalcom should emulate the **principle** that editor features are projections of a semantic database and that queries depend on stable IDs rather than rebuilding analysis in each handler.

### Assumption mismatch

Rust's static item/module/type rules are far more closed and compile-time-governed than Phalcom's dynamic message dispatch/reflection. Do not copy Rust's ability to resolve every method call statically as a Phalcom requirement. Phalcom queries often need bounded candidates/dynamic uncertainty.

### Adoption consequence

A Salsa-like system may eventually help dependency tracking, but current Phalcom already has explicit candidate snapshots and invalidation. Framework adoption should follow measured complexity, not precedent.

## 3. Roslyn: immutable syntax/semantic models

### Problem solved

C#/VB IDE features require stable, concurrent semantic views and rich diagnostics/refactoring.

### Relevant lesson

Immutable published snapshots, identity-aware symbols, and separating syntax trees from semantic models map well to Phalcom. One request should observe one coherent semantic generation.

### Mismatch

C# overload resolution and static binding rules are language semantics. Phalcom should not import type-directed overload machinery into selector dispatch merely because Roslyn exposes “symbol info” for calls.

## 4. TypeScript: pragmatic analysis over dynamic JavaScript

### Problem solved

TypeScript overlays rich static types/flow narrowing on JavaScript's dynamic runtime.

### Relevant lesson

A static domain can describe a dynamic language without becoming runtime dispatch. Flow-sensitive narrowing, control-flow graph reuse, and editor/checker integration are strong precedents.

### Warning

TypeScript historically includes pragmatic unsoundness and compatibility compromises. Phalcom's future correctness-participating typing should explicitly classify any unsound/gradual rules rather than inherit “useful enough” behavior accidentally. `ValueShape` should remain advisory unless bridged by formal rules.

## 5. mypy/Pyright: typed analysis of Python

### Problem solved

They analyze a highly dynamic, reflective language using explicit type annotations, stubs/contracts for native/dynamic libraries, and flow refinement.

### Relevant lesson

Phalcom will need semantic contracts/stubs for Rust/native/core code and an explicit policy for dynamic boundaries. Static module analysis must not execute imports to learn types/surfaces.

### Warning

Python typing has accumulated distinctions among `Any`, `Unknown`, missing annotations, runtime classes, protocols, and stubs. Phalcom should define these categories early and avoid one omnipotent escape value.

## 6. Julia: dynamic multiple dispatch and inference

### Problem solved

Julia performs aggressive inference/specialization over a dynamic language whose runtime method selection is type-based multiple dispatch.

### Relevant lesson

Julia demonstrates that dynamic execution and powerful abstract interpretation can coexist, and that specialization caches need world/version assumptions.

### Critical mismatch

Phalcom's ordinary method identity/dispatch is selector/class based, not Julia-style type tuple dispatch. Do not infer from Julia that future Phalcom type annotations should select overloads. The transferable lesson is world-age/versioned assumptions and bounded inference, not dispatch semantics.

## 7. Smalltalk/Self: message sends, reflection, inline caches

### Problem solved

These systems execute highly dynamic object/message models efficiently.

### Relevant lesson

Keep message-send semantics primary. Static candidate analysis and optimization should preserve fallback lookup and use runtime guards/versioning where open-world mutation prevents permanent devirtualization.

### Phalcom fit

This precedent is particularly close for selector identity, class/object reflection, and cache invalidation. Still, Phalcom's syntax, modules, class/metaclass rules, blocks, and future typing must be specified independently.

## 8. Ruby: open classes, blocks, non-local control

### Relevant lesson

Ruby demonstrates how open object models and blocks complicate static analysis. Block invocation timing, captured state, non-local control, and method mutation must be modeled conservatively. Static “unique target” facts can become invalid if classes are reopened/mutated.

### Phalcom consequence

If Phalcom's reflection permits method mutation, optimizer-strength dispatch facts need version/closed-world assumptions even if source-level class reopening syntax is restricted.

## 9. Kotlin/Swift: flow refinement and rich type checking

Kotlin smart casts illustrate that flow refinement depends on stability: a mutable/captured property cannot always retain a refinement. Swift shows the engineering cost of sophisticated generic/subtyping inference and the importance of diagnostic architecture.

For Phalcom, the transferable rule is: **refinement validity depends on mutation/effect analysis**. Do not borrow their type relations into the semantic engine; checker/type skills own those.

## 10. GHC/OCaml/Scala: type-theory precedents

These systems provide deep lessons on inference, polymorphism, constraints, modules, unions/intersections, and internal IRs. They are relevant when the future checker needs formal type machinery.

This semantic-analysis skill should borrow only infrastructure lessons: normalize shared semantics, preserve source provenance through lowerings, separate inferred facts from source syntax, and solve recursive dependencies explicitly. Full type theory belongs in neighboring skills.

## 11. LLVM/Clang: IR and analysis discipline

LLVM demonstrates the power of a normalized IR with explicit CFG and reusable analyses. Clang demonstrates source fidelity/diagnostics needs.

Phalcom should learn the **two-representation lesson**: source-aware semantic identity and normalized body control/data representation can coexist. It should not lower early to VM bytecode and then try to reconstruct high-level semantics for every tool.

## 12. BEAM/Erlang: concurrency semantics

BEAM shows that concurrency analysis is shaped by isolation/fault semantics. Phalcom fibers are likely cooperative and may share mutable object state, so actor-style reasoning cannot be imported wholesale. The transferable question is to make suspension, message/future completion, cancellation, and shared-state effects explicit in future CFG/effect models.

## 13. Decision matrix

| Precedent | Borrow | Do not cargo-cult |
|---|---|---|
| rust-analyzer | stable IDs, semantic queries, incrementality | Rust's closed static resolution assumptions |
| Roslyn | immutable snapshots, symbol identity, IDE adapters | C# overload binding semantics |
| TypeScript | CFG narrowing, dynamic-language tooling | implicit unsoundness/compat compromises |
| Pyright/mypy | dynamic boundaries, stubs/contracts | Python's accumulated `Any` semantics |
| Julia | abstract interpretation, world/version assumptions | type-based dispatch |
| Smalltalk/Self | message semantics, cache guards | implementation-specific object layout |
| Ruby | reflection/block hazards | Ruby-specific scoping/control rules |
| LLVM/Clang | normalized CFG + source mapping | introducing IR before duplication warrants it |
| BEAM | explicit concurrency/fault model | actor isolation assumptions |

## 14. Review questions

When a proposal says “language X does this,” require answers:

1. Which exact problem in Phalcom is being solved?
2. Which assumptions of language X hold in Phalcom?
3. Does the imported mechanism affect selector identity, reflection, dynamic execution, or compatibility?
4. Is the proposal borrowing an algorithm or accidentally borrowing language semantics?
5. What is the smallest transferable invariant?

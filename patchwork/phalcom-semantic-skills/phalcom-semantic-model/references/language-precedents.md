# Language and Tooling Precedents

Use precedents to understand consequences, not to cargo-cult architecture.

## rustc

Study for:

- separation of source-oriented HIR and control-flow-oriented MIR;
- typed IDs/arenas rather than pervasive long-lived references;
- query/dependency thinking;
- type inference variables and obligations;
- trait solving/canonicalization;
- diagnostics tied to source spans;
- borrow/move analyses over a simplified IR.

Lesson for Phalcom: an analysis representation can differ from execution bytecode. Lower
complex source semantics into the representation where a question becomes simple.

Do not copy Rust's ownership/trait complexity unless Phalcom has the same semantic problem.

## rust-analyzer

Study for:

- IDE-first semantic queries;
- immutable/recomputable facts;
- syntax-to-semantic mapping;
- incremental invalidation;
- separating syntax identity from semantic definitions;
- tolerant analysis of incomplete code.

Lesson: editor performance comes from query boundaries and dependency-aware recomputation,
not from making each LSP request perform a clever local walk.

## C# / Roslyn

Study for:

- immutable syntax trees;
- semantic model as a query surface over syntax;
- symbols distinct from syntax nodes;
- compilations/snapshots;
- refactoring/diagnostics sharing symbol identity.

Lesson: "syntax node" and "symbol" are deliberately different concepts.

## TypeScript compiler

Study for:

- binder before checker;
- symbols/declarations/types as separate layers;
- contextual/bidirectional typing patterns;
- flow narrowing;
- pragmatic gradual/unsound boundaries;
- large-scale caching of type relations.

Lesson: a dynamic-source ecosystem needs explicit treatment of `any`/`unknown`-like states;
collapsing them makes diagnostics and soundness impossible to reason about.

Phalcom should not automatically adopt TypeScript's soundness compromises.

## Kotlin

Study for:

- smart casts / flow-sensitive type refinement;
- nullability integration;
- contracts and dataflow prerequisites for safe smart casts;
- distinction between mutable values and safely refinable values.

Lesson: a refinement is valid only while mutations/aliasing cannot invalidate the predicate.

## Pyright / mypy

Study for:

- overlaying static typing on Python's dynamic runtime semantics;
- module import/type-stub handling;
- gradual boundaries;
- narrowing;
- protocol/structural typing;
- practical error recovery.

Lesson: static semantics must model the dynamic language accurately rather than inventing a
cleaner but false object model.

## Julia

Study for:

- abstract interpretation over dynamic multiple dispatch;
- type/shape inference for optimization;
- union widening;
- method-instance specialization;
- world-age/open-world constraints.

Lesson: inferred runtime possibilities can be extremely useful without becoming declared
source contracts. Phalcom's LSP `ValueShape` has a similar conceptual role at a smaller scale.

## Swift

Study for:

- rich constraint solving;
- bidirectional/contextual type inference;
- protocol/generic constraints;
- diagnostics from failed constraint systems;
- SIL as a distinct lowered semantic/optimization IR.

Lesson: constraint solvers need designed diagnostics/provenance; "solver failed" is not a
useful user error.

## OCaml / Standard ML

Study for:

- principled inference and generalization;
- module/type separation;
- algebraic data types/pattern exhaustiveness;
- compact compiler IR design.

Lesson: type-theoretic elegance helps define invariants, but Phalcom's subtyping/dynamic
object model means whole-program HM inference is not automatically appropriate.

## GHC / Haskell

Study for:

- explicit typed intermediate core;
- constraints/evidence;
- type classes;
- effect/purity discipline;
- exhaustive pattern analysis.

Lesson: keep evidence/constraints explicit when semantics depends on them.

## Smalltalk / Self

Study for:

- message-send semantics;
- object/class/metaclass reflection;
- late binding;
- inline caches;
- dynamic environment tooling.

Lesson: Phalcom's semantic model must preserve message identity and reflective object
semantics even as typing grows.

## Python / CPython

Study for:

- dynamic object model and descriptors;
- module initialization/import cycles;
- C/native extension boundaries;
- runtime introspection;
- difference between typing metadata and runtime dispatch.

Lesson: modules are both namespaces and executable initialization units; typing tools that
ignore runtime import semantics can mislead users.

## Use this comparison method

For any borrowed idea, record:

1. What problem did that system solve?
2. Which assumptions make its solution valid?
3. Does Phalcom share those assumptions?
4. Which semantic commitment would copying it preclude?
5. Is the idea useful as representation, algorithm, language semantics, or only tooling?

Never cite precedent as sufficient justification by itself.

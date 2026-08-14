# Comparative Semantic Architectures

## rustc

Pipeline through AST/HIR/THIR/MIR demonstrates representation specialization. MIR makes control flow and analysis simpler; query system provides incremental dependency structure. Lesson: use the representation suited to the semantic question.

## rust-analyzer

IDE-first semantic database/query architecture, immutable syntax, IDs and demand-driven analysis. Lesson: semantic features should query shared infrastructure, not each parse/infer independently.

## Roslyn

Syntax trees + semantic model + bound trees/operations. Excellent example of public compiler semantic APIs reused by IDE/refactorings/analyzers.

## TypeScript compiler

Binder/checker split, symbol/type identities, control-flow nodes and narrowing. Demonstrates practical integration of dynamic JS semantics with rich static analysis.

## Pyright / mypy

Module/scope/type analysis over Python's dynamic object model; useful for gradual typing and incomplete-source behavior.

## GHC

Renamer -> typechecker -> Core pipeline; strong separation of name resolution from type inference and normalized core representation.

## Swift compiler

AST + constraint solver + SIL. Rich type-system power exposes solver/performance complexity; useful caution for Phalcom.

## Julia

Inference/abstract interpretation over dynamic multiple dispatch and typed IR for optimization. Particularly relevant if Phalcom later adds explicit typed dispatch/specialization.

## Clang/LLVM

AST semantic analysis separated from LLVM IR analyses. Useful for diagnostics/CFG/optimizer architecture, less directly applicable to dynamic dispatch.

## Biome

Its skills emphasize repository-specific invariants, thin/full inference and stale-data avoidance. Key transfer: semantic architecture rules should be operational, not generic textbook advice.

## Selection rule

Borrow architecture only after comparing:

```text
runtime dynamism
open-world mutation
module system
latency target
type soundness goal
reflection
concurrency
```

Phalcom's combination is unusual; no single language is a template.

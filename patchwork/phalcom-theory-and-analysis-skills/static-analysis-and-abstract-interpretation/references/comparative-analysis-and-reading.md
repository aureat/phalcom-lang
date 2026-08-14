# Comparative Static Analysis and Reading

## Core theory

- Cousot & Cousot: abstract interpretation foundations.
- Nielson, Nielson & Hankin: *Principles of Program Analysis*.
- Kildall: unified dataflow framework.
- Kam/Ullman and classic monotone-framework work.
- Cytron et al.: SSA construction.
- Sharir/Pnueli: interprocedural dataflow foundations.
- Reps/Horwitz/Sagiv: IFDS framework for distributive subset problems.
- Muchnick / Cooper & Torczon for compiler analysis engineering.

## Implementations to study

### rustc

MIR-based dataflow, borrow checking, move/init analysis, trait obligation solving, incremental queries. Key lesson: lower to a representation where the analysis is simpler.

### rust-analyzer

Incremental semantic queries and IDE latency constraints. Key lesson: stable identities and demand-driven computations.

### TypeScript

Control-flow narrowing over a highly dynamic/structural type system; demonstrates practical flow-sensitive type facts and complexity tradeoffs.

### Pyright / mypy

Static approximation over Python's dynamic runtime, protocols, gradual `Any`, narrowing and module graphs.

### Julia inference

Abstract interpretation/type inference for dynamic multiple dispatch and optimization; useful comparison for runtime class/type lattices and specialization.

### JVM/.NET analyzers

Roslyn dataflow/operation trees and framework analyzers demonstrate public semantic-model/query architecture.

### LLVM

Dominators, MemorySSA, alias analysis and optimizer pass contracts. Useful later for Phalcom optimizer sophistication.

## Borrow principles, not surface APIs

For each precedent ask:

- Is analysis advisory or correctness-enforcing?
- Is world closed/open?
- Are calls statically resolved?
- Is heap mutable/aliased?
- Is reflection allowed?
- What latency budget exists?

Phalcom often shares dynamic-language constraints more with Python/Julia than with Rust's statically resolved core.

# Pyrefly transfer package for Phalcom

This package turns Pyrefly's implementation ideas into an implementation-ready direction for Phalcom's type checking, annotation, semantic analysis, and incremental tooling.

## Documents

- [Executive report](executive-report.md) — decision summary, efficiency model, architectural lessons, risks, and recommended Phalcom target state.
- [Implementation breakdown](implementation-breakdown.md) — phased design, data structures, crate seams, tests, benchmarks, and acceptance gates.

+## Deep implementation dossiers

These files are the implementation-level companion set. They record concrete execution paths, state machines, data structures, cache keys, ownership, performance mechanisms, and Phalcom transfer rules.

- [Constraint solving and fixed points](01-constraint-solving-and-fixed-points.md)
- [Semantic architecture and execution model](02-semantic-architecture-and-execution-model.md)
- [Semantic behavior: bindings, flow, exports, and modules](03-semantic-behavior-bindings-flow-and-modules.md)
- [Type representation, equality, and canonicalization](04-type-representation-equality-and-canonicalization.md)
- [Answer tables, query cells, and cycle-safe publication](05-answer-tables-query-cells-and-cycle-publication.md)
- [Dependency graph and incremental invalidation](06-dependency-graph-and-incremental-invalidation.md)


## Source snapshot

Pyrefly observations are pinned to commit [`43467e64e36550f232a18e89f24fda79b1020b6b`](https://github.com/facebook/pyrefly/tree/43467e64e36550f232a18e89f24fda79b1020b6b), inspected 2026-08-22. Primary references include:

- [Pyrefly architecture](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/ARCHITECTURE.md)
- [Pyrefly graph indexes](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_graph/src/index.rs)
- [Pyrefly calculation cells](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_graph/src/calculation.rs)
- [Pyrefly type heap](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/heap.rs)
- [Pyrefly type representation](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/types.rs)
- [Pyrefly semantic equality](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/equality.rs)
- [Pyrefly type simplification](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/simplify.rs)
- [Pyrefly staged module state](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/steps.rs)
- [Pyrefly dependency state](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/state.rs)
- [Pyrefly answer solver](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/alt/answers_solver.rs)
- [Pyrefly speed and memory measurements](https://pyrefly.org/blog/speed-and-memory-comparison/)
- [Pyrefly diagnostic invalidation improvements](https://pyrefly.org/blog/2026/02/06/performance-improvements/)

The preliminary Phalcom material remains useful context:

- [`pyrefly_architecture_analysis_for_phalcom_document_1.md`](../pyrefly/pyrefly_architecture_analysis_for_phalcom_document_1.md)
- [`pyrefly_architecture_analysis_for_phalcom_document_2.md`](../pyrefly/pyrefly_architecture_analysis_for_phalcom_document_2.md)
- [`pyrefly_architecture_analysis_for_phalcom_document_3_expanded.md`](../pyrefly/pyrefly_architecture_analysis_for_phalcom_document_3_expanded.md)

## Evidence labels

- **OBSERVED / PYREFLY** — directly inspected in the pinned source or official Pyrefly documentation.
- **CURRENT / PHALCOM** — present in the current checkout, including experimental or dirty work that is not yet accepted as shipped architecture.
- **PROPOSED / PHALCOM** — transfer design or implementation recommendation.
- **DEFERRED / RISK** — deliberately excluded until semantics, measurements, or safety boundaries are established.

## Core thesis

Pyrefly's speed is compositional. It combines cheap semantic identities, dense indexed tables, module-oriented work units, parallel execution, staged immutable publication, dependency-aware invalidation, canonical type operations, cycle-aware answer solving, and explicit complexity guardrails. Phalcom should transfer those seams and invariants rather than copy Python-specific type rules or source code.

The target is a hybrid architecture:

```text
source snapshot
    -> module exports and declaration surfaces
    -> bindings, scopes, flow facts, and dependency keys
    -> demand-driven callable/type queries
    -> constraint worklist and SCC solving
    -> immutable generation snapshot
    -> CLI diagnostics and LSP queries
```

This package is a design and implementation handoff. It does not claim that the full target exists, and it does not modify Phalcom Rust code.

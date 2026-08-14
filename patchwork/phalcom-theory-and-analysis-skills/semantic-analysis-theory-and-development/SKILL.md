---
name: semantic-analysis-theory-and-development
description: Use when architecting, extending, or refactoring Phalcom semantic analysis across parsing, name resolution, semantic IDs, lowering/HIR, control flow, dispatch, facts, callable summaries, module graphs, incremental queries, checker/prover integration, LSP/refactoring APIs, or when deciding which semantic representation should own a new language feature.
compatibility: Designed for Phalcom compiler/LSP/checker/static-prover architecture and Rust implementation; complements phalcom-semantic-model and semantic-analysis-development.
---

# Semantic Analysis Theory and Development for Phalcom

This skill is the architectural bridge between language meaning and concrete analysis passes. It explains where semantic identities, normalized representations, facts, summaries and queries should live as Phalcom grows from LSP intelligence into shared checker/prover/compiler infrastructure.

**REQUIRED BACKGROUND:** `phalcom-semantic-model` defines doctrine; `semantic-analysis-development` is the current code playbook. Use `programming-language-semantics`, `type-theory`, and `static-analysis-and-abstract-interpretation` for theory.

## Target model

```text
source/recovered AST
-> declaration surfaces + lexical scopes
-> resolved semantic identities
-> semantic lowering / HIR when needed
-> body CFG/IR when needed
-> shared facts/summaries
   ├ runtime-shape
   ├ type
   ├ effect
   └ proof/refinement
-> immutable snapshot/query database
-> LSP | checker | prover | lints/refactors | compiler diagnostics | optimizer
```

The stack VM remains the execution target. A semantic IR exists only to make semantic questions shared and explicit.

## Non-negotiable architecture rules

- Bind before infer.
- Use one typed semantic ID space per concept; strings/ranges are not durable identity.
- AST is source structure; introduce HIR/CFG only when consumers repeatedly reconstruct the same semantics/control flow.
- Runtime `ValueShape`, checker `Type`, effects and proof facts are separate domains with explicit bridges.
- Semantic queries return semantic data, not LSP protocol objects.
- Incomplete source must produce coherent partial semantics without panics or false certainty.
- Static dispatch approximation must match selector/class/metaclass/access runtime rules.
- Interprocedural facts use summaries/dependency edges/SCCs.
- Incremental state is dependency-driven and generation-coherent.
- Modules/packages are semantic graph nodes, not textual include mechanics.
- Native/core behavior requires semantic contracts/stubs.
- Facts retain provenance/trust.
- Checker/prover reuse scope/lowering/CFG; they do not build parallel semantic worlds.
- Optimizer consumes only sufficiently sound facts; editor heuristics stay advisory.

## Current starting point

`phalcom-lsp/src/semantic/` already provides semantic IDs, scopes, surfaces, occurrences, runtime-shape/provenance facts, dispatch, structured flow, callable summaries, interprocedural inference, module graph/invalidation, worker engine and immutable snapshots. Treat it as infrastructure to generalize, not disposable LSP code. Re-inspect current source before changes.

## Development workflow

1. State normative dynamic/static rule.
2. Identify semantic entities and ID lifetime.
3. Choose source-surface vs HIR vs CFG ownership.
4. Define recovery-aware resolution/lowering.
5. Define fact domains/program points.
6. Define dispatch/effects/interprocedural dependencies.
7. Define type/proof bridge if needed.
8. Define module/project invalidation.
9. Expose protocol-neutral semantic query.
10. Test semantic core, incremental equivalence and runtime correspondence.
11. Measure rebuild/query performance.

## Reference map

- Pipeline: [references/semantic-analysis-role-and-pipeline.md](references/semantic-analysis-role-and-pipeline.md)
- Binding: [references/binding-and-name-resolution.md](references/binding-and-name-resolution.md)
- IDs/arenas: [references/semantic-identities-arenas-and-interning.md](references/semantic-identities-arenas-and-interning.md)
- HIR: [references/hir-and-semantic-lowering.md](references/hir-and-semantic-lowering.md)
- CFG/IR: [references/control-flow-and-semantic-ir.md](references/control-flow-and-semantic-ir.md)
- Query systems: [references/attribute-grammars-and-query-systems.md](references/attribute-grammars-and-query-systems.md)
- Facts/provenance: [references/facts-provenance-and-uncertainty.md](references/facts-provenance-and-uncertainty.md)
- Dispatch: [references/dynamic-dispatch-and-member-resolution.md](references/dynamic-dispatch-and-member-resolution.md)
- Flow/interprocedural: [references/flow-and-interprocedural-semantics.md](references/flow-and-interprocedural-semantics.md)
- Checker: [references/type-checker-integration.md](references/type-checker-integration.md)
- Prover/effects: [references/prover-effects-and-refinement-integration.md](references/prover-effects-and-refinement-integration.md)
- Modules/packages: [references/modules-packages-and-project-semantics.md](references/modules-packages-and-project-semantics.md)
- Incrementality: [references/incrementality-snapshots-and-queries.md](references/incrementality-snapshots-and-queries.md)
- Recovery: [references/recovery-and-incomplete-programs.md](references/recovery-and-incomplete-programs.md)
- LSP/refactoring: [references/lsp-lints-and-refactoring-consumers.md](references/lsp-lints-and-refactoring-consumers.md)
- Runtime conformance: [references/compiler-runtime-conformance.md](references/compiler-runtime-conformance.md)
- Rust: [references/rust-implementation-patterns.md](references/rust-implementation-patterns.md)
- Migration: [references/architecture-migration-strategy.md](references/architecture-migration-strategy.md)
- Testing/performance: [references/testing-performance-and-observability.md](references/testing-performance-and-observability.md)
- Precedents: [references/comparative-semantic-architectures.md](references/comparative-semantic-architectures.md)
- Review tests: [references/review-and-validation-scenarios.md](references/review-and-validation-scenarios.md)

## IR decision rule

Introduce shared HIR/body IR when several consumers duplicate desugaring/CFG/program-point logic or checker/prover require stable normalized operations. Do not introduce one merely to imitate rustc or LLVM.

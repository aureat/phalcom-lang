# Architecture Migration Strategy

The goal is evolution without rewriting working LSP semantics.

## Phase 1 — codify current semantic API

- document existing IDs/facts/snapshots;
- remove consumer-specific duplicate inference;
- ensure LSP queries use shared semantic engine;
- strengthen tests/invalidation.

## Phase 2 — extract reusable semantic crate boundary

If checker/compiler need direct reuse, move generic semantic components out of `phalcom-lsp` into a crate such as conceptual `phalcom-semantic` without changing behavior.

Candidates:

```text
ids
scope/surface/occurrence
module graph
dispatch
facts/summaries
```

Keep LSP adapter in `phalcom-lsp`.

## Phase 3 — semantic body lowering/CFG

Introduce when type checker/prover/control analyses need common program points. Migrate existing structured-flow tests to assert equivalence.

## Phase 4 — add type domain/checker

Attach `TypeId`/constraints to existing semantic identities. Reuse dispatch/scope/module data.

## Phase 5 — add effect/proof domains

Extend callable summaries/CFG analyses; keep proof solver separate.

## Phase 6 — compiler consumption

Compiler may consume resolved/lowered semantic representation to avoid duplicate name/selector logic, but migrate carefully because compile-time performance and error recovery differ from LSP.

## Compatibility principles

- preserve public LSP behavior during internal migration;
- compare old/new semantic answers on fixture corpus;
- avoid flag-day rewrite;
- keep runtime semantics unchanged unless separately specified;
- measure before/after latency/memory.

## Deletion rule

A migration is incomplete while old parallel inference/resolution remains live. Once new shared owner is verified, delete obsolete path to prevent drift.

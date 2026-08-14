# Phalcom Static Prover Roadmap

A staged prover should deliver value before attempting whole-language theorem proving.

## Stage 0 — shared semantic prerequisites

Require:

- stable semantic IDs;
- reusable CFG/semantic IR;
- precise control outcomes;
- type facts;
- effect/call summaries;
- provenance;
- module dependency revisions.

Do not build prover as an AST-only sidecar.

## Stage 1 — local assertions and contracts

Prove:

- constant boolean assertions;
- local integer comparisons/arithmetic;
- Option/tag presence;
- simple `@requires` at call sites;
- simple `@ensures` over locals/result;
- unreachable branches.

Use abstract interpretation + simple SMT arithmetic.

## Stage 2 — structured control flow

Add:

- branch/path conditions;
- loops with user/derived invariants;
- returns/throws;
- collection length/range checks.

## Stage 3 — modular call proofs

Use verified/trusted callable contracts and effects. Handle recursion through contracts/SCCs.

## Stage 4 — heap/object invariants

Model receiver fields, constructor invariants, public mutator preservation and `old` state.

## Stage 5 — protocols/generics

Substitute generic contracts, prove protocol-level obligations, exploit sealed ADTs/exhaustiveness.

## Stage 6 — stdlib/native models

High-value contracts for:

```text
String/Bytes bounds and encoding
Path lexical operations
IO partial reads/writes
process status
collections
numeric domains
```

## Stage 7 — fibers/effects

Reason across suspension conservatively; later add ownership/interference models if needed.

## Static versus runtime enforcement

For an obligation:

```text
Proven -> omit runtime check in typed mode if assumptions stable
Disproven -> checker diagnostic
Unknown -> require/keep runtime check or allow dynamic mode according to policy
```

This makes static proof and typed-runner complementary.

---

## Architecture gates between stages

The roadmap is capability-gated, not calendar-gated. Do not advance a stage merely because previous feature demos work.

### Gate A: semantic substrate

Before solver integration, require:

- stable semantic IDs and immutable snapshots;
- reusable CFG/control outcomes;
- explicit type/effect/call-summary provenance;
- contract lowering with source origins;
- incremental dependency graph.

### Gate B: proof IR correctness

Before broad SMT use, require:

- sort-checked solver-independent IR;
- solver-independent VC unit tests;
- explicit `UnknownReason` taxonomy;
- negative tests for assumptions/assertions/calls/heap versions;
- backend isolation and budgets.

### Gate C: modular contracts

Before proof-dependent runtime optimization, require:

- verified/trusted contract distinction;
- frame/effect summaries;
- dispatch/open-world policy;
- native trust manifest/conformance;
- proof cache invalidation by semantic dependencies.

### Gate D: optimization consumption

Before eliminating checks:

- proof assumptions must be guaranteed by execution mode;
- compiler preserves correspondence to verified semantics;
- stale proof guards/invalidation tested;
- fallback runtime behavior defined for Unknown.

A narrow but sound prover with excellent diagnostics and integration is more valuable than a broad unsound one.

# Requirements Analysis: Phalcom’s Static-Typing Mission and Pyrefly Transfer

**Status:** Working analysis, implementation-input only<br>
**Date:** 2026-08-22<br>
**Repository baseline inspected:** `078a5e0a` (`docs: complete Pyrefly transfer dossiers`)<br>
**Scope:** Phalcom semantic typing, module compilation, CLI, LSP integration, flow/dispatch, and foundations required before static proving.<br>
**Deliverables:** [Specification 01](01-type-kernel-and-type-language.md), [Specification 02](02-project-semantic-database-and-tooling.md), and [Specification 03](03-flow-dispatch-and-proof-foundations.md).

## 1. Decision context and evidence discipline

Phalcom has crossed the important threshold from a type-design document to an executable local checker. The implementation has a canonicalizing `TypeStore`, a three-state evidence model, basic annotation lowering, subtype and assignability relations, local declaration and expression checking, semantic diagnostics, a compiler entry point, and an LSP diagnostic conversion adapter. That is useful foundation. It is not yet a project semantic system, a complete source type language, or a shared compiler/editor type snapshot.

This analysis distinguishes four kinds of evidence so that a future implementation does not accidentally promote a draft into a current language promise.

| Label | Meaning | Examples in this analysis |
|---|---|---|
| **Observed** | Present in the checked-out Rust source and/or covered by a current test. | `TypeStore`, local `Checker`, one-source CLI checking, syntax-only LSP publication. |
| **Ratified design** | A typing design document states a normative decision, but its implementation may still be absent. | Separate type relations, owner-qualified type parameters, explicit variance. |
| **Proposed design** | Useful direction which needs a language-design decision before it becomes a compatibility commitment. | Strict CLI modes, protocols, static proving surface, full bidirectional checking. |
| **Forward ontology** | The user-provided, currently untracked `docs/spec/typing/ontology.md`; treated as target architecture, not evidence of shipped behavior. | Stratified value/type/kind universe, HKT direction, future `Constraint` kind. |

The Pyrefly material is a repository-grounded architectural transfer study, pinned to Pyrefly commit `43467e64e36550f232a18e89f24fda79b1020b6`. It is evidence for staging, identities, snapshots, query evaluation, invalidation, bounded solving, and measurement. It is **not** authority for importing Python gradual-typing semantics into Phalcom. The transfer’s central constraint is accurate: take architecture and operating discipline; preserve Phalcom message-send semantics, dynamic boundaries, classes/metaclasses, and eventual higher-kinded type theory. See [Pyrefly transfer README](../pyrefly-transfer/README.md) and [transfer architecture](../pyrefly-transfer/11-phalcom-transfer-architecture-and-type-philosophy.md).

## 2. Observed baseline

### 2.1 Type kernel and checker

`phalcom-semantic` represents `Never`, `Unit`, nominal, applied, union, tuple, record, callable, parameter, and inference types in an interned store ([`TypeData`](../../../../phalcom-semantic/src/types/store.rs#L35-L56)). Its union constructor flattens, deduplicates, removes `Never`, and collapses empty/singleton cases ([`TypeStore::union`](../../../../phalcom-semantic/src/types/store.rs#L209-L249)). `TypeKnowledge` deliberately distinguishes `Known`, `Unknown`, and `Dynamic`, together with provenance and a conservative “sound to reject” predicate ([`evidence.rs`](../../../../phalcom-semantic/src/types/evidence.rs)). This is a good policy seam: absence of evidence must not silently become a proof of type safety or a hard error.

The current relation engine handles nominal parent links, unions, tuples, records, callables, and applied types ([`relation.rs`](../../../../phalcom-semantic/src/types/relation.rs)). Its representation capacity exceeds source support: annotation lowering accepts references, builtins, `Never`, `Unit`, `Dynamic`, and unions, while application, tuple, and callable annotation forms are explicitly deferred ([`annotation.rs`](../../../../phalcom-semantic/src/types/annotation.rs#L103-L121)). The AST already has those annotation forms ([`TypeAnnotation`](../../../../phalcom-ast/src/ast.rs#L404-L446)); the parser currently only accepts a qualified reference at that position ([`parse_type_annotation`](../../../../phalcom-ast/src/parser.rs#L1395-L1419)).

The checker pre-registers top-level class surfaces then checks a single parsed program ([`Checker::check_program`](../../../../phalcom-semantic/src/checker/mod.rs)). Local environments are keyed by spelling and lexical scopes; each context instantiates a local surface dispatch resolver and a hardcoded standard native surface ([`CheckerContext`](../../../../phalcom-semantic/src/checker/context.rs)). Expression synthesis covers a meaningful Phase-2 subset, but its `TypedExpression` constraints do not form a general production solve phase ([`expression.rs`](../../../../phalcom-semantic/src/checker/expression.rs)). `LocalConstraintSolver` is intentionally narrow: a one-pass binding map with no occurs check, worklist, substitution environment, or solver status ([`constraint.rs`](../../../../phalcom-semantic/src/types/constraint.rs)).

### 2.2 Module, compiler, CLI, and LSP boundary

The compiler has a strong structural seam: module discovery and linking produce interfaces and distinct semantic/runtime graphs. Its project/package paths discover and link modules but do not typecheck their bodies; inline source and standalone-module paths call `run_semantic_typecheck` ([`compile.rs`](../../../../phalcom-core/src/modules/compile.rs#L125-L450), [`run_semantic_typecheck`](../../../../phalcom-core/src/modules/compile.rs#L513-L552)). The check command likewise parses one path or inline source and invokes that entry point, despite help text still calling it syntax checking ([`cli.rs`](../../../../phalcom-core/bin/phalcom/cli.rs#L107-L152), [`cmd_check`](../../../../phalcom-core/bin/phalcom/cli.rs#L315-L354)). It assigns the checked program `ModuleId::core()`, so a diagnostic has no authentic project/module identity.

`phalcom-modules` currently exports untyped declaration/import/export metadata rather than typed declaration headers ([`interface.rs`](../../../../phalcom-modules/src/interface.rs)). The graph model already distinguishes dependency purposes including interface, type, superclass, protocol, constraint, callback, and ADT edges ([`graph.rs`](../../../../phalcom-modules/src/graph.rs)); this is exactly the right place to add type-header and body-check scheduling without collapsing semantic and runtime cycle policy.

LSP already converts a semantic diagnostic to protocol diagnostics with source `phalcom-typecheck` ([`diagnostics.rs`](../../../../phalcom-lsp/src/diagnostics.rs#L35-L74)), but publication currently sends only parser diagnostics ([`backend.rs`](../../../../phalcom-lsp/src/backend.rs#L298-L315)). Its existing `ValueShape` engine is explicitly a runtime/editor analysis, not a language type ([`facts.rs`](../../../../phalcom-lsp/src/semantic/facts.rs)); it has useful worklist, cancellation, and bounded-convergence mechanics, but it must remain an advisory sibling until the formal checker publishes type facts.

### 2.3 Contracts and proving

`@requires` and `@ensures` have runtime weaving specifications and compiler paths, not static verification. In fact, their current documentation identifies a result-name implementation discrepancy and incomplete metadata plumbing ([requires](../../../spec/current/decorators/requires.md), [ensures](../../../spec/current/decorators/ensures.md)). `phalcom-semantic` presently has no proof IR, verification-condition generator, solver boundary, or proof-result domain despite aspirational crate wording. Any static-proving roadmap must begin by making typed control-flow, effects, call summaries, contracts, and dynamic/native boundaries explicit; it must not claim that runtime guards are proof evidence.

### 2.4 Verification observed during this analysis

The following commands passed against the baseline above:

```text
cargo test -p phalcom-semantic --tests
# 5 unit tests + 5 checker tests + 8 Phase-2 expression tests passed

cargo check -p phalcom-core -p phalcom-lsp
```

This is focused health evidence only. It does not establish sound generic subtyping, project/import-graph checking, incremental invalidation, LSP static diagnostics, editor cancellation correctness, inherited dispatch, type-directed constructor semantics, or static proving.

## 3. Principal gaps and requirements

| ID | Gap observed now | Requirement | Owning specification |
|---|---|---|---|
| RA-01 | Kind records exist but all constructed types default to `Type`; application does not validate constructor kind or arity. | Define a stratified canonical type kernel and kind checker before exposing generic source syntax. | [01](01-type-kernel-and-type-language.md) |
| RA-02 | Applied arguments are universally covariant; parameter identities are bare `u32`; no substitution model exists. | Make variance, substitution, alpha-equivalence, recursive equality, and relation outcomes explicit and terminating. | [01](01-type-kernel-and-type-language.md) |
| RA-03 | AST can represent annotations parser cannot parse; annotation lowering is intentionally partial. | Add a complete, diagnostic-quality annotation grammar and lowering pipeline only after kernel decisions are executable. | [01](01-type-kernel-and-type-language.md) |
| RA-04 | Checker environment is source-local; linked interfaces have no typed headers. | Build typed module interfaces first, then schedule body checking across the semantic import graph. | [02](02-project-semantic-database-and-tooling.md) |
| RA-05 | Snapshots/invalidation are skeletal; source identities and temporary inference IDs can collide across sessions. | Introduce stable persistent identities, revisioned immutable snapshots, dependency recording, and explicit query states. | [02](02-project-semantic-database-and-tooling.md) |
| RA-06 | CLI help and implementation disagree; project/package modes skip typing. | Make CLI mode selection, identities, output schema, exit behavior, and strictness policy deliberate and testable. | [02](02-project-semantic-database-and-tooling.md) |
| RA-07 | LSP checker diagnostics adapter is unused; editor semantic shapes are not formal types. | Publish compiler-owned type diagnostics from a coherent snapshot while retaining `ValueShape` as advisory analysis. | [02](02-project-semantic-database-and-tooling.md) |
| RA-08 | Constraint collection/solving is local; branches/loops do not reach a dataflow fixed point. | Use bidirectional expression checking, provenance-bearing constraints, CFG facts, SCC/worklist solving, and bounded widening. | [03](03-flow-dispatch-and-proof-foundations.md) |
| RA-09 | Dispatch is exact-surface only; `super`, inheritance, class side, generics, protocols, and dynamic/reflection boundaries are incomplete. | Define one ordered formal dispatch query and specialization model consistent with actual message sends. | [03](03-flow-dispatch-and-proof-foundations.md) |
| RA-10 | No static proof substrate exists. | Establish typed/effect/contract facts and `Proven`/`Disproven`/`Unknown` proof interfaces without overclaiming automated verification. | [03](03-flow-dispatch-and-proof-foundations.md) |

## 4. Target architecture

The target is one compiler-owned semantic product, produced in dependency order and published immutably. LSP, CLI, compiler code generation, future prover, and advisory editor analysis consume that product rather than recreating a second type checker.

```text
Source revisions
   │ parse and recover
   ▼
untyped module interfaces ──► typed header shells ──► semantic-interface SCCs
   │                                  │                        │
   │                                  ▼                        ▼
   └──────────────────────────► bindings + CFG ──► demanded type/dispatch queries
                                                             │
                       canonical TypeStore ◄── constraints + relation engine
                                                             │
                                                             ▼
                                         immutable SemanticTypeSnapshot
                                      ╱          │          │            ╲
                           compiler diagnostics  CLI       LSP      future prover
                                                               │
                                            advisory ValueShape remains separate
```

The pipeline must preserve two separations:

1. **Semantic levels.** Evaluation classifies runtime values; the type system classifies semantic type terms; the kind system classifies type constructors. The ontology’s `value : type :: kind` direction is a useful guard against `Type :: Type` collapse and against making reflection dictate static type identity. See [ontology sections 1–4](../../../spec/typing/ontology.md).
2. **Knowledge states.** A known static type, lack of evidence, an explicit dynamic escape, an unresolved dependency, an inference variable, a contradiction, and a proof unknown are distinct states with different diagnostics and cacheability. They must never be encoded as a convenient single `TypeId`.

## 5. What may be transferred from Pyrefly

### Take directly

- Cheap stable IDs, dense tables, staged semantic products, and immutable publication.
- Explicit query-cell lifecycle with cycle/SCC behavior, cancellation, revision stamps, and no publication of partial answers.
- Dependency keys recorded at actual semantic reads, reverse invalidation, and bounded local recomputation.
- Canonical type storage with equality/normalization budget policy separated from user-facing type display.
- Worklist/SCC solver strategy, status-rich outcomes, observability, benchmark fixtures, and deterministic regression tests.

### Adapt to Phalcom

- Module-oriented scheduling must sit on `phalcom-modules` interfaces and semantic graphs, preserving its separate runtime-graph policy.
- Query identities must include Phalcom module/declaration/callable/member side and selector context, rather than Python module/attribute assumptions.
- One worker plus immutable snapshots is the immediate safe concurrency model. Parallel evaluation comes only after dependency and publication invariants are measured.
- LSP can reuse existing cancellation and worklist lessons, while type facts flow from the compiler-owned type snapshot.

### Do not transfer

- Python’s `Any`, `Unknown`, import fallback, class/object, descriptor, overload, and structural-protocol semantics.
- Type-directed changes to Phalcom runtime selector identity or dynamic dispatch. A formal type surface constrains static acceptance; it does not rewrite the value model.
- Raw-pointer/unsafe answer-slot optimizations, global locks around the whole solver, or unbounded fixed-point loops.
- Pyrefly performance numbers as targets without a Phalcom workload and baseline measurement.

## 6. Dependency order and decision gates

The three specifications are deliberately ordered. Specification 01 establishes what a type means and how a relation can be trusted. Specification 02 establishes where type facts live, how module headers become available before bodies, and how every product observes coherent revisions. Specification 03 then uses those prerequisites to type real Phalcom expressions, dispatch, generic applications, flow, and eventually contracts.

Before implementation begins, record decisions for these gates in the typing design series or an ADR:

- Is the untracked ontology ratified as the semantic-level and kinding model? In particular, are `Constraint` and higher kind kinds deferred, or is their representation a current invariant?
- Which source forms are committed for the first generic-annotation release: named applications only, tuple/callable forms, type lambdas, and/or associated types?
- Which nominal classes are mutable, and therefore must use invariant generic parameters or read/write capability splitting?
- What static contract/proof claim is intended: readiness-only facts, bounded automatic checking, or an external proof-engine interface?
- Is `--types=strict` a compatibility contract now, or an experiment behind an explicit unstable flag?

No implementation phase should silently answer those questions by whichever data structure is quickest to add.

## 7. Success definition

The typing mission is achieved incrementally, not by merely accepting more programs. Each specification supplies executable acceptance conditions. Cross-cutting completion means all of the following are true:

- A diagnostic from CLI and LSP can be traced to the same typed snapshot, source revision, stable code, and evidence path.
- A project check schedules typed interface SCCs before bodies and invalidates only dependents of changed semantic facts.
- `Any`, `Dynamic`, `Unknown`, `Never`, inference variables, and proof uncertainty retain distinct formal and diagnostic meaning.
- Generic applications obey declared kind, arity, variance, and capture-safe substitution; no mutable generic becomes covariant by implementation accident.
- Type relation, normalization, constraint solving, CFG dataflow, and query evaluation terminate under cycles and adversarial inputs with deterministic bounded outcomes.
- LSP keeps parser diagnostics and advisory runtime-shape analysis, while static type diagnostics derive only from the shared formal snapshot.
- Static proving, if enabled later, reports proof status honestly and treats dynamic, native, reflective, and effectful boundaries as proof obligations or `Unknown`, never as fabricated facts.

The requirements do not authorize language behavior changes by themselves. They are an implementation contract to be ratified and decomposed into owned work units.

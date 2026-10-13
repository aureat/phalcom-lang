---
id: MODL001.C2.P4
category: MODL
program: MODL001
checkpoint: MODL001.C2
kind: implementation-plan
status: IN_PROGRESS
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - MODL001.C2.P2
  - MODL001.C2.P3
follows: MODL001.C2.P3
supersedes: null
---

# MODL001.C2.P4 — phase-aware source dependencies

> **Approved architectural amendment (2026-10-13):** Execute under the user-supplied `MODL001.C2.P4-architectural-ruling-and-implementer-directive.md` (reviewed collections HEAD `b679028c`). The adopted decisions are recorded in `CHECKPOINT.md`. The task sequence below supersedes the original T1–T4 outline where it differs; historical dependencies on P2/P3 are preserved. This plan does not claim those historical plans completed.

## Approved execution sequence and gates

- **T0:** Record ruling and amend checkpoint/P2/P3/P4 before source edits.
- **T1:** Introduce modules-owned item-level `SourceImportBindingId` and symbolic import table; distinct `SemanticLinkedProgram` has no runtime slots or initialization order. Test aliases/multiple items and pre-finalization symbolic resolution. Add type-enforced compile/materialize stage rejection evidence at this boundary.
- **T2:** Migrate semantic resolver and checker value seeding from executable slots to symbolic targets; retain `BindingId → SourceImportBindingId` provenance.
- **T3:** Retain consumer-owned use evidence with query products; aggregate explicit module coverage and content-sensitive fingerprints. Missing, stale, unsupported, unobserved, or incomplete evidence is Runtime. Test annotations/constraints, direct value/call use, mixed use, shadowing, independent selected items, and cold/incremental parity.
- **T4:** Close generated requirements before DAG validation: superclass, type-form/class objects, contextual realization, patterns, runtime attributes, runtime type/reification roots, selected external callables/witnesses, and static constants. Unsupported families prevent interface-only proof. Do not classify from declaration kind or occurrence role.
- **T5:** Finalize a distinct executable product from symbolic links plus phase evidence plus generated requirements. Jointly project deterministic slots and edges, strictly validate the DAG, publish order atomically. Test unknown fallback, unused imports, whole-module/re-export policy, semantic-cycle success and runtime-cycle rejection.
- **T6:** Cut program lowering/materialization to finalized authority. Missing external mapping is an internal contract error; no late slot insertion. Legacy standalone compatibility must be explicitly isolated from finalized program paths.
- **T7:** Run owner-specific focused tests for all categories in the ruling's verification matrix, including missing-mapping fail-closed and no uncovered `GetLinked`.
- **T8:** Prove the minimal protocol/default → runtime adapter → interface-only protocol fixture and paired runtime-cycle negative. Keep shadow iteration/concurrency source and production bootstrap unchanged.

Modules owns transport, identity, slots and graphs; semantic owns the proof; core owns orchestration and executable semantic projection. `ImportBindingId` is executable-only. Generated requirements may name canonical owners without an authored imported binding. Evidence must survive query reuse and retract on invalidation; a generic snapshot status is not phase completeness. Completion requires T1–T8 evidence, not merely introducing the staged types.

## 0. Executor contract

Implement a sound source-to-runtime dependency path under the architecture below. The work may adapt private helpers and data layout to the live tree. It must not add import syntax, weaken cycle validation, or infer interface-only status from missing runtime edges. Stop for architectural consultation if semantic usage cannot be proven before executable linking, or if any runtime read cannot be represented in the finalized runtime graph.

## 1. Goal

Let canonical semantic analysis prove when an ordinary selected import is needed only to resolve declarations, then build and validate the executable initialization graph from those phase facts while preserving runtime reads and rejecting every runtime cycle.

## 2. Checkpoint acceptance objective

This is a follow-up implementation plan under `MODL001.C2`, not full checkpoint closure. It supplies the module system with a tested boundary between semantic/interface dependencies and eager runtime dependencies. It is a prerequisite for reconsidering the deferred Universe iteration child-module split; it does not itself perform that split or authorize production Universe cutover.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom
branch: feat/telos-collections
revision: e5988ad80a04769740462de98c51d58b56bd314d
predecessor: COLL006.C1 S4 cohesive shadow iteration relocation
```

Verified live facts:

- `phalcom-ast` has only ordinary module imports, selective imports, direct re-exports, and package exposures; no import has a phase modifier.
- `phalcom-modules::ImportSurface` has only `Module`, `Selective`, and `ReExport`.
- `ModuleLinker::collect_imports_and_graphs` currently emits a runtime edge for every source import/re-export and constructs a symbolic read for every imported binding.
- `ModuleLinker` validates the runtime DAG before returning the linked program.
- `ReferenceKind::InterfaceOnly` and `DependencyPhase::InterfaceOnly` exist, but no production source-link path emits an interface-only reference.
- The semantic checker consumes canonical linked import targets. It has no per-import use-phase product; occurrence roles do not distinguish type-position references from runtime reads.
- Runtime linked reads and import binding IDs are currently coupled. Removing only a runtime graph edge would leave a runtime binding/read that the compiler and materializer still consume.
- Existing graph tests construct an `InterfaceOnly` reference by hand; they do not prove a source-to-linker path.

Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

## 4. Required reads before implementation

- `AGENTS.md` and the repository implementation-record lifecycle convention.
- `docs/implementation/MODL001-module-architecture/C2-module-project-model/CHECKPOINT.md`.
- `P2-module-linking-graphs-and-lsp.md` and `P3-module-runtime-initialization-and-execution.md`.
- `docs/specs/README.md`, `docs/specs/universe.md`, `docs/specs/generic-constraints.md`, and the relevant class/trait specifications.
- `packages/universe/docs/iteration-split-incident.md` and `packages/universe/docs/migration-status.md`.
- `phalcom-ast/src/ast.rs`, `phalcom-modules/src/interface.rs`, `phalcom-modules/src/linker.rs`, `phalcom-modules/src/graph.rs`.
- The semantic session/checker import resolution path, source occurrence products, core linked-read compiler/materializer path, and the focused test READMEs named below.

## 5. Normative authority

- `docs/specs/README.md` governs specification precedence. No general normative source-import phase syntax is currently established by this plan.
- `docs/specs/universe.md` governs canonical Universe identity. A phase-aware link must preserve each imported declaration's existing canonical `SymbolId`/`DeclarationId`.
- `docs/specs/generic-constraints.md` and the class/trait specifications govern type and conformance meaning; they do not make a source reference runtime-free by themselves.
- `MODL001.C2.P2` is a **proposed implementation design**, not an accepted language specification. Its intended split is used here only where it agrees with the fixed decisions below: semantic/interface cycles may be solved, runtime initialization cycles remain invalid, whole-module imports and direct re-exports remain runtime dependencies, and unknown phase defaults conservatively to runtime.
- Current behavior is the live implementation described in Section 3. The ability to accept a module cycle through an interface-only edge is a future capability, not a claim about current behavior.

## 6. Takeover state

| Concept | Current owner | Existing interface | Invariant |
|---|---|---|---|
| Source import syntax | `phalcom-ast` | `DependencyDecl`, `ImportDecl` | Ordinary source spelling remains unchanged in this plan. |
| Public export and import identity | `phalcom-modules` | `ImportSurface`, `LinkedModuleInterface`, `SymbolId` | Selected imports resolve to one canonical exported declaration. |
| Semantic/type references | `phalcom-semantic` | linked declarations, binding catalogs, semantic products | Semantic resolution consumes linker-owned names and identities. |
| Runtime initialization dependencies | `phalcom-modules` | `RuntimeDependencyGraph` | Every executable program has an acyclic graph before initialization. |
| Runtime linked reads | `phalcom-core` | `ImportBindingId`, `LinkedReadSpec`, `RuntimeLinkedRead` | Every emitted runtime read resolves to a materialized linked binding. |

`ReferenceKind::InterfaceOnly` is only a graph model hook today. It is not evidence that an import was safely erased from runtime bindings.

## 7. Architecture

Keep the existing source syntax. Phase is an internal semantic fact inferred for each canonical imported binding, not a user assertion and not an independent name resolver.

```text
source imports
    ↓
canonical symbolic linking of all imports/exports
    ↓
semantic checking records joined use phase per imported binding
    ↓
runtime-link finalization projects only proven runtime dependencies/reads
    ↓
strict runtime DAG validation
    ↓
executable lowering and runtime materialization
```

The pre-semantic link product must remain sufficient to resolve every imported name and declaration, including across semantic strongly connected components. It must not claim a validated executable initialization order while phase facts are provisional. After semantic analysis, one canonical phase product is joined per imported binding and consumed by a single runtime-graph finalizer. The runtime compiler/materializer must consume the same finalized read layout; it must not reconstruct phase from syntax or use a second linker.

The phase lattice is conservative:

```text
InterfaceOnly < Runtime
```

Every resolved source import begins as `Runtime` unless semantic analysis proves all uses are interface-only. When uses are observed, their phases join; any runtime use upgrades the binding to `Runtime`. An absent, ambiguous, unsupported, or unclassified use is `Runtime`. Whole-module imports and direct re-exports are always `Runtime`. A selected import remains `InterfaceOnly` only when canonical semantic analysis proves no runtime binding/value is required. Cross-module superclass/class-object references remain `Runtime` unless a separate accepted design proves their materialization is interface-only.

Semantic import identity must remain available for all imports. Runtime import slots are a projection for executable reads, not the semantic import table. The finalized product must map every executable import read to its canonical linked target and must not materialize an interface-only imported value as a runtime read. Runtime dependency edges must include every read emitted by lowering, every runtime declaration materialization requirement, all whole-module imports, and all direct re-exports.

If the current API cannot represent provisional symbol links separately from a validated executable link, introduce one explicit stage boundary. Fail closed: core execution must accept only a finalized, DAG-validated link product.

## 8. Ownership boundaries

### Owns

- Per-import phase evidence from canonical semantic name/reference resolution.
- Separation of semantic import identity from executable runtime reads.
- Final runtime dependency graph construction and strict validation.
- Incremental invalidation/fingerprinting when an imported binding changes between interface-only and runtime use.
- Cross-layer proof that emitted runtime reads are covered by runtime graph edges.

### Does not own

- New source syntax such as `import type` or `from M import type T`.
- Lazy loading, partial module initialization, temporal dead zones, or relaxed runtime SCC rules.
- New type namespaces, new declaration identities, duplicate Universe roots, or a second symbol resolver.
- The six-child iteration file split, adapter redesign, protocol API changes, or `Bytes`/`String` iterator migrations.
- Production Universe source-provider, prelude, native ownership, or bootstrap cutover.
- Broad cleanup of module linking, semantic occurrence indexing, or LSP behavior unrelated to phase evidence.

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Imported declaration identity | Module linker | Semantic analysis, compiler | AST text re-resolution or builtin fallback |
| Use phase of an imported binding | Semantic analysis products | Runtime graph finalizer | Linker guesses from identifier spelling; compiler-specific parallel classifier |
| Executable imported reads | Finalized linked runtime projection | Lowering, VM materializer | Independent runtime import discovery |
| Runtime initialization order | Validated runtime graph | Bootstrap/loader | Partial-module or SCC-order fallback |

## 9. Global invariants

- **INV-01 — Canonical identity:** interface-only classification never changes or recreates the imported `SymbolId`/`DeclarationId`.
- **INV-02 — Phase join:** if any use of one imported binding requires a runtime value/read, its phase is `Runtime`.
- **INV-03 — Unknown is runtime:** unsupported or missing phase evidence cannot remove an eager dependency.
- **INV-04 — Runtime DAG:** every executable link has a validated acyclic runtime graph; runtime cycles remain errors.
- **INV-05 — Complete runtime graph:** every runtime read emitted by lowering and every runtime declaration-materialization requirement has a corresponding runtime dependency.
- **INV-06 — Semantic links remain:** interface-only imports remain resolvable to the canonical declaration during semantic analysis even when no runtime read is produced.
- **INV-07 — Fixed syntax:** this increment does not change import grammar or add an authored phase marker.
- **INV-08 — Incremental parity:** cold and incremental analysis produce the same per-import phases, runtime graph, diagnostics, and initialization order.

## 10. Non-goals

- Making every type-looking use interface-only by spelling alone.
- Treating trait conformance as runtime-free without verifying its semantic/lowering path.
- Removing whole-module imports or direct re-exports from the runtime graph.
- Omitting a runtime edge merely because an import is unused in one inspected body.
- Proving the full Universe shadow package executable or authorizing cutover.

## 11. Expected impact map

### Expected source areas

- `phalcom-modules/src/linker.rs`, `graph.rs`, and linked import products — separate canonical symbolic imports from finalized runtime dependencies.
- `phalcom-semantic/src/session.rs`, checker/import resolution products, and fingerprints — record the strongest semantically proven use phase per imported binding.
- `phalcom-core/src/modules/compile.rs`, compiler linked-read emission, and materialization — consume only the finalized runtime projection and verify read coverage.
- `phalcom-modules/tests`, semantic import/integration tests, and core module tests — prove phase and runtime behavior at each owner.
- `docs/implementation/MODL001-module-architecture/C2-module-project-model/CHECKPOINT.md` — durable plan/gate state.

### Unexpected-touch rule

Entering parser grammar, type namespace semantics, declaration identity, GC, VM initialization rules, LSP ownership, or Universe production bootstrap for an architectural reason is a STOP/CONSULT trigger. Mechanical changes required by the agreed phase boundary may adapt locally.

## 12. Implementer decision authority

### 12.1 FIXED

- Do not add user-facing import phase syntax in this plan.
- Semantic products, not the linker or compiler, own the phase proof.
- All imported names remain canonically linked for semantic analysis.
- Only proven interface-only selective imports may be absent from the executable runtime projection.
- Whole-module imports and re-exports remain runtime dependencies.
- Unknown/unclassified/import materialization cases remain runtime.
- Runtime SCC validation remains strict and occurs before executable initialization.
- Interface-only status is joined per imported binding; a mixed type/value use is runtime.
- Preserve the live declaration identity and the S4 cohesive iteration source.

### 12.2 MECHANICALLY FLEXIBLE

- Private product/type names and file decomposition.
- Whether a pre-semantic program is represented by a distinct Rust type or an explicit provisional/finalized state, provided executable APIs require validated finalization.
- How stable local import keys are represented, provided aliases and individual selected names do not collide or share phase accidentally.
- Which existing semantic reference products are extended, provided the canonical resolver supplies target identity and use context.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Every type/declaration-only imported use is observable through canonical semantic resolution | semantic checking of annotations, constraints, traits, class headers, aliases, ADTs, and bodies | STOP/CONSULT; do not add a source-text scanner as fallback |
| Runtime graph validation can be deferred until semantic phase evidence exists | linker/session/core pipeline and runtime entry points | STOP/CONSULT if executable compilation currently depends on validated order earlier |
| Compiler-generated `GetLinked`/declaration references can be matched to phase evidence | compiler import lowering and declaration materialization | Keep the dependency Runtime until a canonical owner is established |
| Incremental products can invalidate on phase changes without rebuilding unrelated modules | semantic input/product fingerprints and linked component cache | STOP/CONSULT if a second cache or global invalidation is required |

## 13. Global STOP / CONSULT triggers

In addition to repository-wide triggers, stop if:

1. The semantic checker cannot distinguish all runtime references from interface-only references without a syntax change or second source scanner.
2. An interface-only import still requires a runtime class object, module object, export binding, or value slot at initialization.
3. Core execution can observe a provisional graph or materialize an interface-only import as an executable read.
4. A runtime cycle survives after all edges have been classified from canonical evidence.
5. A change would weaken a generic/conformance contract or create a second `Iterator`/`Iterable` identity.
6. Phase changes cannot be represented in incremental fingerprints and cold-analysis parity.

## 14. Debugging budget

Follow repository guidance. A graph mismatch or missing runtime read is semantic/architectural: record observed facts, causal hypothesis, evidence, predicted correction, and one discriminating test before editing. Do not speculate after one serious semantic correction. Runtime-cycle validation and interface-only proof are not candidates for permissive fallback.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariants | Required case |
|---|---|---|---|
| CV-01 | source-to-link success | INV-01, INV-06, INV-07 | ordinary selective import used only in a declaration/type/conformance position resolves to the same canonical target and is finalized interface-only |
| CV-02 | runtime use | INV-02, INV-05 | one expression/value read upgrades the same selected import to Runtime and produces a linked runtime read/edge |
| CV-03 | conservative fallback | INV-03, INV-04 | missing/unsupported phase evidence retains a runtime edge; a resulting runtime cycle is rejected |
| CV-04 | source import distinctions | INV-04 | whole-module import and direct re-export remain runtime and cycle-checked |
| CV-05 | semantic SCC vs runtime DAG | INV-04, INV-06 | a type/conformance cycle resolves semantically while the runtime graph remains acyclic |
| CV-06 | mixed phase | INV-02 | interface use plus runtime use of one binding joins to Runtime |
| CV-07 | incremental invalidation | INV-08 | editing a type-only use into a value use updates the graph/diagnostics and matches cold analysis |
| CV-08 | linked read/materialization | INV-01, INV-05 | every emitted runtime read maps to the canonical symbol and an ordered runtime dependency |
| CV-09 | Universe iteration integration | INV-01, INV-04, INV-06 | protocol defaults may runtime-import adapter classes while adapter conformance imports are interface-only; identities remain single-owner |

## 16. Verification execution budget

### BUILD MODE

- After T1, run only module linker unit/integration filters directly affected by symbolic import and runtime graph finalization.
- After T2, run the exact semantic import-use fixture and its type/value phase assertions.
- After T3, run the smallest compiler/read-layout and core linked-materialization regression.
- Do not run the workspace suite or full Universe runtime corpus during BUILD.

### STABILIZE MODE

- Run all focused CV-01 through CV-09 tests.
- Confirm selected test counts are nonzero.
- Run directly affected `phalcom-modules`, semantic import integration, and core module tests only where their code changed.
- Preserve any unrelated baseline failure with evidence; do not repair it here.

### Explicitly deferred

- Workspace build/test/clippy, broad language corpus, production Universe bootstrap, and performance measurements.
- Six-child iteration module migration and any cutover certification.

## 17. Baseline / unrelated failure policy

Classify unexpected results A/B/C/D per `AGENTS.md`. Do not rerun unchanged baseline failures. Any pre-existing module or semantic issue that blocks the exact phase proof is a blocker to this plan and must be reported; unrelated subsystem failures are recorded and deferred.

## 18. Tasks

### T1 — Separate semantic import links from executable runtime projection

#### Purpose

Give later semantic phase evidence a safe place to land without losing canonical name resolution or exposing an unvalidated runtime graph to execution.

#### Preconditions

- Confirm current linker/session/core call order against the live tree.
- Preserve the existing conservative behavior until phase evidence is connected.

#### Consumes

- `ImportSurface`, resolved paths, `SymbolId`, `LinkedModuleInterface`, and current `LinkedReadSpec` behavior.

#### Produces

- A symbolic linked-import representation sufficient for semantic analysis for every source import.
- A runtime projection/finalization boundary that cannot be mistaken for an executable DAG before validation.
- Stable mapping from semantic imported binding identity to optional runtime read identity.

#### Files and symbols

**Read:**
- `phalcom-modules/src/linker.rs`, `interface.rs`, `graph.rs`, `session.rs`.
- `phalcom-semantic/src/session.rs` and `module_bindings.rs`.
- `phalcom-core/src/modules/{compile,linkage,materialize}.rs` and `compiler/lib/expr.rs`.

**Modify:**
- Only the smallest module-link products and consumers needed to represent symbolic imports separately from runtime reads.

**Tests:**
- Existing selective import link test and runtime-cycle linker test; add one assertion that an ordinary import remains symbolically resolvable before final phase classification.

#### Required implementation shape

- Keep all source imports in the semantic name-resolution surface.
- Do not remove runtime edges based on a caller-supplied missing map or source spelling.
- Keep existing runtime behavior conservative until T2/T3 produce and consume complete phase evidence.
- Executable compile/bootstrap entry points must fail closed if asked to consume a provisional runtime graph.

#### Forbidden approaches

- Deleting runtime graph validation.
- Reusing `ImportBindingId` for a type-only target without an explicit runtime read mapping.
- Making the linker depend on `phalcom-semantic` (crate cycle).
- Introducing a duplicate symbol/import resolver.

#### Tests to run now

Run the smallest `phalcom-modules` linker tests that cover selective import resolution and strict runtime-cycle rejection. Do not broaden after PASS.

#### Tests explicitly deferred

- Semantic phase fixture, core runtime read/materialization, and Universe iteration integration until T2–T4.

#### Acceptance

An ordinary import remains available to semantic resolution; a pre-semantic product cannot be passed to executable initialization as a validated DAG; existing value-import cycles still fail.

#### Local STOP / CONSULT triggers

Stop if current semantic analysis can only resolve imported declarations through runtime linked-read materialization or if runtime link finalization cannot be deferred without changing accepted execution semantics.

### T2 — Publish canonical per-import semantic phase evidence

#### Purpose

Prove the strongest phase required by every selected imported binding from the canonical semantic uses that resolve it.

#### Consumes

- T1 symbolic import links.
- Existing source declaration, checker, and use-resolution products.

#### Produces

- A deterministic semantic product keyed to the importing module and exact imported local binding, carrying canonical target identity and joined `DependencyPhase`.
- Fingerprint/invalidation behavior for phase-only edits.

#### Required implementation shape

- Record interface-only evidence only at semantic sites whose resolved declaration/value is not required at runtime.
- Upgrade to Runtime for value expressions, whole-module use, direct re-export, class/superclass/runtime materialization requirements, and any unresolved/unsupported site.
- Join all uses for one local imported binding. Aliases and separate selected names remain distinct.
- Keep canonical resolution in the existing semantic resolver; do not add identifier-text traversal as an alternate authority.
- Ensure stale summaries cannot survive edits to signatures, bodies, import aliases, or declaration ownership.

#### Forbidden approaches

- Treating all class/trait declarations as interface-only based on declaration kind alone.
- Inferring use phase from occurrence roles that do not carry type/value context.
- Using only source search, a lowering omission, or absent `GetLinked` as proof without checking generated declaration references.
- Defaulting missing evidence to InterfaceOnly.

#### Tests to add

- CV-01, CV-02, CV-03, CV-06, and CV-07 at semantic ownership.
- Include scope shadowing and aliases so only the canonical imported binding contributes to its phase.

#### Tests to run now

The exact new semantic test plus any directly affected query/fingerprint test. Do not run broad semantic suites during BUILD.

#### Acceptance

Type/conformance-only imported references produce `InterfaceOnly`; a value use of the same binding or any unknown use produces `Runtime`; phase edits invalidate and match cold analysis.

#### Local STOP / CONSULT triggers

Stop if semantic analysis cannot represent the phase proof without a second resolver, a full unrelated occurrence-index redesign, or a new user-visible import/type namespace rule.

### T3 — Finalize and validate the executable runtime graph

#### Purpose

Project semantic phase evidence into runtime read layouts and a strict initialization DAG consumed by core execution.

#### Consumes

- T1 symbolic link product.
- T2 canonical per-import phase product.

#### Produces

- A finalized runtime-linked product with runtime read slots, runtime dependency edges, and validated initialization order.
- A structural check that compiler-emitted runtime reads are all covered by Runtime phase edges.

#### Required implementation shape

- Keep semantic and runtime graphs distinct.
- Create runtime edges for Runtime imports and none for proven interface-only selected imports.
- Preserve Runtime edges for whole-module imports and direct re-exports.
- Keep unknown edges Runtime and preserve strict SCC rejection.
- Keep imported canonical semantic targets available even when no runtime slot is present.
- Ensure re-exports, class materialization, static constants, and generated `GetLinked` references retain the currently required behavior.
- Make lowering and materialization consume the finalized runtime slot mapping; an interface-only import must not require a target runtime global slot.

#### Forbidden approaches

- Removing only the graph edge while leaving a runtime read reachable from bytecode.
- Filtering by source import kind without joined semantic phase.
- Tolerant cycle bypass in strict/executable mode.
- Treating all declarations in an interface-only module as initialized or globally available at runtime.

#### Tests to add

- CV-03, CV-04, CV-05, and CV-08.
- Keep the existing value cycle negative control and add mixed interface/value and whole-module/re-export negatives.

#### Tests to run now

Exact linker finalization test, then the smallest core compilation/materialization test for a linked runtime read. Only run once T1/T2 are coherent.

#### Acceptance

Runtime ordering derives from the finalized projection; every emitted read is represented; pure interface edges do not enter runtime SCC detection; actual runtime cycles remain rejected.

#### Local STOP / CONSULT triggers

Stop if any code path can initialize modules from a provisional product or if runtime codegen emits a read that cannot be mapped back to a semantic phase fact.

### T4 — Prove the Universe iteration dependency shape

#### Purpose

Demonstrate that the capability addresses the specific S4 blocker without changing iteration declarations or user-visible methods.

#### Fixture

Use a minimal isolated source graph with a protocol module and one adapter module. The protocol's default method imports and constructs the adapter as a runtime value. The adapter imports the protocol through an ordinary selective import and uses it only for protocol conformance/type references. Add a paired fixture where an adapter's method body reads a value from the protocol module to ensure the mixed dependency upgrades to Runtime and the cycle is rejected.

#### Tests to add

- CV-05 and CV-09: semantic resolution retains one canonical protocol identity, the interface graph retains both references, the finalized runtime graph is acyclic for the type-only conformance edge, and a relevant default method remains executable.
- Paired runtime-use fixture proves cycle rejection.

#### Tests explicitly deferred

- The full sixteen-declaration/six-child iteration source migration, all adapter runtime behavior, and Universe bootstrap/cutover.

#### Acceptance

The minimal ordinary-source fixture links and typechecks with canonical identity preserved, records only the protocol-to-adapter runtime edge, and rejects the paired value-cycle case.

#### Local STOP / CONSULT triggers

Stop if the minimal semantic fixture requires changing protocol contracts, adapter constructors, generic constraints, public imports, or declaration ownership to make the graph acyclic.

## 19. Verification gates

### G1 — Semantic phase proof

Prove CV-01, CV-02, CV-03, CV-06, and CV-07 with focused semantic tests. Confirm positive tests were selected.

### G2 — Runtime graph and read projection

Prove CV-04, CV-05, and CV-08 with module-linker and core linked-read tests. Retain strict runtime-cycle rejection.

### G3 — S4 integration proof

Prove CV-09 with the minimal split fixture. This gate does not accept the six-child source migration or production cutover.

## 20. Final focused acceptance

Report each CV result separately. Completion requires G1–G3. If the interface-only path works generally but not for the iteration pattern, complete only the independently proven work and record the iteration limitation as a blocker; do not mark the six-child split executable-complete.

## 21. Checkpoint bookkeeping

At start, update `MODL001.C2/CHECKPOINT.md` to mark P4 active and record the live baseline. During work, record only stable import product/phase invariants, completed gates, consultations, and blockers. At completion, record the plan status, implementation/verification level, residual failures, and next action.

## 22. Walkthrough and handoff

At completion create:

- `MODL001.C2.P4-walkthrough.md`
- `MODL001.C2.P4-handoff.md`

The walkthrough records actual implementation and tests. The handoff records the finalized/provisional product boundary, phase owner, gate evidence, and whether the iteration split can be reconsidered. Neither document may claim production Universe cutover.

## 23. Completion truth table

```text
symbolic imports resolve             != runtime phase proven
phase product exists                 != runtime graph finalized
runtime graph is acyclic             != emitted reads are covered
focused module tests pass            != Universe iteration integration passes
iteration integration passes         != production cutover certified
```

## 24. Plan self-review

- Every interface-only edge has an explicit semantic evidence owner and conservative fallback.
- Runtime bindings/read slots are separated from semantic import resolution.
- The runtime DAG remains strict and covers emitted reads/materialization.
- No user-facing syntax or duplicate resolver is introduced.
- Focused tests distinguish interface-only from value use and prove the S4 pattern.
- Broad workspace/cutover gates are deferred.

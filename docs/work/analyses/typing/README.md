# Phalcom Typing Platform: Ratified Architecture and Specification Map

**Date:** 2026-08-22
**Status:** Ratified implementation specification series; documents 01–03 are implementation-ready
**Authority:** Forward-looking typing authority after the completed two-axis semantic-tower milestone
**Scope of this delivery:** program map plus implementation architecture, runtime metadata/reification, and reflection API

## 1. Reading contract and evidence labels

This series continues the completed [two-axis semantic-tower plan](../../pending/typing/phase%203/2026-08-22-phalcom-two-axis-semantic-tower-implementation-plan.md) and its [repository-grounded specification](../../pending/typing/phase%203/phalcom_two_axis_semantic_tower_repository_grounded_implementation_spec.md). It does not change Phalcom's runtime class/metaclass model.

Every factual or design claim uses one of these labels:

- **Observed current implementation** — verified against current source after commit `59b3dce4`.
- **Observed test coverage** — evidence reported by the supplied Task 13 acceptance report or visible in current tests; no test was rerun for this documentation task.
- **Ratified/normative design** — final language or architecture decision authorized for implementation.
- **Proposed design needing ratification** — intentionally open; implementation must stop at the stated gate.
- **Untracked forward-design input** — useful input not made normative merely by existing in an untracked document or attachment.
- **Pyrefly architectural transfer** — operational architecture transferred or adapted from the local Pyrefly dossier, never Python semantics.

If a historical typing document conflicts with this series, this series plus the implemented two-axis ontology wins. A proposal is not current implementation until a cited source and test establish it.

## 2. Executive assessment

**Observed current implementation.** Phalcom now has a coherent foundational static checker rather than a syntax-only experiment:

- store-local, densely interned `TypeId` and `KindId` domains, with `Type` and arrow kinds;
- stratified runtime value typing and semantic denotation through `TypeKnowledge`, `SemanticDenotation`, and `ValueSemanticFact`;
- class-object types separated from the nominal forms those class objects denote;
- kind-checked partial type application, owner-qualified type parameters, substitution, declaration surfaces, class/instance-side dispatch, and whole-linked-workspace checking;
- compiler and CLI compilation gated through an analyzed semantic program;
- LSP publication of the compiler-owned static snapshot alongside, not in place of, advisory `ValueShape` analysis;
- parse-once module units, semantic SCC shells, deterministic graph ordering, and stable structural export descriptors.

Representative evidence: [`TypeStore`](../../../../phalcom-semantic/src/types/store.rs#L64), [`SemanticSnapshot`](../../../../phalcom-semantic/src/snapshot.rs#L17), [`analyze_workspace`](../../../../phalcom-semantic/src/workspace.rs#L47), [`AnalyzedProgram`](../../../../phalcom-core/src/modules/compile.rs#L142), and the LSP's explicit [`StaticSemanticSnapshot`](../../../../phalcom-lsp/src/semantic/snapshot.rs#L52).

**Observed test coverage.** Supplied Task 13 evidence reports 1,103 workspace tests passed, six skipped, all doctests passed, deterministic fresh-store structural comparisons, 46 object-model invariants, 131 AST tests, 411 core tests with four ignored, and 48 LSP integration tests with two ignored performance harnesses. The determinism assertion compares diagnostics and store-independent exported type/kind structure in [`workspace.rs`](../../../../phalcom-semantic/tests/workspace.rs#L497). This is strong acceptance evidence for the two-axis milestone, not evidence that the platform specified here already exists.

**Observed current implementation.** Five material gaps define the next platform phase:

1. [`analyze_workspace`](../../../../phalcom-semantic/src/workspace.rs#L47) constructs a fresh store and recomputes the whole linked workspace; `phalcom-semantic` has only a fingerprint registry, while useful reverse invalidation remains in the advisory LSP engine.
2. LSP static analysis silently skips unloadable projects, interface failures, import resolution failures, and linker failures, and converts an impossible runtime cycle into sorted module order in [`run_static_workspace_analysis`](../../../../phalcom-lsp/src/analysis_service.rs#L944).
3. Relations expose booleans or a coarse `Uncertain`; hierarchy and structural recursion have no shared cycle/budget protocol in [`relation.rs`](../../../../phalcom-semantic/src/types/relation.rs#L92).
4. Stable export data is a recursive tree, and compiled module plans do not yet carry typed semantic metadata: [`CompiledTypeRef`](../../../../phalcom-semantic/src/export.rs#L56), [`ModuleMaterializationPlan`](../../../../phalcom-core/src/modules/artifact.rs#L29).
5. Runtime reflection has no type/kind descriptors. Existing module/project reflection caches strongly trace every cached object in [`ReflectionCache::trace`](../../../../phalcom-core/src/modules/reflection_cache.rs#L31), which is unsuitable for unbounded synthetic type forms.

Conclusion: semantic kernel is good and should be retained. Next work is staged-query architecture, durable metadata, and safe explicit reflection—not another type-system rewrite.

## 3. Source-of-truth and authority table

| Subject | Authority | Classification | Rule |
|---|---|---|---|
| Runtime object/class/metaclass behavior | [Current object model](../../../spec/current/object-model.md), accepted ADRs, live VM | **Ratified/normative design** + **Observed current implementation** | Typing does not alter class identity, metaclass wiring, selector identity, allocation, or lookup. |
| Two-axis type/kind ontology | [`ontology.md`](../../../spec/typing/ontology.md) where consistent with live model, completed two-axis plan/spec, live semantic code | **Ratified/normative design** | `value.class`, `value : Type`, and `form :: Kind` are distinct judgments. |
| Existing typing documents 01–03 | `docs/spec/typing/01-*`, `02-*`, `03-*` | **Untracked forward-design input** where contradicted here | Retain useful protocol/generic ideas; discard `Type`-as-protocol, `Type.currentApplication`, applied class-side forwarding, and `out`/`in`. |
| Variance syntax | This series and user direction | **Ratified/normative design** | Declaration-site `+T`, `-T`, and `T`; unary signs are syntax over parameter declarations, conceptually related to `+`/`-` messages, not selector components. |
| Five reviewed language decisions | Peer feedback supplied with this task and ratification instruction | **Ratified/normative design** | See §4. |
| Static semantic facts | `phalcom-semantic::SemanticSnapshot` | **Observed current implementation** | Compiler, CLI, REPL, and LSP must consume one formal semantic source. |
| Advisory editor shapes | `phalcom-lsp::semantic::ValueShape` | **Observed current implementation** | May enrich editor UX; cannot independently reject programs or become a second type checker. |
| Pyrefly dossier | `docs/work/analyses/pyrefly-transfer/` | **Pyrefly architectural transfer** | Transfer identities, staged queries, snapshots, invalidation, bounded solvers, observability; reject Python semantics. |
| Runtime type metadata and reflection | Documents 02–03 in this folder | **Ratified/normative design** | Reifies semantic results without becoming semantic authority. |

## 4. Ratified decision register

### DEC-KIND-POLY — prenex kind polymorphism

**Ratified/normative design.** Phalcom will support prenex kind polymorphism after the present monomorphic kind kernel is mature. Stable generalized binders use `KindParameterId`; solver-local metavariables use `KindVarId` and never escape a solver. No `Type :: Type`, universe polymorphism, dependent kinds, arbitrary kind-level evaluation, or unsolved kind variables in interfaces, snapshots, metadata, or reflection. Semantics are frozen; implementation and public syntax are deferred.

### DEC-RECORD-ROWS — record-specific open rows

**Ratified/normative design.** Structural record types gain `RecordRow` and an explicit `RecordTail::{Closed, Parameter}`. Known fields remain sorted separately from the tail. Record, variant, and effect rows may share implementation mechanisms later but are different semantic domains. Record rows never describe nominal class layouts. Public row syntax remains a **Proposed design needing ratification** and is assigned to the syntax specification.

### DEC-NUMERIC-LITERALS — syntax fixes runtime numeric class

**Ratified/normative design.** `1 : Int`; `1.0 : Float`. Expected types do not reinterpret literal runtime class. `Int` and `Float` are not made subtypes of one another, and assignment does not hide numeric coercion. Mixed arithmetic follows declared method contracts; conversion is explicit. Exact literal knowledge is a `ConstantFact`, not an ordinary singleton `TypeId` unless a later refinement-type feature explicitly promotes it. Current literal synthesis already selects `Int` and `Float` nominal types in [`expression.rs`](../../../../phalcom-semantic/src/checker/expression.rs#L30); the missing work is retaining exact constant facts.

### DEC-TOTALITY — partial correctness plus explicit totality

**Ratified/normative design.** Ordinary callables promise partial correctness. `TerminationRequirement::{Partial, Total}` is separate from `TerminationKnowledge::{ProvenTerminates, Unknown}` and possible later `ProvenDiverges`. A `Total` declaration requires `ProvenTerminates`. `Never` means no normal return value; it does not identify divergence. Return type, effects, exit facts, and termination evidence remain distinct.

### DEC-PROOF-ARTIFACTS — persistent evidence with trust tiers

**Ratified/normative design.** Verification produces durable, fingerprinted artifacts. Evidence distinguishes kernel-checked certificates, trusted-backend attestations, and counterexamples. Trust distinguishes `KernelChecked`, `TrustedBackend`, and explicit assumptions/axioms. Cache keys cover the canonical verification condition, assumptions, referenced interface fingerprints, semantic-model version, backend/version, and proof-kernel version. No `Prop`, proof terms, dependent API programming, or runtime object with implicit proof authority.

### DEC-REFLECTION-EXPLICIT — no ambient generic dispatch context

**Ratified/normative design.** `Type.currentApplication` and transparent applied-type forwarding are rejected. They introduce ambient type-directed runtime behavior and complicate fibers, reentrancy, reflection, security, and cache invalidation. Ordinary construction remains a message to an ordinary class object. Explicit reflective operations go through an immutable `TypingContext`; no type argument changes selector identity or runtime method lookup.

### DEC-METADATA-DAG — durable metadata is versioned and indexed

**Ratified/normative design.** Recursive `CompiledTypeRef` remains a transitional in-memory export helper. Persisted compiler/native/reflection metadata uses a versioned, depth-bounded indexed DAG. Store-local IDs and inference variables never cross this boundary.

### DEC-REFLECTION-IDENTITY — nominal objects stay nominal

**Ratified/normative design.** Reifying a nominal class form returns its existing class object, so reifying `Int` is `Int`. Synthetic forms are immutable descriptor objects canonical within a live VM registry; identity (`===`) is VM-local, while `equivalentTo(_)` is semantic structural equivalence. Runtime `value.class` never claims to recover an erased static type.

## 5. Current-to-target architecture

```text
CURRENT

source/provider
    -> parser + ParsedModuleUnit cache
    -> interface builder + linker + three graphs
    -> analyze_workspace (fresh TypeStore, whole workspace)
    -> SemanticSnapshot
         |-> ProgramAnalyzer -> ProgramCompiler -> runtime materialization
         `-> LSP wrapper snapshot -> diagnostics

TARGET

source/provider + stable workspace/project registry
    -> compiler-owned SemanticDb
         |-> parsed module query
         |-> unlinked/linked interface query
         |-> declaration shell + semantic SCC query
         |-> body/flow/dispatch/effect query
         |-> relation/proof query (bounded result states)
         `-> reverse dependencies + fingerprints
    -> immutable PublishedSemanticSnapshot
         |-> compiler / phalcom check / REPL
         |-> LSP static facts (ValueShape remains advisory)
         `-> versioned SemanticMetadata DAG
                 -> loader-owned RuntimeTypeRegistry
                       |-> existing Class objects for nominal forms
                       `-> weakly cached immutable synthetic descriptors
```

Key ownership rule: `phalcom-modules` owns physical/project/module identity, source products, and graphs; `phalcom-semantic` owns type/kind/relations/query products; `phalcom-core` owns compiled artifacts, runtime registry, materialization, and object reflection; `phalcom-lsp` only adapts published snapshots. No reverse dependency from semantic analysis to VM objects is permitted.

## 6. Specification dependency graph

```text
completed two-axis tower
          |
          v
01 implementation architecture
    |                 |
    v                 v
02 metadata       later checking/flow/effects specs
    |
    v
03 reflection API
    |
    +--> later syntax specification
    +--> later contracts/proof specification
    `--> compiler/CLI/REPL/LSP rollout specification
```

| Document | Owns | Depends on | Must land before |
|---|---|---|---|
| [01 — Implementation Architecture](01-implementation-architecture.md) | IDs, query DB, SCCs, invalidation, relation outcomes, diagnostics pipeline, snapshots | Completed tower | Durable metadata, incremental compiler/LSP, effects/proofs |
| [02 — Runtime Reification and Metadata](02-runtime-reification-and-metadata.md) | Indexed metadata DAG, profiles, loader registry, GC policy, native schema, proof artifact carriage | 01 | Runtime reflection and cross-run caches |
| [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md) | User-visible type/kind/query objects, identity/equality, capabilities, dynamic boundaries | 01–02 | Public typing module and reflection implementation |
| 04 — User-facing syntax (future delivery) | Full annotations, `+`/`-`, kinds, aliases, constraints, rows | 01; semantic decisions here | Parser/AST changes |
| 05 — Advanced kinds, effects, totality, proof artifacts (future) | Kind schemes, rows, effects, VC/prover state | 01–02 | Public prover APIs |
| 06 — Comparative rationale (future) | Lessons and rejected alternatives | All decisions | Non-blocking |
| 07 — Consolidated implementation plan and decision register (future) | Cross-crate work units and rollout | 01–05 | Program execution |

## 7. Feature traceability matrix

| Requirement | Primary spec | Staging/result |
|---|---|---|
| Values/types/kinds/class objects/metaclasses stratified | 01, 03 | Existing tower retained; runtime model unchanged |
| Canonical types and checked kind application | 01 | Existing kernel retained; all public relations become bounded typed outcomes |
| `Any`/`Dynamic`/`Unknown`/missing/`Never`/invalid/infer/proof-unknown separation | 01, later 05 | State algebra frozen; `Any` proper type only when added, never epistemic fallback |
| Owner-qualified parameters, variance, bounds, F-bounds, `Self`, HKT | 01, later 04–05 | IDs already owner-qualified; `+`/`-` and advanced constraints staged |
| Open record rows | 01, later 04–05 | Semantics ratified; syntax deferred |
| Typed interfaces before bodies, semantic SCCs | 01 | Current shells retained; formal staged queries added |
| Immutable snapshots, invalidation, cancellation, budgets | 01 | Compiler-owned query DB; atomic publication |
| Bidirectional checking, inference, flow, dispatch | 01 and later checking spec | Existing baseline retained; query/result protocol generalized |
| Class/instance side, `super`, constructors | 01, 03 | No runtime changes; explicit reflective construction only |
| Native surfaces | 02 | Versioned authoritative metadata; load-time compatibility |
| Reflection, `perform`, DNU, FFI boundaries | 02–03 | Explicit result states, capability checks, proof boundaries |
| Compiler/CLI/REPL/LSP one formal snapshot | 01 | Compiler semantic DB becomes sole formal owner |
| Runtime type objects | 02–03 | Nominals reuse classes; synthetic descriptors ordinary immutable objects |
| Contracts, totality, proofs | 02 and later 05 | Persistent trust-aware artifacts; no false `Proven` |
| Determinism and performance | 01–02 | Structural fingerprints, bounded queries, benchmark gates |

## 8. Migration timeline and ratification gates

### Stage A — harden the completed tower

Implement Spec 01 units A1–A4: validated proper-type construction, explicit relation outcomes, module-owned diagnostic labels, stable workspace identity. Compatibility: no public syntax and no runtime representation change.

Gate A: all existing two-axis tests remain green; no diagnostic disappears merely because link or project analysis failed; no relation or hierarchy walk can diverge.

### Stage B — compiler-owned semantic database

Implement staged query keys, dependency recording, reverse invalidation, semantic SCC iteration, cancellation, budgets, and atomic snapshots. Migrate LSP static analysis from rebuilding/linking its own catalog to requesting a compiler-owned snapshot. Advisory `ValueShape` remains intact.

Gate B: body edit avoids unrelated interface/SCC recomputation; public-interface edit invalidates exact reverse closure; cancellation never publishes partial facts; cold performance does not regress beyond ratified budget.

### Stage C — durable semantic metadata

Implement Spec 02 indexed DAG, schema/version checking, metadata profiles, native-surface convergence, and compiled-module carriage. Retain recursive `CompiledTypeRef` as adapter during migration.

Gate C: deterministic byte-for-byte metadata, hostile-depth validation, no raw store IDs, and version mismatch diagnostics.

### Stage D — runtime reification and reflection

Implement loader registry and Spec 03 APIs. Reuse nominal class objects. Add immutable descriptor classes and weak canonicalization without changing the class/metaclass tower.

Gate D: nominal identity, synthetic equivalence, GC reclamation, access control, world-version invalidation, and dynamic-boundary diagnostics all pass.

### Stage E — syntax and advanced proof platform

Land syntax only after its own grammar ratification. Kind schemes, record rows, effects, totality, and proving land behind the separate gates recorded in §4.

Gate E: no syntax shortcut forces an open semantic decision; proof results are `Proven`, `Disproven`, or reasoned `Unknown` with explicit trust.

## 9. Pyrefly transfer: take directly / adapt / reject

### Take directly

**Pyrefly architectural transfer.** Use cheap dense snapshot-local IDs, canonical stores, staged module queries, immutable publication, dependency recording, reverse invalidation, query state machines, worklists, cancellation, deterministic fixtures, terminal statuses, observability, and benchmarks.

### Adapt

**Pyrefly architectural transfer.** Adapt module queries to Phalcom project identities, selector labels, instance/class-side surfaces, inheritance, `super`, native metadata, semantic SCCs, message-send boundaries, and future kinds/constraints. Adapt editor workers so they consume compiler snapshots rather than owning the static type semantics.

### Reject

**Ratified/normative design.** Reject Python `Any`/unknown behavior, attribute lookup, descriptors, overload selection, protocols, import fallbacks, and editor-only semantics. Reject unsafe cache publication, global mutable solver state, unbounded fixed points, type-directed selector identity, and borrowed Pyrefly performance claims.

## 10. Lessons applied from other languages

| Source | Lesson taken | Boundary Phalcom keeps |
|---|---|---|
| Haskell/ML | Kinds, schemes, principal internal normalization, explicit separation of binders and inference variables | No `Type :: Type`, no dependent calculus, general recursion remains normal |
| OCaml | Row polymorphism is useful for preserving unknown record structure | Record rows remain distinct from class fields, variants, and effects |
| Scala | Declaration-site variance and HKTs are expressive | Use compact `+`/`-`; avoid implicit variance inference and ambient applied-class forwarding |
| Rust | Stable owner-qualified identities, explicit bounds, trust-aware unsafe boundaries | No monomorphized runtime class cloning or borrow semantics imported into Phalcom |
| Swift | Metatype/reflection distinctions expose genuine semantic layers | Do not collapse static types into runtime `Class` or invent `.Type` ambiguity |
| TypeScript/Python | Gradual escape hatches require explicit epistemic states | `Dynamic`, `Unknown`, missing annotation, invalid annotation, and `Any` never collapse |
| Smalltalk | Uniform objects, metaclasses, message sends, and live reflection | Typing never changes selector identity or ordinary dispatch |

## 11. Final developer experience

After the program lands:

- one `phalcom check` path handles inline, file, module, package, and project identities and reports parser, project, link, kind, type, effect, and proof states in one ordered diagnostic contract;
- the REPL assigns every cell a stable session/module identity and invalidates dependent cells coherently;
- editors receive the same formal facts and diagnostic codes as CLI builds, plus clearly marked advisory shapes when formal knowledge is unavailable;
- hover and reflection can show the canonical form, written occurrence, kind, generic parameters, member specialization, effects, and proof status without pretending runtime `.class` is a static type;
- `Unknown`, cancellation, budget exhaustion, opaque native code, reflection, `perform`, DNU, and FFI produce explicit blocked/dynamic boundary explanations rather than fabricated success;
- runtime programs pay no per-instance generic-token cost, and ordinary dispatch remains exactly Phalcom dispatch.

## 12. Intentionally unimplemented or contingent claims

The following are not claimed implemented by this series:

- kind polymorphism syntax or inference;
- record-row syntax and row inference;
- type lambdas, dependent kinds/types, universe polymorphism, or proof terms;
- implicit numeric coercion or singleton types;
- public protocol declaration/conformance syntax, overload syntax, aliases, ADTs, intersections, exhaustiveness, or sealed-hierarchy rules until their dedicated gates;
- full effect inference, totality proving, VC generation, proof-kernel implementation, or solver integration;
- runtime recovery of erased static generic arguments from arbitrary values;
- sound static resolution of arbitrary `perform` or DNU behavior;
- reflection metadata for private source details unless the selected metadata profile and caller authority permit it;
- incremental-performance goals copied from another language implementation.

These omissions are deliberate staging, not an invitation for implementers to invent behavior locally.

---
id: PERF003.C2.P7.PATCH4
category: PERF
program: PERF003
checkpoint: PERF003.C2
parent_plan: PERF003.C2.P7
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - PERF003.C2.P7.PATCH1
  - PERF003.C2.P7.PATCH2
  - PERF003.C2.P7.PATCH3
follows: PERF003.C2.P7.PATCH3
supersedes: null
target_branch: perf/universe-characterization
prepared_against_revision: 2e914b6c8905130780abb675db3401e592214c78
audit_baseline_revision: 2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
executor_profile: GPT-5.6 Luna High / Extra High
---

# PERF003.C2.P7.PATCH4 — Telosc Vertical Loader and P7 Artifact Lifecycle Closure

> **Execution role:** Luna-oriented patch-grade implementation plan.  
> **Parent acceptance objective:** `PERF003.C2.P7` — persisted canonical semantic compiler image (`universe.telosc`).  
> **Patch objective:** close the vertical loader and finalization pipeline from canonical Universe export through deterministic emission, exact-allocation import, source-free snapshot promotion, novel program compilation, and fresh-process source denial.  
> **Working discipline:** drive the end-to-end canonical artifact test continuously, identifying and repairing the first real failing invariant at each step without weakening guards, fabricating defaults, or bypassing accounting.

---

## 0. Executor contract

This plan is designed for an autonomous execution agent operating under strict architectural constraints.

The implementer:
- **must preserve** all non-negotiable invariants:
  - Do not disable or relax allocation-accounting guards.
  - Do not mark a partial snapshot `Complete` without proving all required source-free authority exists.
  - Do not fabricate empty/default semantic products where compiler readers require real authority.
  - Do not retain or serialize session-local/query-local identities as portable identity.
  - Do not reintroduce Universe source/AST dependencies into the artifact load path.
  - Do not reconstruct semantic acceptance by rerunning source analysis.
  - Do not weaken validation, referential-integrity checks, freshness checks, fingerprint checks, or malformed-artifact rejection.
  - Do not modify tests to accept weaker behavior.
  - Do not special-case the Universe where the correct solution belongs in the general artifact/compiler architecture.
  - Do not remove assertions merely because current implementation cannot satisfy them.
  - Do not claim completion based on encoding bytes alone; the artifact must survive its real loader and compiler consumer.
- **may adapt** private helper names, internal APIs, and mechanical signatures to live code;
- **must STOP AND CONSULT** if any core architectural premise is disproved or an invariant must be weakened;
- **must maintain** `CHECKPOINT.md` at meaningful milestones;
- **must produce** the required walkthrough and handoff upon completion.

---

## 1. Goal

Bring P7 to the point where a real compiler artifact (`universe.telosc`) exists, is atomically persisted, loads into an immutable source-free compiler image under exact cumulative allocation accounting, and successfully compiles novel dependent user packages in a separate process with Universe source authority completely unavailable.

---

## 2. Checkpoint acceptance objective

`PERF003.C2` eliminates the Universe compilation floor by transforming canonical Universe compilation from a per-session source-analysis tax into a reusable, immutable persisted image.

`PERF003.C2.P7` delivers the persisted `.telosc` container and compiler image loader. This patch, `PATCH4`, completes the vertical execution path connecting the portable semantic model to the loader, snapshot finalizer, and compiler consumer, satisfying all P7 acceptance criteria.

---

## 3. Repository grounding

- **Target branch:** `perf/universe-characterization`
- **Head commit:** `2e914b6c8905130780abb675db3401e592214c78`
- **Verified live facts:**
  1. The canonical artifact end-to-end test lives in `phalcom-core/tests/core/modules/canonical_artifact.rs` (`build_and_load_universe_telosc_compiles_source_free_inline_user_entry`).
  2. The current first blocker in `build_universe_telosc` is a referential integrity validation error during semantic export:
     `referential integrity error: declaration dispatch callable has no canonical semantic signature: CallableId { owner: Declaration(DeclarationId { module: ModuleId { project: Universe, path: option.option }, name: "Option" }), selector: Selector { base: Named("match"), kind: Method, slots: [Label("some"), Label("none")] }, side: Instance }`.
  3. Causal root: `Option.match(some,none)` is declared in `option.ph` as `native match<R>(some: (T) -> R, none: () -> R) -> R`. Because `Option` is an enum root (`is_enum_root = true`), `phalcom-semantic/src/impls.rs:6834` treated it as `is_requirement = true` due to `!syntax.has_body()`, ignoring `syntax.implementation() == ImplementationKind::Native`. This prevented publication to `callable_signatures` and `callable_definitions`, while emitting spurious `EnumRequirementMissing` diagnostics for `Some` and `None`.
  4. The portable semantic models for callables, expressions, explanations, diagnostics, and proof graphs are merged in `phalcom-compiler-artifact` and `phalcom-semantic`.
  5. The semantic validation and import path remains guarded by `UnaccountedBoundary` until exact cumulative allocation accounting is closed.

---

## 4. Required reads before implementation

```text
- AGENTS.md
- docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md
- docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7-R1/patches/PERF003.C2.P7 Consolidated Patch Program — Part I_ Architecture, Contracts, and Callable Authority.md
- docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7-R1/patches/PERF003.C2.P7 Consolidated Patch Program — Part II_ Semantic Owners, Exact Allocation, and Query Reconstruction.md
- docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7-R1/patches/PERF003.C2.P7 Consolidated Patch Program — Part III_ Container, Loaded Image, and P7 Closure.md
- phalcom-core/tests/core/modules/canonical_artifact.rs
- phalcom-semantic/src/artifact/export.rs
- phalcom-semantic/src/artifact/import.rs
- phalcom-core/src/modules/canonical_artifact.rs
```

---

## 5. Normative authority

- Accepted decisions on `Option`: `docs/decisions/accepted/0006-option-as-abstract-with-some-none.md`, `docs/decisions/accepted/0037-option-bootstrap-formalization-and-defer-niche-encoding.md`.
- Accepted native dispatch surface: `phalcom-native-surface/src/generated.rs` (`Option::match(some,none)`).
- Telosc container specification: `docs/specs/container-format.md` / `telosc` format v1.
- Inherent impl & enum requirement semantics: `docs/specs/enums-and-adts.md`.

---

## 6. Architecture and non-negotiable invariants

```text
portable stable identity
deterministic encoding
source-free semantic authority
immutable loaded compiler state
strict referential validation
transactional bounded allocation
freshness / ABI compatibility checks
query/fingerprint consistency
no hidden source fallback
no session-local identity leakage
```

1. **Owner-authoritative semantic export:** Every declaration, callable, signature, surface, and witness is exported from its canonical owner. No synthetic placeholders.
2. **Deterministic bitwise identity:** Two builds of `build_universe_telosc` from identical compiler state must yield byte-for-byte identical output.
3. **Exact-allocation ledger:** All allocations attributable to artifact load must debit the shared transaction before issuance. No unbudgeted scratch or unchecked heap vectors.
4. **Source-free completeness proof:** A loaded snapshot is only promoted to `Complete` if all tables required for compilation are present and verified without source fallback.
5. **Fresh-process isolation:** Proof of source freedom requires running user compilation in a process where Universe source files are physically absent.

---

## 7. Critical path and 10-step sequence

```text
Step 1: Prove full Universe semantic export succeeds
Step 2: Prove build_universe_telosc() emits deterministic bytes
Step 3: Close exact cumulative semantic-import allocation accounting
Step 4: Reconstruct compiler-required ModuleQueryProducts from validated module authority
Step 5: Finalize/promote the loaded snapshot only after source-free completeness is proven
Step 6: Reconstruct/validate query image and dependency fingerprints
Step 7: Make load_universe_telosc() succeed
Step 8: Make write_universe_telosc() succeed through its mandatory self-load
Step 9: Compile a novel user entry/package against the loaded image
Step 10: Strengthen to a fresh-process/source-denied discriminator
```

---

## 8. Detailed task breakdown

### Task 1 (Step 1): Repair Inherent Native Enum Method Analysis & Complete Export

- **Owner:** `phalcom-semantic/src/impls.rs`, `phalcom-semantic/src/checker/enum_behavior.rs`
- **Issue:** Inherent native methods declared on enum roots (such as `Option.match(some,none)`) are mistakenly classified as variant requirements (`is_requirement = true`) because `!syntax.has_body()`.
- **Invariants:**
  - An inherent method with `syntax.implementation() == ImplementationKind::Native` is an inherent native method, not an unfulfilled variant requirement.
  - Native surface anchoring for ABI-erased primitives must recognize valid declarations registered in `phalcom_native_surface`, including `UniverseKey::Option`.
  - The canonical signature and definition must be published to `callable_signatures` and `callable_definitions`.
  - `check_enum_requirements` must not emit `EnumRequirementMissing` for inherent native methods.
- **Verification:** Run `cargo test -p phalcom-core --test core modules_canonical_artifact::build_and_load_universe_telosc_compiles_source_free_inline_user_entry`. Assert that `build_universe_telosc` completes and emits an artifact without panicking at export preflight/referential validation.

### Task 2 (Step 2): Bitwise Determinism of `build_universe_telosc`

- **Owner:** `phalcom-compiler-artifact`, `phalcom-semantic/src/artifact/export.rs`
- **Invariants:**
  - All serialized structures (sections, directory, maps, sets, strings) must be ordered deterministically.
  - Section digests and container header digest must match across runs.
- **Verification:** Assert `built.bytes == built2.bytes` in `build_and_load_universe_telosc_compiles_source_free_inline_user_entry`.

### Task 3 (Step 3): Close Exact Cumulative Allocation Accounting

- **Owner:** `phalcom-compiler-artifact/src/semantic/`, `phalcom-semantic/src/artifact/import.rs`, `phalcom-type-meta/src/allocation.rs`
- **Invariants:**
  - Remove `UnaccountedBoundary` from `SemanticWorld` validation and import.
  - Every native vector, sorted map, `Arc`, and heap string allocated during `import_semantic_world` and `TypeStore::import_type_graph` must be debited against the cumulative `AllocationBudget`.
  - Roll back child ledgers on validation failure.
  - Enforce max total allocation bytes limit.
- **Verification:** Run focused counting-allocator and exact-fit/N-1 tests; ensure `load_universe_telosc_from_bytes` progresses past `UnaccountedBoundary`.

### Task 4 (Step 4): Reconstruct Compiler-Required `ModuleQueryProducts`

- **Owner:** `phalcom-core/src/modules/canonical_artifact.rs`, `phalcom-semantic/src/canonical_image.rs`
- **Invariants:**
  - Reconstruct `ModuleQueryProducts` (interfaces, declaration lists, dispatch surfaces, exports) from validated module and semantic authority.
  - Do not synthesize dummy/placeholder facts.
- **Verification:** Assert that loaded modules have valid query products for compiler consumers.

### Task 5 (Step 5): Finalize & Promote Loaded Snapshot to Source-Free Complete

- **Owner:** `phalcom-semantic/src/snapshot.rs`, `phalcom-semantic/src/canonical_image.rs`
- **Invariants:**
  - `snapshot.is_source_free_artifact_complete()` must strictly verify the existence and integrity of all required tables (surfaces, signatures, definitions, witness plans, etc.).
  - Sources must be empty (`sources().is_empty()`).
  - No synthetic flag promotion without full proof.
- **Verification:** Assert `loaded.formal_image.snapshot().is_source_free_artifact_complete()` passes.

### Task 6 (Step 6): Reconstruct and Validate Query Image and Dependency Fingerprints

- **Owner:** `phalcom-compiler-artifact/src/query_image.rs`, `phalcom-core/src/modules/canonical_artifact.rs`
- **Invariants:**
  - Deserialized query products must match computed semantic fingerprints.
  - Enforce ABI compatibility checks.
- **Verification:** Query image section verifies cleanly during loader execution.

### Task 7 (Step 7): Make `load_universe_telosc()` Succeed

- **Owner:** `phalcom-core/src/modules/canonical_artifact.rs`
- **Invariants:**
  - `load_universe_telosc(&path, &limits)` and `load_universe_telosc_from_bytes(&bytes, &limits)` return `Ok(LoadedUniverseArtifact)`.
- **Verification:** End-to-end assertions in `canonical_artifact.rs` pass up to user compilation.

### Task 8 (Step 8): Make `write_universe_telosc()` Succeed Through Mandatory Self-Load

- **Owner:** `phalcom-core/src/modules/canonical_artifact.rs`
- **Invariants:**
  - Write atomically via temporary file and rename.
  - Self-load written file and validate before returning `Ok(())`.
- **Verification:** `write_universe_telosc(&path, &built)` succeeds and leaves valid artifact on disk.

### Task 9 (Step 9): Compile Novel Dependent User Entry/Package Against Loaded Image

- **Owner:** `phalcom-core/src/modules/compile.rs`
- **Invariants:**
  - Compiling an inline snippet (`EntrySelection::Inline("print(42);".into())`) or a dependent package against `loaded` succeeds.
  - Universe source provider is not queried; all symbols resolve via loaded artifact.
- **Verification:** `compiled.modules` contains user output, bytecode emission succeeds.

### Task 10 (Step 10): Strengthen to Fresh-Process / Source-Denied Discriminator

- **Owner:** `phalcom-core/tests/core/modules/canonical_artifact.rs`
- **Invariants:**
  - Subprocess A builds and writes `universe.telosc` to a temp directory.
  - Subprocess B runs with no access to Universe sources (isolated environment or denial flags), loads `universe.telosc`, and compiles a non-trivial user program exercising Universe types (`Option`, `List`, `String`).
- **Verification:** Process B succeeds with zero source queries and produces valid bytecode.

---

## 9. Verification gates

| Gate | Scope | Command | Acceptance Standard |
|---|---|---|---|
| **G1** | Export & Determinism | `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_canonical_artifact::build_and_load_universe_telosc` | `build_universe_telosc` succeeds; 2nd build is byte-identical |
| **G2** | Exact Allocation | `RUSTFLAGS='' cargo test -p phalcom-compiler-artifact` | All budget/exact-fit/N-1 tests pass without `UnaccountedBoundary` |
| **G3** | Source-Free Snapshot | `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_canonical_artifact::build_and_load_universe_telosc` | `is_source_free_artifact_complete()` returns true; `sources().is_empty()` |
| **G4** | User Compilation | `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_canonical_artifact::build_and_load_universe_telosc` | `compile_user_entry` succeeds |
| **G5** | Fresh Process / Source Denial | New test in `phalcom-core/tests/core/modules/canonical_artifact.rs` | Independent process compiles user program with sources denied |

---

## 10. Mandatory escalation triggers

Stop and escalate under `shared-consultation-escalation-protocol.md` if:
1. Any invariant would need to be weakened (e.g., removing allocation checks or faking complete snapshot status).
2. A missing authority family cannot be derived or transported from canonical semantic models.
3. ABI or container format changes conflict with normative specs.
4. Memory or allocation architecture requires redesign rather than exact accounting closure.

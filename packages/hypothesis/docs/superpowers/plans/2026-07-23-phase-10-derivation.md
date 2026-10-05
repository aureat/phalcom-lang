# Phase 10 Derived Strategies Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Extend the reflective strategy registry so opt-in data classes and sealed variant hierarchies can be generated automatically, including bounded recursive models, while exact and annotated custom strategies retain precedence.

**Architecture:** `StrategyRegistry` owns exact registrations, annotated-provider installation, applied-type decomposition, a deterministic derived-strategy cache, and resolution-path diagnostics. Passive `@arbitrary` and `@strategy(Type)` attributes carry opt-in metadata; private derivation workers inspect constructor parameter metadata, reject unsafe constructors, build constructor strategies, partition sealed variants into terminal and recursive cases, and use the existing `Gen.recursive` size discipline. Derived values continue to consume ordinary `DrawData` choices and use the shared search and shrink kernel.

**Tech Stack:** Typed Phalcom source, passive reflected attributes, constructor/parameter/class reflection, existing strategy combinators, Python source/static verifiers, ZIP/SHA-256 checkpoint tooling.

## Global Constraints

- Implement only Phase 10; do not add Phase 11 provider hardening or Phase 12 release cleanup.
- Derivation is opt-in through `@arbitrary`; ordinary unmarked classes remain unresolved.
- Exact `register(type:, strategy:)` and installed `@strategy(Type)` providers override automatic derivation.
- Constructor contracts are not translated into filters; constrained constructors fail before search with a custom-strategy recommendation.
- Recursive sealed hierarchies require at least one terminal variant and use `Gen.recursive` rather than unbounded eager recursion.
- Diagnostics preserve the complete nested resolution path.
- Type annotations remain reflective metadata and do not enforce runtime values or alter dispatch.
- Runtime behavior is reported as unobserved when no `phalcom` executable is available.

---

### Task 1: Phase 10 acceptance contract

**Files:**
- Create: `scripts/verify_phase10.py`
- Create: `tests/strategies/derived_data.ph`
- Create: `tests/strategies/sealed_variants.ph`
- Create: `tests/strategies/recursive_variants.ph`
- Create: `tests/strategies/derived_generic_fields.ph`
- Create: `tests/strategies/resolution_path.ph`
- Create: `tests/strategies/constrained_constructor.ph`
- Create: `tests/strategies/annotated_strategy.ph`
- Create: `tests/strategies/custom_registration_precedence.ph`
- Create: `tests/property/inferred_domain_models.ph`

**Interfaces:**
- Consumes: the Phase 10 TDD sequence in the authoritative package plan.
- Produces: a source/static gate that fails on the Phase 09 registry and passes only after derivation, diagnostics, facade, docs, and examples are complete.

- [x] Write all fixtures before production source.
- [x] Run `python3 scripts/verify_phase10.py` and record the expected red result.

### Task 2: Derivation metadata and constructor strategies

**Files:**
- Create: `src/strategies/attributes.ph`
- Create: `src/strategies/derivation.ph`
- Modify: `src/core/errors.ph`

**Interfaces:**
- Consumes: reflected class attributes, constructors, parameters, contracts, and ordinary strategies.
- Produces: passive `arbitrary`, passive `strategy(Type)`, `_ConstructorStrategy`, `_DerivedStrategy`, deterministic metadata fingerprints, and unsafe-constructor diagnostics.

- [x] Define metadata without wrapping or dispatch changes.
- [x] Resolve every typed constructor parameter through the registry.
- [x] Reject missing annotations, rest parameters, multiple constructors, and precondition-constrained constructors.
- [x] Preserve constructor parameter labels and invoke the reflected constructor with ordered arguments.

### Task 3: Registry precedence and nested diagnostics

**Files:**
- Modify: `src/strategies/registry.ph`

**Interfaces:**
- Consumes: built-in registrations, applied types, annotated providers, derivation workers.
- Produces: `register(type:, strategy:)`, compatibility `register(type:, use:)`, `register(provider:)`, exact/custom/derived precedence, derived caching, recursion-safe resolution, and complete path errors.

- [x] Keep exact registrations first.
- [x] Install zero-argument `@strategy(Type)` provider methods as exact entries.
- [x] Recursively resolve generic arguments with path segments.
- [x] Cache successful derivations by canonical type descriptor.
- [x] Include every field/container step in failure diagnostics.

### Task 4: Sealed and recursive derivation

**Files:**
- Modify: `src/strategies/derivation.ph`

**Interfaces:**
- Consumes: sealed-root variant metadata and constructor derivation.
- Produces: terminal `oneOf`, size-aware recursive expansion, nested recursive-field substitution, and non-terminating hierarchy rejection.

- [x] Discover variants in stable name order.
- [x] Partition terminal and recursive variants.
- [x] Build terminal strategies without recursive references.
- [x] Build recursive variants using the child strategy supplied by `Gen.recursive`.
- [x] Reject recursive roots without a terminal variant.

### Task 5: Facade, documentation, examples, and checkpoint

**Files:**
- Modify: `src/hypothesis.ph`
- Modify: `docs/inference.md`
- Create: `docs/design/phase-10-derivation.md`
- Modify: `examples/derived_data.ph`
- Modify: `examples/recursive_expression.ph`
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `phalcom.toml`
- Modify: `CHECKPOINT.md`
- Modify: `TEST-RESULTS.md`
- Regenerate: `SHA256SUMS`
- Create: `phalcom-hypothesis-phase-10-derivation.zip`

**Interfaces:**
- Produces: public derivation attributes, documented precedence and limitations, executable examples, and a complete independently verifiable Phase 10 archive.

- [x] Run Python byte-compilation and Phase 01–10 verifiers in the working tree.
- [x] Run the Phalcom gate only when an executable exists; otherwise record runtime behavior as unobserved.
- [x] Regenerate checksums, build the full archive, extract it cleanly, verify checksums, and rerun all verifiers.
- [x] Record exact counts, archive SHA-256, and Phase 11 as the next phase.

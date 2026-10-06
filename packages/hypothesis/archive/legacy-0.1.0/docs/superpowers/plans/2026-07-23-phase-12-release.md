# Phase 12 Final Integration and Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Produce the release-complete Hypothesis for Phalcom package with no historical implementation artifacts, a single authoritative façade, complete migration/public-API documentation, and independently repeatable release verification.

**Architecture:** Phase 12 changes no search semantics. It removes the temporary monolith/adapters, retains compatibility names only as direct aliases to authoritative modules, publishes a definitive public API inventory, and adds a release verifier that composes all prior static gates. The final ZIP is a full clean snapshot whose checksum manifest and verifier results match a fresh extraction.

**Tech Stack:** Typed Phalcom source, Python 3 static/source verifiers, ZIP and SHA-256 release tooling.

## Global Constraints

- Preserve the single `DrawData` → immutable `Example` → replay/shrink search kernel.
- Remove `legacy/`, `src/_internal/legacy_adapter.ph`, `src/_internal/phase01_surface.ph`, and the obsolete migration generator from the release archive.
- Preserve broad-v1 compatibility names only as direct root aliases to authoritative modules.
- Do not claim Phalcom runtime results when no `phalcom` executable is installed.
- Final version is `0.1.0`; final archive is `phalcom-hypothesis-complete.zip`.

---

### Task 1: Release acceptance gate

**Files:**
- Create: `scripts/verify_phase12.py`
- Create: `scripts/verify_release.py`
- Create: `tests/integration/release_facade.ph`
- Create: `tests/integration/release_compatibility.ph`
- Create: `tests/integration/release_examples.ph`
- Create: `tests/integration/release_persistence.ph`

**Interfaces:**
- Consumes: Phase 01–11 source and verifier contracts.
- Produces: a failing pre-release gate covering artifact removal, façade completeness, docs/version, test/example inventory, and repeatable verification.

- [x] Write the Phase 12 verifier and release orchestration script.
- [x] Run `python3 scripts/verify_phase12.py` against untouched Phase 11 and record the expected failures.
- [x] Run `python3 scripts/verify_release.py --single-pass` and confirm it fails only at Phase 12.

### Task 2: Remove historical implementation boundaries

**Files:**
- Delete: `legacy/`
- Delete: `src/_internal/legacy_adapter.ph`
- Delete: `src/_internal/phase01_surface.ph`
- Delete: `scripts/migrate_phase01.py`
- Move: `tests/legacy/hypothesis_broad_v1_acceptance.ph` → `tests/integration/broad_v1_migration.ph`
- Modify: `src/hypothesis.ph`
- Modify: inherited verifiers that explicitly require removed files

**Interfaces:**
- Consumes: authoritative modules already exported by the root façade.
- Produces: one release façade with no adapter import or duplicate implementation.

- [x] Remove obsolete files and imports.
- [x] Keep `Check`, `CheckConfig`, `PropertyReporter`, `RuleBasedStateMachine`, and `Invariant` as direct compatibility aliases.
- [x] Update historical verifiers to preserve their semantic checks under the final release ownership model.
- [x] Run Phase 01–12 verifiers.

### Task 3: Final public API, migration, and release docs

**Files:**
- Create: `docs/public-api.md`
- Create: `docs/migration-from-monolith.md`
- Create: `docs/design/phase-12-release.md`
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `phalcom.toml`
- Modify: `tests/integration/package_loads.ph`

**Interfaces:**
- Consumes: every root façade alias.
- Produces: exact public-name inventory and import fixture, final package metadata, and migration instructions.

- [x] Document every stable and compatibility export.
- [x] Make `package_loads.ph` import every documented export exactly once.
- [x] Set manifest version to `0.1.0` and finalize release metadata.
- [x] Add migration guidance from monolith/prototype names to authoritative v1 names.
- [x] Run Phase 12 and release verifiers.

### Task 4: Final integration and archive evidence

**Files:**
- Modify: `CHECKPOINT.md`
- Modify: `TEST-RESULTS.md`
- Modify: `SHA256SUMS`
- Create: `/mnt/data/phalcom-hypothesis-complete.zip`

**Interfaces:**
- Consumes: clean release tree and all verification scripts.
- Produces: final full archive and exact static/runtime-boundary evidence.

- [x] Run Python compilation, Phase 01–12 verifiers, mutation verification, and the release verifier twice.
- [x] Run `phalcom test --all` and every example only if a real executable is available; otherwise record the limitation precisely.
- [x] Regenerate `SHA256SUMS` for every project file except itself.
- [x] Build the final ZIP, extract it separately, verify checksums, and rerun all gates.
- [x] Record archive file count and SHA-256.

# Final Checkpoint — Hypothesis for Phalcom 0.1.0

## Status

The release-complete package is finished at the source/static-verification level.

Hypothesis for Phalcom now provides one authoritative implementation for:

- typed primitive choices, semantic spans, immutable examples, generation, and replay;
- typed strategies, combinators, collections, composite draws, recursion, and reflected derivation;
- explicit, reuse, generation, shrink, and verification phases through one search kernel;
- source-aware failures, notes, classifications, event statistics, reporters, and reproduction;
- memory and directory example databases with versioned codecs, bounded retention, corruption recovery, atomic replacement, and merge-on-write;
- rule-based stateful testing with applicability predicates, typed bundles, result references, invariants, teardown, persistence, and structural action shrinking;
- stable provider, strategy, shrink-pass, database, and reporter extension boundaries.

The 0.1.0 release removes:

```text
legacy/
src/_internal/legacy_adapter.ph
src/_internal/phase01_surface.ph
scripts/migrate_phase01.py
tests/legacy/
```

The broad-v1 acceptance fixture is preserved as `tests/integration/broad_v1_migration.ph`. Compatibility names remain direct root aliases to authoritative modules; no compatibility implementation path remains.

## Locked architecture

> Strategies consume typed primitive choices through `DrawData`. Generation records an immutable semantic example. Replay supplies that example again. Shrinking transforms the example and reruns the complete property. Ordinary property testing, `find`, persistent replay, stateful testing, and reflected derivation use the same search kernel.

Phase 12 does not alter this model. It finalizes package ownership, documentation, metadata, release tests, and archive verification.

## Final release additions

- Finalized package metadata at version `0.1.0`.
- Added `docs/public-api.md` as the normative inventory of all 67 root exports.
- Added `docs/migration-from-monolith.md` with direct prototype-to-release migration guidance.
- Added `docs/design/phase-12-release.md` and the completed Phase 12 execution plan.
- Added release integration fixtures for the façade, compatibility aliases, executable examples, and persistent settings.
- Added `scripts/verify_phase12.py` and the two-pass `scripts/verify_release.py` gate.
- Updated every inherited verifier only where final-version syntax or removed adapter ownership superseded its historical assumption; all original feature checks remain active.
- Rewrote the README as final 0.1.0 package documentation.
- Finalized the changelog with the release-complete entry.

## Public façade

`src/hypothesis.ph` imports only authoritative feature modules. Its 67 exports exactly match:

```text
docs/public-api.md
tests/integration/package_loads.ph
```

Broad-v1 compatibility aliases are direct:

```phalcom
const Check = propertyAttributes.WithSettings
const CheckConfig = coreSettings.Settings
const RuleBasedStateMachine = statefulMachine.StateMachine
const Invariant = statefulAttributes.StateInvariant
```

`PropertyReporter.console` remains an authoritative console-reporter compatibility factory.

## TDD evidence

The first Phase 12 verifier run against the untouched Phase 11 checkpoint produced the intended red state:

```text
1 passed, 6 failed
```

The failures covered:

- prerelease package metadata;
- historical monolith and adapter files;
- incomplete root-import coverage;
- missing migrated broad-v1 fixture;
- retired control-flow syntax in the historical fixture;
- missing final migration/release documentation.

After implementation, the Phase 12 gate reports:

```text
8 passed, 0 failed
```

## Final observed source/static summary

```text
All tests passed.
No legacy syntax found.
No placeholder implementations found.
All public façade imports resolved.
```

In this summary, “tests” means the observed Python source/static verification gates. No `phalcom` executable was installed in the checkpoint environment, so the required `phalcom test --all` command, `.ph` fixture execution, executable examples, installed-package runtime comparison, real cross-process persistence, and benchmark timings remain unobserved.

## Source audit

```text
Phase 11 archive files: 215
Final project files: 221
Manifest entries: 220
Unexpected Phase 11 baseline files missing: 0
Intentional historical removals: 5
Missing internal imports: 0
Imbalanced active Phalcom files: 0
Retired syntax files: 0
Placeholder implementation files: 0
Python cache artifacts: 0
Root façade exports: 67
Façade/import-fixture agreement: exact
Façade/public-document agreement: exact
Executable example files: 7
Phalcom executable: not installed
```

The five intentional removals are the historical monolith, the two temporary adapter modules, the obsolete migration generator, and the old legacy test path. The broad-v1 fixture was moved rather than discarded.

## Verification and archive protocol

The release gate performs two complete passes. Each pass runs:

1. the Phase 11 mutation verifier;
2. every Phase 01–12 verifier in order.

`SHA256SUMS` covers every project file except itself. The final `phalcom-hypothesis-complete.zip` is extracted into a separate directory, then subjected to:

- ZIP integrity verification;
- `sha256sum -c SHA256SUMS`;
- Python compilation of every verifier;
- the complete two-pass release gate;
- final source/import/public-API audit.

The archive SHA-256 is reported externally after final construction because an archive cannot contain a stable checksum of itself.

## Release artifact

```text
phalcom-hypothesis-complete.zip
```

This is the final full-package checkpoint. There is no subsequent planned implementation phase.

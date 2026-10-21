# ARTF001.C1.P2.R4 — Artifact Importer Disposition Ledger

## Scope and conclusion

This ledger classifies every module under `phalcom-semantic/src/artifact/` for
R4.T6. The compatibility transaction previously built a parallel semantic
authority through doc-hidden public APIs. It is now gated under `cfg(test)`;
the 20 semantic-artifact behavioral tests were moved into the unit-test build
and pass. T6 remains open until the normal library check and T2–T5 focused
regressions pass after this consolidation.

The canonical production route is:

```text
core canonical artifact loader
  -> validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget
  -> reconstruct_query_image_v4_with_allocation_budget
  -> CanonicalUniverseFormalImage
  -> SemanticWorkspaceSession::from_image
```

There is one production call path for the semantic importer, one for V4 query
reconstruction, one current `TypeStore` relocation, one current
`StaticValueStore` relocation, and one production proof/evidence reconstruction
path under the native importer. Focused allocation tests also call narrow
diagnostic entry points.

## Disposition vocabulary

- **CANONICAL** — current producer or import/query path owns this behavior.
- **RETAIN_HELPER** — shared helper used by the canonical path, without a
  separate semantic authority.
- **HISTORICAL_TEST_ONLY** — legacy staged importer code and fixtures compiled
  only for tests or removed. The entries marked below are now cfg-gated under
  `cfg(test)`.
- **VALIDATION_ONLY** — validates hostile input without reconstructing or
  publishing an alternate semantic generation.

No module is unclassified. The test-only boundary is enforced in code; the
post-consolidation recertification remains outstanding.

## Canonical producer and loader modules

| Module | Disposition | Responsibility / evidence |
|---|---|---|
| `export.rs` | CANONICAL | Current-schema `SemanticWorldV4` and `QueryImageV4` projection called by the core artifact producer. |
| `export/explanation.rs` | RETAIN_HELPER | Stable explanation/provenance projection used by `export.rs`. |
| `export/proof.rs` | RETAIN_HELPER | Callable proof projection and round-trip support used by `export.rs`. |
| `import.rs` | CANONICAL | Native current-main semantic reconstruction; owns type/static relocation, semantic families, callable/proof authority, snapshot construction, and module-owner indexes. |
| `import/diagnostic.rs` | RETAIN_HELPER | Lossless diagnostic relocation used by the canonical importer. |
| `import/explanation.rs` | RETAIN_HELPER | Explanation identity relocation used by callable/proof import. |
| `import/proof.rs` | RETAIN_HELPER | Callable proof graph reconstruction used by the canonical importer. |
| `query.rs` | CANONICAL | Current V4 root validation, dependency closure, relocation, fingerprints, and immutable query image reconstruction; one core production call site. |
| `query_policy.rs` | CANONICAL | Exhaustive current query persistence/support policy consumed by export, manifest, and V4 import. |
| `identity.rs` | RETAIN_HELPER | Project-qualified stable owner catalog shared by current semantic/query reconstruction. |
| `roots.rs` | RETAIN_HELPER | Canonical semantic/type/static export-root collection. |
| `operational.rs` | RETAIN_HELPER | Source-free static-program wire conversion used by canonical export/import. |
| `manifest.rs` | CANONICAL | Explicit snapshot/session/query/compiler-attachment/import-family disposition manifest. |
| `validate.rs` | VALIDATION_ONLY | Fail-closed witness/static and executable validation; does not publish an alternate semantic image. |
| `mod.rs` — `CompilerArtifactImage`, encode/decode | CANONICAL | Portable wire container façade and encoding/validation. `CompilerArtifactImage` remains the source artifact DTO. |

## Legacy staged semantic importer — required test-only boundary

Each module below is invoked by `ArtifactImportTransaction::stage_payload` in
`artifact/mod.rs`. Together, these modules relocate a second set of
type/static, declaration, callable, dispatch, conformance, proof, constructor,
and query products into `ImportedCompilerImage`. They have no production call
sites outside that transaction. Repository callers of the compatibility
staging entry points are semantic artifact tests. The transaction, result type,
wrappers, family modules, and family re-exports are now `cfg(test)`. The former
integration fixture is now `src/artifact/compatibility_tests.rs` and remains
selected by the semantic crate's normal unit-test run.

| Module | Final disposition | Staged family |
|---|---|---|
| `associated_import.rs` | HISTORICAL_TEST_ONLY | Associated/type-member/static-domain authority |
| `callable_binding_import.rs` | HISTORICAL_TEST_ONLY | Callable binding and observed import-phase authority |
| `callable_dispatch_import.rs` | HISTORICAL_TEST_ONLY | Conditional callable dispatch capabilities |
| `callable_evidence_import.rs` | HISTORICAL_TEST_ONLY | Callable-local conformance evidence |
| `callable_expression_import.rs` | HISTORICAL_TEST_ONLY | Callable expression facts and deferred proof authority |
| `callable_import.rs` | HISTORICAL_TEST_ONLY | Callable signatures and definitions |
| `callable_specialization_import.rs` | HISTORICAL_TEST_ONLY | Generic call specialization sidecars |
| `conformance_evidence_import.rs` | HISTORICAL_TEST_ONLY | Exact and derived conformance evidence graph |
| `conformance_import.rs` | HISTORICAL_TEST_ONLY | Explicit conformance heads |
| `constructor_import.rs` | HISTORICAL_TEST_ONLY | Constructor initialization products |
| `denotation_import.rs` | HISTORICAL_TEST_ONLY | Semantic denotation relocation |
| `derived_import.rs` | HISTORICAL_TEST_ONLY | Derived conformance rules and proof rows |
| `diagnostic_import.rs` | HISTORICAL_TEST_ONLY | Staged semantic diagnostics |
| `dispatch_import.rs` | HISTORICAL_TEST_ONLY | Declaration dispatch and field signatures |
| `enum_requirement_import.rs` | HISTORICAL_TEST_ONLY | Enum requirement/status products |
| `inherent_import.rs` | HISTORICAL_TEST_ONLY | Conditional inherent domains and members |
| `module_static_import.rs` | HISTORICAL_TEST_ONLY | Module static binding indexes |
| `query_import.rs` | HISTORICAL_TEST_ONLY | Portable query-owner to live-key conversion for the staged path |
| `query_product_import.rs` | HISTORICAL_TEST_ONLY | Staged structural query-product reconstruction/publication |
| `semantic_import.rs` | HISTORICAL_TEST_ONLY | Staged structural declaration aggregates |
| `static_case_import.rs` | HISTORICAL_TEST_ONLY | Static-case catalogs and signatures |
| `static_constant_import.rs` | HISTORICAL_TEST_ONLY | Checked static constants |
| `static_definition_import.rs` | HISTORICAL_TEST_ONLY | Static definitions/programs |
| `static_dependency_import.rs` | HISTORICAL_TEST_ONLY | Static-program dependency relocation |
| `static_import.rs` | HISTORICAL_TEST_ONLY | Portable static-value relocation for the staged path |
| `static_program_import.rs` | HISTORICAL_TEST_ONLY | Staged canonical static programs and applications |
| `trait_import.rs` | HISTORICAL_TEST_ONLY | Trait headers, requirements, and associated types |
| `type_import.rs` | HISTORICAL_TEST_ONLY | Portable type graph relocation for the staged path |
| `witness_import.rs` | HISTORICAL_TEST_ONLY | Staged witness-plan preflight and selections |
| `mod.rs` — transaction, `ImportedCompilerImage`, compatibility imports | HISTORICAL_TEST_ONLY | Parallel transaction and staged semantic result, gated under tests. |

## Caller and authority census

| Surface | Production count | Test/compatibility callers | Disposition |
|---|---:|---|---|
| Complete native semantic artifact import | 1 | Focused loader allocation tests | Canonical: core loader calls `artifact::import`'s current entry point. |
| V4 query reconstruction | 1 | Focused query allocation tests | Canonical: current `artifact::query` entry point called by the same core loader. |
| Current type relocation | 1 | Native importer unit tests | Canonical: `TypeStore::import_type_graph_with_allocation_budget` in native import. |
| Current static-value relocation | 1 | Native importer unit tests | Canonical: `import_static_value_graph_with_allocation_budget` in native import. |
| Native proof/evidence reconstruction | 1 orchestration path | Native importer family tests | Canonical: proof/witness/callable evidence stages under `artifact::import`. |
| Compatibility staging transaction | 0 production call sites; excluded from non-test builds | `artifact::compatibility_tests`; generic package receipt tests exercise `phalcom-modules` staging | HISTORICAL_TEST_ONLY, enforced with `cfg(test)`. |

The generic `ProjectUniverse`, `TrustedPackageResolver`, and
`stage_artifact_project_authorities` contracts remain in `phalcom-modules`.
Package receipts still bind exact package/version/fingerprint identity, the
artifact digest, and the persistent target identity. No T6 change may replace
those checks with an in-memory-only trust decision.

## T6 closure condition

Before marking R4.T6 complete:

1. Verify a normal library build excludes the legacy transaction,
   `ImportedCompilerImage`, its family staging modules, and compatibility
   wrappers; keep their behavior covered by unit tests.
2. Preserve generic package receipt tests and exact package/version/fingerprint,
   artifact-digest, and target-identity checks.
3. Run migrated compatibility tests and P2 T2–T5 focused regressions; verify
   one production semantic consumer, one production V4 query path, one current
   type/static relocation path, one proof/evidence path, and zero unknown
   dispositions.

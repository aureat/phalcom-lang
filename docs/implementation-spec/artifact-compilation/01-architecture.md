# 01 — Architecture of general artifact compilation

## 1. Goal

A compiler artifact must be buildable for **any** package, not only for
Universe. A package artifact:

- owns exactly the modules of one package project, and the semantic and query
  products rooted in those modules;
- refers to every module, declaration and query that it does not own through
  stable identities that resolve against **dependency artifacts loaded
  earlier**;
- loads source-free: downstream code compiles against it without parsing or
  checking the package's source.

The first proof target is `packages/slotmap` (two source modules plus `main`
and `package`) compiled over the Universe artifact. Executable package
artifacts are **out of scope** (see `09-open-questions.md` Q1). This spec
covers compiler-only package artifacts.

## 2. What exists today

| Concern | Today | Anchor (greppable) |
|---|---|---|
| Artifact container | Four required sections: module world, semantic world, query image, metadata; executable optional | `TeloscContainerBuilder`, `SectionKind` |
| Module world wire | V2 rows + V3 witnesses + V4 bootstrap contract. Every module reference is a dense `ArtifactModuleId` | `ModuleWorldV2`, `ModuleWorldV3`, `ModuleWorldV4` in `phalcom-compiler-artifact/src/module_world.rs` |
| Module world projection | Universe only. Rejects `ProjectIdentity::Resolved`, hard-codes `Builtin{universe,0.1.0}` | `project_module_world`, `project_executable_module_world_with_ids` |
| Closed-world guard | `portable_id` fails on any module outside the artifact | `portable_id` in `phalcom-modules/src/compiler_artifact.rs` |
| Semantic world wire | Uses stable refs (`StableModuleRef`, `StableDeclarationRef`, …), not dense indices | `SemanticWorldV4` |
| Query image wire | Uses stable refs; dependencies on non-owned queries become fingerprint evidence | `QueryImageV4`, `export_query_image_v4_scoped` |
| Ownership scoping | Slices A–D: scope, roots, semantic export, query export | `ArtifactProjectionScope`, `register_semantic_world_roots_scoped`, `export_semantic_world_v4_scoped`, `export_query_image_v4_scoped` |
| Stable identity of a project | `to_stable_project` **panics** on `Resolved`; only `*_with_context` variants handle packages | `to_stable_project`, `to_stable_project_with_context` |
| Package load / compose | Dependency-ordered, Universe as generation 0, one `ExternalModuleAuthoritySet` at the end | `StagedTeloscCompilerWire::compose_artifact_semantic_base_impl`, `plan_package_dependencies` |
| Package trust | Resolver-issued receipt binds `StableProjectRef::Package` to artifact digest and a persistent project | `ValidatedPackageAuthorityReceipt`, `register_trusted_package_authority` |
| Semantic owner resolution on import | Owned modules must exist in the rows. External owners need only an allowlisted project | `StableOwnerCatalog::project_for_module_parts` |

## 3. Target pipeline

```
          ┌──────────── build (per package) ─────────────┐
 source ─►│ compile over composed dependency base        │
          │   → PackageCompilerImage (F2)                │
          │ ArtifactProjectionScope = owned modules (A)  │
          │ roots (B) → semantic (C) → query (D)         │
          │ module world V5 with external table (E)      │
          │ publishing identity for owned project (F1)   │
          │ header + container (F3)                      │
          └──────────────────────────────────────────────┘
                              │ .telosc bytes
                              ▼
          ┌──────────── load (per composition) ──────────┐
          │ plan dependency order (existing)             │
          │ gen 0 = Universe artifact                    │
          │ for each package in order:                   │
          │   authority = Universe + earlier packages    │
          │   load module world V5 against authority (G) │
          │   import semantic/query into shared stores   │
          │ final ExternalModuleAuthoritySet             │
          └──────────────────────────────────────────────┘
                              │
                              ▼
            SemanticWorkspaceSession::from_external_artifact_base
```

## 4. Invariants

These hold across every slice. A change that breaks one is a defect, even
when its tests are green.

- **ARCH-I1 — Ownership is identity-derived.** A module is owned by an
  artifact iff its `ModuleId.project` is the artifact's project. Every
  ownership predicate derives from the module inside the native identity
  (`ArtifactProjectionScope::owns_*`), never from names, targets or file
  paths.
- **ARCH-I2 — One owner per module.** No module row appears in more than one
  artifact. Enforced at load by `ExternalModuleAuthoritySet::from_worlds`
  (duplicate module and project ownership) and at decode by the
  foreign-owner guard in `stage_artifact_project_authorities` /
  `package_import_order`. Under V5 these guards stay. Foreign modules go in
  the external table, never in the rows.
- **ARCH-I3 — Local-only positions stay local.** `entry`,
  `initialization_order`, topology parent/children, the module row a
  semantic row describes, a witness importer, and bootstrap schedule entries
  can only name owned modules. The type system enforces this
  (`ArtifactModuleId`), not runtime checks (rule `MW5-T1`).
- **ARCH-I4 — External references resolve to loaded authority.** Every
  external module reference resolves to a module present in an artifact
  loaded **earlier** in the composition order. A reference to a module not
  yet loaded, or absent, is a load error. It never triggers a source
  fallback.
- **ARCH-I5 — Dependencies are declared.** The project of every external
  module appears in `external_projects`. The dependency planner orders
  artifacts by `external_projects`, so an undeclared dependency could be
  loaded after its importer.
- **ARCH-I6 — Scope agreement.** Root registration, semantic export, query
  export and module-world projection use one `ArtifactProjectionScope`
  value. Universe uses `ArtifactProjectionScope::whole_image()`, and its
  output stays byte-identical to the pre-ARTC builder (regression gate
  `VER-R1`).
- **ARCH-I7 — Path independence.** Artifact bytes do not depend on the
  filesystem location of the package source. No `logical_uri` or
  path-derived fingerprint for an owned or external package may appear in
  any section (rule `EXP-P3`, test `VER-D2`).
- **ARCH-I8 — Fail closed, all or nothing.** Every load step validates
  before it publishes, and charges the caller's `AllocationBudget`. A failure
  leaves the receiving stores and project catalog unchanged (this is the
  existing transaction discipline; new code follows it).

## 5. Responsibilities by crate

| Crate | New responsibility |
|---|---|
| `phalcom-compiler-artifact` | `ModuleWorldV5` wire types, encode/decode, local validation, portable fingerprint, V4→V5 lift; container accepts schema 5. |
| `phalcom-modules` | Package module-world projection (`project_package_module_world_v5`); V5 loader against an `ExternalModuleAuthority`; foreign-owner guards retained. |
| `phalcom-type-meta` / `phalcom-semantic` (metadata) | Publishing-identity override in `StableIdentityContext`. |
| `phalcom-semantic` (artifact) | Identity context threaded through export; `StableOwnerCatalog` checks external module existence. |
| `phalcom-core` | Package compiler image capture, `build_package_telosc`, composition passes earlier-generation authority to each package load, SlotMap proof tests. |

## 6. Non-goals

- Executable package artifacts (`PORTABLE_EXECUTABLE` on a package). Package
  staging continues to reject it (`stage_package_semantic_generation*`).
- Moving Universe to `packages/universe` (ARTF001 P5).
- Changing the Universe module-world wire. Universe keeps emitting V4 in this
  program (ruling `D-MW5-3`).
- Package resolution or distribution (registry, cache layout). The test
  resolver issues receipts directly.

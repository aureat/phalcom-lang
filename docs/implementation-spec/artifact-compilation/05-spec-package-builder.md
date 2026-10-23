# 05 — Spec: package compiler image and builder (slices F2, F3)

Crate: `phalcom-core` (`src/modules/`), with module-world projection in
`phalcom-modules`.

## 1. Inputs and base

**PB-B1. Compile over artifacts, never over dependency source.** The package
is checked and linked over a **composed artifact base**: the Universe
artifact (generation 0) plus any already-built dependency package artifacts.
This means a `LoadedUniverseArtifact` for Universe-only packages, and a
`ComposedArtifactSemanticBase` otherwise. Dependency sources MUST NOT be
parsed or checked. Gate: the package build passes with
`PHALCOM_DENY_SOURCE_BOOTSTRAP=1` and
`canonical_universe_formal_image_build_count() == 0` before and after the
build (same pattern as `fresh_process_artifact_executable_runner`).

**PB-B2. Linking against external authority.** Package modules are linked
with the dependency modules supplied through
`ModuleLinker::with_external_modules` (an `ExternalModuleAuthority`), so the
linked program's module set is **exactly the owned modules**. External
modules are link targets, never linked rows.

**PB-B3. Owned scope.** `ArtifactProjectionScope::new(owned)`, where `owned`
is every module whose `ModuleId.project` is the package's
`ResolvedProjectId`. Assert that the scope equals the linked program's module
key set. A mismatch is a build error (ARCH-I6).

## 2. Package compiler image (F2)

**PB-I1.** Introduce a value that plays the role
`CanonicalCompilerImageView` plays for Universe:

```rust
/// Completed, immutable compiler products of one package compiled over a
/// composed artifact base. Owned rows only; dependency products are
/// referenced, never copied.
pub struct PackageCompilerImage {
    pub project: ResolvedProjectId,
    pub identity: StableProjectRef,                  // EXP-P1, Package variant
    pub scope: ArtifactProjectionScope,              // PB-B3
    pub projects: Arc<ProjectUniverse>,
    pub modules: Arc<CanonicalExternalModuleWorld>,  // owned linked + interfaces + topology
    pub linked: Arc<ExecutableLinkedProgram>,        // owned modules, runtime requirements finalized
    pub snapshot: Arc<SemanticSnapshot>,             // session snapshot (contains external rows; export filters by scope)
    pub queries: Arc<CanonicalSemanticQueryImage>,   // owned queries frozen (PB-I3)
    pub dependency_fingerprints: Arc<CanonicalDependencyFingerprints>, // PB-I4
    pub metadata: Arc<SemanticMetadataBundle>,       // owned modules only (PB-I5)
    pub external: Arc<dyn ExternalModuleAuthority>,  // the base used for linking
    pub source_digest: ArtifactDigest,               // EXP-P4
}
```

The field set is normative. The capture path is the implementer's to
locate. The template is the Universe build
(`canonical_image.rs`: `build` → `SemanticWorkspaceSession` with
`SemanticProductRequest::canonical_compiler_image()` → `freeze`). The
package path starts from
`ComposedArtifactSemanticBase::semantic_session` /
`SemanticWorkspaceSession::from_external_artifact_base` instead of
`from_baseline`.

**PB-I2. Diagnostics block capture.** If the package has any error
diagnostic, or any owned query is `Failed`/`Blocked`, the capture fails with
those diagnostics. No artifact is produced from a package with errors.

**PB-I3. Scoped freeze.** Generalize `CanonicalSemanticQueryImage::freeze`
with a scope parameter. Keys that `scope.owns_query` accepts follow today's
rules (must be committed and ready; census classification unchanged). Keys
owned by no module (`TraitDispatchBucket`, `GenericConformance`;
`query_owner_module` returns `None`) and keys owned by external modules are
**not frozen**: they are dependency evidence. The Universe call passes
`whole_image()` and is unchanged (VER-R1).

**PB-I4. Dependency fingerprints.** For every frozen owned query, each
dependency key resolves to a fingerprint: from the owned image if owned,
otherwise from the external base's `dependency_fingerprint` (the
`ExternalSemanticQueryCatalogue` / Universe image). A dependency with no
fingerprint is a build error ("unresolved external query dependency"). This
satisfies slice D's precondition that the caller supply fingerprints
including external evidence.

**PB-I5. Metadata scope.** The metadata bundle contains entries for owned
modules only. References to external declarations inside owned entries are
permitted only through projects in `external_projects` (this is the existing
payload rule that checks `module_world.external_projects`).

## 3. Package module-world projection (part of E/F3, `phalcom-modules`)

**PB-M1.** New function, sibling of `project_executable_module_world_v4`:

```rust
pub fn project_package_module_world_v5(
    world: &CanonicalExternalModuleWorld,      // owned modules only
    executable: &ExecutableLinkedProgram,      // owned modules only
    external: &dyn ExternalModuleAuthority,
    identity: &StableIdentityContext<'_>,      // EXP-C1 publishing context
    limits: &ValidationLimits,
) -> Result<ModuleWorldV5, ArtifactError>;
```

Rules:

- **PB-M2.** Local ids: owned modules in `ModuleId` order → `ArtifactModuleId`
  (the same ordering rule as Universe).
- **PB-M3.** Module identities via `to_stable_module_with_context(identity)`.
  Owned modules export with the publishing identity, external modules with
  their receipt-bound `Package` or `Builtin` identity.
- **PB-M4.** Replace `portable_id` with
  `module_ref(local_ids, external_index, id) -> Result<ModuleRefV1>`. If `id`
  is owned, return `Local`. Otherwise, if `external.contains(id)`, return
  `External`, interning the stable ref into a `BTreeMap<StableModuleRef, ()>`
  collected during projection. Otherwise fail ("module {id} is neither owned
  nor provided by the dependency base").
- **PB-M5.** After collecting, sort the external refs and assign
  `ExternalModuleIndex` in order. Then **rewrite** provisional external refs
  to final indices. Simplest: do two passes, first collecting the external
  set, then projecting. Do not emit before indices are final.
- **PB-M6.** `external_projects` = the sorted set of projects of
  `external_modules`, plus any project referenced by semantic/query/metadata
  owners (computed by the builder; see PB-H3). The world is valid under
  MW5-V3.
- **PB-M7.** Local-only positions (`entry`, `initialization_order`, topology,
  semantic row nodes, witness importers) use a `local_id` helper that fails
  on non-owned input. Under PB-B2 such input is a projection bug, and it must
  fail loudly.
- **PB-M8.** `bootstrap_modules` and `bootstrap_availability` are empty
  (MW5-V7).
- **PB-M9.** Witnesses for runtime requirements on Universe modules are
  projected from the finalized runtime requirements the same way V3 does
  today, with `External` targets. Before including each one, check that the
  Universe `AvailabilityContract` of the base contains a matching proof
  (target, product, authority, timing). The build fails otherwise. This is
  the build-side twin of LD-W1.
- **PB-M10.** Call `refresh_portable_topology_fingerprint` and
  `validate(limits)` before returning.

## 4. Builder (F3)

**PB-H1. Entry point.**

```rust
pub fn build_package_telosc(
    image: &PackageCompilerImage,
    limits: &ValidationLimits,
) -> Result<BuiltPackageTelosc, TeloscArtifactError>;
```

`BuiltPackageTelosc { bytes, summary, identity: StableProjectRef }`. It MUST
NOT call `build_universe_telosc`, `register_semantic_world_roots` (whole
image), `export_semantic_world_v4_with_runtime_requirements` (whole image) or
`export_query_image_v4` (whole image). It calls only the `_scoped` entry
points with `image.scope`.

**PB-H2. Order of operations.** This mirrors `build_universe_telosc`, with
the scoped and identity-threaded calls.

1. Compute `identity` (EXP-P1/P2) and build the publishing context (EXP-C1).
2. Module world: `project_package_module_world_v5` (PB-M1).
3. Semantic world: `TypeExportContext` / `StaticExportContext` over the
   snapshot stores → `register_semantic_world_roots_scoped(projection, …)` →
   `statics.register_type_roots` (safe after scoped registration, since it
   only walks static values already rooted) → `close_graph` (types, statics)
   → `export_semantic_world_v4_scoped(projection, …, &module_world, image.linked.runtime_requirements(), limits)`.
4. Query image:
   `export_query_image_v4_scoped(&image.scope, &image.queries, &image.dependency_fingerprints, &module_world, &semantic_world, limits)`.
   Both scoped exporters take `&ModuleWorldV2` at the anchor. MW5-M1
   migrates them to `&ModuleWorldV5`.
5. Metadata: `MetadataSectionV1::from_bundle(image.metadata, package_digest, …)`,
   with `package_digest = TeloscPackageIdentity::Package{..}.digest()`.
6. Header (PB-H4), container, self-validation by parsing the built bytes
   (`TeloscContainerView::parse`). Then run **load self-check** (PB-H5).

**PB-H3. Project closure check.** After steps 2–4, collect every
`StableProjectRef` used as an owner in the semantic world and query image
(walk the owner fields; the wire families are enumerated in
`StableOwnerCatalog` users under `phalcom-semantic/src/artifact/*_import.rs`).
Every non-owned project must be in the module world's `external_projects`.
If one is missing, add it and re-project (one retry). Fail if it is still
missing. This prevents import-time "stable module owner project is neither
internal nor allowlisted".

**PB-H4. Header.**

- `package_identity: TeloscPackageIdentity::Package { package, version }`;
- `capabilities: CANONICAL_COMPILER_IMAGE_V1 | RUNTIME_PUBLIC_METADATA`, never
  `PORTABLE_EXECUTABLE`;
- `source_digest` from EXP-P4;
- ABI fields copied from the base Universe artifact header (a package built
  against a different ABI is not loadable with that Universe);
- section digests computed as in the Universe builder;
- `section_count: 4`;
- `MODULE_WORLD_SCHEMA_V5` for the module-world section.

**PB-H5. Load self-check.** Before returning, the builder loads its own
bytes through the production composition path (`06`) over the same base,
with a resolver that issues a receipt for `identity` bound to the built
digest and to `image.project`. A builder whose output its own loader rejects
is a build error. This catches every builder/loader asymmetry at the source.

**PB-H6. Determinism.** Same inputs give byte-identical output: the same
package sources, the same base artifacts, and any filesystem location (tests
VER-D1, VER-D2). No `HashMap` iteration may reach the wire. Sort every set
before emission.

## 5. CLI surface (optional in this program)

**PB-C1.** If added, use
`phalcom artifact build-package <project.toml> --universe <universe.telosc> [--dep <pkg.telosc>]... -o <out.telosc>`.
It is not required for slice H, whose proof is test-driven. If the CLI is
added, it must go through `build_package_telosc` only.

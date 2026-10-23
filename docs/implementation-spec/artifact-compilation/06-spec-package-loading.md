# 06 — Spec: external-aware loading and composition (slice G)

Crates: `phalcom-modules` (module-world loader), `phalcom-semantic`
(`StableOwnerCatalog`), `phalcom-core` (`telosc_compiler_wire.rs`).

## 1. Module-world loader

**LD-M1. Signature.** The module-world loaders take the external authority:

```rust
pub fn load_module_world_v5_with_project_catalog_and_allocation_budget(
    wire: &ModuleWorldV5,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
    projects: Arc<ProjectUniverse>,
    external: Option<&dyn ExternalModuleAuthority>,
) -> Result<CanonicalExternalModuleWorld, ArtifactError>;
```

`None` is allowed only when `wire.external_modules` is empty (the Universe
case); otherwise it is an error. Keep the existing V2 entry points as thin
wrappers (lift + `None`) until every caller has migrated.

**LD-M2. Resolve the external table first.** Before converting any module
row, resolve each `external_modules[i]`:

1. `projects.resolve_stable_project(&r.project)`. Absent: "external module
   project has no exact workspace binding".
2. Build `ModuleId { project, path }` with allocation charging (the pattern
   used for rows in `load_module_world_inner`).
3. `external.contains(&id)`. Absent: "external module {r:?} is not provided
   by any loaded dependency artifact" (ARCH-I4).
4. The resolved `ProjectIdentity` MUST differ from the resolved project of
   every local row. This is defense in depth for MW5-V4 after relocation:
   two distinct stable refs must not bind to one native project.

The result is a vector `external_ids: Vec<ModuleId>` parallel to the table.

**LD-M3. Reference conversion.** Replace the loader's
`module_id(ids, ArtifactModuleId)` with
`module_ref_id(ids, external_ids, ModuleRefV1)`. Local-only fields keep the
local-only helper.

**LD-M4. Binding targets.** For `TargetV2::Binding { module: External(i), name }`
in `linked_reads` or `linked_exports`, require that
`external.linked_module(&external_ids[i])` exports `name`. Otherwise fail
with "external binding {name} is not exported by {module}". For
`UnlinkedTargetV2::CanonicalDeclaration { module: External(i), name }`,
require that `external.interface(&external_ids[i])` declares `name`.

**LD-M5. Topology.** Live `ModuleTopology` nodes are built for local rows
only. External modules are not topology nodes of this world.
`CanonicalExternalModuleWorld::from_parts` requires the same key set for
linked modules, interfaces and topology. That stays true because all three
are local-only. External link targets inside `LinkedModule` values are plain
`ModuleId`s, which `from_parts` does not restrict.

**LD-M6. Project staging.** `stage_artifact_project_authorities` takes
`&ModuleWorldV5`. Its stable-project set is
`external_projects ∪ row projects`; the external table adds nothing, because
MW5-V3 makes it a subset of `external_projects`. Keep its foreign-owner
guard and error text unchanged.

## 2. Availability witnesses across artifacts

**LD-W1.** For each witness whose `dependency` is `External(i)`, resolve the
target to a `ModuleId` (and binding name). Then require that the
**Universe generation's** `AvailabilityContract` contains a proof with an
equal target, product, authority and `available_at`. The composed base
already holds Universe generation 0. Its availability contract is the one
`LoadedUniverseArtifact` retains for `compile_user_entry`. Thread it into
composition as an explicit argument. Do not re-derive it. If the proof is
missing, fail with "external availability witness has no matching
dependency environment proof".

**LD-W2.** External witnesses whose target belongs to a **non-Universe**
dependency package are rejected in this program. Package artifacts carry no
environment contract (MW5-V7), so nothing could justify them. Eager
`runtime_edges` must be used for package-to-package runtime dependencies
(D-MW5-4).

**LD-W3.** `validate_execution_authority` (now on `ModuleWorldV5`) matches
semantic runtime requirements by `StableModuleRef` on both ends. It resolves
a requirement's `dependency` against local rows first and then against
`external_modules`. A dependency found in neither is an error. This replaces
the "runtime requirement dependency is absent from module world" failure for
legitimately external dependencies.

## 3. Semantic owner resolution

**LD-C1.** `StableOwnerCatalog::new` takes `&ModuleWorldV5` and an optional
`&dyn ExternalModuleAuthority` plus the resolved `ProjectUniverse`.

**LD-C2.** For an owner whose project is not internal (today: "allowlisted"
via `external_projects`), additionally resolve the module and require
`external.contains(module_id)`. A missing module fails with "external stable
module owner is not provided by a loaded dependency". With `None` external
authority (Universe), the current behavior is kept.

**LD-C3.** Declaration-level existence for external owners (for example
`DeclarationId(slotmap.main, "SlotMap")` referenced by a downstream package)
is enforced where the importer resolves the declaration in the shared
receiving stores. Hostile test LDH-7 asserts that a forged external
declaration fails import. If the importer accepts it silently, that is an
incident: report it, and do not patch around it.

## 4. Composition

**LD-K1. Prefix authority per package.** In
`compose_artifact_semantic_base_impl`, before staging package `k` (in plan
order), build an authority over every world staged so far:

```rust
let prefix = ExternalModuleAuthoritySet::from_worlds(Arc::new(project_candidate.clone()), worlds.clone())?;
```

Pass it through `stage_package_semantic_generation_into_shared_stores` →
`stage_package_modules` → the V5 loader (LD-M1), and through to
`StableOwnerCatalog` (LD-C1). `from_worlds` runs per step, which is O(n²) in
the number of packages. That is acceptable at current package counts.
Rebuilding per step also catches duplicate ownership at the first offending
package, not at the end.

**LD-K2. Order dependence is a feature.** An artifact that references a
module of a package planned **after** it fails at LD-M2 step 3. The planner
(`plan_package_dependencies`) orders by `external_projects`, and MW5-V3
guarantees that every external module's project is declared. So this
failure only occurs for forged or inconsistent artifacts.

**LD-K3. Final authority unchanged.** After the loop, the final
`ExternalModuleAuthoritySet::from_worlds` and the query-image revalidation
against the final shared stores run as today.

**LD-K4. Decode entry.** `StagedTeloscCompilerWire::decode` decodes the
module world through the schema-dispatching V5 entry point (MW5-L2) and
stores `ModuleWorldV5`. Field paths `module_world.previous.world.*` become
`module_world.*`.

## 5. Session publication

No change. `ComposedArtifactSemanticBase::semantic_session` seeds
`SemanticWorkspaceSession::from_external_artifact_base`. A downstream
project compiles against it with the package's modules supplied as external
authority. Slice H exercises this path.

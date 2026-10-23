# 03 — Spec: `ModuleWorldV5` (slice E)

Crate: `phalcom-compiler-artifact` (`src/module_world.rs`). Loader changes
are in `06`.

## 1. Types

### MW5-T1: Two reference types

```rust
/// Dense index into this artifact's own module rows. Local-only positions
/// keep this type so they cannot name a dependency module (ARCH-I3).
pub struct ArtifactModuleId(pub u32);            // unchanged

/// Index into `ModuleWorldV5::external_modules`.
pub struct ExternalModuleIndex(pub u32);         // new

tagged!(ModuleRefV1 {
    0 => Local { module: ArtifactModuleId },
    1 => External { module: ExternalModuleIndex },
});
```

Provide `ModuleRefV1::local(self) -> Option<ArtifactModuleId>` and
`ModuleRefV1::external(self) -> Option<ExternalModuleIndex>`. Do **not**
provide a conversion that collapses both arms into one integer.

### MW5-T2: New records

The new records replace V1/V2 records in positions that may name a
dependency. Any field not listed is identical to its predecessor.

| New record | Predecessor | Changed fields |
|---|---|---|
| `TargetV2 { 0 => Module { module: ModuleRefV1 }, 1 => Binding { module: ModuleRefV1, name: String } }` | `TargetV1` | `module` |
| `UnlinkedTargetV2` | `UnlinkedTargetV1` | `CanonicalDeclaration.module: ModuleRefV1` |
| `ExportV2` | `ExportV1` | `target: UnlinkedTargetV2` |
| `LinkedExportV2` | `LinkedExportV1` | `target: TargetV2` |
| `ReferenceV2` | `ReferenceV1` | `target: ModuleRefV1` |
| `RuntimeEdgeV2` | `RuntimeEdgeV1` | `dependency: ModuleRefV1` |
| `SemanticNodeV2` | `SemanticNodeV1` | `module: ModuleRefV1` (both arms) |
| `SemanticEdgeV2` | `SemanticEdgeV1` | `target: SemanticNodeV2` |
| `SemanticRowV2 { node: SemanticNodeV1, edges: Vec<SemanticEdgeV2> }` | `SemanticRowV1` | edges only; **the row node stays `SemanticNodeV1` (local)** |
| `RuntimeAvailabilityWitnessV2` | `RuntimeAvailabilityWitnessV1` | `dependency: ModuleRefV1`, `read_target: Option<TargetV2>`, `target: TargetV2`; **`importer` stays `ArtifactModuleId`** |
| `ModuleV3` | `ModuleV2` | `exports: Vec<ExportV2>`, `linked_exports: Vec<LinkedExportV2>`, `linked_reads: Vec<TargetV2>`, `runtime_dependencies: Vec<ModuleRefV1>`, `references: Vec<ReferenceV2>`, `runtime_edges: Vec<RuntimeEdgeV2>`; **`topology_parent`/`topology_children` stay `ArtifactModuleId`** |

`AvailabilityProofV1` (the bootstrap environment contract) is unchanged and
stays local. It describes the Universe artifact's own environment
(`ModuleWorldV4::validate_bootstrap_proof` requires a Universe target).

### MW5-T3: `ModuleWorldV5`

A flat struct, not a `Deref` wrapper:

```rust
pub const MODULE_WORLD_SCHEMA_V5: u32 = 5;

pub struct ModuleWorldV5 {
    pub entry: ArtifactModuleId,
    pub initialization_order: Vec<ArtifactModuleId>,
    pub modules: Vec<ModuleV3>,
    pub semantic_rows: Vec<SemanticRowV2>,
    pub external_projects: Vec<StableProjectRef>,
    pub external_modules: Vec<StableModuleRef>,
    pub topology_fingerprint: u64,
    pub availability_witnesses: Vec<RuntimeAvailabilityWitnessV2>,
    pub bootstrap_modules: Vec<ArtifactModuleId>,
    pub bootstrap_availability: Vec<AvailabilityProofV1>,
}
```

Wire order: `u32` schema, then fields in declaration order using the
existing `Wire` encodings. `external_modules` uses the same
`StableModuleRef` encoding as `ModuleV2.identity`. Decoding charges the
section `AllocationBudget` exactly like V4
(`decode_with_allocation_budget`, `decode_with_budget`, `decode`). It
rejects trailing bytes and runs `validate` before returning.

## 2. Local validation (`ModuleWorldV5::validate`)

Carry over every V2/V3/V4 rule, applied to the new field types. A reference
is in range iff `Local` < `modules.len()` or `External` <
`external_modules.len()`. New or changed rules:

- **MW5-V1.** `external_modules` is strictly increasing in `StableModuleRef`
  order (sorted, no duplicates).
- **MW5-V2.** No external module has a `Session` project.
- **MW5-V3.** Each external module's project is in `external_projects`
  (ARCH-I5).
- **MW5-V4.** No external module's project is the project of any local module
  row (disjoint ownership, ARCH-I2).
- **MW5-V5.** Every `external_modules` entry is referenced by at least one
  `External` reference in this module world (minimality, D-MW5-2).
- **MW5-V6.** Local-only positions are in range for `modules`. They cannot
  be external by type (MW5-T1).
- **MW5-V7.** `bootstrap_modules` and `bootstrap_availability` keep the V4
  rules. A world with any non-empty `external_modules` MUST have both empty:
  a dependent artifact does not re-assert another artifact's environment
  contract.
- **MW5-V8.** The V3 rule pairing `runtime_dependencies` with `runtime_edges`
  applies across both arms: equality is on `ModuleRefV1`.
- **MW5-V9.** V3 witness rules, generalized:
  - The witness `importer` is local.
  - `target` module equals `dependency`.
  - `read_target`, if present, equals `target` and is in the importer's
    `linked_reads`.
  - The timing rule is unchanged.
  - If `dependency` is `External`, then `authority` MUST be one of the
    environment authorities that an external artifact can supply
    (`SemanticIdentityCatalog`, `NativeBindingCatalog`,
    `NativeModuleMaterialization`, `FullyInitializedUniverse`). Whether the
    dependency artifact actually supplies that proof is checked at load
    (`LD-W1`).
- **MW5-V10.** Every linked read that is neither an eager edge nor witnessed
  is rejected (the V3 rule), with `External` reads included.

Error messages MUST say whether a failing reference is local or external,
for example "external module reference 3 out of bounds (2 entries)".

## 3. Portable topology fingerprint

**MW5-F1.** `refresh_portable_topology_fingerprint` /
`portable_topology_fingerprint` hash module references through their
`StableModuleRef` (today `module_id` → `module_identity`). For
`External(i)`, hash `external_modules[i]` the same way. Do not add a
local/external tag byte: ownership is disjoint (MW5-V4), so identities
cannot collide.

**MW5-F2.** For any world whose external table is empty, the V5 hash of the
lifted world equals the V4 hash of the source world. Test: lift the Universe
V4 world, recompute, and compare with `topology_fingerprint`.

## 4. Lift (V2/V3/V4 → V5)

**MW5-L1.** `impl From<ModuleWorldV4> for ModuleWorldV5`: every
may-be-external field becomes `ModuleRefV1::Local`, `external_modules` is
empty, and all other fields copy through. The lift is total and lossless:
lowering any lifted world back to V4 gives an equal value. Provide
`ModuleWorldV5::try_into_v4(&self) -> Option<ModuleWorldV4>`, returning
`None` iff the world contains an `External` reference. The Universe writer
keeps emitting V4 through it.

**MW5-L2.** Section dispatch: in `TeloscContainerView` section validation,
accept `MODULE_WORLD_SCHEMA_V5` for `SectionKind::ModuleWorld`. Add one
decode entry point that dispatches on the schema number and returns
`ModuleWorldV5` (lifting V4; V2/V3 lift through V4 where readers accept them
today). After this slice, everything above the decoder holds `ModuleWorldV5`
(staging struct `StagedTeloscCompilerWire.module_world`, the Universe
loader, `StableOwnerCatalog`, query reference validation).

## 5. Migration of consumers

**MW5-M1.** Change consumer signatures from `&ModuleWorldV2` / `&ModuleWorldV4`
to `&ModuleWorldV5`. Known consumers (re-locate with `rg "ModuleWorldV[234]"`):

- `phalcom-modules/src/compiler_artifact.rs`: `stage_artifact_project_authorities`, `load_module_world*`, `load_module_world_inner`.
- `phalcom-semantic/src/artifact/identity.rs`: `StableOwnerCatalog::new`.
- `QueryImageV4::validate_product_references`.
- `phalcom-semantic/src/artifact/export.rs`: `export_semantic_world_v4_scoped`,
  `export_query_image_v4_scoped` and their whole-image wrappers. These take
  `&ModuleWorldV2` today. Universe callers pass the lifted world.
- `ModuleWorldV3::validate_execution_authority` (becomes a `ModuleWorldV5` method).
- the semantic world import entry
  `validate_and_import_semantic_world_into_shared_stores_with_module_world_and_project_universe_and_allocation_budget`.
- `phalcom-core/src/modules/telosc_compiler_wire.rs`: `.module_world.previous.world` access paths.
- the Universe loader in `phalcom-core/src/modules/canonical_artifact.rs`.

**MW5-M2.** A consumer that cannot handle an `External` arm yet MUST return a
validation error ("external module references are not supported by
<consumer>"). It MUST NOT panic or `unreachable!()`. Universe paths never
see `External`, so this is safe during migration.

**MW5-M3.** Keep the V2/V3/V4 Rust types for decoding and for the Universe
writer. Do not delete them in this program.

## 6. Tests (crate-local, `phalcom-compiler-artifact`)

| ID | Test |
|---|---|
| MW5-T-RT | Round-trip encode/decode of a world with local and external references in every changed position. |
| MW5-T-LIFT | V4 → V5 → V4 is identity on the Universe-shaped fixture; `try_into_v4` is `None` once any `External` exists. |
| MW5-T-FP | MW5-F2 equality on a lifted V4 fixture; the fingerprint changes when an external entry's path changes. |
| MW5-T-V1…V10 | One negative test per rule, each asserting the specific error text. |
| MW5-T-TYPE | Compile-fail is not required. A review checklist item confirms MW5-T1 field types. |
| MW5-T-TRAIL | Trailing bytes and wrong schema number are rejected. |

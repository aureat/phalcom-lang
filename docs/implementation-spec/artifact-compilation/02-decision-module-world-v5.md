---
status: RULED
date: 2026-10-23
supersedes: none
promotable-to: PDR (number to be assigned at commit time)
---

# 02 — Decision: how package module worlds reference dependency modules

## Question

A package's module world must reference modules owned by dependency
artifacts: import references, runtime edges, linked reads, re-exported
bindings, semantic edges, and availability witnesses. Every such reference
is currently an `ArtifactModuleId`, a dense index into the artifact's own
module rows. `portable_id` fails with "module {id} is outside the closed
artifact world". How should the wire encode a reference to a dependency
module?

## Options

**A. `ModuleWorldV5` with a tagged reference.** Add
`external_modules: Vec<StableModuleRef>`. Positions that may name a
dependency use `ModuleRefV1 = Local(ArtifactModuleId) | External(ExternalModuleIndex)`.
Positions that must stay local keep `ArtifactModuleId`.

**B. Index-space extension.** Keep `ArtifactModuleId` everywhere. Indices
`>= modules.len()` mean `external_modules[i - modules.len()]`.

## Ruling: option A

### D-MW5-1: Tagged references in a new schema version

Rationale:

1. **Local-only positions must be unforgeable.** `entry`,
   `initialization_order`, topology, semantic row owners, witness importers
   and bootstrap entries must name owned modules (ARCH-I3). Under B, each of
   these fields needs its own runtime range check, and one forgotten check
   puts a foreign module into the local init schedule. That reopens the
   double-publication hazard the foreign-owner guard exists to block. Under
   A, those fields keep a type that cannot express "external".
2. **B never saves a schema bump.** Every earlier change bumped
   `MODULE_WORLD_SCHEMA_*`. If B reused the V4 number, indices would change
   meaning under an unchanged version. V4 readers would reject with a
   misleading "out-of-range artifact module reference". If B bumps the
   version, the claimed benefit is gone, and the local/external distinction
   is still hidden inside a number.
3. **B is unstable under local edits.** An external index under B is
   `modules.len() + k`. Adding one local module shifts every external
   reference, so the canonical bytes change for a reason unrelated to
   dependencies. A's external index depends only on the sorted external
   table.
4. **Equal work at call sites.** Every decoder, loader and fingerprint site
   that reads a module reference has to decide which table to consult under
   either option. A makes that decision explicit and exhaustive (`match`).

### D-MW5-2: The external table covers module-world references only

`external_modules` lists exactly the dependency modules referenced by the
module world (rule `MW5-V5`). Semantic and query sections already carry
full `StableModuleRef`s. Their external owners are validated against the
loaded external authority by `StableOwnerCatalog` (rule `LD-C2`), not by
table membership. This keeps the table minimal and checkable inside one
section.

### D-MW5-3: Universe keeps the V4 wire in this program

Readers accept V2–V5. V4 decodes and then **lifts** losslessly into the
in-memory V5 form (every reference becomes `Local`, the external table is
empty) (rule `MW5-L1`). Universe artifacts are byte-identical before and
after this program (gate `VER-R1`). Moving the Universe writer to V5 is a
separate, later decision (`09-open-questions.md` Q4).

### D-MW5-4: External eager runtime edges mean artifact-level ordering

An eager runtime edge to an external module is satisfied by the composition
order: the dependency artifact is staged (and, when executable package
artifacts exist, initialized) completely before the importer. No
cross-artifact entry appears in `initialization_order`. Witnessed edges to
Universe modules continue to use availability witnesses (rule `MW5-V9`).

## Consequences

- V5 is the first module-world bump that is not additive. It cannot be a
  `Deref` wrapper over V4, because field types inside module rows change. It
  needs new row records (`ModuleV3`, `TargetV2`, …).
- Code that reads the changed fields must handle `External`. Compile errors
  after the type change are the work list.
- The guards in `stage_artifact_project_authorities` and
  `package_import_order` ("package artifact embeds a foreign module owner")
  become invariants of the V5 design.

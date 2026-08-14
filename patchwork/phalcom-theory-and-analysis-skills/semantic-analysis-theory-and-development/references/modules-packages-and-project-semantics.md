# Modules, Packages, and Project Semantics

## Semantic project graph

As Phalcom modules/packages mature, analyzer needs graph nodes for:

```text
PackageId
ModuleId
source file/unit identity
imports/exports/re-exports
dependency version/source
core/native modules
```

## File is not forever module identity

Today's LSP may use canonical file URI as `ModuleId`. Future package/module semantics may require logical identity distinct from path. Encapsulate construction/resolution so consumers do not hardcode URI strings.

## Export surface

Separate:

- declarations owned by module;
- exported names;
- imported aliases;
- re-exports;
- private/internal visibility.

## Cycles

Build declaration shells/export surfaces before body analysis where cycles are legal. Initialization cycles are runtime semantics; static name graph can exist even when runtime cycle is rejected.

## Package versions

Two package instances with same module name but different versions/sources must not collapse semantic identities.

## Native mixed packages

Rust + Phalcom package needs one module/type/callable surface. Native implementations supply bodies/contracts behind the same semantic declaration IDs where possible.

## Incremental boundary

Changes can affect:

```text
body-only summary
export surface
inheritance/type surface
module graph
package dependency resolution
```

Classify to narrow invalidation safely.

## Registry/lockfile

Project semantics eventually depend on resolved package graph from manifest/lockfile. LSP/checker must use the same resolver as build tooling to avoid editing one dependency graph and compiling another.

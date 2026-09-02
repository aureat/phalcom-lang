# Module resolution

The module resolution layer sits at the boundary between project identity and concrete source lookup. It turns import syntax into a target source unit using project roots, module paths, and package exposure rules. See [resolver.rs](../../../phalcom-modules/src/resolver.rs).

## Entry point: ModuleResolver

`ModuleResolver` is the central abstraction:

```rust
pub struct ModuleResolver<'u, P: SourceProvider> {
    pub universe: &'u ProjectUniverse,
    pub source: &'u P,
    parsed_cache: HashMap<ModuleId, Result<Arc<ParsedModuleUnit>, ModuleLoadError>>,
    interface_cache: HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>,
}
```

Public entry points:

```rust
impl<'u, P: SourceProvider> ModuleResolver<'u, P> {
    pub fn resolve_import(
        &mut self,
        importer: &ModuleId,
        syntax: &ImportPath
    ) -> Result<SourceUnit, ModuleResolutionError>

    pub fn resolve_import_with_trace(
        &mut self,
        importer: &ModuleId,
        syntax: &ImportPath
    ) -> Result<ImportResolutionTrace, ModuleResolutionError>
}
```

The resolver caches both parsed modules and extracted interfaces to avoid re-scanning.

## Import root resolution

Import paths have two forms:

```rust
pub enum ImportRoot {
    Absolute(Identifier),   // e.g., "foo", "universe"
    Relative(NonZeroUsize), // e.g., 1 for ".", 2 for ".."
}
```

### Absolute imports

Absolute imports resolve through import root lookup:

1. Special case: `"universe"` maps to `ImportRootTarget::Universe`
2. Look up root component in the importer's project's import root table
3. Check that the root is known (not legacy `"core"` or `"std"`)
4. Select target project from resolved root

The import root table for each project is precomputed during project resolution and includes:
- The project's own namespace (self-import)
- Resolved dependency aliases
- The universe root

Example:

```phalcom
import universe::collections::map.    "Import from the built-in universe"
import my_project::helpers.             "Self-import within current project"
import other_lib::services::auth.       "Import from a resolved dependency"
```

### Relative imports

Relative imports ascend the package hierarchy:

1. `ModuleId` encodes the importer's `ModulePath` (e.g., `foo::bar::baz`)
2. `.` stays at same level; `..` ascends once
3. Remaining segments are appended (e.g., `..::sibling::module`)
4. Result is validated as an existing module path within the importer's project

Example:

In module `foo::bar::baz`:

```phalcom
import .:          "Import foo::bar (current package root)"
import ..::util.   "Import foo::util (sibling of foo::bar)"
import ..:         "Import foo (parent package)"
```

## External path validation

When an import crosses project boundaries, the resolver validates the target against package exposure rules:

```rust
fn validate_external_path_with_trace(
    &mut self,
    target_project: ResolvedProjectId,
    path: &ModulePath,
) -> Result<ImportResolutionTrace, ModuleResolutionError>
```

The validator walks the path segment-by-segment, checking each `PackagePathSurface::exposed_children` set:

1. Load the package interface for the target root module
2. For each path segment, verify it's in `exposed_children`
3. Load the next level's interface and repeat
4. If any segment is not exposed, reject the import

This is a deliberate protection: not every path is legal just because a project exists. Package boundaries control visibility. See [interfaces.md](interfaces.md).

## Traceability and caching

```rust
pub struct ImportResolutionTrace {
    pub target: SourceUnit,
    pub package_interfaces: BTreeSet<ModuleId>,
}
```

The trace records both the resolved `SourceUnit` and all package interfaces consulted during validation. This makes resolution auditable for debugging and enables semantic tooling to track dependency sources.

## Cross-reference

- See [identity.md](identity.md) for module paths and import root targets
- See [source-providers.md](source-providers.md) for how source units are loaded
- See [interfaces.md](interfaces.md) for how package exposure is determined

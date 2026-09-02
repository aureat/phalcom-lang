# Project structure

The `modules` crate resolves projects via `project.toml` manifests into a `ProjectUniverse`. See [project.rs](../../../phalcom-modules/src/project.rs) and [manifest.rs](../../../phalcom-modules/src/manifest.rs).

## ResolvedProject

A fully resolved project:

```rust
pub struct ResolvedProject {
    pub id: ResolvedProjectId,
    pub name: String,
    pub namespace: ModuleComponent,
    pub root_dir: PathBuf,
    pub source_root: PathBuf,
    pub entry: Option<ModulePath>,
    pub dependencies: BTreeMap<ModuleComponent, ResolvedProjectId>,
    pub import_roots: BTreeMap<ModuleComponent, (ImportRootTarget, bool)>,
    pub source_identity: ProjectSourceIdentity,
    pub persistent_project: bool,
    pub manifest: Option<ValidatedProjectManifest>,
}
```

### Fields

- `id` — unique graph-node identity assigned by the universe
- `name` / `raw_name` — project name for display
- `namespace` — canonical snake_case namespace (from `project.toml`)
- `root_dir` — directory containing `project.toml`
- `source_root` — directory containing source (e.g., `root_dir/src`)
- `entry` / `default_entry` — optional entry module paths
- `dependencies` — resolved dependency aliases and their project IDs
- `import_roots` — precomputed lookup table for absolute imports
- `persistent_project` — `false` for synthetic standalone contexts
- `manifest` — the validated manifest, if from `project.toml`

## ProjectUniverse

The set of all resolved projects in a compilation/analysis session:

```rust
pub struct ProjectUniverse {
    projects: Vec<ResolvedProject>,
    roots: BTreeMap<ProjectSourceIdentity, ResolvedProjectId>,
    synthetic_ids: SyntheticProjectIdAllocator,
}
```

### Operations

```rust
impl ProjectUniverse {
    pub fn new() -> Self
    pub fn projects(&self) -> &[ResolvedProject]
    pub fn get_project(&self, id: ResolvedProjectId) -> Option<&ResolvedProject>
    pub fn allocate_synthetic_id(&mut self) -> SyntheticProjectId
}
```

The universe is typically populated by:

1. Creating a root `ProjectUniverse`
2. Calling `ProjectUniverse::load_root(root_path, provider)` to discover the root project
3. Recursively resolving transitive dependencies via `resolve_project_recursive()`

## Project resolution algorithm

Given a `project.toml` path:

1. **Parse and validate**: Read TOML, parse into `ProjectManifest`, validate via `validate()`
2. **Assign identity**: Allocate a unique `ResolvedProjectId` (unless already seen)
3. **Normalize namespace**: Convert name to canonical `ModuleComponent`
4. **Resolve dependencies**: For each dependency spec, recursively resolve the target project
5. **Precompute import roots**: Build the lookup table for this project's absolute import resolution
6. **Store**: Add to `ProjectUniverse::projects` and index by `ProjectSourceIdentity`

## Manifest validation

Before a manifest becomes a resolved project, `ProjectManifest::validate()` enforces:

- Project name is not empty
- Namespace parses as a valid `ModuleComponent`
- Reserved names (`universe`, `core`, `std`) are rejected for project namespace or dependency aliases
- Dependency aliases don't collide
- Entry points are rooted within the project namespace
- Dependency specs are well-formed (path XOR package+version)

See [project-manifest.md](project-manifest.md) for details.

## Import root table

Each resolved project precomputes an `import_roots` table mapping root component names to their targets:

```rust
pub import_roots: BTreeMap<ModuleComponent, (ImportRootTarget, bool)>
```

Entries:
- `universe` → `(ImportRootTarget::Universe, false)`
- Project namespace → `(ImportRootTarget::Project(self.id), true)` (is_self = true)
- Dependency alias → `(ImportRootTarget::Project(dep_id), false)` (is_self = false)

This table is used by `ModuleResolver::resolve_import()` to translate absolute imports into target projects.

## Dependency resolution

Dependency specs in the manifest are either:

```rust
pub enum DependencySpec {
    Path { path: PathBuf },
    Package { package: String, version: String },
}
```

- **Path dependencies**: Resolved by filesystem lookup; must point to a directory containing `project.toml`
- **Package dependencies**: Resolved by a pluggable `DependencyProvider`; by default rejected (see `NullDependencyProvider`)

The default behavior is to accept only path dependencies, leaving registry/network lookup to higher-level tools.

## Cycles and error handling

If a dependency chain creates a cycle (e.g., A → B → A):

- The resolver tracks visited projects and raises `ProjectError::CircularDependency`
- Manifests with malformed fields raise `ProjectError::InvalidProjectManifest` or `ProjectError::ValidationFailed`

## Cross-reference

- See [identity.md](identity.md) for project and module identities
- See [module-resolution.md](module-resolution.md) for how projects use import root tables
- See [project-manifest.md](project-manifest.md) for manifest validation rules

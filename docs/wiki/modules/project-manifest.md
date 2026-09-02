# Project manifests

The `modules` crate models project structure through `project.toml` manifests. See [manifest.rs](../../../phalcom-modules/src/manifest.rs).

## Manifest structure

Raw manifests are parsed into `ProjectManifest` with two top-level sections:

```rust
pub struct ProjectManifest {
    pub project: ProjectSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, toml::Value>,
}
```

The `project` section contains metadata:

```rust
pub struct ProjectSection {
    pub name: String,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub namespace: Option<String>,  // defaults to normalized name
    #[serde(default = "default_source_root")]
    pub source: PathBuf,             // defaults to "src"
    pub entry: Option<String>,       // optional main module
    pub default_entry: Option<String>,
}
```

### Example

```toml
[project]
name = "my-project"
namespace = "my_project"
version = "0.1.0"
authors = ["Alice"]
source = "src"
entry = "main"

[dependencies]
other_project = { path = "../other" }
```

## Validation

`ProjectManifest::validate()` enforces invariants that guard namespace integrity:

- Project name cannot be empty
- `namespace` must parse as a valid `ModuleComponent` (snake_case, no leading digits)
- Reserved roots (`universe`, `core`, `std`) are rejected for project namespace or dependency aliasing
- Dependency aliases cannot collide with the project namespace or each other after normalization
- Dependency specs must be **either** `path` **or** `package`+`version`, not both
- Entry points must be rooted within the project namespace

Failed validation raises `ProjectError::ValidationFailed`. This guards the namespace layer early.

## Dependency specs

After validation, dependencies are normalized to `DependencySpec`:

```rust
pub enum DependencySpec {
    Path { path: PathBuf },
    Package { package: String, version: String },
}
```

### Path dependencies

```toml
[dependencies]
sibling_project = { path = "../sibling" }
```

Resolved by filesystem lookup; the path must point to a directory containing `project.toml`.

### Package dependencies

```toml
[dependencies]
remote_lib = { package = "my-remote-lib", version = "1.2.3" }
```

Resolved by a pluggable `DependencyProvider` trait. By default, `NullDependencyProvider` rejects all package dependencies, leaving registry lookup to downstream tools. This intentionally keeps the module layer free of network I/O.

## Validated manifests

After validation, `ProjectManifest` is converted to `ValidatedProjectManifest`:

```rust
pub struct ValidatedProjectManifest {
    pub name: String,                                              // canonical normalized name
    pub raw_name: String,                                          // original display name
    pub namespace: ModuleComponent,                                // canonical project namespace
    pub version: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub source: PathBuf,
    pub entry: Option<String>,
    pub default_entry: Option<String>,
    pub dependencies: BTreeMap<ModuleComponent, (String, DependencySpec)>,
}
```

This is the manifest representation used by `ResolvedProject` (see [project-structure.md](project-structure.md)).

## Dependency provider trait

```rust
pub trait DependencyProvider {
    fn resolve_package(
        &self,
        package: &str,
        version: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>>;
}
```

Implementations can fetch from registries, local caches, or other sources. The default is `NullDependencyProvider`, which rejects all package dependencies.

## Cross-reference

- See [project-structure.md](project-structure.md) for how manifests feed into resolved projects
- See [identity.md](identity.md) for module components and project identities

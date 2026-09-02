# Source providers

The `modules` crate abstracts source loading via the `SourceProvider` trait, enabling different source backends (filesystem, in-memory, network). See [source.rs](../../../phalcom-modules/src/source.rs).

## SourceProvider trait

```rust
pub trait SourceProvider {
    fn get_source(
        &self,
        location: &SourceLocation
    ) -> Result<Arc<str>, ModuleLoadError>;

    fn try_get_parsed(
        &self,
        id: ModuleId
    ) -> Result<Option<Arc<ParsedModuleUnit>>, ModuleLoadError>;

    fn list_modules(
        &self,
        id: ModuleId
    ) -> Result<Vec<ModuleComponent>, ModuleLoadError>;
}
```

### Methods

- `get_source(location)` — retrieve raw source text for a module
- `try_get_parsed(id)` — optionally retrieve pre-parsed AST (for caching)
- `list_modules(id)` — list child modules in a package

## SourceUnit

A concrete resolved source unit:

```rust
pub struct SourceUnit {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub kind: ModuleKind,
    pub primary: bool,
}
```

- `source_id` — filesystem or logical location
- `module_id` — canonical module identity
- `kind` — `ModuleKind::Source` or `ModuleKind::Package`
- `primary` — `true` if this is the authoritative source for the module

## ModuleKind

Modules are categorized:

```rust
pub enum ModuleKind {
    Source,    // A `.ph` source file (or synthesized source)
    Package,   // A package root (aggregates children)
}
```

- **Source modules** contain executable Phalcom code
- **Package modules** are organizational containers; they re-export their children

## FilesystemSourceProvider

Default implementation using the filesystem:

```rust
pub struct FilesystemSourceProvider {
    root: PathBuf,
}

impl FilesystemSourceProvider {
    pub fn new(root: PathBuf) -> Self
}
```

Maps module paths to `.ph` files:

- `foo::bar::baz` → `root/foo/bar/baz.ph`
- `foo::bar` (package) → `root/foo/bar/mod.ph`

## ParsedModuleUnit

Cached parsed AST:

```rust
pub struct ParsedModuleUnit {
    pub source_id: SourceId,
    pub program: phalcom_ast::ast::Program,
}
```

Providers can return pre-parsed units to skip lexing/parsing in subsequent accesses.

## EntryOwnership

Controls which module entries are valid:

```rust
pub enum EntryOwnership {
    AnyModule,              // Any module can be an entry point
    ProjectDefault,         // Only the project's default_entry
    ExplicitlyConfigured,   // Only explicitly configured entries
}
```

## Example: custom provider

A network-based provider:

```rust
pub struct NetworkSourceProvider {
    base_url: String,
    cache: HashMap<ModuleId, Arc<str>>,
}

impl SourceProvider for NetworkSourceProvider {
    fn get_source(&self, location: &SourceLocation) -> Result<Arc<str>, ModuleLoadError> {
        let url = format!("{}/{}.ph", self.base_url, location.module.path);
        fetch_url(&url).map(Arc::from)
    }
    // ...
}
```

## Cross-reference

- See [module-resolution.md](module-resolution.md) for how `ModuleResolver` uses `SourceProvider`
- See [interfaces.md](interfaces.md) for how modules are scanned for interfaces

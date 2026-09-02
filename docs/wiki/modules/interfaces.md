# Interfaces

Interfaces capture module-level declarations, exports, and imports. They are extracted from parsed source before linking. See [interface.rs](../../../phalcom-modules/src/interface.rs).

## UnlinkedModuleInterface

Before import paths are resolved to module identities:

```rust
pub struct UnlinkedModuleInterface {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub declarations: BTreeMap<String, DeclarationSurface>,
    pub exports: BTreeMap<String, ExportSurface>,
    pub imports: Vec<ImportSurface>,
    pub exposed_children: BTreeSet<ModuleComponent>,
}
```

### DeclarationSurface

Represents a top-level binding:

```rust
pub struct DeclarationSurface {
    pub name: String,
    pub is_const: bool,
    pub range: SourceRange,
}
```

Example: `const PI = 3.14159;` creates a `DeclarationSurface` named `PI` with `is_const = true`.

### ExportSurface

An exported binding:

```rust
pub struct ExportSurface {
    pub exported_name: String,      // public name
    pub internal_name: String,      // local binding
    pub target: UnlinkedExportTarget,
    pub range: SourceRange,
}

pub enum UnlinkedExportTarget {
    Local(String),
    ReExport { path: ImportPath, remote: String },
    CanonicalDeclaration { module: ModuleId, name: String },
}
```

Example exports in Phalcom:

```phalcom
"Export a local binding"
export const helper.

"Re-export from another module"
export from universe::collections import map.

"Convenience export of a universe declaration"
export universe::core_class as Object.
```

### ImportSurface

Raw import syntax before resolution:

```rust
pub enum ImportSurface {
    Module(ModuleImportDecl),              // import x::y.
    Selective(SelectiveImportDecl),        // import x from y.
    ReExport(ReExportDecl),                // export from x import y.
}
```

### exposed_children

A set of child module names visible to external imports:

```rust
pub exposed_children: BTreeSet<ModuleComponent>
```

When external code imports from this module, each path segment is validated against `exposed_children`. This provides package-level access control.

## LinkedModuleInterface

After linking (import paths resolved to module IDs, symbols bound):

```rust
pub struct LinkedModuleInterface {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub exports: BTreeMap<Box<str>, LinkedExport>,
    pub metadata: ModuleMetadata,
}

pub struct LinkedExport {
    pub public_name: Box<str>,
    pub target: LinkedExportTarget,
    pub range: SourceRange,
}

pub enum LinkedExportTarget {
    Binding(SymbolId),    // References a global symbol
    Module(ModuleId),     // References a whole module
}
```

Linked exports tie public names to either symbol IDs (from [linking-symbols.md](linking-symbols.md)) or module identities.

## InterfaceBuilder

Extracts interfaces from parsed AST:

```rust
pub struct InterfaceBuilder {
    // Internal state
}

impl InterfaceBuilder {
    pub fn build(program: &Program, module_id: ModuleId, kind: ModuleKind) -> Result<UnlinkedModuleInterface, InterfaceError>
}
```

The builder scans:

1. All top-level declarations (bindings, classes, methods)
2. All export statements
3. All import statements
4. Inferred `exposed_children` from package structure

## PackagePathSurface

For package modules, surfaces the hierarchical exposure:

```rust
pub struct PackagePathSurface {
    pub package_path: ModulePath,
    pub exposed_children: BTreeSet<ModuleComponent>,
}
```

When module `foo::bar` imports from `foo::bar::baz::internal`:

1. Load interface for `foo::bar` (root of target path)
2. Check `exposed_children` for `baz` → found, load `foo::bar::baz`
3. Check `exposed_children` for `internal` → if not found, reject import

This hierarchical validation prevents accidental exposure of internal modules.

## Example: Phalcom module interface

```phalcom
"Core utilities module"

"Public helper function"
fn helper(x) { x + 1. }.

"Internal implementation (not exported)"
fn internal_only(x) { x * 2. }.

"Package structure"
include ./submodule.
include ./helpers.

"Selective exports"
export const helper.
export from ./submodule import public_thing.
```

Interface extracted:
- Declarations: `helper`, `internal_only`, `submodule`, `helpers`
- Exports: `helper` (local target), `public_thing` (re-export from submodule)
- exposed_children: `{ submodule, helpers, helper }`

## Cross-reference

- See [module-resolution.md](module-resolution.md) for how package exposure validates external imports
- See [linking-symbols.md](linking-symbols.md) for how interfaces are linked to symbol bindings
- See [dependency-graphs.md](dependency-graphs.md) for semantic relationships between interfaces

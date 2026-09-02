# Linking and symbols

The `modules` crate assigns global symbol identities to module-level bindings and constructs module-level layout tables. See [linker.rs](../../../phalcom-modules/src/linker.rs).

## SymbolId

A canonical identifier for a module-level binding:

```rust
pub struct SymbolId {
    pub module: ModuleId,
    pub name: String,
    pub kind: SymbolKind,
}

pub enum SymbolKind {
    Binding,    // Const or let binding
    Class,      // Class definition
    Method,     // Method or function
}
```

Example: `SymbolId { module: foo::bar, name: "helper", kind: Binding }`

## GlobalBindingId

A flat, globally unique ID assigned to each top-level binding during linking:

```rust
pub struct GlobalBindingId(u32);
```

Used by the runtime to index into flat arrays of binding metadata.

## ModuleLinker

Coordinates linking across modules:

```rust
pub struct ModuleLinker {
    universe: &'u ProjectUniverse,
    interfaces: HashMap<ModuleId, LinkedModuleInterface>,
    bindings: HashMap<SymbolId, GlobalBindingId>,
    import_bindings: HashMap<ImportBindingId, GlobalBindingId>,
}
```

### Public API

```rust
impl<'u> ModuleLinker {
    pub fn new(universe: &'u ProjectUniverse) -> Self

    pub fn link_module(
        &mut self,
        unlinked: UnlinkedModuleInterface
    ) -> Result<LinkedModuleInterface, LinkError>

    pub fn link_program(
        &mut self,
        program: UnlinkedProgram
    ) -> Result<LinkedProgram, LinkError>

    pub fn get_linked_interface(
        &self,
        module: ModuleId
    ) -> Option<&LinkedModuleInterface>
}
```

## LinkedModule

A fully linked module with symbol bindings resolved:

```rust
pub struct LinkedModule {
    pub module: ModuleId,
    pub exports: BTreeMap<Box<str>, LinkedExport>,
    pub global_bindings: BTreeMap<SymbolId, GlobalBindingId>,
    pub module_layout: ModuleBindingLayout,
}

pub struct LinkedExport {
    pub public_name: Box<str>,
    pub target: LinkedExportTarget,
    pub range: SourceRange,
}

pub enum LinkedExportTarget {
    Binding(SymbolId),
    Module(ModuleId),
}
```

## ModuleBindingLayout

Describes the memory layout of module-level bindings:

```rust
pub struct ModuleBindingLayout {
    pub bindings: Vec<(String, GlobalBindingId)>,
    pub module_locals: HashMap<String, usize>,
}
```

Bindings are stored in a flat vector; lookups use name → index mappings.

## ImportBindingId

For imports that create local aliases:

```rust
pub struct ImportBindingId {
    pub module: ModuleId,
    pub name: String,
}
```

Example: `import x from foo.` creates an `ImportBindingId` for the local `x`.

## Linking algorithm

1. **Scan declarations**: Walk the interface's `declarations` map
2. **Assign GlobalBindingId**: For each binding, allocate a unique ID
3. **Process exports**: For each export, resolve the target:
   - If target is a local binding, record the `SymbolId` → `GlobalBindingId` mapping
   - If target is a module, record the module identity directly
4. **Construct layout**: Build `ModuleBindingLayout` from binding assignments
5. **Return LinkedModule**: Package the results with the linked interface

## DependencyPhase

Exports and imports can be categorized by when they're needed:

```rust
pub enum DependencyPhase {
    InterfaceOnly,  // Needed only for type/declaration checking
    Runtime,        // Needed for runtime initialization
}
```

The join operation:

```rust
impl DependencyPhase {
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Runtime, _) | (_, Self::Runtime) => Self::Runtime,
            _ => Self::InterfaceOnly,
        }
    }
}
```

This allows the linker to classify dependencies and determine initialization order. See [dependency-graphs.md](dependency-graphs.md).

## LinkedProgram

A collection of linked modules:

```rust
pub struct LinkedProgram {
    pub modules: BTreeMap<ModuleId, LinkedModule>,
    pub entry_point: Option<ModuleId>,
}
```

Ready for semantic analysis, type checking, or compilation.

## Example: Linking a Phalcom module

Source:

```phalcom
const version = "1.0".

fn greet(name) { "Hello, " + name. }.

export const version.
export fn greet(name) { greet(name). }.
```

Unlinked interface:

```
Declarations:
  - version (const)
  - greet (binding)

Exports:
  - version → Local("version")
  - greet → Local("greet")
```

After linking:

```
SymbolId { module: foo::bar, name: "version", kind: Binding } → GlobalBindingId(42)
SymbolId { module: foo::bar, name: "greet", kind: Method } → GlobalBindingId(43)

LinkedExports:
  - version → LinkedExportTarget::Binding(SymbolId(...))
  - greet → LinkedExportTarget::Binding(SymbolId(...))
```

## Cross-reference

- See [interfaces.md](interfaces.md) for unlinked interface structure
- See [dependency-graphs.md](dependency-graphs.md) for how linked symbols participate in graphs
- See [sessions.md](sessions.md) for incremental relinking on source changes

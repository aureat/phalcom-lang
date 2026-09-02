# Dependency graphs

The `modules` crate maintains three separate dependency graphs: reference, semantic, and runtime. This separation is intentional: a declaration cycle is valid input to a semantic fixed point, but a runtime initialization cycle is not. See [graph.rs](../../../phalcom-modules/src/graph.rs).

## DependencyPhase

Every dependency is classified by when it's needed:

```rust
pub enum DependencyPhase {
    InterfaceOnly,  // Needed only to resolve/check declarations
    Runtime,        // Contributes an eagerly initialized runtime binding
}

impl DependencyPhase {
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Runtime, _) | (_, Self::Runtime) => Self::Runtime,
            _ => Self::InterfaceOnly,
        }
    }
}
```

Examples:

- **InterfaceOnly**: Type annotations refer to a class in another module
- **Runtime**: A top-level binding initializer calls a function from another module
- **Join**: If any use is runtime, the whole dependency is runtime

## ReferenceGraph

Static source references (imports, declarations):

```rust
pub struct ReferenceGraph {
    edges: Vec<ReferenceEdge>,
}

pub struct ReferenceEdge {
    pub from: ModuleId,
    pub to: ModuleId,
    pub kind: ReferenceKind,
    pub range: SourceRange,
}

pub enum ReferenceKind {
    WholeModuleImport,  // import x.
    SelectiveImport,    // import f from x.
    ReExport,           // export from x import y.
    InterfaceOnly,      // Type reference only
}
```

Every import statement creates a `ReferenceEdge`. This graph is useful for:

- Dependency visualization
- Import auditing
- Unused import detection

## SemanticGraph

Logical relationships between declarations:

```rust
pub enum SemanticNodeId {
    Module(ModuleId),
    Declaration { module: ModuleId, name: Box<str> },
}

pub struct SemanticEdge {
    pub from: SemanticNodeId,
    pub to: SemanticNodeId,
    pub kind: SemanticEdgeKind,
}

pub enum SemanticEdgeKind {
    ModuleInterface,      // Module interface dependency
    TypeReference,        // Type/class reference
    Superclass,           // Inheritance
    ProtocolReference,    // Protocol conformance
    ConstraintReference,  // Generic constraint
    CallbackSignature,    // Callback/closure type
}
```

This graph is used by:

- Type inference and checking
- Cycle detection for semantic fixed points
- Protocol/interface validation

A cycle in `SemanticEdgeKind::TypeReference` is legal; a cycle in `RuntimeDependencyReason::Initialization` is not.

## RuntimeDependencyGraph

Eager initialization ordering:

```rust
pub struct RuntimeDependencyGraph {
    edges: Vec<RuntimeDependencyEdge>,
}

pub struct RuntimeDependencyEdge {
    pub from: ModuleId,
    pub to: ModuleId,
    pub reason: RuntimeDependencyReason,
}

pub enum RuntimeDependencyReason {
    Initialization,       // Module init calls another
    Eager,                // Top-level binding initializer needs another
    TransitiveClass,      // Superclass transitively needed
}
```

Used for:

- Determining module initialization order
- Cycle detection (runtime cycles are errors)
- Eager class hierarchy loading

**Invariant**: The runtime graph must be acyclic. A cycle here is a `ModuleGraphError::CircularRuntimeDependency`.

## Example: Three graphs

Modules A, B, C:

**Source**:
```phalcom
// Module A
import from B import type_T.
let x: T = B::create_x().

// Module B
import C for namespace.
class T { ... }.
fn create_x() { C::process(10). }.

// Module C
class Processor { ... }.
fn process(v) { Processor new process: v. }.
```

**ReferenceGraph edges**:
```
A → B (SelectiveImport: import type_T)
B → C (WholeModuleImport: import C)
```

**SemanticGraph edges**:
```
A → T (TypeReference: let x: T)
A → B::create_x (CallbackSignature: invoking create_x)
B → T (ModuleInterface: owns T)
B → C::Processor (TypeReference: in method body)
C → Processor (ModuleInterface: owns Processor)
```

**RuntimeDependencyGraph edges**:
```
A → B (Eager: x's initializer calls create_x)
B → C (Eager: create_x's body calls C::process)
```

All three graphs are acyclic in this example.

## ModuleGraphs

Container for all three graphs:

```rust
pub struct ModuleGraphs {
    pub reference_graph: ReferenceGraph,
    pub semantic_graph: SemanticGraph,
    pub runtime_graph: RuntimeDependencyGraph,
}
```

Constructed after linking all modules; queried by downstream analysis and compilation stages.

## Cycle detection

```rust
pub fn strongly_connected_components(graph: &impl Graph) -> Vec<Vec<NodeId>>
```

Computes SCCs to detect cycles. Cycles in the semantic graph are reported as warnings (allows cycles in type definitions); cycles in the runtime graph are errors.

## Cross-reference

- See [linking-symbols.md](linking-symbols.md) for how linked modules contribute to graphs
- See [interfaces.md](interfaces.md) for interface extraction that feeds graph construction
- See [sessions.md](sessions.md) for incremental graph updates

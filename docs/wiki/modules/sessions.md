# Sessions and incremental updates

The `modules` crate supports workspace sessions that track source state and enable incremental updates without full project reloads. See [session.rs](../../../phalcom-modules/src/session.rs).

## WorkspaceModuleSession

A snapshot of workspace state:

```rust
pub struct WorkspaceModuleSession {
    pub universe: ProjectUniverse,
    pub state: WorkspaceSourceState,
    pub generation: ResolverGeneration,
}

pub struct WorkspaceSourceState {
    resolved_modules: HashMap<ModuleId, WorkspaceSourceMutation>,
    parsed_cache: HashMap<ModuleId, Result<Arc<ParsedModuleUnit>, ModuleLoadError>>,
    interface_cache: HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>,
}
```

### Lifecycle

1. **Create**: `WorkspaceModuleSession::new(universe, provider)`
2. **Update**: Apply `WorkspaceSourceMutation` to source state
3. **Query**: Resolve imports, extract interfaces using cached state
4. **Invalidate**: Mark affected modules as stale on external changes

## WorkspaceSourceMutation

A single source change:

```rust
pub enum WorkspaceSourceMutation {
    Create {
        source_id: SourceId,
        text: Arc<str>,
    },
    Update {
        source_id: SourceId,
        text: Arc<str>,
    },
    Delete {
        source_id: SourceId,
    },
}
```

Mutations are collected into a batch:

```rust
pub struct WorkspaceSourceBatchMutation {
    mutations: Vec<WorkspaceSourceMutation>,
}

impl WorkspaceSourceBatchMutation {
    pub fn apply(&mut self, session: &mut WorkspaceModuleSession) -> Result<(), ModuleLoadError>
}
```

## SourceRevision

Tracks versions of source:

```rust
pub struct SourceRevision {
    pub generation: u64,
    pub content_hash: u64,
}
```

Used to detect stale cached parses and interfaces.

## ResolverGeneration

Monotonic counter tracking analysis generations:

```rust
pub struct ResolverGeneration {
    gen: u64,
}

impl ResolverGeneration {
    pub fn next(&mut self) -> u64
}
```

When the session is updated, the generation is incremented. This enables external tools (LSP servers, type checkers) to detect when cached results are stale.

## Incremental update strategy

1. **Track affected modules**: When a source changes, mark its `ModuleId` as stale
2. **Cascade invalidation**: Any module that imports the changed module is also stale
3. **Reparse on demand**: When a stale module is accessed, reparse and extract interface
4. **Cache hit on stable**: Modules that didn't change remain cached

Example:

```rust
let mut session = WorkspaceModuleSession::new(universe, provider)?;

// Initial state cached
let iface1 = session.get_interface(module_a)?;

// Modify a source file
let mut batch = WorkspaceSourceBatchMutation::new();
batch.push(WorkspaceSourceMutation::Update {
    source_id: source_a,
    text: Arc::from(new_source_text),
});
batch.apply(&mut session)?;

// Generation incremented
let gen_before = session.generation.current();
let gen_after = session.generation.next();
assert!(gen_after > gen_before);

// module_a is reparsed and interface re-extracted on next access
let iface2 = session.get_interface(module_a)?;
```

## Error handling

`WorkspaceModuleSessionError`:

- `SourceNotFound` — mutation references a non-existent source
- `ParseError` — reparsing a changed module failed
- `InterfaceError` — re-extracting interface failed

On error, the session state is rolled back (mutations are transactional).

## LSP integration

Sessions are the foundation for LSP servers:

1. Create session for the workspace
2. On file save, apply a `Create` or `Update` mutation
3. Query changed modules for diagnostics
4. Return updated `generation` to client (enables client-side cache invalidation)

Example LSP flow:

```
Client: open file /project/foo.ph
Server: create session, parse file
Server: on_did_change → apply Update mutation → reparse → emit diagnostics
Client: increment local generation, refresh UI
```

## Performance considerations

- **Batch mutations**: Collect multiple changes before invalidating caches
- **Lazy reparse**: Only re-extract interfaces for modules that are actually queried
- **Stable generation**: External tools use generation to detect stale results

## Cross-reference

- See [module-resolution.md](module-resolution.md) for how resolution uses cached interfaces
- See [interfaces.md](interfaces.md) for interface extraction that's cached per generation
- See [project-structure.md](project-structure.md) for project universe lifecycle

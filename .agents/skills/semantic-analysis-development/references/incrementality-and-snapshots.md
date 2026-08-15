# Incrementality, Invalidation, and Immutable Snapshots

## Update model

Current semantic database supports file/batch replacement. Preserve transaction semantics:

```text
old coherent generation
  -> mutate worker state
  -> recompute affected dependencies
  -> produce new coherent state
  -> publish Arc<SemanticSnapshot>
```

A consumer should never see new scopes with old summaries or vice versa.

## File revision versus generation

Use file revision to identify one document contribution. Use semantic generation for the entire
published semantic world.

A batch of multiple files should advance one generation after all are incorporated.

## Initial affected set

For an updated module include:

- module itself;
- dependent module closure from module graph;
- modules containing callables dependent on old summaries from changed module;
- future type/proof dependents as those graphs are added.

## Expanding frontier

After solving, compare old/new summaries/facts that cross module boundaries. If a semantic
contract changed, add reverse dependents and continue until no new modules are affected.

This is essentially an incremental fixed point over dependency graph.

## File removal

Removal must delete:

- file snapshot;
- class surfaces;
- callable summaries;
- field/parameter contributions;
- module graph node/edges;
- reverse callable edges;
- occurrence/definition targets;
- future type/proof caches.

Then recompute dependents against missing/unresolved state.

Stale deletion bugs are especially dangerous because queries can return plausible old answers.

## Foreign data ownership

Avoid copying full derived structures from another module into a dependent module snapshot.
Instead store target IDs and resolve through current snapshot, or record a compact summary whose
invalidation edge is explicit.

This mirrors the strongest lesson from incremental analyzers such as Biome/rust-analyzer:
staleness must be structurally difficult.

## Cache design checklist

For each cache:

```text
key
value
owner
input dependencies
lifetime
generation/revision
invalidation trigger
memory bound
determinism
```

If one field is unknown, do not add cache yet.

## Query stamps

Long-running LSP requests may compute from a snapshot while a newer generation is published.
This is usually fine if result is tied to request's document version. If applying edits/code
actions, use generation/document stamps to reject stale transformations when necessary.

## Sub-file incrementality

Do not assume sub-file stable AST nodes unless parser provides them.

Whole-file AST rebuild + semantic affected-frontier recomputation is a sound baseline. If future
incremental parser exists, adapt file contribution builder behind a stable semantic interface.

## Module graph refresh

When available modules change, previously unresolved imports can resolve and previously resolved
ones can become unresolved. Refresh resolution globally enough to preserve correctness, then use
edge changes to compute affected dependents.

## Core update

Core source is a high-fanout dependency. Updating it may legitimately invalidate most/all
workspace semantics. Optimize only after profiling; correctness first.

## Tests

### Same-file edit

Change initializer shape and verify only facts after/depending on it change.

### Cross-file call

Change callee return in A; B caller should update; unrelated C should not rebuild if counters
make that observable.

### Import resolution

Create missing imported file; importer should resolve without restart.

### Removal

Delete class/module; completion/definition/summaries disappear.

### No-op reparse

Reparse unchanged file; outputs deterministic, rebuild frontier bounded.

### Batch

Update mutually dependent files together; consumers see one coherent final generation, not an
intermediate error world.

## Debugging stale facts

When a stale fact appears:

1. identify snapshot generation shown to query;
2. identify fact owner/module;
3. trace source contribution;
4. inspect dependency edge from changed source to fact;
5. inspect reverse dependent closure;
6. inspect old/new summary equality;
7. inspect file removal/replacement logic;
8. add a regression test that asserts rebuild trace and query result.

Do not fix by unconditional global invalidation unless it is an intentional temporary safety
fallback with follow-up plan.

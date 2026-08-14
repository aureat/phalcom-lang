# Incrementality, Snapshots, and Queries

## Coherent generations

A semantic snapshot represents one consistent world. Queries should not combine:

```text
new file AST
old class surface
new call summaries
old module graph
```

Publish atomically after affected state converges.

## Current pattern

Phalcom LSP uses mutable worker engine + immutable `SemanticSnapshot` published through `Arc`/locks. This is a sound foundation for editor concurrency.

## Fine-grained evolution

Potential future query keys:

```text
parse(ModuleId, revision)
surface(ModuleId)
scope_graph(ModuleId)
lower_body(CallableId)
summary(CallableId)
type_of(ExprId)
proof(AssertionId)
```

Add gradually; do not split into queries until dependency boundaries are stable.

## Stable versus ephemeral IDs

Cross-query keys need identities stable for query lifetime. File-local numeric IDs are valid within owning revision but should be wrapped with owner/revision when cached globally.

## Reverse dependencies

Maintain edges for summaries/imports/types. A changed callee summary can invalidate callers even when source module imports unchanged.

## Content equality

If recomputation yields identical semantic result, dependents may avoid rebuild. Semantic equality must be deterministic/canonical.

## Cancellation

IDE analysis should support cancelling obsolete generations/work when newer edits arrive. Do not publish partially cancelled state.

## Memory

Snapshots keep old `Arc`s alive while requests run. Avoid embedding huge duplicate AST/fact graphs per derived object. Use IDs/interning/sharing.

## Query instrumentation

Record hit/miss, recomputation count, duration and dependency fanout so optimization is evidence-driven.

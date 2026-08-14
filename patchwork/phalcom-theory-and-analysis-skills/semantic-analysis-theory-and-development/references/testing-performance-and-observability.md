# Testing, Performance, and Observability

## Test layers

### Identity/scope

Shadowing, declarations, imports, exact occurrence targets.

### Surface/dispatch

Inheritance, class-side/metaclass, access, super, selector forms.

### Flow

Branches, loops, assignments, returns, throws, closures, fields.

### Interprocedural

Calls, recursion, higher-order blocks, dynamic dispatch, native summaries.

### Incremental

Edit/remove/re-add module, import graph changes, unchanged-result reuse.

### Consumer

Hover/completion/rename/checker diagnostics consume semantic queries correctly.

## Full rebuild oracle

For any edit sequence:

```text
incremental_snapshot == clean_rebuild_snapshot
```

for semantic results modulo IDs intentionally ephemeral across rebuild. Compare normalized semantic content.

## Property/metamorphic tests

- consistent rename preserves semantics;
- formatting preserves semantics;
- syntactic sugar/desugaring agree;
- traversal order does not change joined facts;
- deterministic repeated analysis.

## Fuzzing

Parser-recovered ASTs should never panic analyzer. Fuzz nested scopes, selectors, blocks, packs, patterns and module graphs.

## Performance budgets

Measure:

```text
initial workspace indexing
single-keystroke file update
completion/hover query latency
memory per snapshot
rebuild frontier
fixed-point iterations
```

## Traceability

Debug mode should answer:

```text
why target resolved this way
why fact has this value
which dependencies caused rebuild
which call summary caused inference
where precision was widened
```

Without observability, semantic engines become impossible to tune safely.

# Attribute Grammars and Query Systems

## Attribute grammar idea

Semantic facts can be seen as attributes of syntax nodes:

- inherited attributes flow context downward (scope, expected type);
- synthesized attributes flow results upward (inferred type, constant value).

This mental model helps avoid global mutable walkers even if Phalcom never uses an attribute-grammar framework.

## Circular attributes

Recursive semantic relationships need fixed-point evaluation. This connects attribute grammars to query systems and abstract interpretation.

## Query-based semantics

A modern compiler can expose functions:

```text
scope_of(node)
resolve_name(node)
type_of(expr)
candidates_of(send)
summary_of(callable)
module_exports(module)
```

Memoization tracks dependencies between queries.

## Advantages

- demand-driven computation;
- natural incremental invalidation;
- consumer reuse;
- isolation of semantic ownership;
- parallel read potential.

## Hazards

- cyclic queries;
- huge keys/results;
- hidden expensive queries called from hot LSP paths;
- caching recovery states forever;
- unstable IDs causing cache misses;
- query purity violations through global mutable state.

## Current Phalcom model

The semantic engine currently rebuilds affected state into immutable snapshots. That is valid and simpler. A future query framework should be introduced incrementally only if rebuild/query complexity warrants it.

## Query tiers

Expose cost/trust tiers conceptually:

```text
O(1)/indexed source target
local scope query
local body analysis
interprocedural summary
whole-project proof
```

LSP completion should not accidentally trigger whole-project SMT.

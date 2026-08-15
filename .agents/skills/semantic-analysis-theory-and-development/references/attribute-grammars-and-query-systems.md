# Attribute Grammars and Query Systems

## 1. Why this theory matters

Semantic analysis is a dependency problem: facts about one node depend on inherited context and other computed facts. Attribute grammars and incremental query systems provide useful mental models, but Phalcom should adopt their principles selectively rather than forcing the codebase into a specific framework.

## 2. Inherited and synthesized attributes

An **inherited attribute** flows context downward/across structure; a **synthesized attribute** is computed from children/local information.

For an expression `e`:

```text
scope(e)        inherited
expected_type(e) inherited (future checker)
resolved_target(e) synthesized from syntax + scope
value_shape(e)  synthesized from target + subexpressions + summaries
```

This vocabulary helps detect cycles and ownership. If `value_shape(e)` needs `callable_summary(f)`, and summary `f` needs a call back to the current callable, the dependency is cyclic and requires fixed-point semantics—not recursive memoization that happens to stack overflow.

## 3. Query model

Treat a semantic result as a pure conceptual query:

```text
Q(key, revision inputs) -> immutable result
```

Examples:

```text
module_surface(ModuleId)
scope_graph(FileId)
resolve_occurrence(FileId, OccurrenceId)
callable_summary(CallableId)
references(SemanticTarget)
```

A query system may memoize results and record dependencies automatically. Correctness still depends on what the query reads and what constitutes equality/change.

## 4. Query purity versus operational state

The conceptual query should be deterministic for one semantic snapshot/configuration. Implementation may use arenas, caches, worker queues, counters, cancellation, or locks, but these should not change semantic result.

Bad query:

```text
infer(expr) -> result depending on whichever mutable global map happened to be updated first
```

Good query:

```text
infer(snapshot, body, program_point, expr) -> deterministic abstract fact
```

## 5. Cycles

Not every cycle is an error. Class inheritance cycles may be a language error; recursive callable summaries are a valid fixed-point problem; module cycles may be allowed for declaration resolution but constrained for initialization.

A query framework's generic “cycle detected” fallback is therefore insufficient. Each cyclic semantic relation needs domain-specific policy:

```text
inheritance: diagnose/recover
call summaries: SCC fixed point
module surfaces: staged SCC indexing
proof goals: recursion invariant/contract rules
```

## 6. Incremental equality

A query can avoid invalidating dependents when recomputation yields semantically equal output. Define equality carefully. Source range changes may matter for diagnostics even if class surface is semantically identical. Split products when appropriate:

```text
ClassSignatureSemanticData
ClassSourceLocations
```

so moving a declaration does not needlessly invalidate all type/dispatch facts while navigation locations still update.

## 7. Demand-driven versus eager

LSP queries favor demand-driven work; batch checking often wants complete diagnostics. A shared query architecture can support both:

```text
editor: request narrow roots -> compute reachable dependencies
batch: request workspace_check roots -> force all required queries
```

However, current interprocedural summaries may benefit from eager SCC/worklist solving. Hybrid architecture is acceptable. Do not contort a natural fixed-point solver into tiny recursive queries if it obscures convergence.

## 8. Provenance and queries

Memoized facts need explanation dependencies. If `summary(A)` depends on `summary(B)`, the semantic result can retain provenance edge `A return <- B return` while the query engine separately records invalidation dependency. They are related but not the same graph: one explains meaning; the other governs recomputation.

## 9. Framework adoption checklist

Before adopting Salsa or a custom query database, verify:

- cyclic fixed-point analyses can be represented cleanly;
- cancellation and snapshot semantics fit editor needs;
- dependency granularity is measurable;
- memory/eviction across many revisions is bounded;
- deterministic diagnostics and provenance remain available;
- integration does not force formal type facts into the same representation as runtime shapes;
- current COW snapshot architecture has a measured problem the framework solves.

## 10. Review questions

1. What are the inherited and synthesized dependencies of this fact?
2. Is a dependency cycle a language error or a fixed-point computation?
3. What equality determines whether dependents need recomputation?
4. Does query memoization preserve provenance separately from invalidation edges?
5. Is demand-driven evaluation appropriate for this analysis?
6. Would a framework simplify dependencies or merely conceal them?

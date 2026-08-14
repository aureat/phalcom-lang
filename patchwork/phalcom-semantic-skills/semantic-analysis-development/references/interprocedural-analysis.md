# Interprocedural Analysis and Callable Summaries

## Why summaries

A call site needs useful callee facts without embedding the callee's internal AST/flow state.
Summaries provide a finite contract for fixed-point solving and invalidation.

## Current summary structure

Current `CallableSummary` includes:

```text
CallableId
Vec<InferredValue> params
InferredValue returns
Vec<CallableId> dependencies
SummaryEffects
SemanticGeneration revision
```

Extend only when a new cross-call semantic question cannot be answered from existing fields.

## Parameter facts

Observed arguments at resolved call sites can be joined by `(CallableId, parameter name)`.
This is useful for advisory LSP inference.

Do not automatically treat observed parameter facts as normative types. External/dynamic call
sites may exist.

## Mapping arguments to parameters

Respect selector labels and pack semantics. A robust call-site record should include:

- static label if known;
- value fact;
- source range;
- referenced binding if any;
- literal block effects if any;
- whether dynamic pack prevents exact mapping.

Do not zip positional vectors blindly for labeled/rest calls.

## Return summaries

For advisory shape:

```text
join all reachable explicit return values + reachable tail value
constructor -> guaranteed instance when semantics says so
no evidence -> Unknown
```

For future language type inference, use the typing spec's join/Unit/recursion rules separately.

## Dependency extraction

Summary dependencies should include statically resolved callable targets whose summary can affect
this callable's summary/effects.

Dynamic sends should set conservative effect/unknown dependency state rather than fabricate a
specific edge.

## Reverse dependencies

Maintain:

```text
callee -> set(callers/dependents)
```

so a changed callee summary can enqueue only affected modules/callables.

Rebuild this map from solved summaries or update transactionally. Do not leave stale reverse
edges after file removal.

## Fixed point

Generic solver:

```text
seed summaries
repeat affected SCC/frontier:
  analyze callable using current summaries
  join/replace according to analysis definition
  if summary changed -> enqueue dependents
until stable
```

The current engine solves affected state in project batches; inspect `infer.rs`/`engine.rs`
before adding another loop.

## Recursion

### Advisory runtime-shape inference

Can conservatively seed recursive returns/parameters as unknown and iterate. Widen if shape
growth exceeds limits.

### Future typed return inference

Typing proposal may require explicit return annotations for recursive methods. That is a checker
policy and can coexist with advisory summary inference.

Do not force the LSP to lose all advisory information because checker inference intentionally
rejects recursive omission.

## Higher-order callables

Current effects can identify callable parameter positions that are invoked. Propagate a literal
block's effects only through parameters known to be invoked.

Future summary dimensions:

```text
invokes parameter may/must
invocation cardinality
escapes/stores parameter
invokes synchronously/deferred
invokes on another fiber
```

Add only if language/library semantics need them.

## Dynamic calls

A dynamic call can affect:

- return precision;
- effect closure;
- invalidation assumptions;
- proof soundness;
- optimizer call graph.

Use conservative fallback. For editor shape inference, unknown return may be acceptable. For a
purity/no-yield proof, dynamic call may force `Unknown`/`MayEffect` unless constrained by a type
or sealed surface.

## Native calls

Native methods need semantic summaries from trusted metadata/source stubs. Include effects that
matter:

- return shape/type;
- may throw;
- may block/yield;
- mutation;
- callback invocation;
- FFI safety boundary.

A missing native summary should degrade to conservative unknown, not a guessed pure call.

## Summary versioning

Summary equality should reflect fields that affect dependents. Provenance-only changes may not
need to invalidate all dependents unless diagnostics query the provenance transitively.

Separate semantic hash/version from display/debug metadata if this becomes a performance issue.

## Tests

- one-hop return propagation;
- two-hop propagation;
- parameter call-site join;
- recursion;
- mutual recursion;
- changed callee invalidates caller;
- unchanged summary does not fan out unnecessarily;
- dynamic call widens/marks effect;
- higher-order block propagation;
- file removal removes summaries/reverse edges;
- same selector in different modules/classes remains distinct.

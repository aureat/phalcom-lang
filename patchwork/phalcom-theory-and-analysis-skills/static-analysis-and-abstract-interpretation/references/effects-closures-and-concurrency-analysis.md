# Effects, Closures, and Concurrency Analysis

## Effect domain

A useful effect summary can be a product of may-properties:

```text
throws: set/error-top
reads/writes fields
reads/writes globals
allocates
IO
blocks_thread
may_yield
spawns_fiber
reflective_mutation
dynamic_send
invokes_params
```

Join is component-wise union/OR for may-effects.

## Pure/impure is too coarse

A single boolean loses the ability to reorder non-conflicting effects or prove contracts. Prefer structured effects where consumers benefit.

## Closure latent effects

Constructing a block and invoking it differ. Store latent effects on callable/block summaries; only apply them when analysis proves the block is invoked.

## Captured writes

A block can modify captured bindings. Higher-order summaries can propagate those writes when the callee invokes the block synchronously.

If invocation timing/cardinality is unknown, use conservative joined state.

## Fiber yield

`may_yield` matters because code between yields may otherwise be considered atomic in a cooperative model. A call that may yield invalidates assumptions about interleaving/shared mutable state.

## Blocking

`blocks_thread` differs from `may_yield`: blocking the VM thread prevents scheduler progress. Standard-library/FFI analysis should expose this distinction.

## Cancellation/cleanup

If fibers gain cancellation, effects include abrupt cancellation edges and cleanup obligations.

## Effect polymorphism

Higher-order helpers can be polymorphic in block effects conceptually:

```text
map(block effects E) -> effects E + collection-allocation
```

No need for user-facing effect types initially; internal summary composition can still model it.

## Optimizer use

Only eliminate/reorder calls when effect summary is trusted/sound enough for optimization. LSP heuristic summaries are not automatically optimizer-proof facts.

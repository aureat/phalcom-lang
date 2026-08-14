# Effects, Closures, Fibers, and Concurrency-Aware Analysis

## Current foothold

Current callable summary effects already model:

- dynamic send;
- callable parameter positions invoked by the body.

Current block analysis can retain:

- non-local return evidence;
- captured lexical writes;
- invoked parameters;
- dynamic send.

This is the right seed for a broader effect domain.

## Effect design rule

Do not add a boolean because one consumer wants it. Define the semantic effect and its
propagation.

For each effect answer:

```text
origin operations
may or must?
call propagation
block/callback propagation
dynamic-call fallback
join
invalidation
native/FFI source
consumer policy
```

## Candidate future effects

### May throw

Origin:

- explicit `throw`;
- calls whose summaries may throw;
- native operations that can fail by exceptions if Phalcom uses them.

Must account for catches/handlers when introduced.

### Does not return

Useful for:

- flow reachability;
- bottom type;
- static proving.

Do not equate "throws" with "does not return" if exceptions can be caught locally.

### May yield fiber

Origin:

- explicit yield/scheduler operations;
- calls known to yield;
- I/O primitives whose Phalcom fiber runtime transparently parks/yields.

This effect can invalidate assumptions about uninterrupted mutation or reentrancy.

### May block OS thread

Distinct from yielding a fiber. Native FFI/process/filesystem calls may block the host thread
unless integrated with scheduler.

This distinction is important for standard-library design and diagnostics.

### May mutate

Potential granularity:

```text
local capture
specific field set
receiver state
global/module state
unknown heap
```

Start only as precise as needed. Unknown reflective call may mutate broadly.

### Escapes closure/parameter

Needed for:

- non-local return safety;
- capture lifetime reasoning;
- future optimization;
- concurrency transfer.

A callable that stores a callback has different semantics from one that invokes it immediately.

### Performs I/O

Useful for effect-aware tooling/proving, but only if Phalcom chooses to expose this semantic
category. Do not infer purity policy prematurely.

## Closure construction versus invocation

Never apply block body effects merely when evaluating a block literal.

Construction may:

- capture bindings;
- allocate closure object;
- record home callable/frame for non-local return semantics.

Invocation may:

- write captures;
- return non-locally;
- throw;
- yield;
- invoke other callbacks.

## Non-local returns

A block's non-local return targets its lexical home callable according to Phalcom block
semantics. If the block can escape beyond the home frame, runtime must trap invalid return;
semantic analysis should not model it as ordinary local return.

Keep target callable identity in evidence.

## Captured writes

When a known callback is invoked, captured writes may update caller flow state.

Need conservative policy when:

- callback may be invoked zero/multiple times;
- callback escapes;
- callback invocation order is unknown;
- multiple fibers may access capture in future.

Do not naively apply a captured write once as if guaranteed.

## Fibers

Before adding fiber-aware semantic rules, read the concurrency spec. Critical questions:

- stackful/stackless representation;
- cooperative scheduling points;
- cancellation semantics;
- exception propagation across fibers;
- fiber-local state;
- shared memory model;
- whether native blocking calls park or block;
- structured concurrency/join semantics.

Semantic effect summaries should be able to answer tooling questions without simulating scheduler.

## Concurrency and refinement stability

A flow fact such as `x != None` is only stable if no concurrent/aliased mutation can change `x`
before use. For lexical immutable bindings, easy. For shared fields/global state under threads,
not necessarily.

Do not introduce aggressive smart casts over mutable shared storage before memory/concurrency
semantics are settled.

## FFI

Rust FFI/native packages may:

- call back into Phalcom;
- yield/block;
- mutate VM state;
- retain Phalcom objects/closures;
- throw/return errors;
- require GC roots.

Semantic metadata for native functions should include enough effect information for checker,
prover and fiber diagnostics. Unknown native code must be conservative.

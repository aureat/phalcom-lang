# Exceptions, Non-local Control, and Effects in Proofs

## Multiple exits

A callable can have:

```text
normal return
throw
non-local return triggered by invoked block
fiber suspension/cancellation
process exit / unrecoverable runtime effect
```

A postcondition for normal result does not prove exceptional behavior.

## Exceptional postconditions

Model either explicit `ensuresThrows`-style contracts or effect summaries:

```text
mayThrow E
```

At a call site, proof state branches into normal and exceptional successors.

## `try`/handlers

Handler path assumes thrown value/type compatible with caught form. Uncaught alternatives propagate.

## Non-local return

A block can cause control to escape its invoker and target a home frame. For modular proof, block summaries need control effects. A higher-order method that invokes its block parameter may therefore inherit non-local-return behavior.

## Ensure/finally

Cleanup runs on specified abrupt paths. VCs must thread heap/effects through cleanup before final outcome.

## Effects as proof preconditions

If a proof relies on field `f` unchanged across call, callee effect summary must exclude writes to `f` (or aliasing that reaches it).

## Dynamic send

Unknown dynamic send can throw message-not-understood, mutate through arbitrary target and invoke user code. It is a proof boundary unless receiver/protocol/effect contract constrains it.

## Pure expressions

Only duplicate/reorder logical encodings of expressions whose language evaluation is pure enough. A source expression with a method send cannot be treated as a mathematical term unless the prover uses a trusted pure contract for that send.

---

## Deep treatment: effect rows for proof control

A useful callable proof summary can separate value contract from effects:

```text
Effects = {
  throws: Set<ErrorDomain>,
  may_nonlocal_return: bool / target set,
  may_yield: bool,
  may_block_thread: bool,
  may_callback: bool,
  may_write: RegionSet,
  may_reflect_dispatch: bool
}
```

This is not necessarily a language-level effect type; it can be semantic-analysis/proof metadata. Keep that distinction explicit.

### Cleanup sequencing

For:

```text
try Body finally Cleanup
```

any outcome `o` from `Body` first executes `Cleanup`. If cleanup completes normally, original `o` resumes; if cleanup itself throws/nonlocally returns according to language semantics, it may replace/transform the original outcome. VC generation must encode the actual precedence rule.

### Non-local return from blocks

If block `b` captures home frame `h`, invoking it may produce:

```text
NonLocalReturn(h, value, H')
```

A higher-order method that invokes `b` cannot advertise a simple normal/throw-only summary unless it catches/transforms that control effect. This directly affects collection methods such as iteration if blocks can non-locally return.

### Effect polymorphism opportunity

Higher-order library methods may forward block effects:

```text
map(block E) has effects E plus collection-specific effects
```

This can be represented in future effect-summary machinery without necessarily exposing effect polymorphism in source typing. Avoid hard-coding every higher-order method as “may do everything” if semantic summaries can safely parameterize callback effects.

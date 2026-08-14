# Semantic Hazards and Invariants

## Identity hazards

### Bare-name class maps

Wrong when two modules declare the same class name. Use module-qualified identity.

### Range-as-identity

Ranges shift after edits. Use ranges to locate source occurrences, not as durable declaration
identity.

### Snapshot-local IDs stored globally

`BindingId`/`ScopeId` may be rebuilt on reparse. Do not retain them beyond snapshot lifetime.

## Scope hazards

### Text search resolution

A global grep for `x` ignores shadowing, declaration order and import aliases.

### Declaration after use

If language semantics restrict visibility before a declaration, scope maps need source-order
checks. A scope containing a name does not mean the name is visible at every offset.

### Pattern binding projection

All names introduced by destructuring need separate identities and projected facts.

## Flow hazards

### Last-write-wins across branches

Analysis traversal order is not runtime certainty. Join predecessor states.

### Unreachable evidence

Facts from unreachable paths should not pollute must/return analyses. If current advisory
analysis intentionally collects some unreachable evidence for editor usefulness, label and
contain that policy rather than treating it as proof.

### Loop single pass

One loop body visit is generally not a fixed point. Use iteration/worklist or conservative
widening where loop-carried facts matter.

## Shape/type hazards

### `Unknown` as union wildcard

Current shape `Unknown` is a loss of knowledge. It should conservatively dominate joins.
Do not interpret it as a concrete dynamic type contract.

### Shape becomes type by rename

Future types need different algebra and correctness semantics. Build a bridge, not a rename.

### Heuristic becomes proof

Use-site heuristics are acceptable for completion ranking, not for rejecting correct code.

### Call-site observations become declared contracts

Open-world dynamic calls may exist outside the observed workspace. Treat parameter call-site
facts as advisory unless typing specification explicitly defines inference from them.

## Dispatch hazards

### Base-name-only lookup

Selector labels/arity/kind are semantically significant.

### Type annotations alter selector identity

Forbidden unless an explicit typed-dispatch feature is ratified.

### `super` modeled as superclass instance

`super` changes lookup origin, not receiver identity.

### Dynamic packs fabricate exact selectors

Computed labels/rest expansion may prevent exact static selector construction. Preserve
uncertainty.

## Interprocedural hazards

### Recursive AST descent

Analyzing callees recursively at every call can loop and duplicates work. Use summaries and
fixed points.

### Unbounded provenance

Every transitive call chain can explode evidence. Cap/arena/compress.

### Missing dynamic effect

Reflection/dynamic sends mean the call graph is not closed. Mark conservative effects.

## Incremental hazards

### Copied foreign facts

Cloned dependent data becomes stale. Prefer identities/references and rebuild by dependency.

### Invalidate syntax only

A source edit can change callable return, parameter facts, field facts or type signatures
without changing a consumer file's syntax.

### Mixed-generation query

Do not publish pieces of semantic state independently if consumers expect coherence.

## Typing/proving hazards

### Equality = subtyping = assignability

These relations differ. Keep APIs explicit.

### Unknown proof = false

Failure to prove is not a counterexample.

### Path refinement survives mutation

If a mutable/aliased value can change, smart-cast-like facts may be invalidated. Track the
conditions that make refinement stable.

### SMT everywhere

Most semantic facts should use cheap dataflow/abstract domains. SMT is a targeted backend,
not the first abstraction.

## Runtime hazards

### Semantic model disagrees with VM

Check compiler/runtime lookup, selector formation, field side, module behavior and native
primitives against spec.

### Native semantic stub stronger than implementation

A tool declaration that promises non-null/no-throw/pure while native code violates it makes
checker/prover/optimizer unsound.

### Concurrency effect ignored

Future fibers/native calls may yield/block/escape closures. Do not assume ordinary calls are
atomic once concurrency semantics say otherwise.

## Security/robustness hazards

Semantic analysis processes malformed/untrusted source. It must avoid:

- panics on invalid AST recovery shapes;
- unbounded recursion on pathological source graphs;
- exponential union/constraint growth;
- unbounded diagnostics/provenance;
- path canonicalization assumptions that escape workspace policy.

Use depth/size caps with explicit conservative fallbacks.

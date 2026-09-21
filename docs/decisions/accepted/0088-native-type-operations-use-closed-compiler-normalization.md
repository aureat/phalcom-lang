# TDR-0088 — Native type operations use a closed compiler-normalized calculus, not VM execution

- Ratified: 2026-09-21
- Status: Accepted
- Date: 2026-09-21
- Supersedes: none
- Amends: [TDR-0082](./0082-declaration-modifiers-and-identity-based-access-domains.md) for native type-member implementation origin
- Depends on: [TDR-0083](./0083-type-level-domains-and-typed-compile-time-parameters.md), [TDR-0084](./0084-canonical-type-level-terms-and-inference-lifetimes.md), [TDR-0085](./0085-parameterized-type-members-and-symbolic-projections.md), [TDR-0086](./0086-type-level-subscript-is-tuple-at.md), [TDR-0087](./0087-structural-map-is-shape-preserving.md)
- Related: LANG007; compiler semantic normalization; Universe intrinsic metadata
- Repository evidence revision: `aureat/phalcom-lang@5b63ade8073d13d6b528af4774181665c30cbd47`

## Context

TDR-0082 established the current declaration-modifier model:

```text
native
    declaration-derived implementation origin

@intrinsic
    Universe-only metadata identity
```

and explicitly rejected deriving implementation semantics or authorization from naming conventions.

LANG007 introduces native type members such as the accepted equivalents of:

```phalcom
@intrinsic
native type Positionals: TypePack

@intrinsic
native type Map<F: Type -> Type>: Type

@intrinsic
native type [I: Int]: Type
```

These declarations are not runtime methods.

Their job is to participate in compile-time semantic normalization over canonical type-level terms.

Without an explicit decision, there are several tempting but damaging implementation shortcuts:

- register native type members as runtime native callables;
- execute type-level `Map` by materializing a runtime Tuple and sending a message;
- introduce bytecode for type-member evaluation;
- invoke arbitrary user methods during type checking;
- treat `@intrinsic` as permission to hard-code behavior by name;
- generalize the feature into an open compile-time interpreter.

Those approaches would make type checking depend on runtime heap state, VM availability, effects, user method dispatch, or non-terminating code. They would also undermine deterministic caching and incremental compilation.

## Decision

### 1. TDR-0082's modifier distinction remains authoritative

This record preserves:

```text
native
    implementation origin

@intrinsic
    builtin metadata / canonical intrinsic association
```

`@intrinsic` is not executable semantics by itself.

`native` does not grant special visibility or runtime authority.

### 2. Native implementation origin is declaration-category specific

For an executable callable declaration:

```text
native
    may denote host/runtime/native callable implementation
```

For a native type member:

```text
native
    denotes host compiler/semantic normalization implementation
```

The common rule is:

> The implementation is supplied by the Phalcom host implementation rather than a Phalcom source body, through the implementation mechanism appropriate to the declaration category.

This is the narrow amendment to TDR-0082.

### 3. Native type-member evaluation occurs in the semantic/type system

A type-member projection such as:

```phalcom
(Int, String)::Map<Option>
```

is normalized over canonical semantic terms.

Its evaluator may inspect:

```text
canonical subject type/domain
canonical member identity
canonical type-member arguments
canonical structural payload types
stable symbolic projections
```

It must not depend on runtime object materialization or VM dispatch.

### 4. Type-level intrinsic evaluation does not execute runtime messages

The implementation must not normalize type members by:

```text
allocating runtime Tuple/Record values
sending [] / Map / At messages
pushing VM frames
executing bytecode
running runtime native callable descriptors
using runtime object addresses
consulting mutable runtime heap state
```

Runtime behavior may be used as semantic precedent, but it is not the execution engine for static type operations.

### 5. LANG007 introduces no type-member VM bytecode

There is no dedicated runtime instruction for:

```text
Positionals
Labeled
At
Map
```

or general type-member projection.

The compiler/semantic layer produces ordinary canonical type results before runtime code generation.

### 6. Intrinsic dispatch uses canonical identity, not spelling

A compiler intrinsic evaluator is selected by canonical type-member identity.

The implementation must not decide behavior with ad hoc string tests such as:

```text
if member_name == "Map"
```

after semantic identity has been established.

Universe source declarations may act as the canonical source/provenance anchors for intrinsic members, but the evaluator binds to semantic identity rather than source spelling.

### 7. Intrinsic normalization is deterministic and side-effect free

A native type operation must be suitable for compiler normalization.

Its behavior must be:

```text
deterministic
free of user-observable side effects
independent of runtime scheduling
independent of mutable heap state
cacheable by semantic inputs
bounded by compiler normalization controls
```

A normalizer may detect:

```text
Stuck
Invalid
cycle
budget exhaustion
cancellation
```

without converting those operational states into unrelated language types.

### 8. The type-level operation set is closed

LANG007 introduces a closed set of compiler-known type-level structural operations accepted by the language specification.

This does not create a general user facility for:

```text
arbitrary recursive type functions
compile-time method execution
type-level loops
compile-time I/O
compile-time mutation
open rewrite-rule registration
runtime reflection as a type-computation oracle
```

Future compiler-known operations may be added through later accepted language decisions.

### 9. Semantic type-level domains do not automatically become runtime object classes

The existence of:

```text
TypePack
LabeledTypePack
RecordRow terms
symbolic type-member projections
compile-time const terms
```

does not require corresponding runtime heap classes or ordinary runtime values.

A future reflection feature may choose to reify descriptors for some of these concepts, but that is a separate decision.

### 10. Stable source ownership and runtime callable ownership remain separate

A canonical Universe declaration may provide:

```text
source location
documentation anchor
intrinsic identity
navigation/reflection provenance
```

for a type member.

That declaration must not automatically enter the runtime native-callable registry merely because it is marked `native`.

Runtime native callable census and compiler-native type-member census are different implementation products.

## Consequences

Type checking remains deterministic, VM-independent, and suitable for incremental caching.

The runtime does not acquire a second type solver.

Universe source can still make builtin type operations discoverable and inspectable without pretending that a type projection is an ordinary message send.

The compiler needs a dedicated semantic intrinsic registry/evaluator path keyed by canonical member identity.

This decision deliberately limits metaprogramming power: LANG007 type computation is an explicitly closed calculus rather than arbitrary compile-time Phalcom execution.

## Rejected alternatives

### Execute native type members as runtime methods

Rejected because type checking must not depend on runtime VM execution or heap values.

### Add type-member bytecodes

Rejected because the operations are compile-time semantic normalization, not runtime program behavior.

### Use `@intrinsic` as the implementation switch

Rejected because TDR-0082 defines it as metadata-only identity, separate from implementation origin.

### Dispatch intrinsics by member-name string

Rejected because semantic identity, not spelling, owns behavior.

### Allow arbitrary user code during type normalization

Rejected because it undermines termination, determinism, caching, reproducibility, and the closed LANG007 semantic model.

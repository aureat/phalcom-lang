# TDR-0090 — Structural static evaluation, Pack/Product/Sum reflection, and binding semantics

- Status: **Accepted**
- Ratified: 2026-10-04
- Checkpoint: `LANG007.C4`
- Supersedes the current binding/field semantics of [TDR-0052](./0052-let-const-bindings-and-field-mutability.md).
- Extends the structural type-level work of TDR-0083 through TDR-0088.
- Related specifications:
  - [Bindings and Fields](../../specs/bindings-and-fields.md)
  - [Static Evaluation](../../specs/static-evaluation.md)
  - [Structural Products](../../specs/structural-products.md)
  - [Structural Sums](../../specs/structural-sums.md)
  - [Arguments and Parameters](../../specs/callables/arguments.md)

## Context

LANG007.C1–C3 established canonical static domains, heterogeneous static arguments,
structural Tuple/Record terms, symbolic relation outcomes, parameterized type members,
incremental semantic products, and compiler-normalized type operations. C4 turns those
foundations into one coherent structural/static language model rather than adding
isolated metaprogramming features.

The design is governed by two constraints:

1. static Telos is ordinary Telos semantic code evaluated by the semantic analyzer,
   not bytecode or arbitrary runtime dispatch;
2. public source semantics are Pack/Product/Sum based even when the compiler retains
   specialized TypePack/row nodes internally for efficiency.

## Decision

### Bindings and fields

```telos
let x = runtimeExpression
let mut y = runtimeExpression
let mut pending: T
const z = staticExpression
```

- `let` is runtime and immutable; an initializer is mandatory.
- `let mut` is runtime and mutable; its initializer may be omitted.
- An initializer-less `let mut` is in an **uninitialized storage state**. That state
  is not `None` and is not part of the binding's value type.
- Reading an uninitialized mutable binding traps at runtime.
- Local flow analysis tracks `Uninitialized | MaybeInitialized | DefinitelyInitialized`.
  Proven reads require no runtime check; definitely invalid reads are diagnosed;
  maybe-initialized reads retain the runtime check.
- `const` is compile-time and immutable; an initializer is mandatory and a closed
  constant must fully reduce.
- Fields are immutable by default. `mut` marks a mutable field.
- Fields must be declared explicitly; assignment never creates an implicit field.
- Immutable fields may be initialized during the constructor lifecycle under definite
  initialization rules and cannot be mutated afterward.

### Static evaluation

Parameterized `type` declarations are the reusable static-computation abstraction:

```telos
type IsSubtype<A: Type, B: Type>: Bool =
    Subtype<A, B>

type Count<*P>: Int {
    let mut count = 0
    for _ in P {
        count += 1
    }
    count
}
```

Static evaluation is deterministic, effect-free semantic evaluation. C4 does not add
const functions or const methods. It supports the ratified ordinary control forms
`if`, `match`, `for`, `while`, local bindings, assignment to evaluator-local
state, `return`, `break`, and `continue`.

`while` remains an explicit semantic control-flow construct. A method or selector
named `whileTrue` has no role in static evaluation.

Generic computations may residualize into normalized semantic residual IR. Raw source
AST, evaluator continuations, VM values, and process-local IDs are not residual
semantics.

Proof predicates such as `Equivalent`, `Subtype`, and `Conforms` have public
result domain `Bool`: proof reduces to `true`, refutation to `false`, and
unresolved open/generic cases remain symbolic. Uncertainty is never coerced to
`false`.

### Pack

`Pack` is a first-class immutable ordered two-lane product: positional values and
ordered Symbol-labeled values. Lane identity, order, labels, and values participate
in structural equality. `Pack()` is distinct from Unit/Tuple and Record.

The implementation is intentionally optimized for packing, rest capture, slicing, and
spreading:

- flat/indexed storage or compiler-virtual equivalents;
- O(1) lane lengths and O(1) rest/slice views;
- no recursive cons representation;
- fused multi-spread construction;
- ordinary fixed-shape calls do not allocate a Pack merely because an argument shape
  is semantically a Pack;
- `*rest`, `**rest`, and `***rest` capture Packs.

Internal TypePack representations may remain compiler optimizations, but the public
model is `PositionalPack<Type>`, `LabeledPack<Type>`, and `CompletePack<Type>`.

### Products and unordered fields

Every transparent product data declaration receives compiler-derived sealed structural
capabilities:

```text
Product
OrderedProduct
UnorderedProduct
```

Ordinary user `impl` declarations cannot forge those capabilities.

Unordered products use:

```telos
type FieldKey = Symbol | String | Int | Bool
```

with a language-defined version-stable total order. Fixed fields, presence, lookup,
and open/captured tails are structural semantics; anonymous open tails are descriptions
of possible structure, not finite iterable sequences.

Static iteration preserves exact per-iteration structural refinement for heterogeneous
products.

### Sums

Every enumerated `data` declaration receives the compiler-derived sealed `Sum`
capability. Reflection exposes ordinary static data:

```telos
Variant(name: Symbol, shape: ProductShape)
type Variants = PositionalPack<Variant>
```

A Symbol denotes a variant family and may correspond to zero, one, or many structural
variants. Variant order is declaration order. C4 exposes ordinary structural variant
information only; public GADT refinement metadata is deferred.

### Recursive reflection and static values

Reflection describes immediate structure and stores contained types as semantic Type
references. It never recursively expands a contained recursive type's reflection.

Reduced structural static values form an acyclic DAG. Recursive semantic entities use
stable semantic references.

Pack and shape views are one-step semantic decompositions over flat/indexed backing
state; they do not require recursively linked representations.

Derived Product/Sum capability and reflection both originate from lower-level canonical
declaration facts; neither recursively proves the other.

### Materialization, metadata, and incrementality

A runtime use of a `const` materializes the already-evaluated semantic value through a
semantic materialization descriptor; the initializer is never rerun.

Cross-module static definitions export checked semantic definition information and
stable residual semantics only as required. Metadata never serializes arena IDs,
VM object identities, or process-local Symbol IDs as semantic identity.

C4 performs a hard public-ontology cutover: old TypePack/RecordFields-facing source
concepts do not remain a parallel first-class model.

## Consequences

- TDR-0052's current `let = mutable`, `const = runtime immutable`, mutable-by-default
  field rules are superseded.
- Static evaluation belongs to `phalcom-semantic`; compiler/runtime lowering consumes
  frozen semantic products.
- The existing pattern/GADT resolver, relation engine, row calculus, query budget, and
  incremental database remain the authorities used by C4.
- Pack's physical representation is deliberately not source-observable; implementations
  may use argument windows, slices, scalar replacement, flat product storage, or
  materialized Pack objects as appropriate.
- No compile-time VM and no arbitrary runtime method execution are introduced.

# Knowledge Domains: Shape, Type, Proof, Effect, Dynamic State

## Why this separation matters

Phalcom is evolving from a dynamic language with strong semantic tooling toward optional
but correctness-participating typing. The existing LSP already infers runtime shapes.
The future checker will reason about language types. Static proving may reason about
propositions. Fibers/concurrency may require effect facts.

These are related but not interchangeable domains.

## Runtime value shape

A runtime shape answers questions like:

- "this expression is probably/exactly an instance of class `Point`";
- "this is the class object `Point`";
- "this list's observed element shapes join to `String | Number`";
- "this is a callable with this semantic identity";
- "this is a method Family with receiver knowledge".

Current `phalcom-lsp::semantic::ValueShape` is this domain.

Shape knowledge is useful for:

- completion;
- hover;
- advisory diagnostics;
- dispatch target inference;
- interprocedural LSP summaries;
- seeding future type inference.

It is not by itself a normative type contract.

## Language type

A language type answers normative questions such as:

- Is value type `A` assignable to declared type `B`?
- Does class `C` conform to protocol `P`?
- What is `List<Int>` after substitution?
- What is the join/meet under the language's type lattice?
- Does a generic argument satisfy bounds?
- Is a method's inferred return compatible with its declared return?

A future type representation needs concepts that runtime shapes do not naturally have:

- generic type parameters and substitutions;
- applied types;
- protocols/structural requirements;
- `Self`;
- `Dynamic`/`Any`/`Nothing` or other special types;
- variance;
- intersections/aliases/refinements as ratified;
- recursive or existential forms if introduced.

Do not stretch `ValueShape` until it becomes a half-correct type algebra.

## Type expression versus resolved type

Preserve the distinction:

```text
source type syntax -> type expression AST -> resolved type identity -> normalized/canonical type
```

Reflection may need lossless source metadata even when the checker has a canonicalized
internal type.

An absent annotation is not the same source fact as an explicit `Dynamic` annotation,
even if a checking rule later treats them similarly.

## Inference variable versus type

During checking, a fresh variable such as `?T42` is not a user-visible type. It is a
solver metavariable with constraints. Do not leak solver identity into reflection or
semantic navigation.

## Runtime dynamic state

Future typing must distinguish at least conceptually:

- unannotated/unknown because no type was declared;
- explicit dynamic escape hatch;
- `Any`/top-like type if Phalcom adopts it;
- type checker failure to infer;
- runtime value whose class is currently unknown to LSP.

These have different soundness and tooling implications.

## Bottom/impossible

`Nothing`/bottom (if ratified) means "no value can inhabit this point" in the type
lattice. This is not ordinary `Unknown`.

Bottom is useful for:

- unreachable branches;
- under-constrained local generic inference if the spec chooses it;
- `throw`/non-returning operations;
- exhaustive match reasoning;
- join identities.

Unknown means loss of information; bottom means impossibility.

## Proof fact

A proof fact is a proposition that holds at a program point, for example:

```text
x != None
n > 0
index < list.size
result.isSome
variant(value) = Some
```

The type system may consume proof facts to refine types, but a proposition is not itself
a type unless Phalcom explicitly adopts refinement/dependent typing semantics.

Static contracts (`@requires`, `@ensures`, `@invariant`) should eventually produce
obligations in a proof domain, not be encoded as ad-hoc type names.

## Effect fact

Potential future effects include:

- may throw;
- may perform dynamic/reflective send;
- may invoke callable parameter N;
- may mutate field/global state;
- may allocate;
- may yield fiber;
- may block OS thread;
- may perform I/O;
- may escape/capture a closure;
- may not return.

Current callable summaries already model `dynamic_send` and invoked callable parameter
positions. Extend this idea rather than bolting effects onto individual consumers.

## Confidence and proof strength

Current semantic inference has confidence categories such as exact, flow,
interprocedural, heuristic. Future typing should not simply inherit those as proof levels.

Recommended conceptual split:

```text
Fact<T> {
  value: T,
  origin/provenance,
  confidence/precision metadata
}

Judgment {
  status: Proved | Refuted | Unknown,
  evidence
}
```

A heuristic shape can be useful for completion while being unusable as checker proof.

## Bridges

### Shape -> type

Safe examples:

- exact literal/runtime class shape can seed a synthesized nominal type;
- exact `ClassObject(C)` can inform a meta/class-object type;
- tuple/record shape can seed a structural/product type if the typing spec defines one.

Unsafe shortcut:

> "The LSP observed only `Int` call sites, therefore the unannotated parameter has the
> normative type `Int`."

Call-site observations are open-world evidence, not automatically a declaration contract.
The checker spec must decide when such evidence is admissible.

### Type -> shape

A declared type may constrain possible runtime shapes, but protocols/unions/generics may
not identify one concrete runtime class. Preserve the abstraction.

### Proof -> type refinement

A branch fact can refine a type:

```text
Option<T> + predicate isSome -> Some<T> / T projection as specified
T | None + x != None -> T
```

Only perform refinements whose predicate semantics are trusted and specified.

### Effect -> static legality

A future no-yield/no-block region may be checked from effect summaries. Do not infer that
contract merely from current absence of observed effects unless the analysis is complete
for all dynamic behavior.

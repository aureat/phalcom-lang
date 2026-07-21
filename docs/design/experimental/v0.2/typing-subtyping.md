# Typed subtyping — conformance termination & override compatibility (proposed)

- Status: **Proposed** (experimental; not ratified) · **soundness teeth**
- Axis: typing (subtype relation, variance, inheritance)
- Resolves: [typing.md](typing.md) Tier-1 gaps #2 (conformance termination) and #3 (override/Liskov)
- Related: [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) (selector identity), [object-model.md](../object-model.md) (single inheritance), [method-lookup.md](../method-lookup.md)

## Problem

[typing.md §5.3](typing.md) states the conformance rule (contravariant params,
covariant result) but leaves two things unspecified, both soundness- or
termination-critical:

1. **Termination.** §5.4/§5.5 add F-bounded generics (`Comparable<Self>`), structural
   matching, and variance. That combination is the classic **undecidable-subtyping**
   trap (F<: subtyping is undecidable; recursive protocols expand forever). A naive
   check of `Int <: Comparable<Int>` recurses without base case.
2. **Override compatibility.** The conformance rule covers *protocols* but nothing
   covers a **subclass overriding** a superclass method under the committed single
   inheritance. An override that narrows a parameter or widens a result is a
   soundness hole.

## Decision

### Conformance is coinductive with an assumption set

To decide `S <: P`, maintain an **assumption stack** of goals in progress. Before
recursing into a sub-goal, check the stack: **if the same `(S, P)` goal recurs,
succeed** (assume-the-goal — Java JLS §4.10, Scala's approach). This gives a finite
proof for `Int <: Comparable<Int>`:

```
Int <: Comparable<Int>              push goal
  ⇒ Int has compareTo(Self=Int) -> Ordering       ✓ (structural)
  ⇒ its use of Comparable<Int> recurs → assumed ✓
```

- **Recursive types compare by name + type arguments, not by structural unfolding.**
  Because protocols and classes are *named* ([typing.md §5.3](typing.md)),
  `Comparable<Int>` is a single node — isorecursive folding at the name. Equirecursive
  unfolding (which reintroduces non-termination) is **not** used. This also settles
  `class Node { next: Option<Node> }`: `Node` folds at its name.
- **Depth cutoff.** A generous configurable bound backstops pathological F-bounds;
  exceeding it emits a diagnostic ("subtype check too deep; annotate to disambiguate")
  rather than looping. Decidable in practice, honest at the edge.

### Override rule (Liskov), one variance rule reused

A subclass method overriding the superclass method for the **same selector** must
have a signature that is a **subtype of the overridden arrow**:

> **params contravariant, result covariant** — an override may *widen* parameter
> types and *narrow* the result, never the reverse. Violation is a compile error.

This is literally the §5.3 conformance rule applied to inheritance instead of
protocols — **one rule, two uses.** `Self` stays bound to the actual receiver, so:

- An override returning `Self` is always compatible.
- An override returning the *fixed superclass type* is **not** a valid override of a
  `Self`-returning method (it breaks per-receiver refinement).

### Variance positions are validated at class-def time

Declaration-site `out`/`in` ([typing.md §5.4](typing.md)) is *checked*, not trusted:
an `out T` parameter appearing in an **input** position (or `in T` in an output
position) is rejected where the class is defined. This is the check that makes
`List<out T>` read-only vs `MutableList<T>` invariant an *enforced* distinction
rather than an aspirational comment.

## Edge cases

| Case | Resolution |
|------|-----------|
| `Int <: Comparable<Int>` | Terminates via assumption stack. |
| `class Node { next: Option<Node> }` | Folds at name `Node`; finite. |
| Mutually recursive protocols `A: B`, `B: A` | Both goals assumed on the stack; succeeds or fails finitely. |
| Override `draw(x: Square)` of super `draw(x: Shape)` | **Rejected** — parameter narrowed (contravariance violated). |
| Override `bounds() -> Square` of super `bounds() -> Shape` | Allowed — result narrowed (covariance). |
| `out T` used as a method parameter | Rejected at class-def (variance-position check). |

## Precludes

- **Equirecursive structural subtyping** — would reintroduce the termination problem.
  Named isorecursive folding stands.
- **Covariant parameter overrides** (the intuitive-but-unsound "specialize the
  argument"). One-way variance is locked; this is the single most common OO type
  bug and it is rejected by construction.

# Typed inference — local type arguments & default return types (proposed)

- Status: **Proposed** (experimental; not ratified)
- Axis: typing (inference)
- Resolves: [typing.md](typing.md) Tier-1 gap #4 (type-argument inference failure modes) and Tier-2 gap #7 (default return type / `Unit`)
- Related: [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md), [blocks.md](../blocks.md), [functions.md](../functions.md)

## Problem

[typing.md §5.8](typing.md) names "local type-argument inference (Pierce–Turner), no
HM" but does not specify how constraints are collected, how they are solved under
variance, what happens when a type argument is **under-** or **over-constrained**, or
what an **un-annotated method** returns. As written it is not implementable.

## Decision

### Local type-argument inference (confined to one send)

For a generic send `r.m<T…>(args)`:

1. **Collect.** Each argument of type `A` at parameter type `P(T)` yields the
   constraint `A <: P(T)`, decomposed by the variance of `T`'s position:
   covariant position ⇒ *lower* bound (`T :> A`), contravariant ⇒ *upper* bound
   (`T <: A`).
2. **Solve.** Each `T` = the **least upper bound** of its lower bounds (default,
   covariant), then checked against any declared bound (`T: Comparable<T>`). Failure
   of the bound is a diagnostic.
3. **Expected type flows in.** In *check* mode the result-position constraint is
   added first: `let xs: List<Int> = ys.map { … }` seeds `U = Int` before the block is
   even checked (bidirectional inward flow).
4. **No cross-statement propagation.** Inference is scoped to the single send; nothing
   leaks to the next statement. (Scala/Swift-style locality.)

### Failure modes are explicit, never silent

| Situation | Result |
|-----------|--------|
| **Under-constrained** `T` (no lower bound, e.g. `[]` alone) | Solve to `Nothing`; if that breaks a declared bound, **require an explicit type argument** (`List<Int>()`), with a diagnostic. Never fall back to `Any`. |
| **Over-constrained** `T` (incompatible bounds from two positions) | Compile error naming the two conflicting argument positions. |
| **Ambiguous** (multiple maximal solutions, no LUB) | Compile error asking for an explicit `<…>`. |

Silent `Any` inference is banned: it would hide exactly the errors the checker
exists to catch.

### Default return type — inferred, not `self`, not `Unit`

An un-annotated method's result type is the **join of its `return`/tail-expression
types** (checker-internal, erased — no runtime effect):

- Smalltalk defaults to returning `self`; Phalcom methods instead carry an explicit
  tail-expression value ([functions.md](../functions.md)), so *inferring* is both more
  precise and still erasable.
- A method whose body yields no meaningful value (all statements, no value tail) has
  result **`Unit`**. `Unit` is a real single-valued type (its one value is the
  receiver, upholding the "everything returns something" convention) so `-> Unit`
  methods still chain as `self`.

### Recursion guard

A method whose inferred return type would depend on its own inferred return type
(unannotated (mutual) recursion) **requires an explicit result annotation**, with the
diagnostic "cannot infer the return type of a recursive method; annotate it." This is
the standard restriction (Scala requires result types on recursive defs) and keeps
inference a finite bottom-up pass.

## Edge cases

| Case | Resolution |
|------|-----------|
| `[].map { x => x }` | element `Nothing`; block param `x: Nothing`; result `List<Nothing>` — usually forced by an expected type. |
| `let n = 1 + 2` | synthesized `Int` (no annotation needed). |
| `id<T>(x: T) -> T` applied as `id(1)` | `T = Int` from the single lower bound. |
| `pick<T>(a: T, b: T)` as `pick(1, "x")` | `T = LUB(Int, String) = Any`; usable only for `Any`-level messages, else annotate. |
| recursive `fact(n) => n <= 1 ? 1 : n * fact(n - 1)` | needs `-> Int` (recursion guard). |

## Precludes

- **Global / whole-program inference (HM)** — locked out by subtyping + structural
  types + label-encoded selectors (restates [typing.md §5.8](typing.md)).
- **Silent `Any` fallback** for under-constrained type arguments — replaced by an
  explicit-annotation requirement, so inference never launders a type error into
  `Any`.

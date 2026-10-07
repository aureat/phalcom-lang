# 1. Executive architectural recommendation

## Decision in one page

Telos should adopt a **capability-oriented numerical foundation** with a small
nominal runtime core and no universal implicit numeric tower.

1. Keep `Number` as the runtime behavioral root for the built-in numeric values,
   but do not use it as the contract for every generic algorithm. `Number` is a
   dynamic dispatch category, not a proof that an operation is closed, ordered,
   exact, or field-like.
2. Keep `Int` as one exact, unbounded surface type with hidden immediate and
   `LargeInt` representations. Keep `Float` as IEEE-754 binary64. These are
   already the right concrete foundations in the current implementation and
   accepted numeric decisions.
3. Add `Rational` as a concrete, normalized, exact data value in a later
   foundation slice. Recommend `Rational` with an implementation-owned integer
   representation, not public `Rational<T>` in V1. Its public semantics should
   be independent of whether the backing integers are immediate or heap-backed.
4. Add `Complex<T>` as a generic data family only after generic construction,
   associated behavior, and numeric capability evidence are stable. The first
   useful specialization is `Complex<Float>`; `Complex<Rational>` and
   `Complex<Int>` should be valid only where operation-specific laws make sense.
5. Represent generic mathematics with **small independent traits/capabilities**
   and associated types, not a Rust-like inheritance tower. Use existing
   `trait`, `impl`, `where T |= Trait`, associated types, `Self`, and evidence
   machinery. Do not invent trait inheritance or trait objects.
6. Make conversion explicit at API boundaries. Mixed operators may continue to
   use the existing built-in bilateral operator machinery, but third-party
   numeric types must participate through explicit conversion/promotion
   witnesses rather than a compiler-wide conversion lattice.
7. Keep `universe.math` as a lightweight facade. It owns Float elementary
   functions, constants, angle conversion, and exact integer algorithms. A
   future `universe.numbers` owns Rational/Complex and numeric protocols only
   when those types are ready; it must not force a heavyweight scientific stack.
8. Treat complex functions as a separate backend contract. Do not expose
   `sqrt`, `log`, `pow`, or trigonometric functions for complex values until
   branch cuts, signed zero, infinities, NaNs, and accuracy guarantees are
   specified per function.

## What this avoids

- A nominal taxonomy where `Real`, `Exact`, and `Approximate` imply operations
  that a type cannot actually provide.
- Python-style reverse-method boilerplate as the primary extension mechanism.
- Julia-style global promotion rules whose interaction with selector identity
  and user-defined classes would be difficult to make predictable.
- Treating mathematical real numbers as though they were the current `Float`.
- Making all future math generic before Telos has a stable type-level operation
  vocabulary.

## Recommended architectural invariant

> A generic algorithm asks for the smallest named capability it needs; a numeric
> representation advertises exactness, ordering, conversion, and special-value
> behavior separately; no type name alone silently supplies a stronger law.

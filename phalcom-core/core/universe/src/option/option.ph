
// Absence is an Option (ADR-0007), not a surface `nil`. `Option` is abstract;
// `Some` wraps one value and `None` is an immediate variant. These are
// bootstrapped in Rust (universe.rs): the classes, the `Some(_)` construction
// primitive, and the `match(some:, none:)` eliminator. The skeletons below only
// *reopen* those bootstrapped rows so the class names are surface-visible.
//
// U-CORE-2 (catalog-delta.md §2.2) adds the four combinators that make
// `ifTrue`/`ifFalse`'s newly-well-formed `Option` result actually chainable —
// `ifNone(_)`, `orElse(_)`, `isSome`, `isNone`, every one defined over `match`
// (values-and-absence.md §3.3), so `Some>>_` / `None>>_` branching stays
// dispatch, not a variant check. The richer suite (`map`, `flatMap`,
// `filter`, `ifSome`, `unwrapOr`, …) is still deliberately NOT defined here —
// that remains U-STD's job. Do not add those bodies to this skeleton.
//
// `None` (the name) resolves to the immediate *value*, not the `None`
// class; that global is bound in Rust (VM::install_core).
//
// `None` has a source presentation below. The compiler preserves its special
// public immediate-value binding while reopening the hidden class row.

@native
class Option<T> is Object {

  @native
  match(some: Dynamic, none: Dynamic) -> Dynamic

  // Runs `f` (0-arity) for its side effect when `self` is `None`; passes
  // `Some` through untouched. Never extracts — returns `self` so calls chain
  // (values-and-absence.md §3.3's "Effect" group).
  ifNone(_ f) -> Self {
    match(
      some: |v| self, 
      none: || { 
        f.call()
        self 
      }
    )
  }

  // `Some` passes through unchanged; `None` becomes `f`'s (0-arity) `Option`
  // result (values-and-absence.md §3.3's "Transform" group). This is the
  // `??` operator's target (§3.4: `a ?? b` === `a.orElse || { b }`).
  orElse(_ f) -> Self | Dynamic {
    match(
      some: |v| self, 
      none: || f.call()
    )
  }

  isSome -> Bool { 
    match(
      some: |v| true, 
      none: || false
    ) 
  }

  isNone -> Bool { 
    match(
      some: |v| false, 
      none: || true
    ) 
  }

  // U-STD (values-and-absence.md §3.3's "Transform" group; catalog-delta §2.2):
  // `Some(v)` becomes `Some(f(v))`; `None` passes through untouched. `f` is a
  // 1-arity block over the wrapped value; the result is re-wrapped so the
  // chain stays an `Option`.
  map(_ f) -> Self | Option<Dynamic> {
    match(
      some: |v| Some(f.call(v)), 
      none: || self 
    )
  }

  // U-STD (values-and-absence.md §3.3's "Transform" group): like `map`, but `f`
  // already returns an `Option`, so its result is used directly rather than
  // re-wrapped — the monadic bind (`>>=`). `None` short-circuits to `self`.
  flatMap(_ f) -> Self | Option<Dynamic> {
    match(
      some: |v| f.call(v), 
      none: || self
    )
  }

  // U-STD (values-and-absence.md §3.3's "Filter" group): `Some(v)` stays `Some(v)`
  // when `pred(v)` is `true`, otherwise collapses to immediate `None`;
  // `None` passes through. `pred` must return a real `Bool` (ADR-0021).
  filter(_ pred) -> Self | Option<Dynamic> {
    match(
      some: |v| { if (pred.call(v)) { self } else { None } }, 
      none: || { self }
    )
  }

  // U-STD (values-and-absence.md §3.3's "Effect" group; mirror of `ifNone`): runs
  // the 1-arity block `f` for its side effect on the wrapped value when `Some`,
  // then returns `self` so calls chain; a `None` is passed through untouched.
  ifSome(_ f) -> Self {
    match(
      some: |v| { 
        f.call(v)
        self 
      }, 
      none: || self
    )
  }

  // U-STD (values-and-absence.md §3.3's "Extract" group): unwraps a `Some` to its
  // value, or yields `default` for a `None`. The eager sibling of `orElse`
  // (which takes a block); here `default` is an already-evaluated fallback value.
  unwrapOr<U>(_ default: U) -> U {
    match(
      some: |v| v, 
      none: || default
    )
  }

  // Display (values-and-absence §3, U-CORE-4, R-INV-4.3). Derived over
  // `match`, so a user-overridden `match` is respected (R-INV-2.4) and the
  // inner value is rendered via its OWN `toString` message (so a
  // value-typed payload agrees with the print path, R-INV-4.1).
  toString -> String { 
    match(
      some: |v| "Some(" + v.toString + ")", 
      none: || "None" 
    ) 
  }

  // absence -> error bridge (error-handling.md §5, result.md §2, ADR-0007):
  // `Some(v)` already carries a real value, so no reason is needed; `None`
  // has no value, so `err` fills in the failure reason. Round-trips with
  // `Result#ok()` below (`Some(v).okOr(_)` -> `Ok(v)` -> `.ok()` -> `Some(v)`).
  okOr<E>(_ err) -> Result<T, E> {
    match(
      some: |v| Ok.new(v), 
      none: || Err.new(err) 
    )
  }

  ==(_ other) -> Bool {
    other.is(Option).ifFalse { 
      return false 
    }

    match(
      some: |v| { 
        other.match(
          some: |ov| v == ov, 
          none: || false
        ) 
      },
      none: || other.isNone
    )
  }

  hash -> Int { 
    self.match(
      some: |v| v.hash, 
      none: || 0 
    ) 
  }
}

@native
class Some<T> is Option<T> {

  @class
  @native
  call(_ value: Dynamic) -> Some

  @class
  @native
  new(_ value: Dynamic) -> Some

}

@native
class None is Option {}

@native
class Unit is Object {

  toString -> String { "()" }

  hash -> Int { 0 }

}

// `Result`/`Ok`/`Err` (U-ERR, result.md §1-§3; ADR-0008 the error model,
// ADR-0007 the abstract-root-plus-two-subclasses machinery `Option`/`Some`/
// `None` already established). Unlike `Some`/`None` — bootstrapped natively
// because U6 predated U7's user-facing `@constructor` — `Result`/`Ok`/`Err` are
// **pure `.ph`**: U7's `@constructor` + `_`-prefixed instance fields need no
// floor primitive at all (net floor delta for this whole file: **0**).
//
// `Result` gets its **own** `match(ok:,err:)`, deliberately not reusing
// `Option`'s native one (forward-compat.md §2: the two must not couple, so a
// future migration of `Option` to `.ph` stays symmetric and doesn't touch
// `Result`).

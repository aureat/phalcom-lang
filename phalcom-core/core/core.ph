class Object {}

class Class {}
 
class Metaclass {}

class Number {}

class String {}

class Bool {}

// The boolean tower (ADR-0004): `Bool` is abstract; `True` and `False` are its
// two concrete singleton subclasses — the surface classes of `true`/`false`
// (so `true.class == True`). Their control-flow behaviour (`not`/`and`/`or`/
// `ifTrue`/`ifFalse`/`ifTrue:ifFalse:`) lives on `Bool` as sacred native
// primitives and is reached by inheritance (KEEP; see floor-census.md §2.6/§5),
// so these bodies are intentionally empty. The globals are already bound in Rust
// (VM::install_core, add_class!) — unlike `None`, they name the class objects,
// so these reopens re-emit the identical DefineGlobal binding (a harmless no-op).
class True {}

class False {}

class Symbol {}

// Absence is an Option (ADR-0007), not a surface `nil`. `Option` is abstract;
// `Some` wraps one value and `None` is a single shared singleton. These are
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
// `None` (the name) resolves to the shared singleton *value*, not the `None`
// class; that global is bound in Rust (VM::install_core).
//
// There is deliberately NO `class None {}` reopen here (unlike `Option`/
// `Some`): `Statement::Class` unconditionally emits `DefineGlobal` at the end
// of every class body, reopen or not (compiler/lib.rs). For every other core
// class that's a harmless no-op — the global already points at that same
// class object — but `None`'s global is bound to the *singleton instance*,
// not the class, so reopening it here would silently clobber that binding
// back to the class object the moment core.ph runs. See DEFERRED.md: a
// future unit that needs to add real members to `None` must fix that
// compiler special case first, not just re-add this skeleton.

class Option {
  // Runs `f` (0-arity) for its side effect when `self` is `None`; passes
  // `Some` through untouched. Never extracts — returns `self` so calls chain
  // (values-and-absence.md §3.3's "Effect" group).
  ifNone(f) {
    return self.match(some: { v => self }, none: { f.call(); self })
  }

  // `Some` passes through unchanged; `None` becomes `f`'s (0-arity) `Option`
  // result (values-and-absence.md §3.3's "Transform" group). This is the
  // `??` operator's target (§3.4: `a ?? b` === `a.orElse { b }`).
  orElse(f) {
    return self.match(some: { v => self }, none: { f.call() })
  }

  isSome => self.match(some: { v => true }, none: { false })

  isNone => self.match(some: { v => false }, none: { true })

  // U-STD (values-and-absence.md §3.3's "Transform" group; catalog-delta §2.2):
  // `Some(v)` becomes `Some(f(v))`; `None` passes through untouched. `f` is a
  // 1-arity block over the wrapped value; the result is re-wrapped so the
  // chain stays an `Option`.
  map(f) {
    return self.match(some: { v => Some.new(f.call(v)) }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Transform" group): like `map`, but `f`
  // already returns an `Option`, so its result is used directly rather than
  // re-wrapped — the monadic bind (`>>=`). `None` short-circuits to `self`.
  flatMap(f) {
    return self.match(some: { v => f.call(v) }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Filter" group): `Some(v)` stays `Some(v)`
  // when `pred(v)` is `true`, otherwise collapses to the shared `None` singleton;
  // `None` passes through. `pred` must return a real `Bool` (ADR-0021).
  filter(pred) {
    return self.match(some: { v => if (pred.call(v)) { self } else { None } }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Effect" group; mirror of `ifNone`): runs
  // the 1-arity block `f` for its side effect on the wrapped value when `Some`,
  // then returns `self` so calls chain; a `None` is passed through untouched.
  ifSome(f) {
    return self.match(some: { v => f.call(v); self }, none: { self })
  }

  // U-STD (values-and-absence.md §3.3's "Extract" group): unwraps a `Some` to its
  // value, or yields `default` for a `None`. The eager sibling of `orElse`
  // (which takes a block); here `default` is an already-evaluated fallback value.
  unwrapOr(default) {
    return self.match(some: { v => v }, none: { default })
  }
}

class Some {}

// Kernel List (ADR-0020): a native array-backed heap object (ListObject),
// not an InstanceObject — bootstrapped in Rust (universe.rs) with five floor
// primitives (rawLength/rawAt/rawSet/rawPush, plus native `new()`). This
// skeleton reopens that bootstrapped row to define the public protocol over
// those primitives (ADR-0019's "hybrid: native primitives, self-defined
// control"). `toString` is ALSO a native primitive this unit, not defined
// here — see the U-LIST return contract for why (element-value stringification
// is blocked on U-CORE-4; DEFERRED.md #19). U-STD (catalog-delta §2.4;
// DEFERRED.md #18/#20/#25) discharges the deferral for the combinator layer:
// `map`/`reduce`/`filter`/`includes`/`isEmpty` and the `at(_:put:)` wrapper
// over `rawSet` now live below, all pure `.ph` over the floor. Only
// **list-literal syntax** `[a, b, c]` remains deferred (it needs a new ADR +
// parser work; DEFERRED.md #6) — do not add that here.

class List {
  size => self.rawLength

  at(i) {
    return self.rawAt(i)
  }

  add(v) {
    self.rawPush(v)
    return self
  }

  each(f) {
    var i = 0
    while (i < self.size) {
      f.call(self.at(i))
      i = i + 1
    }
  }

  // U-STD (catalog-delta §2.4): a new `List` holding `f(x)` for each element,
  // in order. Built over `each`/`add`/`List.new` — never stringifies an
  // element (the `toString`-message trap, DEFERRED.md #19), so it is safe.
  map(f) {
    var result = List.new()
    self.each { x => result.add(f.call(x)) }
    return result
  }

  // U-STD (catalog-delta §2.4): a new `List` of the elements for which
  // `pred(x)` holds, in order. `pred` must yield a real `Bool` (ADR-0021);
  // the `ifTrue` result is discarded (used only for its side effect).
  filter(pred) {
    var result = List.new()
    self.each { x => pred.call(x).ifTrue { result.add(x) } }
    return result
  }

  // U-STD (catalog-delta §2.4; DEFERRED.md #25): fold `f(acc, x)` across the
  // elements left-to-right, seeded with `init`. `f` is a 2-arity block; the
  // final accumulator is returned. This is the shape `blocks_argument_to_method`
  // waited on. Selector `reduce(_:_:)` — the trailing block desugars to the
  // second positional argument (`reduce(init) { acc, x => ... }`).
  reduce(init, f) {
    var acc = init
    self.each { x => acc = f.call(acc, x) }
    return acc
  }

  // U-STD (catalog-delta §2.4): `true` when any element is `== x`, else
  // `false`. `==` is an ordinary send (value/identity via `object_eq`); no
  // element stringification.
  includes(x) {
    var found = false
    self.each { e => (e == x).ifTrue { found = true } }
    return found
  }

  // U-STD (catalog-delta §2.4): `true` when the list has no elements. `size`
  // is a real `Number`, so the `== 0` condition is a well-formed `Bool`.
  isEmpty => self.size == 0

  // U-STD (DEFERRED.md #18): the public `.ph` wrapper over the `rawSet(_,_)`
  // floor primitive — writes `put` at index `i` and returns `self` so writes
  // chain (mirrors `add`). Selector `at(_:put:)` matches `rawSet`'s 2 args;
  // the labeled parameter is named `put` (label == name, parser convention).
  at(i, put:) {
    self.rawSet(i, put)
    return self
  }
}

class System {
  static print() {
    // Native print function
  }
}

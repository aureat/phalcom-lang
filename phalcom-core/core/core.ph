class Object {
  // Is-kind-of test: true iff `cls` is the receiver's class or an ancestor of
  // it (object-model.md §8). Derived purely over the floor — class/==/superclass
  // — so it needs no native primitive (ADR-0019/0023). The superclass chain is a
  // run of class objects terminating in the `None` singleton (class_superclass
  // returns `None` at the root), so the walk stops on `c == None`. The `ifTrue`
  // result is in pop (statement) position, so U-CORE-2's Some-lift is elided;
  // the body neither reads nor depends on `ifTrue`'s return shape.
  isA(cls) {
    var c = self.class
    while (c != None) {
      (c == cls).ifTrue { return true }
      c = c.superclass
    }
    return false
  }
}

class Class {}

class Metaclass {}

// `Error` (U-CORE-6, ADR-0008) is bootstrapped natively with `_message` at
// fixed slot 0 (`message`/`raise` are native primitives reading/using it
// directly — `primitive/error.rs`), but ships with no user-visible way to
// *set* that slot: bare `Error.new()` (the generic 0-arg allocator) leaves it
// unset, and — per ADR-0011/U-INH §3.5 "fields stack, never alias" — a `.ph`
// *subclass* that independently assigns a same-named `_message` field gets
// its own **fresh** slot, not this one (`subclass_field_offset_stability`).
// So the only way to reach the native slot 0 from `.ph` is a `construct`
// declared on `Error` **itself**: this reopen (recognised at compile time as
// reopening the already-registered native class, not a fresh `extends`) adds
// exactly that, giving `Error.new(msg)` a working 1-arg constructor and every
// subclass a `super.new(msg)` to route through (error-handling.md §1,
// U-ERR: `throw ArgumentError("age must be >= 0")`-style user `Error`
// subclasses need this to carry a real `message`).
class Error {
  // Preserved bare 0-arg form (many pre-U-ERR call sites — `Error.new()` in
  // Fiber/Future fixtures — rely on it; declaring *any* `new`-named
  // `construct` drops the generic inherited bare allocator, U7, so this must
  // be declared explicitly alongside the 1-arg form below, not left implicit).
  construct new() { }
  construct new(msg) { _message = msg }
}

class Number {}

class String {
  // Display (U-CORE-4, R-INV-4.1): a string's display *is* itself — no
  // representation read, so this is `.ph`-derivable rather than a floor
  // primitive (ADR-0019's derivability test).
  toString => self
}

class Bool {
  // Display (U-CORE-4, R-INV-4.1): derived over the sacred `ifTrue(_,
  // ifFalse)` selector (proven syntax:
  // `control-flow/control_flow_send_equivalence.ph` L9). This `toString` is
  // NOT itself sacred (floor-census §5's `bool_sacred_pristine` tracks only
  // the six original selectors), so adding it does not trip the inliner
  // deopt.
  toString {
    return self.ifTrue({ "true" }, ifFalse: { "false" })
  }
}

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

  // Display (values-and-absence §3, U-CORE-4, R-INV-4.3). Derived over
  // `match`, so a user-overridden `match` is respected (R-INV-2.4) and the
  // inner value is rendered via its OWN `toString` message (so a
  // value-typed payload agrees with the print path, R-INV-4.1).
  toString => self.match(some: { v => "Some(" + v.toString + ")" }, none: { "None" })

  // absence -> error bridge (error-handling.md §5, result.md §2, ADR-0007):
  // `Some(v)` already carries a real value, so no reason is needed; `None`
  // has no value, so `err` fills in the failure reason. Round-trips with
  // `Result#ok()` below (`Some(v).okOr(_)` -> `Ok(v)` -> `.ok()` -> `Some(v)`).
  okOr(err) {
    return self.match(some: { v => Ok.new(v) }, none: { Err.new(err) })
  }
}

class Some {}

// `Result`/`Ok`/`Err` (U-ERR, result.md §1-§3; ADR-0008 the error model,
// ADR-0007 the abstract-root-plus-two-subclasses machinery `Option`/`Some`/
// `None` already established). Unlike `Some`/`None` — bootstrapped natively
// because U6 predated U7's user-facing `construct` — `Result`/`Ok`/`Err` are
// **pure `.ph`**: U7's `construct` + `_`-prefixed instance fields need no
// floor primitive at all (net floor delta for this whole file: **0**).
//
// `Result` gets its **own** `match(ok:,err:)`, deliberately not reusing
// `Option`'s native one (forward-compat.md §2: the two must not couple, so a
// future migration of `Option` to `.ph` stays symmetric and doesn't touch
// `Result`).
class Result {
  isOk => self.match(ok: { v => true }, err: { e => false })

  isErr => self.match(ok: { v => false }, err: { e => true })

  // Transforms the `Ok` value; an `Err` passes through unchanged (never
  // raises — a pure value transform, result.md §2).
  map(f) {
    return self.match(ok: { v => Ok.new(f.call(v)) }, err: { e => self })
  }

  // Transforms the `Err` reason; an `Ok` passes through unchanged — the
  // symmetric counterpart of `map` (result.md §2).
  mapErr(f) {
    return self.match(ok: { v => self }, err: { e => Err.new(f.call(e)) })
  }

  // Chains an `Ok` -> `Result` function (flat-map/monadic bind); short-
  // circuits on `Err` (result.md §2).
  andThen(f) {
    return self.match(ok: { v => f.call(v) }, err: { e => self })
  }

  // The `Ok` value, or **re-`throw`** the `Err` reason (`throw expr` ===
  // `expr.raise()`, ADR-0031 §1) — the value -> exception bridge
  // (error-handling.md §5, result.md §3). If the `Err` reason is not itself
  // an `Error` (a user built `Err.new(42)`), `raise` misses and this
  // surfaces the ordinary `doesNotUnderstand` miss — consistent with the
  // only-`Error`-throwable rule, not special-cased here.
  unwrap => self.match(ok: { v => v }, err: { e => e.raise() })

  unwrapOr(default) {
    return self.match(ok: { v => v }, err: { e => default })
  }

  // The `Err` reason, or re-`throw` if `Ok` — symmetric to `unwrap`.
  unwrapErr => self.match(ok: { v => v.raise() }, err: { e => e })

  // `Result` -> `Option`: drops the failure reason (result.md §2/§5). Round-
  // trips with `Option#okOr(_)` above.
  ok() {
    return self.match(ok: { v => Some.new(v) }, err: { e => None })
  }

  // Display: each arm renders its payload via its OWN `toString` (agrees
  // with `Option#toString`'s pattern, R-INV-4.1).
  toString => self.match(ok: { v => "Ok(" + v.toString + ")" }, err: { e => "Err(" + e.toString + ")" })
}

class Ok extends Result {
  construct new(v) { _value = v }

  match(ok:, err:) => ok.call(_value)
}

class Err extends Result {
  construct new(e) { _error = e }

  match(ok:, err:) => err.call(_error)
}

// The throw -> value bridge (error-handling.md §5): runs `self` (0-arity),
// capturing a `throw` into `Err(e)`; success is `Ok(v)`. Pure `.ph` over
// `on(_)(_)` (U-ERR, ADR-0038) — no floor cost. Installed on the abstract
// `Function` root so both `Block` and (reflectively) `Method` inherit it,
// mirroring how `call`/`on`/`ensure` are native on both.
class Function {
  // Explicit `()` (a method, not a getter — `attempt() { … }` vs `attempt =>
  // …`) so the call-site selector encodes as `attempt()`, matching the
  // spec's `{ risky() }.attempt()` call form (error-handling.md §5) exactly.
  attempt() {
    return { Ok.new(self.call()) }.on(Error) { e => Err.new(e) }
  }
}

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

  // U-STD item 4 (U-ITER-FIX plan §"Not in this unit", DEC-ITER-A resolved):
  // drives the cursor protocol (`iterate(_)`/`iteratorValue(_)`, ADR-0035 §1)
  // rather than a raw `size`/`at(_)` index walk. `for (x in self)` compiles
  // to the same `Invoke`-only `iterate`/`iteratorValue`/`isSome` loop as any
  // user iterable (spec §3.1) — no `block_call`, no index math — so `each`
  // (and everything below built over it: `map`/`filter`/`reduce`/`includes`)
  // is protocol-driven behavior-preservingly.
  each(f) {
    for (x in self) {
      f.call(x)
    }
  }

  // Cursor iteration protocol (ADR-0035 §1, iteration.md §1). `List` is the
  // reference iterable: the cursor is the integer index, and `Option` carries
  // the "more?" signal so no surface `nil` ever appears (Invariant 4). Written
  // purely in `.ph` over the existing `size`/`at(_)` floor — zero primitives.
  //
  // `iterate(_)` is given the previous cursor (or `None` to start) and returns
  // `Some(nextIndex)` while elements remain, `None` at the end. `cursor.map { c
  // => c + 1 }` advances a `Some(i)` to `Some(i+1)` and passes `None` through;
  // `.unwrapOr(0)` collapses that to the raw next index (`None` → 0). The
  // sacred `ifTrue { … }` already wraps its arm in `Some` and yields `None` on
  // the false branch (U-CORE-2), so `(next < self.size).ifTrue { next }` is
  // exactly `Some(next)` in range and `None` past the end — no explicit `Some`
  // wrap, no `Option#unwrap` (which this surface does not define).
  iterate(cursor) {
    let next = cursor.map { c => c + 1 }.unwrapOr(0)
    return (next < self.size).ifTrue { next }
  }

  // Given a live cursor (an index the `for` desugar extracted from a `Some`
  // that `iterate(_)` just returned), yields the element there (ADR-0035 §1,
  // iteration.md §1). Only ever called with an in-range index, so it defers to
  // `at(_)` directly.
  iteratorValue(cursor) => self.at(cursor)

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

  // U-CORE-5 (decisions.md Q5, R-INV-5.3 E1-E5): structural equality —
  // element-wise, order-sensitive, via each element's own `==`. Guarded by
  // `isA(List)` so a non-List `other` is simply unequal (E2), never a dNU.
  // Derived entirely over the floor (`size`/`at`/`isA`/`while`/`and`/`!`) —
  // no new native primitive (ADR-0019 unchanged). `and`/`!` are the
  // language's infix/prefix operator forms (`Bool#and(_:)`/`Bool#not()`
  // dispatched by the compiler, not dotted-call syntax — `and`/`not` are
  // reserved words and cannot follow `.` as a bare identifier).
  ==(other) {
    if (other.isA(List)) {
      var same = (self.size == other.size)
      var i = 0
      // `and` is lazy (short-circuits); once `same` is false the loop
      // condition is false without evaluating `i < self.size`, so the loop
      // exits before `at(i)` can run out of bounds.
      while (same and (i < self.size)) {
        same = (self.at(i) == other.at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  // U-CORE-5 (R-INV-5.3 E6): `!=` MUST route through `==`. The floor
  // `Object#!=` (`object_neq`) negates identity `value_eq` directly, NOT
  // `self.==` — without this override `list != other` would stay
  // identity-based and contradict the structural `==` above (the `==`⊗`!=`
  // decoupling hazard).
  !=(other) {
    return !(self == other)
  }
}

// Kernel Map/Set (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 1): native
// insertion-ordered hash collections — Object::Map/Object::Set, sharing the
// MapObject backing struct (DEC-CT-B) but with distinct raw-primitive
// bindings and distinct classes. This skeleton reopens the bootstrapped rows
// to define the public protocol over the raw floor (ADR-0019's "hybrid: native
// primitives, self-defined control"). Both are MUTABLE, so neither installs a
// `hash` override — they inherit Object#hash (identity), so per Q5
// (decisions.md, collection-protocol.md law 4) neither is a valid Map/Set key;
// `rawPut`/`rawAdd` enforce this (DEC-CT-C) by rejecting a mutable-collection
// key (List/Map/Set) with a raised Error.

class Map {
  size => self.rawSize

  // Total lookup: Some-shaped by convention (raw value on hit, the None
  // singleton on miss — mirrors List#at's rawAt shape, U-CORE-2/ADR-0021).
  at(k) => self.rawGet(k)

  // Insert/overwrite; returns self so `put` calls chain.
  at(k, put:) {
    self.rawPut(k, put)
    return self
  }

  includes(k) => self.rawHas(k)

  // Idempotent: removing an absent key is a no-op, still returns self.
  remove(k) {
    self.rawRemove(k)
    return self
  }

  // A fresh List of keys, in iteration (insertion) order.
  keys {
    var result = List.new()
    var i = 0
    while (i < self.size) {
      result.add(self.rawKeyAt(i))
      i = i + 1
    }
    return result
  }

  // A fresh List of values, in iteration (insertion) order.
  values {
    var result = List.new()
    var i = 0
    while (i < self.size) {
      result.add(self.rawValueAt(i))
      i = i + 1
    }
    return result
  }

  // 2-arg block `{ k, v => ... }` per entry, in iteration order.
  each(f) {
    var i = 0
    while (i < self.size) {
      f.call(self.rawKeyAt(i), self.rawValueAt(i))
      i = i + 1
    }
  }

  // Cursor iteration protocol (ADR-0035 §1, iteration.md §1) — identical
  // shape to List's, over `size`/`rawKeyAt`. DEC-CT-E: the cursor value
  // `iteratorValue` yields is the KEY (both Map and Set yield keys); `each(_)`
  // above remains the 2-arg entry form for Map.
  iterate(cursor) {
    let next = cursor.map { c => c + 1 }.unwrapOr(0)
    return (next < self.size).ifTrue { next }
  }

  iteratorValue(cursor) => self.rawKeyAt(cursor)

  // Structural equality: same key set, pairwise-== values (order-independent
  // over keys — `includes`/`at` do the membership + value work, not raw
  // index comparison). Guarded by `isA(Map)` so a non-Map is simply unequal.
  ==(other) {
    if (other.isA(Map)) {
      var same = (self.size == other.size)
      var i = 0
      while (same and (i < self.size)) {
        var k = self.rawKeyAt(i)
        same = other.includes(k) and (self.rawValueAt(i) == other.at(k))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  // MUST route through == (the ==/!= decoupling hazard) — Object#!= negates
  // identity, not this structural ==.
  !=(other) {
    return !(self == other)
  }
}

class Set {
  size => self.rawSize

  add(v) {
    self.rawAdd(v)
    return self
  }

  includes(v) => self.rawHas(v)

  remove(v) {
    self.rawRemove(v)
    return self
  }

  // Positional read in insertion order — not in the map-and-set.md selector
  // table, but a direct, zero-floor-cost derivation over rawAt that the
  // U-CORE-5 conformance harness (collection-protocol.md §2) needs, and a
  // natural extension of the sequence protocol every collection instantiates.
  at(i) => self.rawAt(i)

  each(f) {
    var i = 0
    while (i < self.size) {
      f.call(self.rawAt(i))
      i = i + 1
    }
  }

  // Cursor iteration protocol — identical shape to List's, over
  // `size`/`rawAt`.
  iterate(cursor) {
    let next = cursor.map { c => c + 1 }.unwrapOr(0)
    return (next < self.size).ifTrue { next }
  }

  iteratorValue(cursor) => self.rawAt(cursor)

  // Structural equality: same members, order-independent. Same-size plus
  // "every element of self is in other" is sufficient since neither set
  // holds duplicates (rawAdd is idempotent).
  ==(other) {
    if (other.isA(Set)) {
      var same = (self.size == other.size)
      var i = 0
      while (same and (i < self.size)) {
        same = other.includes(self.rawAt(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(other) {
    return !(self == other)
  }
}

// Kernel Tuple (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 2): a native fixed
// arity immutable slice — Object::Tuple, mirroring List's shape but with NO
// mutation selector (immutability is structural, TupleObject's Box<[Value]>).
// This is also the (a, b) literal's construction target (already emitted by
// U-COLL's parser as `Tuple.fromList(List.new().add(a).add(b))`), so the
// literal graduates automatically the moment this class exists.

class Tuple {
  size => self.rawSize

  at(i) => self.rawAt(i)

  each(f) {
    var i = 0
    while (i < self.size) {
      f.call(self.rawAt(i))
      i = i + 1
    }
  }

  // Cursor iteration protocol (ADR-0035 §1) — identical shape to List's.
  iterate(cursor) {
    let next = cursor.map { c => c + 1 }.unwrapOr(0)
    return (next < self.size).ifTrue { next }
  }

  iteratorValue(cursor) => self.rawAt(cursor)

  // Structural equality: same arity, pairwise-==. Guarded by isA(Tuple) so a
  // non-Tuple (including a same-elements List — cross-kind, E2) is unequal.
  ==(other) {
    if (other.isA(Tuple)) {
      var same = (self.size == other.size)
      var i = 0
      while (same and (i < self.size)) {
        same = (self.at(i) == other.at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(other) {
    return !(self == other)
  }

  // Value hash (DEC-CT-D): a .ph fold over each element's own `hash` —
  // order-sensitive, zero new floor. Consistent with == by construction (a
  // deterministic function of the same elements == compares), and survives
  // the future Int/Float split for free (forward-compat §4): it folds
  // *mathematical-value* hashes (whatever Number#hash decides), never bits.
  // Bounded by a large prime modulus so the accumulator stays a stable,
  // comparable Number regardless of tuple length.
  hash {
    var acc = 17
    var i = 0
    while (i < self.size) {
      acc = (acc * 31 + self.at(i).hash) % 999999937
      i = i + 1
    }
    return acc
  }
}

// Kernel Range (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 3): a native lazy
// numeric interval — Object::Range, three raw bound-field getters and NOTHING
// else on the floor. `each`/`toList`/`size`/`includes`/`first`/`last` are all
// derived here over `rawStart`/`rawEnd`/`rawInclusive` + Number arithmetic —
// `each` GENERATES elements, never allocates (RG-2 laziness), so
// `Range.new(1, 1000000, true)` stays O(1) to construct and each step of
// `each` is O(1). Bound convention (RG-1): `a..b` inclusive (`rawInclusive`
// true), `a...b` exclusive (false) — the reserved `..`/`...` literal's
// committed meaning, honored now via the explicit constructor.

class Range {
  first => self.rawStart

  last {
    return self.rawInclusive.ifTrue({ self.rawEnd }, ifFalse: { self.rawEnd - 1 })
  }

  size {
    var n = self.rawInclusive.ifTrue({ self.rawEnd - self.rawStart + 1 }, ifFalse: { self.rawEnd - self.rawStart })
    return (n < 0).ifTrue({ 0 }, ifFalse: { n })
  }

  includes(n) {
    var upper = self.rawInclusive.ifTrue({ n <= self.rawEnd }, ifFalse: { n < self.rawEnd })
    return (self.rawStart <= n) and upper
  }

  // Total indexed read (mirrors every other collection's `at(_)`): raw value
  // on hit, the None singleton on miss — not part of map-and-set.md's/
  // tuple-and-range.md's enumerated Range selector table, but a direct,
  // zero-floor-cost derivation the U-CORE-5 conformance harness (every
  // collection instantiates it) needs (U-COLLTYPES plan.md §7).
  at(i) {
    return (i >= 0 and (i < self.size)).ifTrue({ self.rawStart + i }, ifFalse: { None })
  }

  // Generates start, start+1, … up to the bound — no allocation (RG-2).
  each(f) {
    var i = 0
    while (i < self.size) {
      f.call(self.rawStart + i)
      i = i + 1
    }
  }

  // The explicit materialization escape hatch (RG-2).
  toList {
    var result = List.new()
    self.each { x => result.add(x) }
    return result
  }

  // Cursor iteration protocol (ADR-0035 §1) — identical shape to List's;
  // `iteratorValue` is `start + cursor`, never a materialized index into an
  // element buffer (there is none).
  iterate(cursor) {
    let next = cursor.map { c => c + 1 }.unwrapOr(0)
    return (next < self.size).ifTrue { next }
  }

  iteratorValue(cursor) => self.rawStart + cursor

  // Structural equality over the normalized bound fields (start/end/
  // inclusive) — not the generated sequence, which would defeat laziness for
  // a large range. Guarded by isA(Range).
  ==(other) {
    if (other.isA(Range)) {
      var sameStart = (self.rawStart == other.rawStart)
      var sameEnd = (self.rawEnd == other.rawEnd)
      var sameBound = (self.rawInclusive == other.rawInclusive)
      return sameStart and sameEnd and sameBound
    } else {
      return false
    }
  }

  !=(other) {
    return !(self == other)
  }

  // Value hash (immutable ⇒ hashable, Q5): a .ph fold over the three bound
  // fields' own `hash` — zero new floor, consistent with == by construction.
  hash {
    var acc = 17
    acc = (acc * 31 + self.rawStart.hash) % 999999937
    acc = (acc * 31 + self.rawEnd.hash) % 999999937
    acc = (acc * 31 + self.rawInclusive.hash) % 999999937
    return acc
  }
}

class System {
  static print() {
    // Native print function
  }
}

// `Future` (U-FUTURE Slice A; concurrency.md §2; ADR-0030 §1): a settle-once
// state machine over a fulfilled/rejected result. Slice A ships the
// **scheduler-free** half of the spec surface only — `value(_)`/`error(_)`
// construct an already-settled future, `isReady`/`value` read it, and
// `then`/`map`/`catch` fire synchronously because a Slice-A future is
// *always* already settled by the time `.ph` code can observe it. This is a
// **plain `InstanceObject`** (concurrency.md §2 "Implementation" ¶1) — zero
// new floor, zero native code, no `Fiber` dependency (a settled future never
// suspends). `async(_)`/`await` and the pending→settle drain need a native
// ready-queue (`System.schedule(_)`) and `Fiber#isDone`, neither of which is
// landed; that is Slice B, gated on DEC-FUT-SCHED
// (`docs/forge/units/U-FUTURE/plan.md` §9) — deliberately NOT built here.
//
// State lives in three private fields (plan §6.1): `_state` (one of the
// strings `"pending"`, `"fulfilled"`, `"rejected"`), `_value` (the settled
// value or the captured `Error`), and `_waiters` (a `List`, always empty in
// Slice A — no suspender ever registers on it; kept as a field now so Slice
// B's pending→settle drain is an additive change to this same layout, not a
// field-layout break).
class Future {
  // Builds an already-`fulfilled` future wrapping `v` (concurrency.md §2
  // `construct value(_)`). Goes through the pending→`settleValue` path
  // rather than setting `_state`/`_value` directly so construction and
  // post-construction settlement share one settle-once code path.
  construct value(v) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleValue(v)
  }

  // Builds an already-`rejected` future wrapping `e` (concurrency.md §2
  // `construct error(_)`); see `value(_)` for why this routes
  // through `settleError` instead of assigning state directly.
  construct error(e) {
    _state = "pending"
    _value = None
    _waiters = List.new()
    self.settleError(e)
  }

  // `true` once `self` has settled (`fulfilled` or `rejected`); `false`
  // while `pending`. Never suspends (C-FUT-8) — Slice A has no suspension
  // mechanism to trigger in the first place.
  isReady => _state != "pending"

  // Settles `self` as `fulfilled` with `v`, unless already settled (settle-
  // once, C-FUT-3): a `self.isReady` receiver is a no-op that returns `self`
  // unchanged, so a second `settleValue`/`settleError` can never clobber the
  // first result. Returns `self` either way so callers can chain.
  settleValue(v) {
    if (self.isReady) {
      return self
    } else {
      _state = "fulfilled"
      _value = v
      return self
    }
  }

  // Settles `self` as `rejected` with `e` (an `Error`), unless already
  // settled — the rejection sibling of `settleValue(_)`; see it for
  // the settle-once contract (C-FUT-3).
  settleError(e) {
    if (self.isReady) {
      return self
    } else {
      _state = "rejected"
      _value = e
      return self
    }
  }

  // The settled value as an `Option` (concurrency.md §2): `Some(v)` once
  // `fulfilled`, `None` while `pending` or once `rejected` (the rejection
  // reason is reached via `catch(_)`/`then(_)`, not `value`). Never
  // suspends (C-FUT-8) — a plain field read, no `Fiber` involved.
  value => (_state == "fulfilled").ifTrue({ Some.new(_value) }, ifFalse: { None })

  // Registers a continuation on the settled/fulfilled path (concurrency.md
  // §2 `then(_)`). Slice A receivers are always already settled, so this
  // fires synchronously: on `fulfilled`, calls `f` with the value and wraps
  // its result in a fresh fulfilled future (`Future.value(f.call(_value))`,
  // plan §6.2); on `rejected`, the rejection propagates unchanged (`self`)
  // without invoking `f`. A `pending` receiver can only occur under Slice
  // B's scheduler (no Slice-A constructor ever produces one) — raises
  // rather than silently hanging, since Slice A has no drain to fire the
  // continuation later.
  then(f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.value(f.call(_value))
      } else {
        return self
      }
    } else {
      return Error.new().raise()
    }
  }

  // `then(_)` restricted to the fulfilled path (concurrency.md §2 `map(_)`):
  // identical settled-synchronous behavior to `then(_)` in Slice A
  // — both fire `f` only on `fulfilled` and propagate `rejected` unchanged —
  // kept as a distinct selector because the spec keeps them distinct (§4:
  // "do not introduce aliases") and Slice B's suspending forms diverge.
  map(f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.value(f.call(_value))
      } else {
        return self
      }
    } else {
      return Error.new().raise()
    }
  }

  // Registers an error handler on the rejected path (concurrency.md §2
  // `catch(_)`): on `rejected`, calls `f` with the captured `Error` and
  // wraps its result in a fresh **fulfilled** future — `catch` recovers, it
  // does not re-reject (`Future.value(f.call(_value))`, plan §6.2); on
  // `fulfilled`, passes through unchanged (`self`) without invoking `f`. A
  // `pending` receiver is Slice B territory; see `then(_)` for why
  // this raises rather than hangs.
  catch(f) {
    if (self.isReady) {
      if (_state == "rejected") {
        return Future.value(f.call(_value))
      } else {
        return self
      }
    } else {
      return Error.new().raise()
    }
  }
}

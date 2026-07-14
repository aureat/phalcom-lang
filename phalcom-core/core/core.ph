class Object {
  // Is-kind-of test: true iff `cls` is the receiver's class or an ancestor of
  // it (object-model.md §8, is-tests.md — U-IS). Derived purely over the floor
  // — class/==/superclass — so it needs no native primitive (ADR-0019/0023).
  // The superclass chain is a run of class objects terminating in the `None`
  // singleton (class_superclass returns `None` at the root), so the walk
  // stops on `c == None`. The `ifTrue` result is in pop (statement) position,
  // so U-CORE-2's Some-lift is elided; the body neither reads nor depends on
  // `ifTrue`'s return shape.
  //
  // No RHS-is-a-class guard: a non-class `cls` never equals any `c` in the
  // chain, so the walk naturally falls through to `false` (is-tests.md I-4,
  // ratified `false`). Do not add a `cls.isA(...)`-style guard here — it
  // would recurse through the `isA` alias below forever, and it would target
  // `Behavior`, which is not bootstrapped in this codebase (ADR-0003 designs
  // it; core.ph has only `Object`/`Class`/`Metaclass`).
  is(cls) {
    var c = self.class
    while (c != None) {
      (c == cls).ifTrue { return true }
      c = c.superclass
    }
    return false
  }

  // Exact test: true iff `cls` is the receiver's *live, direct* class —
  // no superclass walk. Backs the `x is! T` surface (is-tests.md).
  isExactly(cls) => self.class == cls

  // Back-compat alias (U-CORE-1) — `is(_)` is now the primary kind-of test;
  // `isA` is retained so existing internal (`List#==` etc.) and user callers
  // keep working unchanged.
  isA(cls) => self.is(cls)
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

// Design-by-Contract failure classes (U-ANNOT-CONTRACTS,
// `docs/spec/v0.2/experimental/annotations-contracts.md`). Each is raised by
// the woven `@requires`/`@ensures`/`@invariant` check emitted by
// `phalcom-core/src/compiler/attributes.rs`'s `build_check_stmt` — zero Rust,
// zero new primitive, ordinary `Error` subclasses that inherit `message`/
// `raise` from the `_message`-slot machinery above.
class PreconditionError extends Error {}

class PostconditionError extends Error {}

class InvariantError extends Error {}

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
// primitives (length_/at_/set_/push_, plus native `new()`). This
// skeleton reopens that bootstrapped row to define the public protocol over
// those primitives (ADR-0019's "hybrid: native primitives, self-defined
// control"). `toString` is ALSO a native primitive this unit, not defined
// here — see the U-LIST return contract for why (element-value stringification
// is blocked on U-CORE-4; DEFERRED.md #19). U-STD (catalog-delta §2.4;
// DEFERRED.md #18/#20/#25) discharges the deferral for the combinator layer:
// `map`/`reduce`/`filter`/`includes`/`isEmpty` and the `at(_:put:)` wrapper
// over `set_` now live below, all pure `.ph` over the floor. Only
// **list-literal syntax** `[a, b, c]` remains deferred (it needs a new ADR +
// parser work; DEFERRED.md #6) — do not add that here.

class Iterable {
  // Generic index-cursor walk over `self.size` (ADR-0048 §1/§3). A subclass whose
  // cursor is not a 0..size index (none in-kernel today) overrides this.
  iterate(cursor) {
    let next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + 1 })
    return (next < self.size).ifTrue({ next }, ifFalse: { None })
  }

  each(f) {
    for (x in self) {
      f.call(x)
    }
  }

  // map/filter/reduce/includes walk `iterate`/`iteratorValue` DIRECTLY, not
  // `self.each` — `Map` overrides `each` with an incompatible 2-arity (k, v)
  // selector (DEC-CT-E); routing these through `self.each` would silently call a
  // 1-arity block with 2 arguments the moment `Map` inherits them. See the rubric.
  // U-SEQ DEC-SEQ-A branch (A): `map` is redefined lazy (was eager List-returning under U-ITERABLE —
  // BREAKING, see plan.md §8 migration-audit gate). `filter` is NOT touched (stays the eager U-ITERABLE
  // selector); `where` is the new Wren-parity lazy filter. `.toList` is the materializer for all of them.
  map(f) {
    return MapView.new(self, f)
  }

  filter(pred) {
    var result = List.new()
    var c = self.iterate(None)
    while (c != None) {
      var x = self.iteratorValue(c)
      pred.call(x).ifTrue({ result.add(x) }, ifFalse: { None })
      c = self.iterate(c)
    }
    return result
  }

  reduce(init, f) {
    var acc = init
    var c = self.iterate(None)
    while (c != None) {
      acc = f.call(acc, self.iteratorValue(c))
      c = self.iterate(c)
    }
    return acc
  }

  includes(x) {
    var found = false
    var c = self.iterate(None)
    while (c != None) {
      (self.iteratorValue(c) == x).ifTrue({ found = true }, ifFalse: { None })
      c = self.iterate(c)
    }
    return found
  }

  isEmpty => self.size == 0

  // U-SEQ (iteration.md §5 extension, wren_core.wren Sequence L7-119 precedent): combinator breadth
  // Phalcom lacked. All written over `for (x in self)`, never `self.each { }` (Map's `each(f)` is 2-arg —
  // see plan.md §3.1) and never index math. Zero new floor primitives.

  all(f) {
    for (x in self) {
      f.call(x).ifFalse { return false }
    }
    return true
  }

  any(f) {
    for (x in self) {
      f.call(x).ifTrue { return true }
    }
    return false
  }

  count {
    var n = 0
    for (x in self) { n = n + 1 }
    return n
  }

  count(f) {
    var n = 0
    for (x in self) { f.call(x).ifTrue { n = n + 1 } }
    return n
  }

  find(f) {
    for (x in self) {
      f.call(x).ifTrue { return Some.new(x) }
    }
    return None
  }

  join => self.join("")

  join(sep) {
    var first = true
    var result = ""
    for (x in self) {
      first.ifFalse { result = result + sep }
      first = false
      result = result + x.toString
    }
    return result
  }

  toList {
    var result = List.new()
    for (x in self) { result.add(x) }
    return result
  }

  where(pred) {
    return WhereView.new(self, pred)
  }

  skip(n) {
    return SkipView.new(self, n)
  }

  take(n) {
    return TakeView.new(self, n)
  }
}

class List {
  size => self.length_

  at(i) {
    return self.at_(i)
  }

  add(v) {
    self.push_(v)
    return self
  }

  // U-STD item 4 (U-ITER-FIX plan §"Not in this unit", DEC-ITER-A resolved):
  // drives the cursor protocol (`iterate(_)`/`iteratorValue(_)`, ADR-0035 §1)
  // rather than a raw `size`/`at(_)` index walk. `for (x in self)` compiles
  // to the same `Invoke`-only `iterate`/`iteratorValue`/`isSome` loop as any
  // user iterable (spec §3.1) — no `block_call`, no index math — so `each`
  // (and everything below built over it: `map`/`filter`/`reduce`/`includes`)
  // is protocol-driven behavior-preservingly.
  // Given a live cursor, yields the element there (ADR-0035 §1,
  // iteration.md §1). Only ever called with an in-range index, so it defers to
  // `at(_)` directly.
  iteratorValue(cursor) => self.at(cursor)

  // U-STD (DEFERRED.md #18): the public `.ph` wrapper over the `set_(_,_)`
  // floor primitive — writes `put` at index `i` and returns `self` so writes
  // chain (mirrors `add`). Selector `at(_:put:)` matches `set_`'s 2 args;
  // the labeled parameter is named `put` (label == name, parser convention).
  at(i, put:) {
    self.set_(i, put)
    return self
  }

  // U-INDEX (ADR-0060): `[]` is its own dedicated, user-overridable
  // selector — not `at`'s call-site sugar — so `List` must opt in
  // explicitly with a thin delegation, same as any other collection
  // author would. `xs[i]` sends `[_]`; `xs[i] = v` sends `[_,put]`.
  [i] { return self.at(i) }

  [i, put:] { return self.at(i, put: put) }

  // U-CORE-5 (decisions.md Q5, R-INV-5.3 E1-E5): structural equality —
  // element-wise, order-sensitive, via each element's own `==`. Guarded by
  // `isA(List)` so a non-List `other` is simply unequal (E2), never a dNU.
  // Derived entirely over the floor (`size`/`at`/`isA`/`while`/`and`/`not`) —
  // no new native primitive (ADR-0019 unchanged). `and`/`not` are the
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
    return not (self == other)
  }
}

// Kernel Map/Set (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 1): native
// insertion-ordered hash collections — Object::Map/Object::Set, sharing the
// MapObject backing struct (DEC-CT-B) but with distinct native-primitive
// bindings and distinct classes. This skeleton reopens the bootstrapped rows
// to define the public protocol over the native floor (ADR-0019's "hybrid: native
// primitives, self-defined control"). Both are MUTABLE, so neither installs a
// `hash` override — they inherit Object#hash (identity), so per Q5
// (decisions.md, collection-protocol.md law 4) neither is a valid Map/Set key;
// `put_`/`add_` enforce this (DEC-CT-C) by rejecting a mutable-collection
// key (List/Map/Set) with a raised Error.

class Map {
  size => self.size_

  // Total lookup: Some-shaped by convention (raw value on hit, the None
  // singleton on miss — mirrors List#at's at_ shape, U-CORE-2/ADR-0021).
  at(k) => self.get_(k)

  // Insert/overwrite; returns self so `put` calls chain.
  at(k, put:) {
    self.put_(k, put)
    return self
  }

  // U-INDEX (ADR-0060): `[]` is its own dedicated, user-overridable
  // selector — a thin delegation to `at`, same shape as `List`'s.
  // `m[k]` sends `[_]`; `m[k] = v` sends `[_,put]`.
  [k] { return self.at(k) }

  [k, put:] { return self.at(k, put: put) }

  includes(k) => self.has_(k)

  // Idempotent: removing an absent key is a no-op, still returns self.
  remove(k) {
    self.remove_(k)
    return self
  }

  // A fresh List of keys, in iteration (insertion) order.
  keys {
    var result = List.new()
    var i = 0
    while (i < self.size) {
      result.add(self.keyAt_(i))
      i = i + 1
    }
    return result
  }

  // A fresh List of values, in iteration (insertion) order.
  values {
    var result = List.new()
    var i = 0
    while (i < self.size) {
      result.add(self.valueAt_(i))
      i = i + 1
    }
    return result
  }

  // 2-arg block `{ k, v => ... }` per entry, in iteration order.
  each(f) {
    var i = 0
    while (i < self.size) {
      f.call(self.keyAt_(i), self.valueAt_(i))
      i = i + 1
    }
  }

  // DEC-CT-E: the cursor value `iteratorValue` yields is the KEY (both Map and Set yield keys);
  // `each(_)` above remains the 2-arg entry form for Map.

  iteratorValue(cursor) => self.keyAt_(cursor)

  // Structural equality: same key set, pairwise-== values (order-independent
  // over keys — `includes`/`at` do the membership + value work, not raw
  // index comparison). Guarded by `isA(Map)` so a non-Map is simply unequal.
  ==(other) {
    if (other.isA(Map)) {
      var same = (self.size == other.size)
      var i = 0
      while (same and (i < self.size)) {
        var k = self.keyAt_(i)
        same = other.includes(k) and (self.valueAt_(i) == other.at(k))
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
    return not (self == other)
  }
}

class Set {
  size => self.size_

  add(v) {
    self.add_(v)
    return self
  }

  includes(v) => self.has_(v)

  remove(v) {
    self.remove_(v)
    return self
  }

  // Positional read in insertion order — not in the map-and-set.md selector
  // table, but a direct, zero-floor-cost derivation over at_ that the
  // U-CORE-5 conformance harness (collection-protocol.md §2) needs, and a
  // natural extension of the sequence protocol every collection instantiates.
  at(i) => self.at_(i)



  iteratorValue(cursor) => self.at_(cursor)

  // Structural equality: same members, order-independent. Same-size plus
  // "every element of self is in other" is sufficient since neither set
  // holds duplicates (add_ is idempotent).
  ==(other) {
    if (other.isA(Set)) {
      var same = (self.size == other.size)
      var i = 0
      while (same and (i < self.size)) {
        same = other.includes(self.at_(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(other) {
    return not (self == other)
  }
}

// Kernel Tuple (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 2): a native fixed
// arity immutable slice — Object::Tuple, mirroring List's shape but with NO
// mutation selector (immutability is structural, TupleObject's Box<[Value]>).
// This is also the (a, b) literal's construction target (already emitted by
// U-COLL's parser as `Tuple.fromList(List.new().add(a).add(b))`), so the
// literal graduates automatically the moment this class exists.

class Tuple {
  size => self.size_

  at(i) => self.at_(i)

  // U-INDEX (ADR-0060): read-only — `tup[i]` sends `[_]`. No `[_,put]`
  // wrapper: `Tuple` is immutable (no `at(_,put:)` either), so `tup[i] = v`
  // correctly `doesNotUnderstand` rather than raising a bespoke error.
  [i] { return self.at(i) }

  iteratorValue(cursor) => self.at_(cursor)

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
    return not (self == other)
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
// numeric interval — Object::Range, three native bound-field getters and NOTHING
// else on the floor. `each`/`toList`/`size`/`includes`/`first`/`last` are all
// derived here over `start_`/`end_`/`inclusive_` + Number arithmetic —
// `each` GENERATES elements, never allocates (RG-2 laziness), so
// `Range.new(1, 1000000, true)` stays O(1) to construct and each step of
// `each` is O(1). Bound convention (RG-1): `a..b` inclusive (`inclusive_`
// true), `a...b` exclusive (false) — the reserved `..`/`...` literal's
// committed meaning, honored now via the explicit constructor.

class Range {
  first => self.start_

  last {
    return self.inclusive_.ifTrue({ self.end_ }, ifFalse: { self.end_ - 1 })
  }

  size {
    var n = self.inclusive_.ifTrue({ self.end_ - self.start_ + 1 }, ifFalse: { self.end_ - self.start_ })
    return (n < 0).ifTrue({ 0 }, ifFalse: { n })
  }

  includes(n) {
    var upper = self.inclusive_.ifTrue({ n <= self.end_ }, ifFalse: { n < self.end_ })
    return (self.start_ <= n) and upper
  }

  // Total indexed read (mirrors every other collection's `at(_)`): raw value
  // on hit, the None singleton on miss — not part of map-and-set.md's/
  // tuple-and-range.md's enumerated Range selector table, but a direct,
  // zero-floor-cost derivation the U-CORE-5 conformance harness (every
  // collection instantiates it) needs (U-COLLTYPES plan.md §7).
  at(i) {
    return (i >= 0 and (i < self.size)).ifTrue({ self.start_ + i }, ifFalse: { None })
  }

  // Generates start, start+1, … up to the bound — no allocation (RG-2).


  // The explicit materialization escape hatch (RG-2).
  toList {
    var result = List.new()
    self.each { x => result.add(x) }
    return result
  }

  // Cursor iteration protocol (ADR-0035 §1) — identical shape to List's;
  // `iteratorValue` is `start + cursor`, never a materialized index into an
  // element buffer (there is none).


  iteratorValue(cursor) => self.start_ + cursor

  // Structural equality over the normalized bound fields (start/end/
  // inclusive) — not the generated sequence, which would defeat laziness for
  // a large range. Guarded by isA(Range).
  ==(other) {
    if (other.isA(Range)) {
      var sameStart = (self.start_ == other.start_)
      var sameEnd = (self.end_ == other.end_)
      var sameBound = (self.inclusive_ == other.inclusive_)
      return sameStart and sameEnd and sameBound
    } else {
      return false
    }
  }

  !=(other) {
    return not (self == other)
  }

  // Value hash (immutable ⇒ hashable, Q5): a .ph fold over the three bound
  // fields' own `hash` — zero new floor, consistent with == by construction.
  hash {
    var acc = 17
    acc = (acc * 31 + self.start_.hash) % 999999937
    acc = (acc * 31 + self.end_.hash) % 999999937
    acc = (acc * 31 + self.inclusive_.hash) % 999999937
    return acc
  }
}

// Lazy view classes (wren_core.wren MapSequence/WhereSequence/SkipSequence L121-152, 168-182), ported
// to Phalcom's bare-cursor protocol (post-U-ITERABLE: `iterate` returns the raw next cursor, or the
// `None` singleton at exhaustion — never Some-wrapped). `extends Iterable` so combinators, `for`,
// and every other view work on a view for free.

class MapView extends Iterable {
  construct new(source, f) {
    _source = source
    _f = f
  }
  iterate(cursor) => _source.iterate(cursor)
  iteratorValue(cursor) => _f.call(_source.iteratorValue(cursor))
}

class WhereView extends Iterable {
  construct new(source, pred) {
    _source = source
    _pred = pred
  }
  iterate(cursor) {
    var cur = cursor
    while (true) {
      cur = _source.iterate(cur)
      (cur == None).ifTrue { return None }
      _pred.call(_source.iteratorValue(cur)).ifTrue { return cur }
    }
  }
  iteratorValue(cursor) => _source.iteratorValue(cursor)
}

class SkipView extends Iterable {
  construct new(source, count) {
    (count.isA(Number) and (count >= 0)).ifFalse {
      throw Error("skip: count must be a non-negative Number")
    }
    _source = source
    _count = count
  }
  iterate(cursor) {
    (cursor != None).ifTrue { return _source.iterate(cursor) }
    var cur = _source.iterate(None)
    var n = _count
    while ((n > 0) and (cur != None)) {
      cur = _source.iterate(cur)
      n = n - 1
    }
    return cur
  }
  iteratorValue(cursor) => _source.iteratorValue(cursor)
}

class TakeView extends Iterable {
  construct new(source, count) {
    (count.isA(Number) and (count >= 0)).ifFalse {
      throw Error("take: count must be a non-negative Number")
    }
    _source = source
    _count = count
  }
  iterate(cursor) {
    var srcCursor = None
    var taken = 0
    (cursor != None).ifTrue {
      srcCursor = cursor.at(0)
      taken = cursor.at(1)
    }
    ((taken + 1) > _count).ifTrue { return None }
    var next = _source.iterate(srcCursor)
    (next == None).ifTrue { return None }
    return (next, taken + 1)
  }
  iteratorValue(cursor) => _source.iteratorValue(cursor.at(0))
}

class System {
  static print() {
    // Native print function
  }

  // U-SCHED: the `.ph`-callable counterpart to `VM::run`'s native
  // root-drive belt-and-suspenders pump (`vm/dispatch.rs`) — pumps
  // `System.nextScheduled` to exhaustion, `try()`-resuming each queued
  // fiber (capture-not-propagate, so one scheduled task's uncaught raise
  // cannot abort another) — including any fiber a running scheduled fiber
  // itself schedules mid-drain, since `nextScheduled` is re-read every
  // iteration. Deliberately does **not** unwrap via `.match(some:none:)`
  // (which runs its arm through `Block#call`'s native re-entrant
  // `run_until`, forbidding a fiber switch underneath, ADR-0030 §4): `f.try()`
  // must run at this method's own top level, not nested inside a block a
  // native primitive is driving, so the receiver is unwrapped via
  // `unwrapOr(_)` into a plain local first, and `try()` sent as its own
  // statement.
  static runScheduled() {
    var next = System.nextScheduled
    while (next.isSome) {
      var f = next.unwrapOr(None)
      f.try()
      next = System.nextScheduled
    }
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
  // Builds a pending future (U-FUTURE Slice B).
  construct new() {
    _state = "pending"
    _value = None
    _waiters = List.new()
  }

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
  // while `pending`.
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
      self.drain()
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
      self.drain()
      return self
    }
  }

  // Reschedules all waiters once settled.
  drain() {
    _waiters.each { w =>
      System.schedule(w)
    }
    _waiters = List.new()
  }

  // The settled value as an `Option` (concurrency.md §2): `Some(v)` once
  // `fulfilled`, `None` while `pending` or once `rejected` (the rejection
  // reason is reached via `catch(_)`/`then(_)`, not `value`).
  value => (_state == "fulfilled").ifTrue({ Some.new(_value) }, ifFalse: { None })

  // Suspends the current fiber until settled (U-FUTURE Slice B).
  // If called on the root fiber, degrades to "pump until settled" (pumps
  // System.runScheduled) because the root fiber cannot yield.
  await {
    if (not self.isReady) {
      _waiters.add(Fiber.current)
      let res = { Fiber.yield(None) }.attempt()
      if (res.isErr) {
        let err = res.unwrapErr
        if (err.isA(CannotYieldAcrossNativeFrame)) {
          return err.raise()
        }
        // Yield failed because we are on the root fiber!
        _waiters = _waiters.filter { w => w != Fiber.current }
        while (not self.isReady) {
          System.runScheduled()
        }
      }
    }
    if (_state == "rejected") {
      return _value.raise()
    }
    return _value
  }

  // Runs `action` on a fresh fiber and settles the returned future with its
  // result (or captured error if it fails).
  static async(action) {
    let f = Future.new()
    let driver = Fiber.new {
      let fib = Fiber.new(action)
      let res = fib.try()
      if (fib.error.isSome) {
        f.settleError(fib.error.unwrapOr(None))
      } else {
        f.settleValue(res)
      }
    }
    System.schedule(driver)
    return f
  }

  // Registers a continuation on the settled/fulfilled path (concurrency.md
  // §2 `then(_)`). If pending, registers a continuation that will settle
  // the returned future when this receiver settles.
  then(f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.value(f.call(_value))
      } else {
        return self
      }
    } else {
      let f_next = Future.new()
      _waiters.add({
        if (_state == "fulfilled") {
          let fib = Fiber.new({ f.call(_value) })
          let res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            f_next.settleValue(res)
          }
        } else {
          f_next.settleError(_value)
        }
      })
      return f_next
    }
  }

  // `then(_)` restricted to the fulfilled path (concurrency.md §2 `map(_)`).
  map(f) {
    if (self.isReady) {
      if (_state == "fulfilled") {
        return Future.value(f.call(_value))
      } else {
        return self
      }
    } else {
      let f_next = Future.new()
      _waiters.add({
        if (_state == "fulfilled") {
          let fib = Fiber.new({ f.call(_value) })
          let res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            f_next.settleValue(res)
          }
        } else {
          f_next.settleError(_value)
        }
      })
      return f_next
    }
  }

  // Registers an error handler on the rejected path (concurrency.md §2
  // `catch(_)`).
  catch(f) {
    if (self.isReady) {
      if (_state == "rejected") {
        return Future.value(f.call(_value))
      } else {
        return self
      }
    } else {
      let f_next = Future.new()
      _waiters.add({
        if (_state == "rejected") {
          let fib = Fiber.new({ f.call(_value) })
          let res = fib.try()
          if (fib.error.isSome) {
            f_next.settleError(fib.error.unwrapOr(None))
          } else {
            f_next.settleValue(res)
          }
        } else {
          f_next.settleValue(_value)
        }
      })
      return f_next
    }
  }
}

// `Tracer` (decorators-dispatch-observability.md D-2, ratified 2026-07-13):
// the pluggable observability sink `@traced`'s `sink:` argument targets —
// duck-typed (`enter`/`exit`/`threw`), so any object answering this protocol
// drops in. `Tracer.stdout` is the shipped default, routing through
// `System.print` (Phalcom has no dedicated logging primitive, system.md).
// Ships standalone: `@traced` itself is Install/Dispatch/Runtime-tier
// decorator-mechanism work, not yet built (see PLAN-DECORATORS.md), so this
// class has no caller yet — the sink protocol is ready when it lands.
class Tracer {
  static stdout => Tracer.new()

  enter(name, args) { System.print("-> " + name.toString + " " + args.toString) }
  exit(name, result, elapsed) { System.print("<- " + name.toString + " = " + result.toString) }
  threw(name, err) { System.print("!! " + name.toString + " threw " + err.toString) }
}

// `OffBehavior` (decorators-dispatch-observability.md D-3, ratified
// 2026-07-13): `@featureFlag`'s off-path — what a gated call does when its
// flag reads false. `applyTo(inv)` (invoked by the not-yet-built
// `@featureFlag` Runtime interceptor's `aroundSend` hook against a
// not-yet-defined `inv` envelope) is deliberately NOT implemented here —
// this class ships now as pure value semantics only; wiring it to a real
// interception envelope is Install/Dispatch/Runtime mechanism work.
class OffBehavior {
  static raise => OffBehavior.new("raise", None)
  static fallback(sel) => OffBehavior.new("fallback", Some.new(sel))
  static skip(value) => OffBehavior.new("skip", Some.new(value))

  construct new(kind, payload) { _kind = kind; _payload = payload }

  kind => _kind
  payload => _payload
}

// `Backoff` (decorators-behavioral.md B-2, ratified 2026-07-13): `@retry`'s
// backoff strategy. `.none` is fully usable today — no suspension needed,
// matching `@retry`'s own default. `.fixed(ms)`/`.exponential(base:,max:)`
// need a real suspending wait between attempts, which needs `System.sleep(_)`
// — explicitly **not landed** (system.md: "still open", gated on a
// timer-completion-source follow-on unit, itself gated on U-SCHED's ready-
// queue/timer split per open-questions.md §15). Rather than silently busy-
// waiting or lying about elapsed time, `.fixed`/`.exponential`'s
// `waitBefore` raises until that primitive exists — a real gap, not a stub
// pretending to work.
class Backoff {
  static none => Backoff.new("none", 0, 0)
  static fixed(ms) => Backoff.new("fixed", ms, 0)
  static exponential(base:, max:) => Backoff.new("exponential", base, max)

  construct new(kind, a, b) { _kind = kind; _a = a; _b = b }

  waitBefore(attempt) {
    if (_kind == "none") {
      return None
    } else {
      return Error.new("Backoff." + _kind + " needs System.sleep(_), not yet landed (system.md)").raise()
    }
  }
}

// `Attribute`/`On`/`Tier` (M-ATTR-ROOT, attribute-classes.md §"Decision"/
// §"`@On`"/§"Bootstrap"): the reified-descriptor root every attribute class
// extends, the `@On` builtin attribute carrying legality + declared tier, and
// the tier marker classes. `@Name(args)` desugars, at the enclosing class's
// definition time, to `Name.new(args)` + `artifact.__attach(_a)`
// (`compiler::attributes`/`compiler::lib::class_decl`) — the constructed
// instance is retained on the decorated artifact's native `_attributes` store
// (`ClassObject`/`MethodObject`/`ModuleObject`, `primitive/attribute.rs`),
// reflectable via `Behavior#attributes`/`Method#attributes` below.
//
// **Forced deviation (positional-only args, filed to `docs/forge/DEFERRED.md`):**
// attribute-arg lists are positional-only — `parser.rs`'s
// `parse_attribute_arg_list` has no label grammar — so `On`'s own
// constructors are positional (`On.new(target)` / `On.new(target, tier)`),
// not the spec's labeled `tier:`/`inherited:` form. `inherited:` is dropped
// entirely for the same reason (v0.3 follow-on once labeled attribute args
// exist). A single `target` (not a list) is stored, since the parser also has
// no list-literal syntax yet (`core.ph` L306) to build a multi-target list at
// a use site — multi-target `@On` is deferred alongside labeled args.

// Root. Every attribute extends this — usage (retention, `resolves_to_
// attribute_class`'s `extends` chain walk) is fixed in
// `compiler::attributes` at this root.
class Attribute {}

// Builtin attribute carrying legality + declared tier (A-1) — recursion
// bottoms out here: `On` is itself an `Attribute` subclass, so `@On(...)` on
// an `Attribute` subclass's own header is retained/reflectable like any other
// attribute. `tier` is `None` for passive metadata (no hook selector may be
// implemented — `attr.undeclared_hook`) or one of the `Tier` marker classes
// below (`Install`/`Dispatch`/`Runtime` — `Compile`/`Layout` are reserved for
// compiler-native hooks only, `attr.compile_tier_reserved`).
class On extends Attribute {
  var _targets
  var _tier

  construct new(target) { _targets = target; _tier = None }
  construct new(target, tier) { _targets = target; _tier = tier }

  targets => _targets
  tier => _tier
}

// The tier marker classes (attribute-classes.md: "same pattern Phalcom
// already uses for `Bool`'s `True`/`False`" — real singleton objects, not
// symbols; a bare class, used purely by identity, is the same pattern
// `True`/`False` already establish). `Compile`/`Layout` are reserved for
// compiler-native hooks only; `Install`/`Dispatch`/`Runtime` are the
// user-facing tiers (M-INSTALL/M-DISPATCH/M-RUNTIME, PLAN-DECORATORS.md).
class Tier {}
class Compile extends Tier {}
class Layout extends Tier {}
class Install extends Tier {}
class Dispatch extends Tier {}
class Runtime extends Tier {}

// `Behavior#attributes`/`#attributesOfType(_)` (object-model.md's metaclass
// tower superclass of `Class`+`Metaclass`) — the reflection surface over the
// native `_attributes` store every class object carries (M-ATTR-ROOT).
// Method-only reopen (no new fields) — safe on a bootstrap class (a
// reopen-with-fields would trip read-before-write).
class Behavior {
  attributes => self.__attributes
  attributesOfType(cls) => self.__attributes.filter { a => a.isA(cls) }
}

// `Method#attributes`/`#attributesOfType(_)` — the same reflection surface
// as `Behavior` above, for the reified `Method` object a class's method
// dictionary holds.
class Method {
  attributes => self.__attributes
  attributesOfType(cls) => self.__attributes.filter { a => a.isA(cls) }
}

class Object {}

class Class {}
 
class Metaclass {}

class Number {}

class String {}

class Bool {}

class Symbol {}

// Absence is an Option (ADR-0007), not a surface `nil`. `Option` is abstract;
// `Some` wraps one value and `None` is a single shared singleton. These are
// bootstrapped in Rust (universe.rs): the classes, the `Some(_)` construction
// primitive, and the `match(some:, none:)` eliminator. The skeletons below only
// *reopen* those bootstrapped rows so the class names are surface-visible.
//
// The rich combinator suite (`map`, `flatMap`, `filter`, `orElse`, `ifSome`,
// `unwrapOr`, …) is deliberately NOT defined here — it is U-STD's job, layered
// over `match` in Phalcom. Do not add combinator bodies to these skeletons.
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

class Option {}

class Some {}

// Kernel List (ADR-0020): a native array-backed heap object (ListObject),
// not an InstanceObject — bootstrapped in Rust (universe.rs) with five floor
// primitives (rawLength/rawAt/rawSet/rawPush, plus native `new()`). This
// skeleton reopens that bootstrapped row to define the public protocol over
// those primitives (ADR-0019's "hybrid: native primitives, self-defined
// control"). `toString` is ALSO a native primitive this unit, not defined
// here — see the U-LIST return contract for why. `rawSet` has no `.ph`
// wrapper yet (no `at(_:put:)` selector) — deliberately deferred to U-STD,
// along with `map`/`reduce`/`filter`/literal syntax. Do not add those bodies
// to this skeleton.

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
}

class System {
  static print() {
    // Native print function
  }
}

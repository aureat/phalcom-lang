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

class Option {}

class Some {}

class None {}

class System {
  static print() {
    // Native print function
  }
}
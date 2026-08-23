@native
class Object {

  @native
  name -> Dynamic

  @native
  class -> Dynamic

  @native
  class=(put value: Dynamic) -> Dynamic

  @native
  toString -> String

  @native
  hash -> Int

  @native
  ==(_ other: Dynamic) -> Bool

  @native
  !=(_ other: Dynamic) -> Bool

  @native
  ===(_ other: Dynamic) -> Bool

  @native
  matches(_ other: Dynamic) -> Bool

  @native
  understands(_ selector: Dynamic) -> Bool

  @native
  perform(_ selector: Dynamic, ***args: Dynamic) -> Dynamic

  @native
  respondsTo(_ selector: Dynamic) -> Bool

  @native
  doesNotUnderstand(_ message: Dynamic) -> Dynamic

  @native
  methodFor(_ selector: Dynamic) -> Dynamic

  @internal
  @native
  _$invariantEnter() -> Dynamic

  @internal
  @native
  _$invariantExit() -> Dynamic

  @internal
  @native
  _$attributes -> Dynamic

  @internal
  @native
  _$attach(_ attribute: Dynamic) -> Dynamic

  @internal
  @native
  _$freezeAttributes() -> Dynamic

  // Is-kind-of test: true iff `cls` is the receiver's class or an ancestor of
  // it (object-model.md §8, is-tests.md — U-IS). Derived purely over the floor
  // — class/==/superclass — so it needs no native primitive (ADR-0019/0023).
  // The superclass chain is a run of class objects terminating in the `None`
  // immediate `None` value (class_superclass returns `None` at the root), so the walk
  // stops on `c == None`. The `ifTrue` result is in pop (statement) position,
  // so U-CORE-2's Some-lift is elided; the body neither reads nor depends on
  // `ifTrue`'s return shape.
  //
  // No RHS-is-a-class guard: a non-class `cls` never equals any `c` in the
  // chain, so the walk naturally falls through to `false` (is-tests.md I-4,
  // ratified `false`). Do not add a `cls.is(...)`-style guard here — it
  // would recurse through the `isA` alias below forever, and it would target
  // `Behavior`, which is not bootstrapped in this codebase (ADR-0003 designs
  // it; core.ph has only `Object`/`Class`/`Metaclass`).
  is(_ cls) {
    let c = class
    while c != None {
      if c == cls { 
        return true 
      }
      c = c.superclass
    }
    false
  }

  // Exact test: true iff `cls` is the receiver's *live, direct* class —
  // no superclass walk. Backs the `x is! T` surface (is-tests.md).
  is!(_ cls) { class == cls }

  // Default ordering relations are derived from the bilateral compare
  // protocol. Numeric classes retain their specialized primitive methods;
  // other classes can implement only compare(_) and inherit this surface.
  <(_ other) { (self <=> other).kind === #less }

  <=(_ other) {
    let order = (self <=> other)

    (order.kind === #less) or (order.kind === #equal)
  }

  >(_ other) { (self <=> other).kind === #greater }

  >=(_ other) {
    let order = (self <=> other)
    
    (order.kind === #greater) or (order.kind === #equal)
  }
}

class Object {
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
  // ratified `false`). Do not add a `cls.isA(...)`-style guard here — it
  // would recurse through the `isA` alias below forever, and it would target
  // `Behavior`, which is not bootstrapped in this codebase (ADR-0003 designs
  // it; core.ph has only `Object`/`Class`/`Metaclass`).
  is(_ cls) {
    let c = self.class
    while (c != None) {
      (c == cls).ifTrue || { return true }
      c = c.superclass
    }
    return false
  }

  // Exact test: true iff `cls` is the receiver's *live, direct* class —
  // no superclass walk. Backs the `x is! T` surface (is-tests.md).
  isExactly(_ cls) { self.class == cls }

  // Back-compat alias (U-CORE-1) — `is(_)` is now the primary kind-of test;
  // `isA` is retained so existing internal (`List#==` etc.) and user callers
  // keep working unchanged.
  isA(_ cls) { self.is(cls) }
}

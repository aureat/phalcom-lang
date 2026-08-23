
// The throw -> value bridge (error-handling.md §5): runs `self` (0-arity),
// capturing a `throw` into `Err(e)`; success is `Ok(v)`. Pure `.ph` over
// `on(_)(_)` (U-ERR, ADR-0038) — no floor cost. Installed on the abstract
// `Function` root so both `Closure` and (reflectively) `Method` inherit it,
// mirroring how `call`/`on`/`ensure` are native on both.
@native
class Function is Object {
  // Explicit `()` (a method, not a getter — `attempt() { … }` vs `attempt {
  // …`) so the call-site selector encodes as `attempt()`, matching the
  // spec's `{ risky() }.attempt()` call form (error-handling.md §5) exactly.
  attempt() {
    return || { Ok.new(self.call()) }.on(Error) |e| { Err.new(e) }
  }
}

// Kernel List (ADR-0020): a native array-backed heap object (ListObject),
// not an InstanceObject — bootstrapped in Rust (universe.rs) with five floor
// implementation primitives (`_$length`/`_$at`/`_$set`/`_$push`, plus native `new()`). This
// skeleton reopens that bootstrapped row to define the public protocol over
// those primitives (ADR-0019's "hybrid: native primitives, self-defined
// control"). `toString` is ALSO a native primitive this unit, not defined
// here — see the U-LIST return contract for why (element-value stringification
// is blocked on U-CORE-4; DEFERRED.md #19). U-STD (catalog-delta §2.4;
// DEFERRED.md #18/#20/#25) discharges the deferral for the combinator layer:
// `map`/`reduce`/`filter`/`includes`/`isEmpty` and the `at(_:put:)` wrapper
// over `_$set` now live below, all pure `.ph` over the floor. Only
// **list-literal syntax** `[a, b, c]` remains deferred (it needs a new ADR +
// parser work; DEFERRED.md #6) — do not add that here.

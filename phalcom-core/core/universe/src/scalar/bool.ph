@native
class Bool is Object {
  @class
  @native
  new() -> Bool
  @class
  @native
  new(_ value: Dynamic) -> Bool
  @native
  and(_ other: Bool) -> Bool
  @native
  or(_ other: Bool) -> Bool
  @native
  not -> Bool
  @native
  ifTrue(_ then: Dynamic) -> Dynamic
  @native
  ifFalse(_ otherwise: Dynamic) -> Dynamic
  @native
  ifTrue(_ then: Dynamic, ifFalse: Dynamic) -> Dynamic
  @native
  hash -> Int
  // Display (U-CORE-4, R-INV-4.1): derived over the sacred `ifTrue(_,
  // ifFalse)` selector (proven syntax:
  // `control-flow/control_flow_send_equivalence.ph` L9). This `toString` is
  // NOT itself sacred (floor-census §5's `bool_sacred_pristine` tracks only
  // the six original selectors), so adding it does not trip the inliner
  // deopt.
  toString {
    // self.ifTrue { "true" } ifFalse: { "false" }
    return self.ifTrue(|| "true", ifFalse: || "false")
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
@native
class True is Bool {}

@native
class False is Bool {}

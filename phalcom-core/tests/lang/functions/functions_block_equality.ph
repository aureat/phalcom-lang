// area: functions
// spec: object-model.md; ADR-0006
// status: PASS
// Ported from Wren `test/core/function/equality.wren`: blocks are not
// structurally equal (each literal is a distinct closure allocation, even
// with identical source), unequal to any other type, but equal to
// themselves by identity — the default `Object#==` a `Block` inherits
// (nothing overrides it). Closures built from the same literal across
// loop iterations are still each their own allocation, so unequal.

// Not structurally equal.
System.print({ 123 } == { 123 })
System.print({ 123 } != { 123 })

// Not equal to other types.
System.print({ 123 } == 1)
System.print({ 123 } == false)
System.print({ 123 } == "fn 123")
System.print({ 123 } != 1)
System.print({ 123 } != false)
System.print({ 123 } != "fn 123")

// Equal by identity.
const f = { 123 }
System.print(f == f)
System.print(f != f)

// Closures for the same literal are not equal.
const fns = List.new()
let i = 0
while (i < 2) {
  fns.add({ 123 })
  i = i + 1
}
System.print(fns.at(0) == fns.at(1))

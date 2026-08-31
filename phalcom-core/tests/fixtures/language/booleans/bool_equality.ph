// area: booleans
// spec: values-and-absence.md; messages-and-selectors.md
// status: PASS
// Ported from Wren `test/core/bool/equality.wren`: `Bool#==`/`Bool#!=`
// compare by value between two `Bool`s, and are unconditionally `false`/
// `true` (respectively) against any other type — no coercion, no
// truthiness (ADR-0021).

System.print(true == true)
System.print(true == false)
System.print(false == true)
System.print(false == false)

// Not equal to other types.
System.print(true == 1)
System.print(false == 0)
System.print(true == "true")
System.print(false == "false")
System.print(false == "")

System.print(true != true)
System.print(true != false)
System.print(false != true)
System.print(false != false)

// Not equal to other types.
System.print(true != 1)
System.print(false != 0)
System.print(true != "true")
System.print(false != "false")
System.print(false != "")

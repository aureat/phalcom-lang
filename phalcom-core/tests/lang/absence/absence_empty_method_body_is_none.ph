// area: absence
// spec: values-and-absence.md; control-flow.md; ADR-0007; U6-plan.md §4
// status: PASS
// U6: a method whose body leaves no value on the stack (here an empty body)
// falls off its end and surfaces to the `None` singleton, not the receiver
// `self`. This is the non-inlined mirror of the empty-block cases; without the
// fix `compile_block` returned `self` and this printed `false`.

class C { m() { } }
System.print(C.new().m() == None)

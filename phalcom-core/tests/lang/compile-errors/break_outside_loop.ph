// C-ITER-7 (ADR-0035 §3, spec §6): a `break` with no enclosing loop is a
// compile error with a span at the keyword — never a runtime value.
System.print("unreachable")
break

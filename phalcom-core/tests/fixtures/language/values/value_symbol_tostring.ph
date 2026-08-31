// area: values
// spec: values-and-absence.md §1; U-CORE-4 (R-INV-4.1, BD-CORE4-2 Option A)
// status: PASS
// A label-free symbol's message (`symbol_tostring`) and print
// (`Value::to_string`'s `Symbol` arm) paths now agree on the `#{text}` form
// — byte-stable without U-LEX's `#…` literal syntax.

System.print(Symbol.new("foo").toString)

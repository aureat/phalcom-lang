// area: lexical
// spec: selectors.md §2
// status: PASS
// A bare `#name` name symbol prints as `#name`. Two independently interned
// occurrences of the same name compare equal — exercises both the lexer's
// name-symbol token and the U-LEX-HASH coupled `value_eq` fix
// (`(Value::Symbol, Value::Symbol)` content equality).

System.print(#move)
System.print(#move == #move)
System.print(#move == Symbol.new("move"))
System.print(#move == #size)

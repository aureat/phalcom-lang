// area: boolean tower
// spec: ADR-0004; U-CORE-2 (Option Some-lift)
// status: PASS
// ifTrue/ifFalse still return a well-formed Option after the class split:
// the inherited bool_if_true primitive Some-lifts through True/False.
System.print(true.ifTrue || { 42 }.isSome)
System.print(false.ifTrue || { 42 }.isNone)
System.print(true.ifTrue || { 42 }.unwrapOr(0))
System.print(false.ifFalse || { 7 }.isSome)

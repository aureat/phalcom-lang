// area: string
// spec: values-and-absence.md; control-flow.md §1
// status: NEGATIVE
// Wren precedent: test/core/string/not.wren (Wren: every string, including
// `""`, is truthy, so `!"s"` and `!""` both print `false`). Phalcom has no
// truthy-coercion model for arbitrary objects (values-and-absence.md;
// `ifTrue`/`ifTrue(_:ifFalse:)` require a real `Bool` receiver, never an
// implicit "truthy" cast) and `String` does not define `not` — `not expr`
// desugars to a `not` getter send (parser.rs `UnaryOp::Not`), so `not "s"` reaches
// `doesNotUnderstand` instead of Wren's `false`. This fixture pins the
// divergence: Wren's `!string` is a coercion; Phalcom's `not string` is a
// message the class must define, which `String` does not. (U-NEG: prefix `!`
// retired; `not` is the sole prefix-negation surface.)
System.print(not "s")

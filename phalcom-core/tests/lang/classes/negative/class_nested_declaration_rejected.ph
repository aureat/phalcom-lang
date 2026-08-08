// U-CLASSCLOSE §6, decision 0065 ruling 5: class declarations are module
// top-level only. A `class` nested inside a method body parses (so the
// error carries a real span) and is then rejected with
// `class.nested_declaration` — the ban is a syntax rule enforced by position,
// not a compile-time invariant.
class Outer make => {
        class Inner {}
        Inner.new()
    }
}

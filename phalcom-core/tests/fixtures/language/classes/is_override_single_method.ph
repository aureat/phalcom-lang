// area: classes
// spec: next/is-tests.md
// status: PASS
// U-IS: only two magic methods exist (`is(_)`/`is!(_)`) — negation is
// a compile-time `.not` wrap, not a selector. So a structural override of
// `is(cls)` alone governs both `is` and `is not` with consistent polarity,
// "for free": no `isNot` to separately override or get out of sync.

class Drawable {}

class Shape {
  is(_ cls) {
    (cls == Drawable).ifTrue || { return true }
    return super.is(cls)
  }
}

let s = Shape.new()
System.print(s is Drawable)
System.print(s is not Drawable)

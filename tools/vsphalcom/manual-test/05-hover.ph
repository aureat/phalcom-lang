// 05-hover.ph
//
// Manual steps:
//
// 1. Hover the bare word `throw` on any line below (e.g. inside
//    `riskyOperation`) — expect the keyword blurb explaining it desugars to
//    `expr.raise()`.
//
// 2. Hover `isA` on the `obj.isA(Number)` call below — expect a selector
//    signature hover showing `isA(_) — method on Object (core.ph)`.
//
// 3. Hover `describe` on its own declaration line (`describe {`) — the
//    doc block immediately above it should surface as the hover's summary
//    paragraph (this is the local, same-document Phaldoc scanner — it only
//    works because the /// block is adjacent, in this file, to this exact
//    declaration line).
//
// 4. Hover `describe` again but from its call site further down in
//    `callSite` — expect NO doc summary this time (single-document scanner,
//    not a project index; only the signature line, if `describe` also
//    happens to be a recognized core selector — otherwise no hover at all,
//    which is expected/documented scope).

class Inspector {
  riskyOperation(_ x) {
    (x < 0).ifTrue { throw ArgumentError.new("x must be >= 0") }
    return x
  }

  checkType(_ obj) {
    return obj.isA(Number)
  }

  /// Summarizes this inspector's current state as a short label.
  ///
  /// This second paragraph should NOT appear in the hover — only the first
  /// paragraph above is the summary Phaldoc harvests.
  describe {
    return "Inspector"
  }

  callSite {
    return self.describe
  }
}

// area: strings
// spec: leaf toString fast path (CB-6 follow-up) — `Value::to_display_string`
//       fast-paths `Number`/`Symbol`/`Str` while their override-epoch flag
//       (`Universe::number_tostring_pristine` et al.) is pristine, but must
//       fall back to a real `toString` send the instant a leaf class is
//       reopened.
// status: PASS
// Regression guard: reopening `Number#toString` after bootstrap must be
// picked up by BOTH `System.print` (a `List` of numbers, exercising the fast
// path through `list_to_string`'s per-element render) and `\(…)` string
// interpolation on a bare `Number`. If the fast path's override-epoch flag
// or selector encoding is wrong, these two lines stop agreeing with each
// other and with the reopened body.
class Number {
  toString => "N"
}

System.print([1])
System.print("\(1)")

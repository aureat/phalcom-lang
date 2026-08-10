// area: strings
// spec: DEFERRED CB-6 fix — `Value::to_display_string` (System.print) and
//       `\(…)` string interpolation must agree on the same container value.
// status: PASS
// Regression guard for CB-6 Site A: `to_display_string` used to skip the
// `toString` send for `List`/`Map` (among others), so `System.print` could
// disagree with `"\(…)"` on the exact same value. Both paths below print the
// same object back to back — they must be identical.

class Secret { toString { "<redacted>" }
}
const s = Secret.new()

const list = [s]
System.print(list)
System.print("\(list)")

const m = Map.new()
m.at("k", put: s)
System.print(m)
System.print("\(m)")

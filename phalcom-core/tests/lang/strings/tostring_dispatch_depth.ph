// area: strings
// spec: DEFERRED CB-6 fix — `to_display_string` sends `toString`
//       unconditionally; `list_to_string` sends per element.
// status: PASS
// Regression guard: a user `toString` override must be honoured at any
// nesting depth, through any mix of `List`/`Map` containers, not just at
// the top level.

class Secret { toString => "<redacted>"
}
const s = Secret.new()

// A `List` nested inside a `List`.
System.print([[s]])

// A `Map` nested inside a `List`.
const m = Map.new()
m.at("k", put: s)
System.print([m])

// A `List` nested inside a `Map`.
const m2 = Map.new()
m2.at("k", put: [s])
System.print(m2)

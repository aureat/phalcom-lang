// area: strings
// spec: DEFERRED CB-6 fix — `to_display_string` sends `toString`
//       unconditionally; `list_to_string` sends per element.
// status: PASS
// Regression guard: a user `toString` override must be honoured at any
// nesting depth, through any mix of `List`/`Map` containers, not just at
// the top level.

class Secret { toString { "<redacted>" }
}
let s = Secret.new()

// A `List` nested inside a `List`.
let inner: List<Secret> = [s]
let outer: List<List<Secret>> = [inner]
System.print(outer)

// A `Map` nested inside a `List`.
let m = Map.new()
m.at("k", put: s)
let maps: List<Dynamic> = [m]
System.print(maps)

// A `List` nested inside a `Map`.
let m2 = Map.new()
let nested: List<Secret> = [s]
m2.at("k", put: nested)
System.print(m2)

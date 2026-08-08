// area: strings
// spec: ADR-0022 §"Stringification target" (as amended 2026-07-15);
//       ADR-0015 (Object default toString); DEFERRED CB-1
// status: PASS
// **Regression guard for DEFERRED CB-1.** `"\(x)"` must send `toString`, so a
// user's override is honoured and interpolation agrees with `System.print` and
// with an explicit `.toString` send — all three, for the same object.
//
// Before the fix the desugar wrapped each interpolation as `String.new(expr)`,
// which renders through the native `Value::to_string` and falls to
// `Value::to_debug` for anything it has no bespoke renderer for. So a plain
// instance printed `<redacted>` via System.print but `<Secret instance>` via
// `"\(p)"` — the override silently bypassed at by far the most common
// stringify site.
//
// Why this matters beyond tidiness (CB-1's security note): ADR-0015's default
// `toString` is redaction-safe — a `SecretKey` renders as `<SecretKey>`, not its
// contents — and that property did not survive interpolation. The leak was
// bounded only because `to_debug` happens to print no field contents today;
// enriching it (an ordinary debug convenience someone will want) would have
// turned every interpolation site into a field-disclosure bug.

class Secret {
  toString => "<redacted>"
}
const p = Secret.new()

// The three paths must agree.
System.print(p)
System.print("\(p)")
System.print(p.toString)

// A class object is the other case `to_string` got wrong (it fell to to_debug's
// `<class Secret>` while print sent toString).
System.print("\(Secret)")

// Values the native path DID render correctly must not regress. Each of these
// reaches a real `toString`: Number/Symbol/List native, String `=> self`, Bool /
// Map / Set / Tuple / Range derived in core.ph.
const m = Map.new()
m.at("a", put: 1)
System.print("\(1) \("s") \(true) \([1, 2]) \(m) \((1, 2)) \(1..=5)")

// Nested: the payload renders via its own toString, not to_debug — because
// `Option#toString` and the new `Map#toString` are BOTH derived in core.ph, so
// they recurse by sending `toString`.
System.print("\(Some.new(p))")
const m2 = Map.new()
m2.at("k", put: p)
System.print("\(m2)")

// **DEFERRED CB-6, fixed.** `List#toString` (`list_to_string`) is a NATIVE
// primitive; it used to recurse through `Value::to_string`, which never sends
// a message, so an override nested in a `List` was bypassed while the same
// value nested in a `Map` (a `.ph` toString, which does send) rendered
// correctly. `list_to_string` now sends `toString` per element
// (`Value::to_display_string`), so both agree.
System.print("\([p])")

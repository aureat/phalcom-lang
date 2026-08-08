// area: variadics
// spec: U9-implementation-spec.md §2; method-lookup.md §1-2; ADR-0012
// status: PASS
// A miss with no variadic entry (no `bogus(*)` selector exists anywhere in
// the hierarchy) falls through to U8's `doesNotUnderstand(_:)` path exactly
// as before — proves the variadic probe doesn't swallow the dNU fallback.

class Proxy3 {
  doesNotUnderstand(_ msg) {
    return "proxied"
  }
}
const p = Proxy3.new()
System.print(p.bogus(1, 2, 3))

// area: collections
// spec: lexical-structure.md §6; ADR-0032 §1, §3.1; DEC-COLL-B; U-COLLTYPES plan.md
// status: PASS
// The `{ IDENT : … }` map literal (U-COLL's §6 disambiguation) now desugars
// to a native `Map` construction chain (`Map.new().at(Symbol.new("a"),
// put: 1)…`, wired at U-COLLTYPES Phase 1 landing) — bare-identifier keys
// are symbols (ADR-0032 §3.1). Supersedes the retired
// `negative/map_literal_pending.ph` (the map literal no longer raises the
// "pending" diagnostic it used to pin).
//
// NOTE: keyed retrieval by an independently-reconstructed symbol
// (`m.at(Symbol.new("a"))`) is not exercised here — it depends on
// `Symbol#==` comparing equal for two separately-interned-but-identical
// symbols, a pre-existing gap outside U-COLLTYPES's scope (see
// DEFERRED.md). `keys`/`values`/`each(_)` read the map's own stored keys
// back directly (no re-comparison needed) and are exercised instead.

let m = {a: 1, b: 2}
System.print(m)
System.print(m.size)
System.print(m.keys)
System.print(m.values)
m.each { k, v =>
  System.print(k)
  System.print(v)
}

// area: collections
// spec: lexical-structure.md §6; ADR-0032 §1, §3.1; DEC-COLL-B
// status: NEGATIVE
// The `{ IDENT : … }` map literal is *recognised* by the §6 disambiguation but
// its runtime lowering is deferred to the collection-runtime unit
// (U-COLLTYPES), so it raises a precise "pending" diagnostic. This pins the
// map branch as classified (not silently mis-parsed as a block) — the highest
// -risk line of the U4 brace interaction.

let m = {a: 1, b: 2}
System.print(m)

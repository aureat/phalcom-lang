// area: collections
// spec: lexical-structure.md §7; ADR-0032 §1, §3.2; DEC-COLL-A
// status: PASS — graduated at U-COLLTYPES Phase 2 (native `Tuple` arm landed)
// The tuple `(a, b)` arm desugars to `Tuple.__fromList(List.new().add(a).add(b))`.
// The parser lowering shipped in U-COLL; the native `Tuple` class (U-COLLTYPES
// Phase 2) resolves the desugar target, so this now runs to completion: a
// `Tuple` distinct from `List` (the typing surface requires the distinction).

const t = (3, 4)
System.print(t.size)
System.print(t.at(0))
System.print(t.class)

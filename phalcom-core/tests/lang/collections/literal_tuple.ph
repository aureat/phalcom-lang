// area: collections
// spec: lexical-structure.md §7; ADR-0032 §1, §3.2; DEC-COLL-A
// status: PASS — graduated at U-COLLTYPES Phase 2 (native `Tuple` arm landed)
// The compiler emits `BuildTuple` for `(a, b)`; no constructor selector is
// exposed to source code. The native `Tuple` class (U-COLLTYPES Phase 2)
// realizes that instruction, so this now runs to completion: a
// `Tuple` distinct from `List` (the typing surface requires the distinction).

const t = (3, 4)
System.print(t.size)
System.print(t.at(0))
System.print(t.class)

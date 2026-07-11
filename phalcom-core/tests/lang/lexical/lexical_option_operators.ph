// area: lexical/operators
// spec: lexical-structure.md §9; values-and-absence.md §3.4; ADR-0007
// status: PASS
// D5 coverage: the `??` null-coalescing and `?.` optional-send operators were
// landed by U6 (parser desugars `a ?? b` → `a.orElse { b }` and `opt?.m` →
// `opt.map { recv => recv.m }`). This fixture exercises them end-to-end under
// the `lexical` label; U-LEX adds no machinery here.
//
// `??` replaces a `None` with the right-hand `Option` and passes a `Some`
// through unchanged.
System.print((None ?? Some.new(5)).match(some: { v => v }, none: { 0 }))
System.print((Some.new(3) ?? Some.new(5)).match(some: { v => v }, none: { 0 }))
// `?.` maps over a `Some` (yielding a `Some`) and short-circuits a `None`.
System.print(Some.new(3)?.toString.isSome)
System.print(None?.toString.isNone)

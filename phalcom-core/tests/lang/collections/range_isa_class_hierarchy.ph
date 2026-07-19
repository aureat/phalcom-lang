// area: collections
// spec: tuple-and-range.md §2; object-model.md
// status: PASS
// Wren range/type.wren, adapted: Phalcom has no `is` binary type-test
// operator (docs/spec/v0.2/next/is-tests.md is a future surface, unlanded)
// and no generic `Sequence`/`Iterable` root a `Range` conforms to yet
// (U-ITERABLE, unlanded) — `isA(_)` is the current membership test, and
// `.class` the current type-identity read (Wren's `.type`).

const r = Range.new(2, 5, true)
System.print(r.isA(Range))
System.print(r.isA(Object))
System.print(r.isA(String))
System.print(r.class == Range)

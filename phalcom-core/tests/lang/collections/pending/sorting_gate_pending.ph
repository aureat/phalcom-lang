// area: collections
// spec: D.3 sorting gate pending
// status: PENDING

// Verification checklist for Ordering variants:
// Ordering.less / Ordering.equal / Ordering.greater
// Since sorting methods are NOT implemented yet, we document/comment out/write inactive checks
// and verify the gate does not expose them.

System.print("sorting gate check")

// Let's verify sorting methods are NOT defined on List / Iterable.
// We can do this by showing that calling them raises doesNotUnderstand.
// But wait! If we do it, it's a runtime exception. In pending tests, we can assert that they are not implemented,
// or we can write a commented-out test demonstrating how sorting *would* work when implemented,
// as requested in §20 of the implementation plan:
// "- Verification checklist for Ordering variants.
//  - Callback count expectations for sorted(on:).
//  - In-place List#sort returning Unit."

/*
// Ordering variants checklist:
// Ordering.less is returned when a < b
// Ordering.greater is returned when a > b
// Ordering.equal is returned when a == b

// Callback count expectations for sorted(on:) with N elements:
// sorted(on:) should invoke the selector block exactly N times (one per element) and cache the keys.

// In-place List#sort returns Unit:
// let list = [3, 1, 2]
// list.sort() == Unit
*/

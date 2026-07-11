// area: blocks
// spec: blocks.md; functions.md
// status: PENDING
// BLOCKED (not on U10): this fixture reaches for `List.reduce(_)` with a block
// argument, but `reduce`/`map`/`filter` and `[...]` list literals are not in the
// kernel yet — they are deferred to U-STD (core.ph §"do not add those bodies";
// DEFERRED.md). Non-local return itself (U10) needs none of these; it is fully
// covered by ../blocks_non_local_return.ph (each-based multi-level unwind).
// Rewrite this against the real `reduce` protocol once U-STD lands.
let numbers = [1, 2, 3, 4]
let sum = numbers.reduce(0) { acc, n => acc + n }
System.print(sum)

// U-ITER-FIX item 1(a) / U-REOPEN-FIX (ADR-0035 §3, iteration.md §3): a `break`
// reached through a block the inliner *materializes* as a real closure —
// here a non-Bool `if` condition (`Truthy`) forcing the sacred inliner's
// deopt fallback — cannot statically jump into the enclosing loop's own
// chunk (it lives in a different compiler function). Rather than the silent
// no-op U-ITER originally shipped with, this now fails **loudly**: an
// `Error.new(message).raise()` trap fires, carrying a descriptive message
// ("...materialized block..."), and the program halts. The common
// `if (Bool) { break }` path is unaffected (inliner fast path never takes
// this deopt twin for a real Bool).
//
// NEGATIVE lane (U-REOPEN-FIX graduation from `iteration/pending`): stdout is
// only "1" — the trap fires on the first iteration, before "done" (and
// before x=2/x=3) ever print — and the process exits non-zero with the
// diagnostic on stdout/stderr.
class Truthy {
    ifTrue(_ block) { return block.call() }
}

for x in [1, 2, 3] {
    System.print(x)
    if (Truthy.new()) { break }
}
System.print("done")

// U-ITER-FIX item 1(a) / U-REOPEN-FIX (ADR-0035 §3, iteration.md §3): a
// `continue` reached through a materialized block — here a non-Bool `if`
// condition (`Truthy`) forcing the inliner's deopt fallback — cannot
// statically jump into the enclosing loop's own chunk (it lives in a
// different compiler function). Rather than the silent no-op U-ITER
// originally shipped with, this now fails **loudly**: an
// `Error.new(message).raise()` trap fires on the very first iteration,
// carrying a descriptive message ("...materialized block..."), and the
// program halts before anything prints. The common `if (Bool) { continue }`
// path is unaffected (inliner fast path).
//
// NEGATIVE lane (U-REOPEN-FIX graduation from `iteration/pending`): the
// process exits non-zero with the diagnostic on stdout/stderr.
class Truthy {
    ifTrue(_ block) { return block.call() }
}

for (x in [1, 2, 3]) {
    if (Truthy.new()) { continue }
    System.print(x)
}
System.print("done")

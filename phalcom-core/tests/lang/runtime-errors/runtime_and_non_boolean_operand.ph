// area: errors
// spec: control-flow.md; messages-and-selectors.md
// status: NEGATIVE
// U5: `and`/`or` are now ordinary, overridable sends (control-flow.md §2),
// so `true and 1` is no longer erroneous — it dispatches to Bool::and(_)
// and evaluates the block, returning 1. This case now tests the send
// actually failing to resolve: Number has no `and(_)` method.

System.print(1 and true)

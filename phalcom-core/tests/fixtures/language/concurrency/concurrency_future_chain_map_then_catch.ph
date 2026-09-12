// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A adversarial: a `map` -> `then` -> `catch` chain fires
// through scheduled callbacks at each hop. From a
// `fulfilled` start, `map`/`then` both run (mutating the value) and the
// trailing `catch` is a passthrough (its handler never fires, since the
// chain is still `fulfilled`). From a `rejected` start, `map`/`then` are
// both skipped (the rejection propagates through them unchanged) and only
// the trailing `catch` fires, recovering to `fulfilled`.
const chained = Future.value(2)
    .map |v| { v + 1 }
    .map |v| { v * 2 }
    .catch |e| { -1 }

System.runScheduled()
System.print(chained.value)

const rejectedChain = Future.error(Error.new())
    .map |v| { v + 1 }
    .map |v| { v * 2 }
    .catch |e| { -1 }

System.runScheduled()
System.print(rejectedChain.value)

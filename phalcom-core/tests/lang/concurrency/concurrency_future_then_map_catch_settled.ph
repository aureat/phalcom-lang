// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A: `then`/`map`/`catch` over an already-settled receiver
// fire synchronously (plan §6.2). `then`/`map` run their block on the
// `fulfilled` path and propagate a `rejected` receiver untouched; `catch`
// runs its block on the `rejected` path (recovering to `fulfilled`) and
// propagates a `fulfilled` receiver untouched.

System.print(Future.value(10).then |v| { v + 1 }.value)
System.print(Future.error(Error.new()).then |v| { v + 1 }.isReady)

System.print(Future.value(10).map |v| { v * 2 }.value)
System.print(Future.error(Error.new()).map |v| { v * 2 }.isReady)

System.print(Future.value(10).catch |e| { 0 }.value)
System.print(Future.error(Error.new()).catch |e| { 99 }.value)

// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A: `Future.value(_)`/`Future.error(_)` build already-settled
// futures; `isReady`/`value` read them without ever suspending (C-FUT-8).
// `value` is the settled value as an `Option` — `Some(v)` once `fulfilled`,
// `None` once `rejected` (the rejection reason is reached via `catch`/`then`,
// not `value`).

const ok = Future.value(42)
System.print(ok.isReady)
System.print(ok.value)

const bad = Future.error(Error.new())
System.print(bad.isReady)
System.print(bad.value)

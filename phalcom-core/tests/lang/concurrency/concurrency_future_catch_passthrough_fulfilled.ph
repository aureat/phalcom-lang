// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A adversarial: `catch(_)` on an already-`fulfilled`
// receiver is a passthrough — the handler block is never invoked (only the
// `rejected` path fires it), so the returned future carries the *original*
// value through unchanged, and the receiver itself is untouched (still
// `fulfilled` with its original value, readable independently afterward).
const f = Future.value(3)
const caught = f.catch |e| { 999 }
System.print(caught.value)
System.print(f.value)

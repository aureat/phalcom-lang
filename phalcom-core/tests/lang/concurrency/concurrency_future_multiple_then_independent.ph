// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A adversarial: `then(_)` never mutates its receiver —
// registering two independent continuations on the *same* settled fulfilled
// future produces two independent derived futures, each seeing the base's
// original value, and the base itself is left unchanged after both fire.
const base = Future.value(5)
const a = base.then { v => v + 1 }
const b = base.then { v => v * 10 }
System.print(a.value)
System.print(b.value)
System.print(base.value)

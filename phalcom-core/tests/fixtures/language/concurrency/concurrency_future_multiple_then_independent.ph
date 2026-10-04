// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A adversarial: `then(_)` never mutates its receiver —
// registering two independent continuations on the *same* settled fulfilled
// future produces two independent derived futures, each seeing the base's
// original value, and the base itself is left unchanged after both fire.
let base = Future.value(5)
let a = base.then |v| { Future.value(v + 1) }
let b = base.then |v| { Future.value(v * 10) }
System.runScheduled()
System.print(a.value)
System.print(b.value)
System.print(base.value)

// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// U-FUTURE Slice A adversarial: `then(_)`/`map(_)` on a `rejected` receiver
// never invoke their block — the rejection propagates through unchanged
// (`self`, plan §6.2), so the derived future is still `rejected` and its
// `value` is `None`; the original rejection reason is only reachable via
// `catch(_)`, confirmed here by feeding the *same* rejected future through
// both `then` and `catch` and observing the identical captured `Error`.
const boom = Error.new()
const rejected = Future.error(boom)
const skipped = rejected.then |v| { v + 1 }
System.print(skipped.isReady)
System.print(skipped.value)
const recovered = rejected.catch |e| { (e == boom) }
System.print(recovered.value)

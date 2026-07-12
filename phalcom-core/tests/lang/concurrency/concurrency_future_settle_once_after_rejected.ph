// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// C-FUT-3 adversarial: settle-once holds symmetrically from the `rejected`
// side too — once a future has settled `rejected` with `firstErr`, *both* a
// later `settleError(_)` (a different `Error` instance) and a later
// `settleValue(_)` are no-ops that leave the original rejection completely
// untouched. Confirmed not just by `isReady`/`value` (which can't distinguish
// "still the same rejection" from "some other rejection") but by recovering
// the captured error through `catch(_)` and checking its *identity* against
// `firstErr`.
let firstErr = Error.new()
let secondErr = Error.new()
let f = Future.error(firstErr)
f.settleError(secondErr)
f.settleValue(1)
System.print(f.isReady)
System.print(f.value)
let caught = f.catch { e => (e == firstErr) }
System.print(caught.value)

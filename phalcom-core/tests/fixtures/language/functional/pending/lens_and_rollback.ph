// area: functional
// status: PENDING
from universe.functional import State, StateStep, Lens

data Counter(_ count: Int, _ label: String)
data Model(_ counter: Counter, _ enabled: Bool)

let count = Lens<Counter, Int>.new(
  get: |c| { c.count },
  set: |c, n| { Counter(n, c.label) }
)
let counter = Lens<Model, Counter>.new(
  get: |model| { model.counter },
  set: |model, c| { Model(c, model.enabled) }
)
let original = Model(Counter(3, "kept"), true)
let focused = counter.compose(count)
let updated = focused.modify(original, using: |n| { n + 5 })
System.print(original.counter.count)
System.print(updated.counter.count)
System.print(updated.counter.label)
System.print(updated.enabled)
let increment = State<Int, Int>.new(|n| { StateStep<Int, Int>(state: n + 1, value: n) })
let zoomed = increment.zoom(through: focused).run(original)
System.print(zoomed.value)
System.print(zoomed.state.counter.count)
System.print(zoomed.state.counter.label)

let failure = State<Int, Result<Int, String>>.new(|n| {
  StateStep<Int, Result<Int, String>>(state: n + 99, value: Result<Int, String>::Error("bad"))
})
let failed = failure.rollbackOnError<Int, String>().run(5)
System.print(failed.state)
System.print(failed.value.err.unwrap)
let success = State<Int, Result<Int, String>>.new(|n| {
  StateStep<Int, Result<Int, String>>(state: n + 1, value: Result<Int, String>::Ok(42))
})
let committed = success.rollbackOnError<Int, String>().run(5)
System.print(committed.state)
System.print(committed.value.ok.unwrap)

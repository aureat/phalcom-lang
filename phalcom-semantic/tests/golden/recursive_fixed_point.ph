// LAW CHAIN
// 1. Item<T> carries a recursive Item<T> link and exposes T through members.
// 2. Walker<T>#visit and #revisit form a mutually recursive callable SCC.
// 3. IntWalker specializes Walker<T> at T=Int and publishes the recursive result.
// 4. Service mutates loop state across a back-edge with continue, break, return,
//    and throw paths; Probe observes the composed tuple plus an independent Int.
//
// OBSERVATIONS
// 01 Item<T> recursive field/member substitution.
// 02 Walker<T> generic parameter identity across both recursive callables.
// 03 visit -> revisit resolved callable edge.
// 04 revisit -> visit resolved callable edge.
// 05 recursive normal-return fixed point remains T, not fabricated Dynamic.
// 06 IntWalker inheritance specializes T=Int.
// 07 Service candidate/current binding joins across loop iterations.
// 08 continue contributes a back-edge without a normal value.
// 09 break contributes the loop exit state.
// 10 throw and return paths contribute no normal loop value.
// 11 Service -> IntWalker -> Walker dependency chain is retained.
// 12 Probe tuple publication preserves independent Int evidence.

class Item<T> {
  _value: T
  _next: Item<T>

  @constructor
  new(_ value: T, _ next: Item<T>) {
    _value = value
    _next = next
  }

  value() -> T { _value }
  next() -> Item<T> { _next }
}

class Walker<T> {
  @constructor
  new() {}

  visit(_ item: Item<T>, _ fuel: Int) -> T {
    if fuel <= 0 {
      return item.value()
    } else {
      self.revisit(item.next(), fuel - 1)
    }
  }

  revisit(_ item: Item<T>, _ fuel: Int) -> T {
    if fuel <= 0 {
      return item.value()
    } else {
      self.visit(item.next(), fuel - 1)
    }
  }
}

class IntWalker is Walker<Int> {
  @constructor
  new() {}

  run(_ item: Item<Int>, _ fuel: Int) -> Int {
    self.visit(item, fuel)
  }
}

class Service {
  @class
  execute(_ start: Item<Int>, _ limit: Int, _ abort: Bool) {
    let current = 0
    let index = 0
    let walker = IntWalker.new()

    while index < limit {
      if abort {
        throw "aborted"
      }

      let candidate = walker.run(start, index)
      current = candidate
      index = index + 1

      if candidate == 0 {
        continue
      }

      if index >= limit {
        break
      }
    }

    (current, index)
  }
}

class Probe {
  @class
  run(_ start: Item<Int>, _ limit: Int, _ abort: Bool) {
    let result = Service.execute(start, limit, abort)
    let (value, count) = result
    let independent = 42
    (value, count, independent)
  }
}

// LAW CHAIN
// 1. Apply contextually types closure parameter x as Int.
// 2. Captured base retains outer BindingId and is not mutated by closure flow.
// 3. Closure branches join and publish an Int return.
// 4. Apply -> Service -> Probe composes the callable dependency chain.

class Apply {
  @class
  apply(_ value: Int, with f: (Int) -> Int) -> Int {
    f(value)
  }
}

class Service {
  @class
  run(_ flag: Bool) {
    let mut base = 1

    let mut transform = |x| {
      let mut local = if flag {
        x
      } else {
        base
      }
      local
    }

    let mut result = Apply.apply(42, with: transform)
    let mut stillBase = base
    (result, stillBase)
  }
}

class Probe {
  @class
  run(_ flag: Bool) {
    Service.run(flag)
  }
}

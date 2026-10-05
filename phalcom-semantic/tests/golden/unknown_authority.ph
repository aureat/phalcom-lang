// LAW CHAIN
// 1. Known constructor evidence validates certain.
// 2. Opaque mystery remains incomplete and may be assumed, never established.
// 3. Branch join preserves reachable weakness.
// 4. Independent certain.value() remains established Int.

class CellNum {
  @constructor new() {}
  value() -> Int { 1 }
}

class Factory {
  @class
  known() -> CellNum {
    CellNum.new()
  }

  @class
  opaque() {
    mystery()
  }
}

class Service {
  @class
  run(_ chooseKnown: Bool) {
    let mut certain: CellNum = Factory.known()
    let mut uncertain: CellNum = Factory.opaque()

    let mut selected = if chooseKnown {
      certain
    } else {
      uncertain
    }

    let mut independent = certain.value()
    (selected, independent)
  }
}

class Probe {
  @class
  run(_ flag: Bool) {
    Service.run(flag)
  }
}

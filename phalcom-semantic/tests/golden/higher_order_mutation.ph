// LAW CHAIN
// 1. Factory.make returns a nested callable with a mutable captured binding.
// 2. The inner add closure and outer twice closure share callable/capture facts.
// 3. Aliasing the returned callable preserves its contextual (Int) -> Int type.
// 4. Apply -> Service -> Probe observes mutation, alias selection, and tuple publication.
//
// OBSERVATIONS
// 01 Factory.make publishes a callable return type.
// 02 add captures total by binding identity, not by value copy.
// 03 add's mutation effect is visible across repeated invocations.
// 04 twice captures add as a first-class callable.
// 05 twice's nested calls preserve Int parameter and result facts.
// 06 original and alias share callable compatibility without duplicating captures.
// 07 conditional callable selection joins equivalent callable shapes.
// 08 Apply.apply checks the selected callable against (Int) -> Int.
// 09 direct invocation after Apply observes the same captured state.
// 10 Service publishes both call results as an Int tuple.
// 11 Service -> Factory -> Apply callable dependency chain is retained.
// 12 Probe preserves independent Int evidence beside higher-order flow.

class Apply {
  @class
  apply(_ value: Int, with f: (Int) -> Int) -> Int {
    f(value)
  }
}

class Factory {
  @class
  make(_ seed: Int) -> (Int) -> Int {
    let total = seed

    let add: (Int) -> Int = |delta| {
      total = total + delta
      total
    }

    let twice: (Int) -> Int = |value| {
      let once = add(value)
      add(once)
    }

    twice
  }
}

class Service {
  @class
  run(_ seed: Int, _ chooseAlias: Bool) {
    let original = Factory.make(seed)
    let alias = original

    let selected = if chooseAlias {
      alias
    } else {
      original
    }

    let first = Apply.apply(1, with: selected)
    let second = original(2)
    (first, second)
  }
}

class Probe {
  @class
  run(_ seed: Int, _ chooseAlias: Bool) {
    let result = Service.run(seed, chooseAlias)
    let (first, second) = result
    let independent = 42
    (first, second, independent)
  }
}

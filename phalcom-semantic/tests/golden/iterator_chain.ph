// LAW CHAIN
// 1. Strange<A, B> exposes Entry<B, A> through iteratorValue.
// 2. Strange<Int, String> iterates Entry<String, Int>.
// 3. Nested member and Decoder calls preserve those substitutions.
// 4. Loop break leaves last and tuple publication precise.

class Entry<K, V> {
  mut _key: K
  mut _value: V

  @constructor
  new(_ key: K, _ value: V) {
    _key = key
    _value = value
  }

  key() -> K { _key }
  value() -> V { _value }
}

class Strange<A, B> {
  iteratorValue(_ cursor: Int) -> Entry<B, A> {
    mystery()
  }
}

class Decoder {
  @class
  decode(_ value: Int) -> Int { value }
}

class Service {
  @class
  collect(_ source: Strange<Int, String>, _ stop: Bool) {
    let mut last = 0

    for entry in source {
      let mut key = entry.key()
      let mut value = Decoder.decode(entry.value())
      last = value

      if stop {
        break
      }
    }

    (last, "done")
  }
}

class Probe {
  @class
  run(_ source: Strange<Int, String>, _ stop: Bool) {
    let mut result = Service.collect(source, stop)
    let mut (last, status) = result
  }
}

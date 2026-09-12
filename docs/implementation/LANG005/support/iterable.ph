// LANG005 — proposed kernel trait surface.
// Target-facing syntax: not expected to parse before LANG005 lands.
//
// Iteration remains the two-selector cursor protocol. Cursor progression returns
// Option<Cursor>; the compiler may optimize the wrapper away when proven.

trait Iterable {
  type Item
  type Cursor

  iterate(_ cursor: Option<Cursor>) -> Option<Cursor>
  iteratorValue(_ cursor: Cursor) -> Item

  each(_ f) {
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      f.call(self.iteratorValue(live))
      cursor = self.iterate(Some(live))
    }
    ()
  }

  map(_ f) {
    const result = List.new()
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      result.append(f.call(self.iteratorValue(live)))
      cursor = self.iterate(Some(live))
    }
    result
  }

  filter(_ predicate) {
    const result = List.new()
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      const value = self.iteratorValue(live)
      if predicate.call(value) {
        result.append(value)
      }
      cursor = self.iterate(Some(live))
    }
    result
  }

  contains(_ expected) -> Bool {
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      if self.iteratorValue(live) == expected {
        return true
      }
      cursor = self.iterate(Some(live))
    }
    false
  }

  all(where predicate) -> Bool {
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      if not predicate.call(self.iteratorValue(live)) {
        return false
      }
      cursor = self.iterate(Some(live))
    }
    true
  }

  any(where predicate) -> Bool {
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      if predicate.call(self.iteratorValue(live)) {
        return true
      }
      cursor = self.iterate(Some(live))
    }
    false
  }

  none(where predicate) -> Bool {
    not self.any(where: predicate)
  }

  count -> Int {
    let n = 0
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      n = n + 1
      cursor = self.iterate(Some(live))
    }
    n
  }

  find(where predicate) -> Option<Item> {
    let cursor = self.iterate(None)
    while cursor.isSome {
      const live = cursor.unwrap
      const value = self.iteratorValue(live)
      if predicate.call(value) {
        return Some(value)
      }
      cursor = self.iterate(Some(live))
    }
    None
  }

  toList -> List<Item> {
    const result = List.new()
    self.each(|value| { result.append(value) })
    result
  }

  toSet -> Set<Item> {
    const result = Set.new()
    self.each(|value| { result.add(value) })
    result
  }
}

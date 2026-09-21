native class Record is Object {
  intrinsic type Map<F: Type -> Type>: Type

  internal native rawSize -> Int
  internal native rawLabelAt(_ index: Int) -> Symbol
  internal native rawValueAt(_ index: Int) -> Dynamic
  size { self.rawSize }
  labelAt(_ index) { self.rawLabelAt(index) }

  // Safe label lookup for record patterns and ordinary reflection. The
  // Option result distinguishes a missing label from a stored None value.
  get(_ key) {
    let i = 0
    while (i < self.size) {
      if (self.labelAt(i) === key) { return Some(self.rawValueAt(i)) }
      i = i + 1
    }
    None
  }

  ==(_ other) {
    if (other.is(Record)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        let label = self.rawLabelAt(i)
        let j = 0
        let found = false
        while ((not found) and (j < other.size)) {
          if (other.rawLabelAt(j) == label) {
            found = (self.rawValueAt(i) == other.rawValueAt(j))
          }
          j = j + 1
        }
        same = found
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  hash {
    let acc = 101 + self.size
    let i = 0
    while (i < self.size) {
      // Record equality is key-set based, so the hash contribution must be
      // commutative over presentation order as well. The native view keeps
      // encounter order observable, but that order is not semantic identity.
      acc = (acc + ((self.rawLabelAt(i).hash * 31) + self.rawValueAt(i).hash)) % 999999937
      i = i + 1
    }
    return acc
  }
}

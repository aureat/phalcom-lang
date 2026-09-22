/// Immutable labeled collection with insertion-ordered fields.
native class Record is Object {
  intrinsic type Map<F: Type -> Type>: Type

  internal native rawSize -> Int
  internal native rawLabelAt(_ index: Int) -> Symbol
  internal native rawValueAt(_ index: Int) -> Dynamic
  size { self.rawSize }
  labelAt(_ index) { self.rawLabelAt(index) }

  /// Returns the value for `key`, or `None` when the label is absent.
  get(_ key) {
    let i = 0
    while i < self.size {
      if self.labelAt(i) === key {
        return Some(self.rawValueAt(i))
      }
      i = i + 1
    }
    None
  }

  /// Compares records by label set and corresponding value equality.
  ==(_ other) {
    if other is not Record {
      return false
    }
    let same = self.size == other.size
    let i = 0
    while same and i < self.size {
      let label = self.rawLabelAt(i)
      let j = 0
      let found = false
      while not found and j < other.size {
        if other.rawLabelAt(j) == label {
          found = self.rawValueAt(i) == other.rawValueAt(j)
        }
        j = j + 1
      }
      same = found
      i = i + 1
    }
    same
  }

  hash {
    let acc = 101 + self.size
    let i = 0
    while i < self.size {
      acc = (acc + ((self.rawLabelAt(i).hash * 31) + self.rawValueAt(i).hash)) % 999999937
      i = i + 1
    }
    return acc
  }
}

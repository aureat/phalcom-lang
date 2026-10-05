intrinsic type data RecordFields

type FieldKey = Symbol | String | Int | Bool

export FieldKey
export RecordFields

impl RecordFields {
  intrinsic type Map<F: Type -> Type>: RecordFields
}

intrinsic data Record<**Fs> {
  **Fs
}

intrinsic type EmptyRecord = Record<>

/// Immutable labeled collection with insertion-ordered fields.
impl<**Fs: RecordFields> Record<**Fs> {
  internal native rawSize -> Int
  internal native rawLabelAt(_ index: Int) -> Symbol
  internal native rawValueAt(_ index: Int) -> Record<**Fs>::Value
  size { self.rawSize }
  labelAt(_ index) { self.rawLabelAt(index) }

  private rawValueAtChecked(_ index: Int) -> Record<**Fs>::Value {
    self.rawValueAt(index)
  }

  /// Returns the value for `key`, or `None` when the label is absent.
  get(_ key) -> Option<Record<**Fs>::Value> {
    let mut i = 0
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
    let mut same = self.size == other.size
    let mut i = 0
    while same and i < self.size {
      let mut label = self.rawLabelAt(i)
      let mut j = 0
      let mut found = false
      while not found and j < other.size {
        if other.rawLabelAt(j) == label {
          found = self.rawValueAtChecked(i) == other.rawValueAtChecked(j)
        }
        j = j + 1
      }
      same = found
      i = i + 1
    }
    same
  }

  hash {
    let mut acc = 101 + self.size
    let mut i = 0
    while i < self.size {
      acc = (acc + ((self.rawLabelAt(i).hash * 31) + self.rawValueAtChecked(i).hash)) % 999999937
      i = i + 1
    }
    return acc
  }
}

impl<**Fs: RecordFields> Record<**Fs> {
  intrinsic type Fields: RecordFields
  intrinsic type Value: Type
  intrinsic type Map<F: Type -> Type>: Type
}

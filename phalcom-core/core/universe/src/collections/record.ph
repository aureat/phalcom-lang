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
  internal native rawKeyAt(_ index: Int) -> FieldKey
  internal native rawValueAt(_ index: Int) -> Record<**Fs>::Value
  internal native rawLogicalValueAt(_ index: Int) -> Record<**Fs>::Value
  internal native rawSameKeys(_ other: Dynamic) -> Bool
  internal native rawKeyHashAt(_ index: Int) -> Int
  internal native rawLookup(_ key: FieldKey) -> Option<Record<**Fs>::Value>
  size { self.rawSize }
  labelAt(_ index) { self.rawLabelAt(index) }
  keyAt(_ index) { self.rawKeyAt(index) }

  /// Returns the value for `key`, or `None` when the label is absent.
  get(_ key: FieldKey) -> Option<Record<**Fs>::Value> { self.rawLookup(key) }

  /// Compares records by canonical key set and corresponding value equality.
  ==(_ other) {
    if other is not Record {
      return false
    }
    if self.size != other.size {
      return false
    }
    if not self.rawSameKeys(other) {
      return false
    }
    let mut same = true
    let mut i = 0
    while same and i < self.size {
      same = self.rawLogicalValueAt(i) == other.rawLogicalValueAt(i)
      i = i + 1
    }
    same
  }

  hash {
    let mut acc = 5391693 + self.size
    let mut i = 0
    while i < self.size {
      acc = (acc * 31 + (self.rawKeyHashAt(i) * 31) + self.rawLogicalValueAt(i).hash) % 999999937
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

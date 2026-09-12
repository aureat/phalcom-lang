// LANG005 — proposed kernel conformances.
//
// Existing concrete members witness requirements where signatures match.
// Bodies are supplied where the old Iterable superclass supplied behavior.
//
// No blanket Hashable-for-Object rule is introduced here.

// -----------------------------------------------------------------------------
// Iterable
// -----------------------------------------------------------------------------

impl<T> Iterable for List<T> {
  type Item = T
  type Cursor = Int

  iterate(_ previous: Option<Int>) -> Option<Int> {
    const next = previous.match(
      some: |cursor| { cursor + 1 },
      none: || { 0 },
    )
    if next < self.size { Some(next) } else { None }
  }

  iteratorValue(_ cursor: Int) -> T {
    self[cursor]
  }
}

impl<T> Iterable for Set<T> {
  type Item = T
  type Cursor = Int

  iterate(_ previous: Option<Int>) -> Option<Int> {
    const next = previous.match(
      some: |cursor| { cursor + 1 },
      none: || { 0 },
    )
    if next < self.size { Some(next) } else { None }
  }

  iteratorValue(_ cursor: Int) -> T {
    self.at(cursor)
  }
}

impl<K, V> Iterable for Map<K, V> {
  // Current Phalcom Map iteration yields keys.
  type Item = K
  type Cursor = Int

  iterate(_ previous: Option<Int>) -> Option<Int> {
    const next = previous.match(
      some: |cursor| { cursor + 1 },
      none: || { 0 },
    )
    if next < self.size { Some(next) } else { None }
  }

  iteratorValue(_ cursor: Int) -> K {
    self._$keyAt(cursor)
  }
}

impl Iterable for Tuple {
  // Runtime Tuple is heterogeneous. Static tuple types may later refine Item
  // to the union of their element types.
  type Item = Dynamic
  type Cursor = Int

  iterate(_ previous: Option<Int>) -> Option<Int> {
    const next = previous.match(
      some: |cursor| { cursor + 1 },
      none: || { 0 },
    )
    if next < self.size { Some(next) } else { None }
  }

  iteratorValue(_ cursor: Int) -> Dynamic {
    self[cursor]
  }
}

impl Iterable for Bytes {
  type Item = Int
  type Cursor = Int

  iterate(_ previous: Option<Int>) -> Option<Int> {
    const next = previous.match(
      some: |cursor| { cursor + 1 },
      none: || { 0 },
    )
    if next < self.size { Some(next) } else { None }
  }

  iteratorValue(_ cursor: Int) -> Int {
    self[cursor]
  }
}

impl Iterable for Range {
  type Item = Number
  type Cursor = Number

  // Existing Range#iterate(_) and Range#iteratorValue(_) are witnesses.
}

// -----------------------------------------------------------------------------
// Sized
// -----------------------------------------------------------------------------

impl<T> Sized for List<T> {}
impl<T> Sized for Set<T> {}
impl<K, V> Sized for Map<K, V> {}
impl Sized for Tuple {}
impl Sized for Record {}
impl Sized for String {}
impl Sized for Bytes {}

// Range is omitted until LANG005 fixes whether every Range is finite/sized.

// -----------------------------------------------------------------------------
// Indexable
// -----------------------------------------------------------------------------

impl<T> Indexable for List<T> {
  type Index = Int
  type Item = T
}

impl<T> MutableIndexable for List<T> {
  type Index = Int
  type Item = T
}

impl Indexable for Tuple {
  type Index = Int
  type Item = Dynamic
}

impl Indexable for Bytes {
  type Index = Int
  type Item = Int
}

// String is omitted until its exact indexing unit is a stable trait contract.

// -----------------------------------------------------------------------------
// Comparable
// -----------------------------------------------------------------------------

impl Comparable<Number> for Number {
  // Existing Number#compare(Number) is the witness.
}

// -----------------------------------------------------------------------------
// Hashable
// -----------------------------------------------------------------------------

impl Hashable for Number {}
impl Hashable for String {}
impl Hashable for Symbol {}
impl Hashable for Tuple {}
impl Hashable for Record {}
impl Hashable for Range {}
impl Hashable for Bytes {}

// No blanket `impl<T> Hashable for T where T <: Object` here.
// Universal-vs-capability hashing remains an explicit LANG005 decision.

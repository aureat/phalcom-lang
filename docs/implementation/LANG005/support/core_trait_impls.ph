// LANG005 — proposed future kernel conformance sketches.
//
// These declarations are migration input for C6, not production Universe
// sources. Current production Iterable/for behavior remains cursor-based.
// No blanket Hashable-for-Object rule is introduced here.

// -----------------------------------------------------------------------------
// Stateful Iterator / Iterable target
// -----------------------------------------------------------------------------

// Target conformance sketches after C6 provides nested conformance and
// associated-equality evidence. The relationship between Item and Iterator is
// deliberately not encoded here as provisional syntax.

impl<T> Iterable for List<T> {
  type Item = T
  type Iterator = ListIterator<T>
}

impl<T> Iterator for ListIterator<T> {
  type Item = T
}

impl<T> Iterable for Set<T> {
  type Item = T
  type Iterator = SetIterator<T>
}

impl<T> Iterator for SetIterator<T> {
  type Item = T
}

impl<K, V> Iterable for Map<K, V> {
  type Item = (key: K, value: V)
  type Iterator = MapIterator<K, V>
}

impl<K, V> Iterator for MapIterator<K, V> {
  type Item = (key: K, value: V)
}

impl Iterable for Tuple {
  // Runtime Tuple is heterogeneous; retain Dynamic for the initial target.
  type Item = Dynamic
  type Iterator = TupleIterator
}

impl Iterator for TupleIterator {
  type Item = Dynamic
}

impl Iterable for Bytes {
  type Item = Int
  type Iterator = BytesIterator
}

impl Iterator for BytesIterator {
  type Item = Int
}

impl Iterable for Range {
  type Item = Number
  type Iterator = RangeIterator
}

impl Iterator for RangeIterator {
  type Item = Number
}

// Target laws for these future conformances:
// - source iterator getters create fresh stateful traversals;
// - iterator.iterator returns self;
// - next returns Option<Item>, with stable None exhaustion;
// - ordinary errors propagate unchanged;
// - no public Cursor/hasNext/reset/clone protocol is required.
//
// Current Map direct iteration still yields keys through the cursor-based
// production Iterable until C6 performs the migration to entry tuples.

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

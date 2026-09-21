from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable

// Kernel Tuple (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 2): a native fixed
// arity immutable slice — Object::Tuple, mirroring List's shape but with NO
// mutation selector (immutability is structural, TupleObject's Box<[Value]>).
// Product literals compile directly to native build bytecodes.

native class Tuple is Collection {
  intrinsic type Positionals: TypePack
  intrinsic type Labeled: LabeledTypePack
  intrinsic type At<I: Int>: Type
  intrinsic type Map<F: Type -> Type>: Type

  @class
  internal native rawFromList(_ value: List<Dynamic>) -> Tuple
  internal native rawSize -> Int
  internal native rawAt(_ index: Int) -> Dynamic
  internal native rawPositionalSize -> Int
  internal native rawLabelAt(_ index: Int) -> Symbol
  internal native rawPositionals -> List<Dynamic>
  internal native rawLabeled -> Map<Symbol, Dynamic>
  internal native rawSlice(_ start: Int, _ end: Int) -> Tuple
  size { self.rawSize }
  positionals { self.rawPositionals }
  labeled { self.rawLabeled }
  labelAt(_ index) { self.rawLabelAt(index) }

  first {
    if (self.size == 0) { return None }
    return Some(self.at(0))
  }

  last {
    if (self.size == 0) { return None }
    return Some(self.at(self.size - 1))
  }

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Tuple` rendering exactly — `(a, b)`, `()` when empty. Derived over
  // `size_`/`at_`; each element renders via its OWN `toString`.
  toString {
    let s = "("
    let i = 0
    while (i < self.rawSize) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self.rawAt(i).toString
      i = i + 1
    }
    return s + ")"
  }

  at(_ i) { self.rawAt(i) }

  private findLabel(_ sym) {
    let num_labeled = self.size - self.rawPositionalSize
    let i = 0
    while (i < num_labeled) {
      if (self.rawLabelAt(i) == sym) {
        return Some(self.rawPositionalSize + i)
      }
      i = i + 1
    }
    return None
  }

  private access(_ key) {
    if (key.is(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { Some(self.rawAt(idx)) },
        none: || { None }
      )
    }
    let raw = self.rawAt(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return Some(raw)
    }
    return None
  }

  get(_ key) { self.access(key) }

  [_ key] {
    if (key.is(Range)) {
      return key.rawSliceBounds(self.size).match(
        ok: |bounds| {
          let start = bounds[0]
          let end = bounds[1]
          if (start > end) { end = start }
          self.rawSlice(start, end)
        },
        err: |error| { error.raise() }
      )
    }
    if (key.is(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self.rawAt(idx) },
        none: || { throw KeyError.new("Tuple label not found") }
      )
    }
    let raw = self.rawAt(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    throw IndexError.new("Tuple index out of range")
  }

  [_ key, default] {
    if (key.is(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self.rawAt(idx) },
        none: || { default }
      )
    }
    let raw = self.rawAt(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return default
  }

  get(_ key, orElse) {
    if (key.is(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self.rawAt(idx) },
        none: || { orElse.call(key) }
      )
    }
    let raw = self.rawAt(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return orElse.call(key)
  }

  doesNotUnderstand(_ message) {
    if (message.args == ()) {
      let idx = self.findLabel(message.selector)
      return idx.match(
        some: |i| { self.rawAt(i) },
        none: || { super.doesNotUnderstand(message) }
      )
    }
    return super.doesNotUnderstand(message)
  }

  iterator -> TupleIterator { TupleIterator.new(self) }

  zipped -> TupleZipView { TupleZipView.new(self) }

  // Structural equality: same arity, pairwise-==. Guarded by isA(Tuple) so a
  // non-Tuple (including a same-elements List — cross-kind, E2) is unequal.
  ==(_ other) {
    if (other.is(Tuple)) {
      let same = (self.size == other.size) and (self.rawPositionalSize == other.rawPositionalSize)
      let i = 0
      while (same and (i < self.size - self.rawPositionalSize)) {
        same = (self.rawLabelAt(i) == other.rawLabelAt(i))
        i = i + 1
      }
      i = 0
      while (same and (i < self.size)) {
        same = (self.at(i) == other.at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  !=(_ other) {
    return not (self == other)
  }

  // Value hash (DEC-CT-D): a .ph fold over each element's own `hash` —
  // order-sensitive, zero new floor. Consistent with == by construction (a
  // deterministic function of the same elements == compares), and survives
  // the future Int/Float split for free (forward-compat §4): it folds
  // *mathematical-value* hashes (whatever Number#hash decides), never bits.
  // Bounded by a large prime modulus so the accumulator stays a stable,
  // comparable Number regardless of tuple length.
  hash {
    let acc = 17 + self.rawPositionalSize
    let i = 0
    while (i < self.size) {
      acc = (acc * 31 + self.at(i).hash) % 999999937
      i = i + 1
    }
    i = 0
    while (i < self.size - self.rawPositionalSize) {
      acc = (acc * 31 + self.rawLabelAt(i).hash) % 999999937
      i = i + 1
    }
    return acc
  }
}

class TupleIterator {
  _source: Tuple
  _index: Int
  _done: Bool

  @constructor
  new(_ source: Tuple) {
    _source = source
    _index = 0
    _done = false
  }

  next -> Option<Dynamic> {
    if (_done) { return None }
    if (_index >= _source.size) {
      _done = true
      return None
    }
    let value = _source.at(_index)
    _index = _index + 1
    Some(value)
  }

  iterator { self }
}

impl CanonicalIterator for TupleIterator {
  type Item = Dynamic
}

impl CanonicalIterable for Tuple {
  type Item = Dynamic
  type Iterator = TupleIterator
  iterator -> TupleIterator { TupleIterator.new(self) }
}

class TupleZipIterator {
  _iterators: List<Dynamic>
  _done: Bool

  @constructor
  new(_ tuple: Tuple) {
    _iterators = List<Dynamic>.new()
    let i = 0
    while (i < tuple.size) {
      _iterators.rawPush(tuple.at(i).iterator)
      i = i + 1
    }
    _done = false
  }

  next -> Option<Dynamic> {
    if (_done or _iterators.size == 0) {
      _done = true
      return None
    }
    let firstStep = _iterators.at(0).next
    if (firstStep.isNone) {
      let i = 1
      while (i < _iterators.size) {
        if (_iterators.at(i).next.isSome) {
          _done = true
          throw ZipLengthMismatchError.new("strict zip lanes ended at different times")
        }
        i = i + 1
      }
      _done = true
      return None
    }
    let items = List<Dynamic>.new()
    items.rawPush(firstStep.match(some: |v| { v }, none: || { None }))
    let i = 1
    while (i < _iterators.size) {
      let step = _iterators.at(i).next
      if (step.isNone) {
        _done = true
        throw ZipLengthMismatchError.new("strict zip lanes ended at different times")
      }
      items.rawPush(step.match(some: |v| { v }, none: || { None }))
      i = i + 1
    }
    return Some(Tuple.rawFromList(items))
  }

  iterator { self }
}

class TupleZipView is Collection {
  _tuple: Tuple

  @constructor
  new(_ tuple: Tuple) { _tuple = tuple }

  iterator -> TupleZipIterator { TupleZipIterator.new(_tuple) }
}

impl CanonicalIterator for TupleZipIterator {
  type Item = Dynamic
}

impl CanonicalIterable for TupleZipView {
  type Item = Dynamic
  type Iterator = TupleZipIterator
  iterator -> TupleZipIterator { self.iterator }
}

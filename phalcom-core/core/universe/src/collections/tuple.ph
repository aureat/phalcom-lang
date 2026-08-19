class Unit {
  toString { "()" }
  hash { 0 }
}

// Kernel Tuple (ADR-0032 §1, ADR-0039, U-COLLTYPES Phase 2): a native fixed
// arity immutable slice — Object::Tuple, mirroring List's shape but with NO
// mutation selector (immutability is structural, TupleObject's Box<[Value]>).
// Product literals compile directly to native build bytecodes.

class Tuple {
  size { self._$size }
  positionals { self._$positionals }
  labeled { self._$labeled }
  labelAt(_ index) { self._$labelAt(index) }

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
    while (i < self._$size) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self._$at(i).toString
      i = i + 1
    }
    return s + ")"
  }

  at(_ i) { self._$at(i) }

  @private
  findLabel(_ sym) {
    let num_labeled = self.size - self._$positionalSize
    let i = 0
    while (i < num_labeled) {
      if (self._$labelAt(i) == sym) {
        return Some(self._$positionalSize + i)
      }
      i = i + 1
    }
    return None
  }

  @private
  access(_ key) {
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { Some(self._$at(idx)) },
        none: || { None }
      )
    }
    let raw = self._$at(key)
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
    if (key.isA(Range)) {
      return key._$sliceBounds(self.size).match(
        ok: |bounds| {
          let start = bounds[0]
          let end = bounds[1]
          if (start > end) { end = start }
          self._$slice(start, end)
        },
        err: |error| { error.raise() }
      )
    }
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self._$at(idx) },
        none: || { throw KeyError.new("Tuple label not found") }
      )
    }
    let raw = self._$at(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    throw IndexError.new("Tuple index out of range")
  }

  [_ key, default] {
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self._$at(idx) },
        none: || { default }
      )
    }
    let raw = self._$at(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return default
  }

  get(_ key, orElse) {
    if (key.isA(Symbol)) {
      return self.findLabel(key).match(
        some: |idx| { self._$at(idx) },
        none: || { orElse.call(key) }
      )
    }
    let raw = self._$at(key)
    let len = self.size
    let i = key
    if (i < 0) { i = len + i }
    if (i >= 0 and i < len) {
      return raw
    }
    return orElse.call(key)
  }

  iteratorValue(_ cursor) { self._$at(cursor) }

  // Structural equality: same arity, pairwise-==. Guarded by isA(Tuple) so a
  // non-Tuple (including a same-elements List — cross-kind, E2) is unequal.
  ==(_ other) {
    if (other.isA(Tuple)) {
      let same = (self.size == other.size) and (self._$positionalSize == other._$positionalSize)
      let i = 0
      while (same and (i < self.size - self._$positionalSize)) {
        same = (self._$labelAt(i) == other._$labelAt(i))
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
    let acc = 17 + self._$positionalSize
    let i = 0
    while (i < self.size) {
      acc = (acc * 31 + self.at(i).hash) % 999999937
      i = i + 1
    }
    i = 0
    while (i < self.size - self._$positionalSize) {
      acc = (acc * 31 + self._$labelAt(i).hash) % 999999937
      i = i + 1
    }
    return acc
  }
}

@native
class Set is Iterable {
  size { self._$size }

  // Display (U-CORE-4, R-INV-4.1; DEFERRED CB-1). Mirrors `Value::to_string`'s
  // native `Set` rendering exactly — `Set(a, b)`, `Set()` when empty. Derived
  // over `size_`/`at_`; each element renders via its OWN `toString`.
  toString {
    let s = "Set("
    let i = 0
    while (i < self._$size) {
      s = s + (i > 0).ifTrue(|| { ", " }, ifFalse: || { "" })
      s = s + self._$at(i).toString
      i = i + 1
    }
    return s + ")"
  }

  add(_ v) {
    self._$add(v)
    return self
  }

  contains(_ v) { self._$has(v) }
  includes(_ v) { self.contains(v) }

  remove(_ v) {
    self._$remove(v)
    return self
  }

  // Positional read in insertion order — not in the map-and-set.md selector
  // table, but a direct, zero-floor-cost derivation over at_ that the
  // U-CORE-5 conformance harness (collection-protocol.md §2) needs, and a
  // natural extension of the sequence protocol every collection instantiates.
  at(_ i) { self._$at(i) }

  iteratorValue(_ cursor) { self._$at(cursor) }

  // Structural equality: same members, order-independent. Same-size plus
  // "every element of self is in other" is sufficient since neither set
  // holds duplicates (add_ is idempotent).
  ==(_ other) {
    if (other.is(Set)) {
      let same = (self.size == other.size)
      let i = 0
      while (same and (i < self.size)) {
        same = other.includes(self._$at(i))
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
}

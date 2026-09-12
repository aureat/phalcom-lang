// LANG005 — proposed kernel comparison capability.
//
// Rhs is explicit rather than assuming consumed Self.

trait Comparable<Rhs> {
  compare(_ other: Rhs) -> Ordering

  <(_ other: Rhs) -> Bool {
    self.compare(other) === Ordering::Less
  }

  <=(_ other: Rhs) -> Bool {
    const ordering = self.compare(other)
    (ordering === Ordering::Less) or (ordering === Ordering::Equal)
  }

  >(_ other: Rhs) -> Bool {
    self.compare(other) === Ordering::Greater
  }

  >=(_ other: Rhs) -> Bool {
    const ordering = self.compare(other)
    (ordering === Ordering::Greater) or (ordering === Ordering::Equal)
  }
}

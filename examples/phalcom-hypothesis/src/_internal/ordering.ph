// Private lexicographic primitives used by deterministic engine orderings.

class _Ordering {
  @class
  integers(left: Int, right: Int) -> Int {
    if left < right {
      return -1
    }
    if left > right {
      return 1
    }
    return 0
  }

  @class
  strings(left: String, right: String) -> Int {
    if left < right {
      return -1
    }
    if left > right {
      return 1
    }
    return 0
  }
}

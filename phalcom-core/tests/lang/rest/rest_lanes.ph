// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §11-13
// status: PASS
// F.3 lane-aware rest capture: positional, labeled, split, and complete.

class RestLanes {
  positional(_ fixed, *tail) {
    return tail.size
  }
  labeled(timeout, **extra) {
    return extra.size
  }
  split(_ fixed, *tail, timeout, **extra) {
    return tail.size + extra.size
  }
  complete(_ fixed, timeout, ***remaining) {
    return remaining.size
  }
}

const lanes = RestLanes.new()
System.print(lanes.positional(1, 2, 3))
System.print(lanes.labeled(timeout: 1, debug: 2))
System.print(lanes.split(1, 2, 3, timeout: 4, debug: 5))
System.print(lanes.complete(1, timeout: 2, debug: 3))

// Exact lookup completes across inheritance before rest fallback starts.
class ExactParent {
  choose(_ left, _ right) { return 42 }
}
class RestChild is ExactParent {
  choose(*items) { return items.size }
}
System.print(RestChild.new().choose(1, 2))

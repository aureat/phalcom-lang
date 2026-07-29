// area: errors
// spec: error-handling.md §2; ADR-0031
// status: PASS
// First-listed = innermost = first-checked = first-match-wins: `on A` before
// `on B`, a `B` throw is checked against `A` (misses, re-propagated verbatim)
// then `B` (matches). Neither handler fires for an unrelated `C`, which
// reaches the outer `catch` (the eventual catch-all).

class AErr is Error {
  @constructor
  new(msg) { super.new(msg) }
}
class BErr is Error {
  @constructor
  new(msg) { super.new(msg) }
}
class CErr is Error {
  @constructor
  new(msg) { super.new(msg) }
}

try {
  throw BErr.new("b")
} on AErr e {
  System.print("A")
} on BErr e {
  System.print("B: " + e.message)
}

try {
  throw CErr.new("c")
} on AErr e {
  System.print("A")
} on BErr e {
  System.print("B")
} catch e {
  System.print("caught-all: " + e.message)
}

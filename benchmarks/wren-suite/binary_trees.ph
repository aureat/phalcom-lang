// Ported from wren/test/benchmark/binary_trees.wren. `_left == null` ->
// `_left == None` (Phalcom's absence value; unassigned fields read None,
// there is no surface `nil`). Both `for`-range forms replaced with
// while-counters (no range-literal parser production). String interpolation
// `%()` -> `\()` (ADR-0022).
class Tree {
  @constructor
  new(_ item, _ depth) {
    _item = item
    if (depth > 0) {
      let item2 = item + item
      depth = depth - 1
      _left = Tree.new(item2 - 1, depth)
      _right = Tree.new(item2, depth)
    }
  }

  check {
    if (_left == None) {
      return _item
    }

    return _item + _left.check - _right.check
  }
}

let minDepth = 4
let maxDepth = 12
let stretchDepth = maxDepth + 1

System.print("stretch tree of depth \(stretchDepth) check: " +
    "\(Tree.new(0, stretchDepth).check)")

let longLivedTree = Tree.new(0, maxDepth)

// iterations = 2 ** maxDepth
let iterations = 1
let d = 0
while (d < maxDepth) {
  iterations = iterations * 2
  d = d + 1
}

let depth = minDepth
while (depth < stretchDepth) {
  let check = 0
  let i = 1
  while (i <= iterations) {
    check = check + Tree.new(i, depth).check + Tree.new(0 - i, depth).check
    i = i + 1
  }

  System.print("\(iterations * 2) trees of depth \(depth) check: \(check)")
  iterations = iterations / 4
  depth = depth + 2
}

System.print(
    "long lived tree of depth \(maxDepth) check: \(longLivedTree.check)")

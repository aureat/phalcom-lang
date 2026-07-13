// Ported from wren/test/benchmark/binary_trees.wren. `_left == null` ->
// `_left == None` (Phalcom's absence value; unassigned fields read None,
// there is no surface `nil`). Both `for`-range forms replaced with
// while-counters (no range-literal parser production). String interpolation
// `%()` -> `\()` (ADR-0022).
class Tree {
  construct new(item, depth) {
    _item = item
    if (depth > 0) {
      var item2 = item + item
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

var minDepth = 4
var maxDepth = 12
var stretchDepth = maxDepth + 1

System.print("stretch tree of depth \(stretchDepth) check: " +
    "\(Tree.new(0, stretchDepth).check)")

var longLivedTree = Tree.new(0, maxDepth)

// iterations = 2 ** maxDepth
var iterations = 1
var d = 0
while (d < maxDepth) {
  iterations = iterations * 2
  d = d + 1
}

var depth = minDepth
while (depth < stretchDepth) {
  var check = 0
  var i = 1
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

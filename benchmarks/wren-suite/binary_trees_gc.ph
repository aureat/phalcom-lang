// Ported from wren/test/benchmark/binary_trees_gc.wren. Same as
// binary_trees.ph, minus explicit `System.gc()` calls — Phalcom has no
// `System.gc` primitive (nothing surfaces manual collection control; see
// docs/adr/accepted/0050-non-moving-mark-sweep-collector.md, in-flight). Kept as a
// separate file anyway: still exercises the same allocation-pressure shape
// (no `for (i in 1...1000) System.gc()` "give GC a shot" hook is the delta),
// useful once GC control lands to see if inserting it changes anything.
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

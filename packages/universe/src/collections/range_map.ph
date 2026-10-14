//! Persistent ordered balanced interval map.
//!
//! Provides `RangeMap<K, V>` mapping pairwise-disjoint `Interval<K>` regions
//! to values `V` with O(log n) point lookup and structural sharing.

from universe.collections.interval import Bound, Interval
from universe.collections.iteration import Iterator as CanonicalIterator, Iterable as CanonicalIterable

/// Internal persistent AVL tree node.
class RangeMapNode<K, V> {
  _interval: Interval<K>
  _value: V
  _left: Option<RangeMapNode<K, V>>
  _right: Option<RangeMapNode<K, V>>
  _height: Int

  @constructor
  new(
    _ interval: Interval<K>,
    _ value: V,
    left: Option<RangeMapNode<K, V>>,
    right: Option<RangeMapNode<K, V>>,
    height: Int
  ) {
    _interval = interval
    _value = value
    _left = left
    _right = right
    _height = height
  }

  interval -> Interval<K> { self._interval }
  value -> V { self._value }
  left -> Option<RangeMapNode<K, V>> { self._left }
  right -> Option<RangeMapNode<K, V>> { self._right }
  height -> Int { self._height }

  @class
  nodeHeight(_ node: Option<RangeMapNode<K, V>>) -> Int {
    match node {
      Option::Some(n) => n.height
      _ => 0
    }
  }

  @class
  balanceFactor(_ node: Option<RangeMapNode<K, V>>) -> Int {
    match node {
      Option::Some(n) => RangeMapNode.nodeHeight(n.left) - RangeMapNode.nodeHeight(n.right)
      _ => 0
    }
  }

  @class
  create(
    _ interval: Interval<K>,
    _ value: V,
    left: Option<RangeMapNode<K, V>>,
    right: Option<RangeMapNode<K, V>>
  ) -> RangeMapNode<K, V> {
    let lh = RangeMapNode.nodeHeight(left)
    let rh = RangeMapNode.nodeHeight(right)
    let h = (if lh > rh { lh } else { rh }) + 1
    RangeMapNode<K, V>.new(interval, value, left: left, right: right, height: h)
  }

  @class
  rotateRight(_ y: RangeMapNode<K, V>) -> RangeMapNode<K, V> {
    let x = y.left.unwrap
    let t2 = x.right
    let newY = RangeMapNode.create(y.interval, y.value, left: t2, right: y.right)
    RangeMapNode.create(x.interval, x.value, left: x.left, right: Option::Some(newY))
  }

  @class
  rotateLeft(_ x: RangeMapNode<K, V>) -> RangeMapNode<K, V> {
    let y = x.right.unwrap
    let t2 = y.left
    let newX = RangeMapNode.create(x.interval, x.value, left: x.left, right: t2)
    RangeMapNode.create(y.interval, y.value, left: Option::Some(newX), right: y.right)
  }

  @class
  rebalance(_ node: RangeMapNode<K, V>) -> RangeMapNode<K, V> {
    let bf = RangeMapNode.balanceFactor(Option::Some(node))
    if bf > 1 {
      let leftChild = node.left.unwrap
      if RangeMapNode.balanceFactor(node.left) < 0 {
        let rotatedLeft = RangeMapNode.rotateLeft(leftChild)
        let newSelf = RangeMapNode.create(node.interval, node.value, left: Option::Some(rotatedLeft), right: node.right)
        return RangeMapNode.rotateRight(newSelf)
      }
      return RangeMapNode.rotateRight(node)
    }
    if bf < -1 {
      let rightChild = node.right.unwrap
      if RangeMapNode.balanceFactor(node.right) > 0 {
        let rotatedRight = RangeMapNode.rotateRight(rightChild)
        let newSelf = RangeMapNode.create(node.interval, node.value, left: node.left, right: Option::Some(rotatedRight))
        return RangeMapNode.rotateLeft(newSelf)
      }
      return RangeMapNode.rotateLeft(node)
    }
    node
  }

  // --- Search & Insertion ---

  @class
  findPoint(_ node: Option<RangeMapNode<K, V>>, _ point: K) -> Option<V> {
    match node {
      Option::None => Option::None
      Option::Some(n) => {
        if n.interval.contains(point) {
          return Option::Some(n.value)
        }
        let cmp = match n.interval.lower {
          Unbounded => Ordering::Greater
          Included(l) => {
            let o = point <=> l
            if o == Ordering::Unordered {
              InvalidIntervalError.new("Point comparison yielded Ordering::Unordered").raise()
            }
            o
          }
          Excluded(l) => {
            let o = point <=> l
            if o == Ordering::Unordered {
              InvalidIntervalError.new("Point comparison yielded Ordering::Unordered").raise()
            }
            if o == Ordering::Equal { Ordering::Less } else { o }
          }
        }
        if cmp == Ordering::Less {
          RangeMapNode.findPoint(n.left, point)
        } else {
          RangeMapNode.findPoint(n.right, point)
        }
      }
    }
  }

  @class
  insert(
    _ node: Option<RangeMapNode<K, V>>,
    _ interval: Interval<K>,
    _ value: V
  ) -> RangeMapNode<K, V> {
    match node {
      Option::None => RangeMapNode.create(interval, value, left: Option::None, right: Option::None)
      Option::Some(n) => {
        if n.interval.overlaps(interval) {
          RangeOverlapError.new("Inserted interval overlaps with existing RangeMap entry").raise()
        }
        let ord = Interval.compareLower(interval.lower, n.interval.lower)
        let updated = if ord == Ordering::Less {
          let newLeft = RangeMapNode.insert(n.left, interval, value)
          RangeMapNode.create(n.interval, n.value, left: Option::Some(newLeft), right: n.right)
        } else {
          let newRight = RangeMapNode.insert(n.right, interval, value)
          RangeMapNode.create(n.interval, n.value, left: n.left, right: Option::Some(newRight))
        }
        RangeMapNode.rebalance(updated)
      }
    }
  }

  /// Rebuilds a pivot between ordered left/right trees without mutating either input.
  /// Recursing down the taller spine keeps the result AVL-balanced in O(height difference).
  @class
  joinWithPivot(
    _ left: Option<RangeMapNode<K, V>>,
    _ interval: Interval<K>,
    _ value: V,
    _ right: Option<RangeMapNode<K, V>>
  ) -> RangeMapNode<K, V> {
    let leftHeight = RangeMapNode.nodeHeight(left)
    let rightHeight = RangeMapNode.nodeHeight(right)
    if leftHeight > rightHeight + 1 {
      let leftRoot = left.unwrap
      let joinedRight = RangeMapNode.joinWithPivot(leftRoot.right, interval, value, right)
      let rebuilt = RangeMapNode.create(leftRoot.interval, leftRoot.value, left: leftRoot.left, right: Option::Some(joinedRight))
      return RangeMapNode.rebalance(rebuilt)
    }
    if rightHeight > leftHeight + 1 {
      let rightRoot = right.unwrap
      let joinedLeft = RangeMapNode.joinWithPivot(left, interval, value, rightRoot.left)
      let rebuilt = RangeMapNode.create(rightRoot.interval, rightRoot.value, left: Option::Some(joinedLeft), right: rightRoot.right)
      return RangeMapNode.rebalance(rebuilt)
    }
    RangeMapNode.create(interval, value, left: left, right: right)
  }

  /// Joins two AVL trees whose in-order keys are strictly separated.
  @class
  join(
    _ left: Option<RangeMapNode<K, V>>,
    _ right: Option<RangeMapNode<K, V>>
  ) -> Option<RangeMapNode<K, V>> {
    match left {
      Option::None => right
      Option::Some(leftRoot) => {
        match right {
          Option::None => Option::Some(leftRoot)
          Option::Some(rightRoot) => {
            let extracted = RangeMapNode.extractMin(rightRoot)
            let joined = RangeMapNode.joinWithPivot(
              Option::Some(leftRoot),
              extracted[1].interval,
              extracted[1].value,
              extracted[0]
            )
            Option::Some(joined)
          }
        }
      }
    }
  }

  /// Removes the smallest node, rebuilding only its search path.
  @class
  extractMin(_ node: RangeMapNode<K, V>) -> (Option<RangeMapNode<K, V>>, RangeMapNode<K, V>) {
    match node.left {
      Option::None => (node.right, node)
      Option::Some(leftRoot) => {
        let extracted = RangeMapNode.extractMin(leftRoot)
        let rebuilt = RangeMapNode.create(node.interval, node.value, left: extracted[0], right: node.right)
        let balanced = RangeMapNode.rebalance(rebuilt)
        (Option::Some(balanced), extracted[1])
      }
    }
  }

  /// Removes the largest node, rebuilding only its search path.
  @class
  extractMax(_ node: RangeMapNode<K, V>) -> (Option<RangeMapNode<K, V>>, RangeMapNode<K, V>) {
    match node.right {
      Option::None => (node.left, node)
      Option::Some(rightRoot) => {
        let extracted = RangeMapNode.extractMax(rightRoot)
        let rebuilt = RangeMapNode.create(node.interval, node.value, left: node.left, right: extracted[0])
        let balanced = RangeMapNode.rebalance(rebuilt)
        (Option::Some(balanced), extracted[1])
      }
    }
  }

  /// Finds the smallest node without rebuilding its search path.
  @class
  minimum(_ node: Option<RangeMapNode<K, V>>) -> Option<RangeMapNode<K, V>> {
    match node {
      Option::None => Option::None
      Option::Some(root) => match root.left {
        Option::None => Option::Some(root)
        Option::Some(_) => RangeMapNode.minimum(root.left)
      }
    }
  }

  /// Finds the greatest node without rebuilding its search path.
  @class
  maximum(_ node: Option<RangeMapNode<K, V>>) -> Option<RangeMapNode<K, V>> {
    match node {
      Option::None => Option::None
      Option::Some(root) => match root.right {
        Option::None => Option::Some(root)
        Option::Some(_) => RangeMapNode.maximum(root.right)
      }
    }
  }

  /// Splits the tree at a lower-bound key: left keys are strictly less than pivot.
  @class
  splitByLower(
    _ node: Option<RangeMapNode<K, V>>,
    _ pivot: Bound<K>
  ) -> (Option<RangeMapNode<K, V>>, Option<RangeMapNode<K, V>>) {
    match node {
      Option::None => (Option::None, Option::None)
      Option::Some(root) => {
        let order = Interval.compareLower(root.interval.lower, pivot)
        if order == Ordering::Less {
          let split = RangeMapNode.splitByLower(root.right, pivot)
          let left = RangeMapNode.joinWithPivot(root.left, root.interval, root.value, split[0])
          (Option::Some(left), split[1])
        } else {
          let split = RangeMapNode.splitByLower(root.left, pivot)
          let right = RangeMapNode.joinWithPivot(split[1], root.interval, root.value, root.right)
          (split[0], Option::Some(right))
        }
      }
    }
  }

  /// Counts a discarded middle range in O(k), where k is the affected entry count.
  @class
  countNodes(_ node: Option<RangeMapNode<K, V>>) -> Int {
    match node {
      Option::None => 0
      Option::Some(root) => 1 + RangeMapNode.countNodes(root.left) + RangeMapNode.countNodes(root.right)
    }
  }

  /// Removes the only node left of the lower cut that can cross into the range.
  @class
  removeLeftBoundary(
    _ left: Option<RangeMapNode<K, V>>,
    _ interval: Interval<K>
  ) -> (Option<RangeMapNode<K, V>>, Option<RangeMapNode<K, V>>, Int, Int) {
    match left {
      Option::None => (Option::None, Option::None, 0, 0)
      Option::Some(root) => {
        let extracted = RangeMapNode.extractMax(root)
        let mut leftResult: Option<RangeMapNode<K, V>> = extracted[0]
        let boundary = extracted[1]
        if not boundary.interval.overlaps(interval) {
          leftResult = Option::Some(RangeMapNode.joinWithPivot(leftResult, boundary.interval, boundary.value, Option::None))
          return (leftResult, Option::None, 0, 0)
        }

        let remainders = boundary.interval.remaindersAfterRemoval(interval)
        let mut rightBoundary: Option<RangeMapNode<K, V>> = Option::None
        let mut remainderCount = 0
        let mut i = 0
        while i < remainders.size {
          let rem = remainders.at(i)
          if Interval.compareLower(rem.lower, interval.lower) == Ordering::Less {
            leftResult = Option::Some(RangeMapNode.joinWithPivot(leftResult, rem, boundary.value, Option::None))
          } else {
            let emptyLeft: Option<RangeMapNode<K, V>> = Option::None
            let emptyRight: Option<RangeMapNode<K, V>> = Option::None
            rightBoundary = Option::Some(RangeMapNode.create(rem, boundary.value, left: emptyLeft, right: emptyRight))
          }
          remainderCount = remainderCount + 1
          i = i + 1
        }
        (leftResult, rightBoundary, 1, remainderCount)
      }
    }
  }

  /// Discards fully covered middle nodes and preserves the single right boundary.
  @class
  removeMiddleBoundary(
    _ middle: Option<RangeMapNode<K, V>>,
    _ left: Option<RangeMapNode<K, V>>,
    _ right: Option<RangeMapNode<K, V>>,
    _ interval: Interval<K>
  ) -> (Option<RangeMapNode<K, V>>, Option<RangeMapNode<K, V>>, Int, Int) {
    match middle {
      Option::None => (left, right, 0, 0)
      Option::Some(root) => {
        let mut affectedCount = RangeMapNode.countNodes(middle)
        let extracted = RangeMapNode.extractMax(root)
        let last = extracted[1]
        let mut leftResult: Option<RangeMapNode<K, V>> = left
        let mut rightResult: Option<RangeMapNode<K, V>> = right
        let mut remainderCount = 0
        let emptyRoot: Option<RangeMapNode<K, V>> = Option::None
        if last.interval.overlaps(interval) {
          let remainders = last.interval.remaindersAfterRemoval(interval)
          let mut i = 0
          while i < remainders.size {
            let rem = remainders.at(i)
            if Interval.compareLower(rem.lower, interval.lower) == Ordering::Less {
              leftResult = Option::Some(RangeMapNode.joinWithPivot(leftResult, rem, last.value, Option::None))
            } else {
              rightResult = Option::Some(RangeMapNode.joinWithPivot(emptyRoot, rem, last.value, rightResult))
            }
            remainderCount = remainderCount + 1
            i = i + 1
          }
        } else {
          // Defensive preservation for a degenerate ordered-domain boundary.
          rightResult = Option::Some(RangeMapNode.joinWithPivot(emptyRoot, last.interval, last.value, rightResult))
          affectedCount = affectedCount - 1
        }
        (leftResult, rightResult, affectedCount, remainderCount)
      }
    }
  }

}

/// Immutable persistent range map mapping disjoint intervals to values.
class RangeMap<K, V> is Collection {
  _root: Option<RangeMapNode<K, V>>
  _size: Int

  @constructor
  new() {
    _root = Option::None
    _size = 0
  }

  @constructor
  new(root: Option<RangeMapNode<K, V>>, size: Int) {
    _root = root
    _size = size
  }

  size -> Int { self._size }
  isEmpty -> Bool { self._size == 0 }

  get(_ point: K) -> Option<V> {
    RangeMapNode.findPoint(self._root, point)
  }

  containsPoint(_ point: K) -> Bool {
    match self.get(point) {
      Option::Some(_) => true
      _ => false
    }
  }

  /// Strict insertion: rejects invalid/empty/overlapping intervals and merges
  /// touching neighbors whose values compare equal.
  inserting(_ interval: Interval<K>, _ value: V) -> RangeMap<K, V> {
    if not interval.isValid or interval.isEmpty {
      InvalidIntervalError.new("Cannot insert invalid or empty interval into RangeMap").raise()
    }
    let split = RangeMapNode.splitByLower(self._root, interval.lower)
    let left = split[0]
    let right = split[1]
    let leftNeighbor = RangeMapNode.maximum(left)
    let rightNeighbor = RangeMapNode.minimum(right)

    match leftNeighbor {
      Option::Some(neighbor) => {
        if neighbor.interval.overlaps(interval) {
          RangeOverlapError.new("Inserted interval overlaps with existing RangeMap entry").raise()
        }
      }
      _ => {}
    }
    match rightNeighbor {
      Option::Some(neighbor) => {
        if neighbor.interval.overlaps(interval) {
          RangeOverlapError.new("Inserted interval overlaps with existing RangeMap entry").raise()
        }
      }
      _ => {}
    }

    // Perform all user-defined value equality callbacks before rebuilding or
    // publishing the new root. The source roots remain immutable on failure.
    let mergeLeft = match leftNeighbor {
      Option::Some(neighbor) => neighbor.interval.touchesWithoutGap(interval) and neighbor.value == value
      _ => false
    }
    let mergeRight = match rightNeighbor {
      Option::Some(neighbor) => neighbor.interval.touchesWithoutGap(interval) and neighbor.value == value
      _ => false
    }

    let mut newLeft = left
    let mut newRight = right
    let mut mergedInterval = interval
    let mut mergedCount = 0
    if mergeLeft {
      let neighbor = leftNeighbor.unwrap
      let extracted = RangeMapNode.extractMax(left.unwrap)
      newLeft = extracted[0]
      mergedInterval = neighbor.interval.unionIfContiguous(mergedInterval).unwrap
      mergedCount = mergedCount + 1
    }
    if mergeRight {
      let neighbor = rightNeighbor.unwrap
      let extracted = RangeMapNode.extractMin(right.unwrap)
      newRight = extracted[0]
      mergedInterval = mergedInterval.unionIfContiguous(neighbor.interval).unwrap
      mergedCount = mergedCount + 1
    }

    let newRoot = RangeMapNode.joinWithPivot(newLeft, mergedInterval, value, newRight)
    RangeMap<K, V>.new(root: Option::Some(newRoot), size: self._size + 1 - mergedCount)
  }

  /// Coverage subtraction: removes coverage of `interval`.
  removing(_ interval: Interval<K>) -> RangeMap<K, V> {
    if not interval.isValid {
      InvalidIntervalError.new("Cannot remove invalid interval from RangeMap").raise()
    }
    if interval.isEmpty or self.isEmpty {
      return self
    }

    // Split at the lower boundary. The greatest entry strictly to its left is
    // the only left-side entry that can overlap `interval`.
    let lowerSplit: (Option<RangeMapNode<K, V>>, Option<RangeMapNode<K, V>>) = match interval.lower {
      Unbounded => (Option::None, self._root)
      _ => RangeMapNode.splitByLower(self._root, interval.lower)
    }
    let leftEdit = RangeMapNode.removeLeftBoundary(lowerSplit[0], interval)
    let leftResult: Option<RangeMapNode<K, V>> = leftEdit[0]
    let afterLower: Option<RangeMapNode<K, V>> = lowerSplit[1]

    // The complement of the range's upper bound is the first lower key that
    // cannot overlap it. An unbounded upper range has no right partition.
    let upperSplit: (Option<RangeMapNode<K, V>>, Option<RangeMapNode<K, V>>) = match interval.upper {
      Unbounded => (afterLower, Option::None)
      Included(value) => RangeMapNode.splitByLower(afterLower, Bound<K>::Excluded(value))
      Excluded(value) => RangeMapNode.splitByLower(afterLower, Bound<K>::Included(value))
    }
    let middle: Option<RangeMapNode<K, V>> = upperSplit[0]
    let right: Option<RangeMapNode<K, V>> = upperSplit[1]
    let middleEdit = RangeMapNode.removeMiddleBoundary(middle, leftResult, right, interval)
    let leftBoundary: Option<RangeMapNode<K, V>> = leftEdit[1]
    let middleRight: Option<RangeMapNode<K, V>> = middleEdit[1]
    let rightResult = RangeMapNode.join(leftBoundary, middleRight)

    let root = RangeMapNode.join(middleEdit[0], rightResult)
    let newSize = self._size - leftEdit[2] - middleEdit[2] + leftEdit[3] + middleEdit[3]
    RangeMap<K, V>.new(root: root, size: newSize)
  }

  /// Overlay: replaces mapping over `interval` with `value`.
  overlaying(_ interval: Interval<K>, _ value: V) -> RangeMap<K, V> {
    if not interval.isValid {
      InvalidIntervalError.new("Cannot overlay invalid interval").raise()
    }
    if interval.isEmpty {
      return self
    }
    self.removing(interval).inserting(interval, value)
  }

  /// Snapshot array of normalized entries in sorted order.
  entries -> Array<(interval: Interval<K>, value: V)> {
    let result: Array<(interval: Interval<K>, value: V)> = [*self]
    result
  }

  iterator -> RangeMapIterator<K, V> {
    RangeMapIterator<K, V>.new(self._root)
  }

  ==(_ other) -> Bool {
    if not (other is RangeMap) { return false }
    let a = self.entries
    let b = other.entries
    if a.size != b.size { return false }
    let mut i = 0
    while i < a.size {
      let ea = a.at(i)
      let eb = b.at(i)
      if not (ea.interval == eb.interval) or not (ea.value == eb.value) {
        return false
      }
      i = i + 1
    }
    true
  }

  hash -> Int {
    let ents = self.entries
    let mut h = 17
    let mut i = 0
    while i < ents.size {
      let e = ents.at(i)
      h = h * 31 + e.interval.hash + e.value.hash
      i = i + 1
    }
    h
  }

  toString -> String {
    let ents = self.entries
    let mut s = "RangeMap("
    let mut i = 0
    while i < ents.size {
      if i > 0 { s = s + ", " }
      let e = ents.at(i)
      s = s + "\(e.interval): \(e.value)"
      i = i + 1
    }
    s + ")"
  }
}

class RangeMapIterator<K, V> {
  mut _pending: List<RangeMapNode<K, V>>

  @constructor
  new(_ root: Option<RangeMapNode<K, V>>) {
    _pending = List<RangeMapNode<K, V>>.new()
    self.pushLeftSpine(root)
  }

  pushLeftSpine(_ node: Option<RangeMapNode<K, V>>) {
    let mut current = node
    while current.isSome {
      let entry = current.unwrap
      _pending.append(entry)
      current = entry.left
    }
  }

  next -> Option<(interval: Interval<K>, value: V)> {
    let candidate = _pending.popLast
    if candidate.isNone { return Option::None }
    let entry = candidate.unwrap
    self.pushLeftSpine(entry.right)
    Option::Some((interval: entry.interval, value: entry.value))
  }

  iterator { self }
}

impl<K, V> CanonicalIterator for RangeMapIterator<K, V> {
  type Item = (interval: Interval<K>, value: V)
}

impl<K, V> CanonicalIterable for RangeMap<K, V> {
  type Item = (interval: Interval<K>, value: V)
  type Iterator = RangeMapIterator<K, V>
  iterator -> RangeMapIterator<K, V> { self.iterator }
}

export RangeMap, RangeMapIterator

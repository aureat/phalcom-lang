//! Persistent ordered balanced interval map.
//!
//! Provides `RangeMap<K, V>` mapping pairwise-disjoint `Interval<K>` regions
//! to values `V` with O(log n) point lookup and structural sharing.

from universe.collections.interval import Bound, Interval

/// Internal persistent AVL tree node.
class _RangeMapNode<K, V> {
  _interval: Interval<K>
  _value: V
  _left: Option<_RangeMapNode<K, V>>
  _right: Option<_RangeMapNode<K, V>>
  _height: Int

  @constructor
  new(
    _ interval: Interval<K>,
    _ value: V,
    left: Option<_RangeMapNode<K, V>>,
    right: Option<_RangeMapNode<K, V>>,
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
  left -> Option<_RangeMapNode<K, V>> { self._left }
  right -> Option<_RangeMapNode<K, V>> { self._right }
  height -> Int { self._height }

  @class
  nodeHeight(_ node: Option<_RangeMapNode<K, V>>) -> Int {
    match node {
      Option::Some(n) => n.height
      _ => 0
    }
  }

  @class
  balanceFactor(_ node: Option<_RangeMapNode<K, V>>) -> Int {
    match node {
      Option::Some(n) => _RangeMapNode.nodeHeight(n.left) - _RangeMapNode.nodeHeight(n.right)
      _ => 0
    }
  }

  @class
  create(
    _ interval: Interval<K>,
    _ value: V,
    left: Option<_RangeMapNode<K, V>>,
    right: Option<_RangeMapNode<K, V>>
  ) -> _RangeMapNode<K, V> {
    let lh = _RangeMapNode.nodeHeight(left)
    let rh = _RangeMapNode.nodeHeight(right)
    let h = (if lh > rh { lh } else { rh }) + 1
    _RangeMapNode<K, V>.new(interval, value, left: left, right: right, height: h)
  }

  @class
  rotateRight(_ y: _RangeMapNode<K, V>) -> _RangeMapNode<K, V> {
    let x = y.left.unwrap
    let t2 = x.right
    let newY = _RangeMapNode.create(y.interval, y.value, left: t2, right: y.right)
    _RangeMapNode.create(x.interval, x.value, left: x.left, right: Option::Some(newY))
  }

  @class
  rotateLeft(_ x: _RangeMapNode<K, V>) -> _RangeMapNode<K, V> {
    let y = x.right.unwrap
    let t2 = y.left
    let newX = _RangeMapNode.create(x.interval, x.value, left: x.left, right: t2)
    _RangeMapNode.create(y.interval, y.value, left: Option::Some(newX), right: y.right)
  }

  @class
  rebalance(_ node: _RangeMapNode<K, V>) -> _RangeMapNode<K, V> {
    let bf = _RangeMapNode.balanceFactor(Option::Some(node))
    if bf > 1 {
      let leftChild = node.left.unwrap
      if _RangeMapNode.balanceFactor(node.left) < 0 {
        let rotatedLeft = _RangeMapNode.rotateLeft(leftChild)
        let newSelf = _RangeMapNode.create(node.interval, node.value, left: Option::Some(rotatedLeft), right: node.right)
        return _RangeMapNode.rotateRight(newSelf)
      }
      return _RangeMapNode.rotateRight(node)
    }
    if bf < -1 {
      let rightChild = node.right.unwrap
      if _RangeMapNode.balanceFactor(node.right) > 0 {
        let rotatedRight = _RangeMapNode.rotateRight(rightChild)
        let newSelf = _RangeMapNode.create(node.interval, node.value, left: node.left, right: Option::Some(rotatedRight))
        return _RangeMapNode.rotateLeft(newSelf)
      }
      return _RangeMapNode.rotateLeft(node)
    }
    node
  }

  // --- Search & Insertion ---

  @class
  findPoint(_ node: Option<_RangeMapNode<K, V>>, _ point: K) -> Option<V> {
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
          _RangeMapNode.findPoint(n.left, point)
        } else {
          _RangeMapNode.findPoint(n.right, point)
        }
      }
    }
  }

  @class
  insert(
    _ node: Option<_RangeMapNode<K, V>>,
    _ interval: Interval<K>,
    _ value: V
  ) -> _RangeMapNode<K, V> {
    match node {
      Option::None => _RangeMapNode.create(interval, value, left: Option::None, right: Option::None)
      Option::Some(n) => {
        if n.interval.overlaps(interval) {
          RangeOverlapError.new("Inserted interval overlaps with existing RangeMap entry").raise()
        }
        let ord = Interval.compareLower(interval.lower, n.interval.lower)
        let updated = if ord == Ordering::Less {
          let newLeft = _RangeMapNode.insert(n.left, interval, value)
          _RangeMapNode.create(n.interval, n.value, left: Option::Some(newLeft), right: n.right)
        } else {
          let newRight = _RangeMapNode.insert(n.right, interval, value)
          _RangeMapNode.create(n.interval, n.value, left: n.left, right: Option::Some(newRight))
        }
        _RangeMapNode.rebalance(updated)
      }
    }
  }

  // Collect in-order (interval, value) pairs into List
  @class
  inOrderCollect(_ node: Option<_RangeMapNode<K, V>>, _ acc: List<(interval: Interval<K>, value: V)>) {
    match node {
      Option::None => {}
      Option::Some(n) => {
        _RangeMapNode.inOrderCollect(n.left, acc)
        acc.append((interval: n.interval, value: n.value))
        _RangeMapNode.inOrderCollect(n.right, acc)
      }
    }
  }
}

/// Immutable persistent range map mapping disjoint intervals to values.
class RangeMap<K, V> is Collection {
  _root: Option<_RangeMapNode<K, V>>
  _size: Int

  @constructor
  new() {
    _root = Option::None
    _size = 0
  }

  @constructor
  new(root: Option<_RangeMapNode<K, V>>, size: Int) {
    _root = root
    _size = size
  }

  @class
  new() -> RangeMap<K, V> {
    RangeMap<K, V>.new()
  }

  size -> Int { self._size }
  isEmpty -> Bool { self._size == 0 }

  get(_ point: K) -> Option<V> {
    _RangeMapNode.findPoint(self._root, point)
  }

  containsPoint(_ point: K) -> Bool {
    match self.get(point) {
      Option::Some(_) => true
      _ => false
    }
  }

  /// Strict insertion: rejects invalid/empty intervals and any overlap.
  inserting(_ interval: Interval<K>, _ value: V) -> RangeMap<K, V> {
    if not interval.isValid or interval.isEmpty {
      InvalidIntervalError.new("Cannot insert invalid or empty interval into RangeMap").raise()
    }
    let newRoot = _RangeMapNode.insert(self._root, interval, value)
    RangeMap<K, V>.new(root: Option::Some(newRoot), size: self._size + 1)
  }

  /// Coverage subtraction: removes coverage of `interval`.
  removing(_ interval: Interval<K>) -> RangeMap<K, V> {
    if not interval.isValid {
      InvalidIntervalError.new("Cannot remove invalid interval from RangeMap").raise()
    }
    if interval.isEmpty or self.isEmpty {
      return self
    }
    let allEntries = self.entries
    let mut newMap = RangeMap<K, V>.new()
    let mut i = 0
    while i < allEntries.size {
      let entry = allEntries.at(i)
      let remainders = entry.interval.remaindersAfterRemoval(interval)
      let mut j = 0
      while j < remainders.size {
        let rem = remainders.at(j)
        newMap = newMap.inserting(rem, entry.value)
        j = j + 1
      }
      i = i + 1
    }
    newMap
  }

  /// Overlay: replaces mapping over `interval` with `value`.
  overlaying(_ interval: Interval<K>, _ value: V) -> RangeMap<K, V> {
    if not interval.isValid or interval.isEmpty {
      InvalidIntervalError.new("Cannot overlay invalid or empty interval").raise()
    }
    self.removing(interval).inserting(interval, value)
  }

  /// Normalized entries list, coalescing adjacent contiguous intervals with equal values.
  entries -> List<(interval: Interval<K>, value: V)> {
    let raw = List<(interval: Interval<K>, value: V)>.new()
    _RangeMapNode.inOrderCollect(self._root, raw)
    if raw.size <= 1 { return raw }

    let normalized = List<(interval: Interval<K>, value: V)>.new()
    let mut current = raw.at(0)
    let mut i = 1
    while i < raw.size {
      let nextEntry = raw.at(i)
      if current.value == nextEntry.value and current.interval.touchesWithoutGap(nextEntry.interval) {
        let merged = current.interval.unionIfContiguous(nextEntry.interval).unwrap
        current = (interval: merged, value: current.value)
      } else {
        normalized.append(current)
        current = nextEntry
      }
      i = i + 1
    }
    normalized.append(current)
    normalized
  }

  iterator -> RangeMapIterator<K, V> {
    RangeMapIterator<K, V>.new(self.entries)
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
      if ea.interval != eb.interval or ea.value != eb.value {
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
  mut _entries: List<(interval: Interval<K>, value: V)>
  mut _index: Int

  @constructor
  new(_ entries: List<(interval: Interval<K>, value: V)>) {
    _entries = entries
    _index = 0
  }

  next -> Option<(interval: Interval<K>, value: V)> {
    if _index < _entries.size {
      let item = _entries.at(_index)
      _index = _index + 1
      return Option::Some(item)
    }
    return Option::None
  }
}

export RangeMap, RangeMapIterator

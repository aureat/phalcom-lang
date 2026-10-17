from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable
from universe.text.builder import StringBuilder

native class List<T> is Collection {
  @class
  native new() -> List<T>
  @class
  internal native rawWithCapacity(_ capacity: Int) -> List<T>
  internal native rawLength -> Int
  internal native rawAt(_ index: Int) -> Dynamic
  internal native rawSet(_ index: Int, _ value: Dynamic) -> Dynamic
  internal native rawPush(_ value: Dynamic) -> Dynamic
  internal native rawReplaceSlice(_ start: Int, _ end: Int, _ replacement: List<T>) -> Dynamic
  native toString -> String

  /// Creates an empty list after reserving capacity under the shared List
  /// backing-storage budget. The reservation happens before callers fill it.
  @class
  withCapacity(_ capacity: Int) -> List<T> {
    if capacity < 0 { ArgumentError.new("List capacity must be nonnegative").raise() }
    return self.rawWithCapacity(capacity)
  }

  size { self.rawLength }

  first {
    if self.size == 0 { return None }
    return Some(self.at(0))
  }

  last {
    if self.size == 0 { return None }
    return Some(self.at(self.size - 1))
  }

  /// Returns a fresh iterator over the list's current elements.
  iterator -> ListIterator<T> { ListIterator.new(self) }

  at(_ i) {
    return self.rawAt(i)
  }

  get(_ index: Int) -> Option<T> {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return Some(raw)
    }
    return None
  }

  private sliceByRange(_ range) {
    return range.rawSliceBounds(self.size).match(
      ok: |bounds| {
        let mut start = bounds[0]
        let mut end = bounds[1]
        if start > end {
          end = start
        }
        let mut result = List<T>.new()
        let mut i = start
        while i < end {
          result.rawPush(self.rawAt(i))
          i = i + 1
        }
        result
      },
      err: |error| { error.raise() }
    )
  }

  [_ index] {
    if index is Range {
      return self.sliceByRange(index)
    }
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    throw IndexError.new("List index out of range")
  }

  [_ index, default] {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    return default
  }

  get(_ index, orElse) {
    let mut raw = self.rawAt(index)
    let mut len = self.size
    let mut i = index
    if i < 0 { i = len + i }
    if i >= 0 and i < len {
      return raw
    }
    return orElse.call(index)
  }

  append(_ value) {
    self.rawPush(value)
    return ()
  }

  /// Joins all elements into a single string with an optional separator.
  join() -> String {
    self.join("")
  }

  /// Joins all elements into a single string separated by `sep`.
  join(_ sep) -> String {
    let mut sb = StringBuilder.new()
    let mut i = 0
    let total = self.size
    while i < total {
      if i > 0 {
        sb.append(sep)
      }
      let item = self.rawAt(i)
      sb.append(item.toString)
      i = i + 1
    }
    sb.toString
  }

  prepend(_ value) {
    let mut oneElementList = List<T>.new()
    oneElementList.rawPush(value)
    self.rawReplaceSlice(0, 0, oneElementList)
    return ()
  }

  clear {
    let mut emptyList = List<T>.new()
    self.rawReplaceSlice(0, self.size, emptyList)
    return ()
  }

  insert(_ value, at index) {
    let mut n = self.size
    let mut p = index
    if p < 0 { p = n + p }
    if p < 0 or p > n {
      return Result::Error(IndexError.new("List#insert: index out of bounds"))
    }
    let mut oneElementList = List<T>.new()
    oneElementList.rawPush(value)
    self.rawReplaceSlice(p, p, oneElementList)
    let mut ok: Result<Unit, Error> = Result::Ok(())
    return ok
  }

  remove(at index) {
    let mut n = self.size
    let mut p = index
    if p < 0 { p = n + p }
    if p < 0 or p >= n {
      return Result::Error(IndexError.new("List#remove: index out of bounds"))
    }
    let mut captured = self.rawAt(p)
    let mut emptyList = List<T>.new()
    self.rawReplaceSlice(p, p + 1, emptyList)
    let mut ok: Result<T, Error> = Result::Ok(captured)
    return ok
  }

  popFirst {
    let mut n = self.size
    if n == 0 { return None }
    let mut captured = self.rawAt(0)
    let mut emptyList = List<T>.new()
    self.rawReplaceSlice(0, 1, emptyList)
    return Some(captured)
  }

  popLast {
    let mut n = self.size
    if n == 0 { return None }
    let mut captured = self.rawAt(n - 1)
    let mut emptyList = List<T>.new()
    self.rawReplaceSlice(n - 1, n, emptyList)
    return Some(captured)
  }

  removeAll(where predicate) {
    let mut retained = List<Dynamic>.new()
    let mut count = 0
    let mut i = 0
    let mut n = self.size
    while i < n {
      let mut x = self.rawAt(i)
      if predicate.call(x) {
        count = count + 1
      } else {
        retained.rawPush(x)
      }
      i = i + 1
    }
    self.rawReplaceSlice(0, self.size, retained)
    return count
  }

  swap(first a, second b) {
    let mut n = self.size
    let mut idxA = a
    if idxA < 0 { idxA = n + idxA }
    if idxA < 0 or idxA >= n {
      return Result::Error(IndexError.new("List#swap: first index out of bounds"))
    }
    let mut idxB = b
    if idxB < 0 { idxB = n + idxB }
    if idxB < 0 or idxB >= n {
      return Result::Error(IndexError.new("List#swap: second index out of bounds"))
    }
    if idxA == idxB {
      let mut ok: Result<Unit, Error> = Result::Ok(())
      return ok
    }
    let mut valA = self.rawAt(idxA)
    let mut valB = self.rawAt(idxB)
    self.rawSet(idxA, valB)
    self.rawSet(idxB, valA)
    let mut ok: Result<Unit, Error> = Result::Ok(())
    return ok
  }

  /// Replaces the element at `i` and returns the list.
  at(_ i, put) {
    let mut len = self.size
    let mut norm = i
    if norm < 0 { norm = len + norm }
    if norm < 0 or norm >= len {
      throw IndexError.new("Expected an in-range index, got an out-of-range Number")
    }
    self.rawSet(i, put)
    return self
  }

  /// Replaces the selected range with elements from a list.
  replace(_ range, with replacements) {
    if range is not Range {
      return Result::Error(SliceError.new("List#replace: first argument must be a Range"))
    }
    if replacements is not List {
      return Result::Error(SliceError.new("List#replace: replacement must be a List"))
    }
    return range.rawSliceBounds(self.size).match(
      ok: |bounds| {
        let mut start = bounds[0]
        let mut end = bounds[1]
        if start > end {
          end = start
        }
        self.rawReplaceSlice(start, end, replacements)
        let mut ok: Result<Unit, Error> = Result::Ok(())
        ok
      },
      err: |error| { Result::Error(error) }
    )
  }

  /// Assigns an element or replaces a range through the subscript protocol.
  [_ i]=(_ val) {
    if i is Range {
      return self.replace(i, with: val).unwrap
    }
    return self.at(i, put: val)
  }

  /// Compares lists by size and corresponding element equality.
  ==(_ other) {
    if other is List {
      let mut same = self.size == other.size
      let mut i = 0
      while same and i < self.size {
        same = self.at(i) == other.at(i)
        i = i + 1
      }
      return same
    }
    return false
  }

  /// Returns the inverse of structural list equality.
  !=(_ other) {
    return not (self == other)
  }

  toList { self }

  toSet { self.iterator.toSet }

  toMap { self.iterator.toMap }

  toMap(merging block) { self.iterator.toMap(merging: block) }
}

class ListIterator<T> {
  mut _values: List<T>
  mut _index: Int

  @constructor new(_ values: List<T>) {
    _values = values
    _index = 0
  }

  next -> Option<T> {
    if _index < _values.size {
      let mut value = _values[_index]
      _index = _index + 1
      return Some(value)
    }
    None
  }

  iterator { self }
}

impl<T> CanonicalIterator for ListIterator<T> {
  type Item = T
}

impl<T> CanonicalIterable for List<T> {
  type Item = T
  type Iterator = ListIterator<T>
  iterator -> ListIterator<T> { ListIterator.new(self) }
}

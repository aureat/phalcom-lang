```ph
class Countdown is Iterable {
  @constructor
  from(_ n) { _n = n }

  iterate -> Option<Int> {
		Option(_n >= 0)
  }

  iterate(_ cursor: Int) -> Option<Int> {
    const next = cursor - 1
		(next >= 0).optionally(next)
  }

  iteratorValue(_ cursor: Int) -> Int {
    cursor
  }
}
```

```ph
enum InterleaveCursor {
  Left(_ index: Int)
  Right(_ index: Int)
}

class Interleave<T> is Iterable {
  @constructor
  new(left: List<T>, right: List<T>) {
    _left = left
    _right = right
  }

  iterate -> Option<InterleaveCursor> {
    if not _left.isEmpty {
      Some(InterleaveCursor::Left(0))
    } else if _right.isNotEmpty {
      Some(InterleaveCursor::Right(0))
    } else {
      None
    }
  }

  iterate(_ cursor: InterleaveCursor) -> Option<InterleaveCursor> {
    match cursor {
      Left(index) => {
        if index < _right.size {
          Some(InterleaveCursor::Right(index))
        } else if index + 1 < _left.size {
          Some(InterleaveCursor::Left(index + 1))
        } else {
          None
        }
      }

      Right(index) => {
        if index + 1 < _left.size {
          Some(InterleaveCursor::Left(index + 1))
        } else if index + 1 < _right.size {
          Some(InterleaveCursor::Right(index + 1))
        } else {
          None
        }
      }
    }
  }

  iteratorValue(_ cursor: InterleaveCursor) -> T {
    match cursor {
      Left(index)  => _left[index]
      Right(index) => _right[index]
    }
  }
}
```
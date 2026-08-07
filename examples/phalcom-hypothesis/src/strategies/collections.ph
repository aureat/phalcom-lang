// Collection and sum-type strategies.

import Strategy from "strategies/strategy"
import base from "strategies/combinators"
import DrawData from "choices/data"
import errors from "core/errors"

class _ListStrategy<T> is base.StrategyBase<List<T>> {
  @constructor
  new(elements: Strategy<T>, minSize: Int, maxSize: Int) {
    _CollectionBounds.validate(
      kind: "list",
      minSize: minSize,
      maxSize: maxSize
    )
    _elements = elements
    _minSize = minSize
    _maxSize = maxSize
  }

  draw(data: DrawData) -> List<T> {
    return data.withSpan(label: #list, discardable: false) {
      const length = data.drawInt(
        min: _minSize,
        max: _maxSize,
        shrinkTowards: _minSize,
        label: Some.new(#length)
      )
      const values = List.new()
      let index = 0
      while index < length {
        const value = data.withSpan(label: #element, discardable: true) {
          _elements.draw(data)
        }
        values.add(value)
        index++
      }
      return values
    }
  }

  fingerprint -> String {
    return "list(" + _elements.fingerprint + "," +
      _minSize.toString + "," + _maxSize.toString + ")"
  }
}

class _SetStrategy<T> is base.StrategyBase<Set<T>> {
  @constructor
  new(elements: Strategy<T>, minSize: Int, maxSize: Int) {
    _CollectionBounds.validate(
      kind: "set",
      minSize: minSize,
      maxSize: maxSize
    )
    _elements = elements
    _minSize = minSize
    _maxSize = maxSize
  }

  draw(data: DrawData) -> Set<T> {
    return data.withSpan(label: #set, discardable: false) {
      const target = data.drawInt(
        min: _minSize,
        max: _maxSize,
        shrinkTowards: _minSize,
        label: Some.new(#length)
      )
      const values = Set.new()
      let attempts = 0
      const maxAttempts = 100 + (target * 10)
      while values.size < target and attempts < maxAttempts {
        const value = data.withSpan(label: #element, discardable: true) {
          _elements.draw(data)
        }
        if values.includes(value) {
          data.recordRejection("set element was not unique")
        } else {
          values.add(value)
        }
        attempts++
      }
      if values.size < target {
        throw errors._RejectedExample.new(
          "set strategy could not generate enough unique values"
        )
      }
      return values
    }
  }

  fingerprint -> String {
    return "set(" + _elements.fingerprint + "," +
      _minSize.toString + "," + _maxSize.toString + ")"
  }
}

class _MapStrategy<K, V> is base.StrategyBase<Map<K, V>> {
  @constructor
  new(
    keys: Strategy<K>,
    values: Strategy<V>,
    minSize: Int,
    maxSize: Int
  ) {
    _CollectionBounds.validate(
      kind: "map",
      minSize: minSize,
      maxSize: maxSize
    )
    _keys = keys
    _values = values
    _minSize = minSize
    _maxSize = maxSize
  }

  draw(data: DrawData) -> Map<K, V> {
    return data.withSpan(label: #map, discardable: false) {
      const target = data.drawInt(
        min: _minSize,
        max: _maxSize,
        shrinkTowards: _minSize,
        label: Some.new(#length)
      )
      const out = Map.new()
      const seen = Set.new()
      let attempts = 0
      const maxAttempts = 100 + (target * 10)
      while seen.size < target and attempts < maxAttempts {
        data.withSpan(label: #entry, discardable: true) {
          const key = _keys.draw(data)
          const value = _values.draw(data)
          if seen.includes(key) {
            data.recordRejection("map key was not unique")
          } else {
            seen.add(key)
            out.at(key, put: value)
          }
        }
        attempts++
      }
      if seen.size < target {
        throw errors._RejectedExample.new(
          "map strategy could not generate enough unique keys"
        )
      }
      return out
    }
  }

  fingerprint -> String {
    return "map(" + _keys.fingerprint + "," + _values.fingerprint + "," +
      _minSize.toString + "," + _maxSize.toString + ")"
  }
}

class _TupleStrategy is base.StrategyBase<Tuple> {
  @constructor
  new(elements: List<Strategy<Any>>) {
    _elements = base._StrategyCopies.list(elements)
  }

  draw(data: DrawData) -> Tuple {
    return data.withSpan(label: #tuple, discardable: false) {
      const values = List.new()
      let index = 0
      for strategy in _elements {
        const value = data.withSpan(label: #element, discardable: false) {
          strategy.draw(data)
        }
        values.add(value)
        index++
      }
      return Tuple.__fromList(values)
    }
  }

  fingerprint -> String {
    const parts = List.new()
    for strategy in _elements {
      parts.add(strategy.fingerprint)
    }
    return "tuple(" + parts.join(",") + ")"
  }
}

class _OptionStrategy<T> is base.StrategyBase<Option<T>> {
  @constructor
  new(value: Strategy<T>) {
    _value = value
  }

  draw(data: DrawData) -> Option<T> {
    const present = data.drawBool(
      shrinkTowards: false,
      label: Some.new(#present)
    )
    if not present {
      return None
    }
    return Some.new(_value.draw(data))
  }

  fingerprint -> String => "option(" + _value.fingerprint + ")"
}

class _ResultStrategy<T, E> is base.StrategyBase<Result<T, E>> {
  @constructor
  new(ok: Strategy<T>, error: Strategy<E>) {
    _ok = ok
    _error = error
  }

  draw(data: DrawData) -> Result<T, E> {
    const success = data.drawBool(
      shrinkTowards: true,
      label: Some.new(#result)
    )
    if success {
      return Ok.new(_ok.draw(data))
    }
    return Err.new(_error.draw(data))
  }

  fingerprint -> String {
    return "result(" + _ok.fingerprint + "," + _error.fingerprint + ")"
  }
}

class _CollectionBounds {
  @class
  validate(kind: String, minSize: Int, maxSize: Int) -> None {
    if minSize < 0 or minSize > maxSize {
      throw errors._InvalidStrategy.new(
        "invalid " + kind + " size bounds"
      )
    }
  }
}

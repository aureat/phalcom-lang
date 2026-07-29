// Strategy combinators shared by every concrete strategy implementation.

import Strategy from "strategies/strategy"
import DrawData from "choices/data"
import errors from "core/errors"

class StrategyBase<T> {
  draw(data: DrawData) -> T {
    throw errors._InvalidStrategy.new("Strategy.draw(_) is abstract")
  }

  map<U>(transform: [T] -> U) -> Strategy<U> {
    return _MappedStrategy.new(inner: self, transform: transform)
  }

  filter(predicate: [T] -> Bool) -> Strategy<T> {
    return _FilteredStrategy.new(
      inner: self,
      predicate: predicate,
      maxAttempts: 100
    )
  }

  filter(predicate: [T] -> Bool, maxAttempts: Int) -> Strategy<T> {
    return _FilteredStrategy.new(
      inner: self,
      predicate: predicate,
      maxAttempts: maxAttempts
    )
  }

  flatMap<U>(transform: [T] -> Strategy<U>) -> Strategy<U> {
    return _FlatMappedStrategy.new(inner: self, transform: transform)
  }

  named(label: Symbol) -> Strategy<T> {
    return _NamedStrategy.new(inner: self, label: label)
  }

  label -> Option<Symbol> => None

  fingerprint -> String => self.class.name.toString
}

class _JustStrategy<T> is StrategyBase<T> {
  @constructor
  new(value: T) {
    _value = value
  }

  draw(data: DrawData) -> T => _value

  fingerprint -> String {
    return "just(" + _value.class.name.toString + ")"
  }
}

class _MappedStrategy<T, U> is StrategyBase<U> {
  @constructor
  new(inner: Strategy<T>, transform: [T] -> U) {
    _inner = inner
    _transform = transform
  }

  draw(data: DrawData) -> U {
    return _transform.call(_inner.draw(data))
  }

  fingerprint -> String => "map(" + _inner.fingerprint + ")"
}

class _FilteredStrategy<T> is StrategyBase<T> {
  @constructor
  new(
    inner: Strategy<T>,
    predicate: [T] -> Bool,
    maxAttempts: Int
  ) {
    if maxAttempts <= 0 {
      throw errors._InvalidStrategy.new(
        "filter maxAttempts must be greater than zero"
      )
    }
    _inner = inner
    _predicate = predicate
    _maxAttempts = maxAttempts
  }

  draw(data: DrawData) -> T {
    let attempt = 0
    while attempt < _maxAttempts {
      const value = _inner.draw(data)
      if _predicate.call(value) {
        return value
      }
      data.recordRejection(
        "filtered candidate did not satisfy the predicate"
      )
      attempt++
    }

    throw errors._RejectedExample.new(
      "strategy filter rejected every candidate"
    )
  }

  fingerprint -> String => "filter(" + _inner.fingerprint + ")"
}

class _FlatMappedStrategy<T, U> is StrategyBase<U> {
  @constructor
  new(inner: Strategy<T>, transform: [T] -> Strategy<U>) {
    _inner = inner
    _transform = transform
  }

  draw(data: DrawData) -> U {
    const seed = _inner.draw(data)
    const derived = _transform.call(seed)
    if not derived.respondsTo(#draw) {
      throw errors._InvalidStrategy.new(
        "flatMap transform must return a Strategy"
      )
    }
    return derived.draw(data)
  }

  fingerprint -> String => "flatMap(" + _inner.fingerprint + ")"
}

class _NamedStrategy<T> is StrategyBase<T> {
  @constructor
  new(inner: Strategy<T>, label: Symbol) {
    _inner = inner
    _label = label
  }

  draw(data: DrawData) -> T {
    return data.withSpan(label: _label, discardable: false) {
      return data.withLabel(_label) {
        _inner.draw(data)
      }
    }
  }

  label -> Option<Symbol> => Some.new(_label)

  fingerprint -> String {
    return "named(" + _label.toString + "," + _inner.fingerprint + ")"
  }
}

class _OneOfStrategy<T> is StrategyBase<T> {
  @constructor
  new(strategies: List<Strategy<T>>) {
    if strategies.size == 0 {
      throw errors._InvalidStrategy.new(
        "oneOf requires at least one strategy"
      )
    }
    _strategies = _StrategyCopies.list(strategies)
  }

  draw(data: DrawData) -> T {
    const index = data.drawIndex(
      size: _strategies.size,
      shrinkTowards: 0,
      label: Some.new(#branch)
    )
    return _strategies.at(index).draw(data)
  }

  fingerprint -> String {
    const parts = List.new()
    for strategy in _strategies {
      parts.add(strategy.fingerprint)
    }
    return "oneOf(" + parts.join(",") + ")"
  }
}

class _StrategyCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}

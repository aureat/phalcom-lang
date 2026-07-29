// Explicit composite draws, deferred factories, and size-bounded recursion.

import Strategy from "strategies/strategy"
import base from "strategies/combinators"
import DrawData from "choices/data"
import errors from "core/errors"

class Draw {
  @constructor
  new(data: DrawData) {
    _data = data
  }

  from<T>(strategy: Strategy<T>) -> T {
    return strategy.draw(_data)
  }

  size -> Int => _data.size
}

class _BuildStrategy<T> is base.StrategyBase<T> {
  @constructor
  new(builder: [Draw] -> T) {
    _builder = builder
  }

  draw(data: DrawData) -> T {
    return data.withSpan(label: #build, discardable: false) {
      _builder.call(Draw.new(data))
    }
  }

  fingerprint -> String => "build"
}

class _DeferredStrategy<T> is base.StrategyBase<T> {
  @constructor
  new(factory: [] -> Strategy<T>) {
    _factory = factory
  }

  draw(data: DrawData) -> T {
    const resolved = _factory.call()
    if not resolved.respondsTo(#draw) {
      throw errors._InvalidStrategy.new(
        "deferred factory must return a Strategy"
      )
    }
    return resolved.draw(data)
  }

  fingerprint -> String => "deferred"
}

class _SizedStrategy<T> is base.StrategyBase<T> {
  @constructor
  new(inner: Strategy<T>, size: Int) {
    if size < 0 {
      throw errors._InvalidStrategy.new(
        "strategy generation size cannot be negative"
      )
    }
    _inner = inner
    _size = size
  }

  draw(data: DrawData) -> T {
    return data.withGenerationSize(_size) {
      _inner.draw(data)
    }
  }

  fingerprint -> String {
    return "sized(" + _size.toString + "," + _inner.fingerprint + ")"
  }
}

class _RecursiveStrategy<T> is base.StrategyBase<T> {
  @constructor
  new(
    base: Strategy<T>,
    extend: [Strategy<T>] -> Strategy<T>
  ) {
    _base = base
    _extend = extend
  }

  draw(data: DrawData) -> T {
    if data.size == 0 {
      return _base.draw(data)
    }

    const expand = data.drawBool(
      shrinkTowards: false,
      label: Some.new(#recursive)
    )
    if not expand {
      return _base.draw(data)
    }

    const child = _SizedStrategy.new(
      inner: self,
      size: data.size - 1
    )
    const extended = _extend.call(child)
    if not extended.respondsTo(#draw) {
      throw errors._InvalidStrategy.new(
        "recursive extension must return a Strategy"
      )
    }

    return data.withSpan(label: #recursiveBranch, discardable: true) {
      return data.withGenerationSize(data.size - 1) {
        extended.draw(data)
      }
    }
  }

  fingerprint -> String => "recursive(" + _base.fingerprint + ")"
}

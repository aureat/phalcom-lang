// Primitive strategies over typed DrawData operations.

import Strategy from "strategies/strategy"
import base from "strategies/combinators"
import DrawData from "choices/data"
import errors from "core/errors"

class _IntStrategy is base.StrategyBase<Int> {
  @constructor
  unbounded() {
    _min = None
    _max = None
  }

  @constructor
  bounded(min: Int, max: Int) {
    if min > max {
      throw errors._InvalidStrategy.new(
        "integer strategy minimum exceeds maximum"
      )
    }
    _min = Some.new(min)
    _max = Some.new(max)
  }

  draw(data: DrawData) -> Int {
    if _min.isSome || {
      return data.drawInt(
        min: _min.unwrap,
        max: _max.unwrap,
        shrinkTowards: _PrimitiveTargets.integer(
          min: _min.unwrap,
          max: _max.unwrap
        ),
        label: None
      )
    }

    const magnitude = 1 << data.size
    return data.drawInt(
      min: 0 - magnitude,
      max: magnitude,
      shrinkTowards: 0,
      label: None
    )
  }

  fingerprint -> String {
    if _min.isNone || {
      return "int"
    }
    return "int(" + _min.unwrap.toString + "," + _max.unwrap.toString + ")"
  }
}

class _BoolStrategy is base.StrategyBase<Bool> {
  draw(data: DrawData) -> Bool {
    return data.drawBool(shrinkTowards: false, label: None)
  }

  fingerprint -> String => "bool"
}

class _FloatStrategy is base.StrategyBase<Float> {
  @constructor
  new(min: Float, max: Float) {
    if min > max {
      throw errors._InvalidStrategy.new(
        "float strategy minimum exceeds maximum"
      )
    }
    _min = min
    _max = max
  }

  draw(data: DrawData) -> Float {
    const minUnits = _FloatEncoding.toUnits(_min)
    const maxUnits = _FloatEncoding.toUnits(_max)
    const units = data.drawInt(
      min: minUnits,
      max: maxUnits,
      shrinkTowards: _PrimitiveTargets.integer(
        min: minUnits,
        max: maxUnits
      ),
      label: None
    )
    return _FloatEncoding.fromUnits(units)
  }

  fingerprint -> String {
    if _min == -1000000.0 and _max == 1000000.0 {
      return "float"
    }
    return "float(" + _min.toString + "," + _max.toString + ")"
  }
}

class _BytesStrategy is base.StrategyBase<Bytes> {
  @constructor
  new(minSize: Int, maxSize: Int) {
    _PrimitiveBounds.validateSizes(
      kind: "bytes",
      minSize: minSize,
      maxSize: maxSize
    )
    _minSize = minSize
    _maxSize = maxSize
  }

  draw(data: DrawData) -> Bytes {
    return data.drawBytes(
      minSize: _minSize,
      maxSize: _maxSize,
      shrinkTowards: Bytes.zeroed(_minSize),
      label: None
    )
  }

  fingerprint -> String {
    if _minSize == 0 and _maxSize == 64 {
      return "bytes"
    }
    return "bytes(" + _minSize.toString + "," + _maxSize.toString + ")"
  }
}

class _SampledFromStrategy<T> is base.StrategyBase<T> {
  @constructor
  new(values: List<T>) {
    if values.size == 0 {
      throw errors._InvalidStrategy.new(
        "sampledFrom requires at least one value"
      )
    }
    _values = base._StrategyCopies.list(values)
  }

  draw(data: DrawData) -> T {
    const index = data.drawIndex(
      size: _values.size,
      shrinkTowards: 0,
      label: None
    )
    return _values.at(index)
  }

  fingerprint -> String => "sampledFrom(" + _values.size.toString + ")"
}

class _TextStrategy is base.StrategyBase<String> {
  @constructor
  new(
    alphabet: Strategy<Int>,
    minSize: Int,
    maxSize: Int
  ) {
    _PrimitiveBounds.validateSizes(
      kind: "text",
      minSize: minSize,
      maxSize: maxSize
    )
    _alphabet = alphabet
    _minSize = minSize
    _maxSize = maxSize
  }

  draw(data: DrawData) -> String {
    return data.withSpan(label: #text, discardable: false) {
      const length = data.drawInt(
        min: _minSize,
        max: _maxSize,
        shrinkTowards: _minSize,
        label: Some.new(#length)
      )
      const codePoints = List.new()
      let index = 0
      while index < length {
        const point = data.withSpan(label: #character, discardable: true) {
          _alphabet.draw(data)
        }
        codePoints.add(point)
        index++
      }
      return String.fromCodePoints(codePoints)
    }
  }

  fingerprint -> String {
    if _minSize == 0 and _maxSize == 64 {
      return "text"
    }
    return "text(" + _alphabet.fingerprint + "," +
      _minSize.toString + "," + _maxSize.toString + ")"
  }
}

class _PrimitiveBounds {
  @class
  validateSizes(
    kind: String,
    minSize: Int,
    maxSize: Int
  ) -> None {
    if minSize < 0 or minSize > maxSize {
      throw errors._InvalidStrategy.new(
        "invalid " + kind + " size bounds"
      )
    }
  }
}

class _PrimitiveTargets {
  @class
  integer(min: Int, max: Int) -> Int {
    if min > 0 {
      return min
    }
    if max < 0 {
      return max
    }
    return 0
  }
}

class _FloatEncoding {
  @class
  scale -> Int => 1000000

  @class
  toUnits(value: Float) -> Int {
    return (value * self.scale).round.toInt
  }

  @class
  fromUnits(value: Int) -> Float {
    return Float.fromInt(value) / Float.fromInt(self.scale)
  }
}

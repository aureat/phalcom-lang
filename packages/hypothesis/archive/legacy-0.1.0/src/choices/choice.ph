// Immutable typed primitive choices recorded in semantic examples.

@data
@immutable
@sealed
class Choice {
  @variant Integer(value:, min:, max:, shrinkTowards:, label:)
  @variant Boolean(value:, shrinkTowards:, label:)
  @variant Index(value:, size:, shrinkTowards:, label:)
  @variant Bytes(value:, minSize:, maxSize:, shrinkTowards:, label:)

  @class
  @requires(min <= max)
  @requires(value >= min)
  @requires(value <= max)
  @requires(shrinkTowards >= min)
  @requires(shrinkTowards <= max)
  integer(
    value: Int,
    min: Int,
    max: Int,
    shrinkTowards: Int,
    label: Option<Symbol>
  ) -> Choice {
    return Integer.new(
      value: value,
      min: min,
      max: max,
      shrinkTowards: shrinkTowards,
      label: label
    )
  }

  @class
  boolean(
    value: Bool,
    shrinkTowards: Bool,
    label: Option<Symbol>
  ) -> Choice {
    return Boolean.new(
      value: value,
      shrinkTowards: shrinkTowards,
      label: label
    )
  }

  @class
  @requires(size > 0)
  @requires(value >= 0)
  @requires(value < size)
  @requires(shrinkTowards >= 0)
  @requires(shrinkTowards < size)
  index(
    value: Int,
    size: Int,
    shrinkTowards: Int,
    label: Option<Symbol>
  ) -> Choice {
    return Index.new(
      value: value,
      size: size,
      shrinkTowards: shrinkTowards,
      label: label
    )
  }

  @class
  @requires(minSize >= 0)
  @requires(minSize <= maxSize)
  @requires(value.size >= minSize)
  @requires(value.size <= maxSize)
  @requires(shrinkTowards.size >= minSize)
  @requires(shrinkTowards.size <= maxSize)
  bytes(
    value: Bytes,
    minSize: Int,
    maxSize: Int,
    shrinkTowards: Bytes,
    label: Option<Symbol>
  ) -> Choice {
    return Bytes.new(
      value: _ChoiceBytes.copy(value),
      minSize: minSize,
      maxSize: maxSize,
      shrinkTowards: _ChoiceBytes.copy(shrinkTowards),
      label: label
    )
  }

  value -> Any {
    return self.match(
      integer: |item| { item.value },
      boolean: |item| { item.value },
      index: |item| { item.value },
      bytes: |item| { _ChoiceBytes.copy(item.value) }
    )
  }

  label -> Option<Symbol> {
    return self.match(
      integer: |item| { item.label },
      boolean: |item| { item.label },
      index: |item| { item.label },
      bytes: |item| { item.label }
    )
  }

  shrinkTarget -> Any {
    return self.match(
      integer: |item| { item.shrinkTowards },
      boolean: |item| { item.shrinkTowards },
      index: |item| { item.shrinkTowards },
      bytes: |item| { _ChoiceBytes.copy(item.shrinkTowards) }
    )
  }

  // Compatibility getters used by the temporary integer-only shrinker.
  min -> Int {
    return self.match(
      integer: |item| { item.min },
      boolean: |_| { 0 },
      index: |_| { 0 },
      bytes: |item| { item.minSize }
    )
  }

  max -> Int {
    return self.match(
      integer: |item| { item.max },
      boolean: |_| { 1 },
      index: |item| { item.size - 1 },
      bytes: |item| { item.maxSize }
    )
  }

  withValue(value: Any) -> Choice {
    return self.match(
      integer: |item| {
        Choice.integer(
          value: value,
          min: item.min,
          max: item.max,
          shrinkTowards: item.shrinkTowards,
          label: item.label
        )
      },
      boolean: |item| {
        Choice.boolean(
          value: value,
          shrinkTowards: item.shrinkTowards,
          label: item.label
        )
      },
      index: |item| {
        Choice.index(
          value: value,
          size: item.size,
          shrinkTowards: item.shrinkTowards,
          label: item.label
        )
      },
      bytes: |item| {
        Choice.bytes(
          value: value,
          minSize: item.minSize,
          maxSize: item.maxSize,
          shrinkTowards: item.shrinkTowards,
          label: item.label
        )
      }
    )
  }

  simplifications -> List<Any> {
    return self.match(
      integer: |item| {
        _ChoiceSimplifier.integers(
          value: item.value,
          min: item.min,
          max: item.max,
          target: item.shrinkTowards
        )
      },
      boolean: |item| {
        const values = List.new()
        if item.value != item.shrinkTowards || {
          values.add(item.shrinkTowards)
        }
        return values
      },
      index: |item| {
        _ChoiceSimplifier.integers(
          value: item.value,
          min: 0,
          max: item.size - 1,
          target: item.shrinkTowards
        )
      },
      bytes: |item| {
        const values = List.new()
        if item.value != item.shrinkTowards || {
          values.add(item.shrinkTowards)
        }
        return values
      }
    )
  }

  signaturePart -> String {
    return self.match(
      integer: |item| { "i:" + item.value.toString },
      boolean: |item| { "b:" + item.value.toString },
      index: |item| { "x:" + item.value.toString },
      bytes: |item| { "y:" + item.value.toString }
    )
  }
}

class _ChoiceSimplifier {
  @class
  integers(value: Int, min: Int, max: Int, target: Int) -> List<Int> {
    const out = List.new()
    self.add(out, candidate: target, value: value, min: min, max: max)

    let offset = 1
    while offset <= 16 {
      self.add(out, candidate: target + offset, value: value, min: min, max: max)
      self.add(out, candidate: target - offset, value: value, min: min, max: max)
      offset++
    }

    let probe = value
    while _ChoiceNumbers.abs(probe - target) > 1 {
      probe = target + ((probe - target) ~/ 2)
      self.add(out, candidate: probe, value: value, min: min, max: max)
    }
    return out
  }

  @class
  add(
    values: List<Int>,
    candidate: Int,
    value: Int,
    min: Int,
    max: Int
  ) -> None {
    if candidate >= min and candidate <= max and candidate != value {
      if not values.includes(candidate) {
        values.add(candidate)
      }
    }
  }
}

class _ChoiceNumbers {
  @class
  abs(value: Int) -> Int {
    if value < 0 {
      return 0 - value
    }
    return value
  }
}

class _ChoiceBytes {
  @class
  copy(value: Bytes) -> Bytes {
    const copied = Bytes.zeroed(value.size)
    let position = 0
    while position < value.size || {
      copied[position] = value[position]
      position++
    }
    return copied
  }
}

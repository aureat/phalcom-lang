// Typed primitive choice requests.
//
// Requests describe the domain presented to a provider. The shrink target is
// semantic metadata retained on the resulting choice; providers do not shrink.

@data
@immutable
@sealed
class ChoiceRequest {
  @variant Integer(min:, max:, shrinkTowards:, label:)
  @variant Boolean(shrinkTowards:, label:)
  @variant Index(size:, shrinkTowards:, label:)
  @variant Bytes(minSize:, maxSize:, shrinkTowards:, label:)

  @class
  @requires(min <= max)
  @requires(shrinkTowards >= min)
  @requires(shrinkTowards <= max)
  integer(
    min: Int,
    max: Int,
    shrinkTowards: Int,
    label: Option<Symbol>
  ) -> ChoiceRequest {
    return Integer.new(
      min: min,
      max: max,
      shrinkTowards: shrinkTowards,
      label: label
    )
  }

  @class
  boolean(
    shrinkTowards: Bool,
    label: Option<Symbol>
  ) -> ChoiceRequest {
    return Boolean.new(
      shrinkTowards: shrinkTowards,
      label: label
    )
  }

  @class
  @requires(size > 0)
  @requires(shrinkTowards >= 0)
  @requires(shrinkTowards < size)
  index(
    size: Int,
    shrinkTowards: Int,
    label: Option<Symbol>
  ) -> ChoiceRequest {
    return Index.new(
      size: size,
      shrinkTowards: shrinkTowards,
      label: label
    )
  }

  @class
  @requires(minSize >= 0)
  @requires(minSize <= maxSize)
  @requires(shrinkTowards.size >= minSize)
  @requires(shrinkTowards.size <= maxSize)
  bytes(
    minSize: Int,
    maxSize: Int,
    shrinkTowards: Bytes,
    label: Option<Symbol>
  ) -> ChoiceRequest {
    return Bytes.new(
      minSize: minSize,
      maxSize: maxSize,
      shrinkTowards: _ChoiceRequestBytes.copy(shrinkTowards),
      label: label
    )
  }

  label -> Option<Symbol> {
    return self.match(
      integer: { value => value.label },
      boolean: { value => value.label },
      index: { value => value.label },
      bytes: { value => value.label }
    )
  }

  shrinkTarget -> Any {
    return self.match(
      integer: { value => value.shrinkTowards },
      boolean: { value => value.shrinkTowards },
      index: { value => value.shrinkTowards },
      bytes: { value => _ChoiceRequestBytes.copy(value.shrinkTowards) }
    )
  }
}

class _ChoiceRequestBytes {
  @class
  copy(value: Bytes) -> Bytes {
    const copied = Bytes.zeroed(value.size)
    let position = 0
    while position < value.size {
      copied[position] = value[position]
      position++
    }
    return copied
  }
}

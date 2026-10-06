/// A typed primitive decision recorded in an example tape.
data Choice {
  /// An integer drawn from an inclusive range.
  Integer(
    _ value: Int,
    _ min: Int,
    _ max: Int,
    _ shrinkTowards: Int
  )

  // A boolean decision with a preferred shrink value.
  Boolean(
    _ value: Bool,
    _ shrinkTowards: Bool
  )

  /// An index drawn from the half-open range `[0, size)`.
  Index(
    _ value: Int,
    _ size: Int,
    _ shrinkTowards: Int
  )

  /// A byte sequence drawn from the configured size range.
  Bytes(
    _ value: Bytes,
    _ minSize: Int,
    _ maxSize: Int,
    _ shrinkTowards: Bytes
  )
}

impl Choice {
  /// Copies mutable payloads so later byte-buffer writes cannot alter a tape.
  copy -> Choice {
    match self {
      Choice::Integer(value, min, max, target) =>
        Choice::Integer(value, min, max, target)
      Choice::Boolean(value, target) => Choice::Boolean(value, target)
      Choice::Index(value, size, target) => Choice::Index(value, size, target)
      Choice::Bytes(value, minSize, maxSize, target) =>
        Choice::Bytes(
          ByteCopies.copy(value),
          minSize,
          maxSize,
          ByteCopies.copy(target)
        )
    }
  }
}

class ByteCopies {
  @class
  copy(value: Bytes) -> Bytes {
    let copied = Bytes.zeroed(value.size)
    let mut index = 0

    while index < value.size {
      copied[index] = value[index]
      index += 1
    }

    copied
  }
}

export Choice

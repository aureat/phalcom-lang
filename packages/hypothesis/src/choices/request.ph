/// The typed domain requested from a choice provider.
data ChoiceRequest {
  /// Requests an integer from an inclusive range.
  Integer(
    _ min: Int,
    _ max: Int,
    _ shrinkTowards: Int
  ),

  /// Requests a boolean choice.
  Boolean(
    _ shrinkTowards: Bool
  ),

  /// Requests an index from the half-open range `[0, size)`.
  Index(
    _ size: Int,
    _ shrinkTowards: Int
  ),

  /// Requests bytes whose length is within the configured range.
  Bytes(
    _ minSize: Int,
    _ maxSize: Int,
    _ shrinkTowards: Bytes
  ),
}

export ChoiceRequest

/// Half-open source interval in token (for text sources: byte) offsets.
data Span(
  start: Int,
  end: Int
)

/// A value paired with the exact source interval it was parsed from.
data Spanned<T>(
  value: T,
  span: Span
)

export Span, Spanned

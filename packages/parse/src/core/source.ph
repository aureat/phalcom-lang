/// Sequential input abstraction shared by every parser.
///
/// A `Source` exposes a stable, indexable token sequence. Implementations
/// must not semantically mutate underneath a running parser; adapters in
/// `parse.source` snapshot caller-owned collections where necessary.
trait Source {
  type Token

  /// Number of tokens in the sequence.
  size -> Int

  /// Token at `index`. Valid for `0 <= index < size`.
  at(_ index: Int) -> Self::Token
}

export Source

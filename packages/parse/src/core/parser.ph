from .cursor import Cursor
from .result import ParseResult

/// The parser conformance surface.
///
/// A parser is parameterized by its source type while its produced value
/// is a property of the implementation under that source, expressed as
/// the associated type `Output`.
///
/// In v1, `Rule<S, T>` is the composition currency: concrete leaf
/// implementations conform to `Parser<S>`, and adapters in `parse.source`
/// and `parse.text` erase leaves into `Rule<S, T>` values. This keeps
/// source and result types statically preserved (`Rule` never erases `S`
/// or `T`) while avoiding a `Dynamic` boundary anywhere in composition.
trait Parser<S> {
  type Output

  parse(_ cursor: Cursor<S>) -> ParseResult<S, Self::Output>
}

export Parser

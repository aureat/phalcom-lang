// Immutable semantic span over a half-open choice range [start, end).

@data
@immutable
class Span {
  _id: Int
  _label: Symbol
  _start: Int
  _end: Int
  _parent: Option<Int>
  _discardable: Bool

  @class
  @requires(id >= 0)
  @requires(start >= 0)
  @requires(start <= end)
  create(
    id: Int,
    label: Symbol,
    start: Int,
    end: Int,
    parent: Option<Int>,
    discardable: Bool
  ) -> Span {
    return Span.new(
      id: id,
      label: label,
      start: start,
      end: end,
      parent: parent,
      discardable: discardable
    )
  }

  length -> Int { _end - _start }

  contains(choiceIndex: Int) -> Bool {
    return choiceIndex >= _start and choiceIndex < _end
  }
}

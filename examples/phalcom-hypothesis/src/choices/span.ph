// Immutable semantic span over a half-open choice range [start, end).

@data
@immutable
class Span {
  const _id: Int
  const _label: Symbol
  const _start: Int
  const _end: Int
  const _parent: Option<Int>
  const _discardable: Bool

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

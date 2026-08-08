// Immutable semantic example: normalized primitive choices, semantic spans,
// and the generation size used by size-sensitive strategies.

import Choice from "choices/choice"
import Span from "choices/span"

@data
@immutable
class Example {
  const _choiceValues: List<Choice>
  const _spanValues: List<Span>
  const _generationSize: Int

  @class
  empty -> Example {
    return Example.from(
      choices: const [],
      spans: const [],
      generationSize: 0
    )
  }

  @class
  @requires(generationSize >= 0)
  from(
    choices: List<Choice>,
    spans: List<Span>,
    generationSize: Int
  ) -> Example {
    return Example.new(
      choiceValues: _ExampleCopies.choices(choices),
      spanValues: _ExampleCopies.spans(spans),
      generationSize: generationSize
    )
  }

  size -> Int => _choiceValues.size

  at(index: Int) -> Choice => _choiceValues.at(index)

  choices -> List<Choice> => _ExampleCopies.choices(_choiceValues)

  spans -> List<Span> => _ExampleCopies.spans(_spanValues)

  spanWithId(id: Int) -> Option<Span> {
    for span in _spanValues {
      if span.id == id {
        return Some.new(span)
      }
    }
    return None
  }

  replace(index: Int, choice: Choice) -> Example {
    const next = List.new()
    let position = 0
    while position < _choiceValues.size {
      if position == index {
        next.add(choice)
      } else {
        next.add(_choiceValues.at(position))
      }
      position++
    }
    return Example.from(
      choices: next,
      spans: _spanValues,
      generationSize: _generationSize
    )
  }

  prefix(size: Int) -> Example {
    const count = _ExampleNumbers.clamp(size, min: 0, max: _choiceValues.size)
    const nextChoices = List.new()
    let position = 0
    while position < count {
      nextChoices.add(_choiceValues.at(position))
      position++
    }

    const nextSpans = List.new()
    for span in _spanValues {
      if span.start < count {
        nextSpans.add(
          Span.create(
            id: span.id,
            label: span.label,
            start: span.start,
            end: _ExampleNumbers.min(span.end, count),
            parent: span.parent,
            discardable: span.discardable
          )
        )
      }
    }

    return Example.from(
      choices: nextChoices,
      spans: nextSpans,
      generationSize: _generationSize
    )
  }

  @requires(start >= 0)
  @requires(start <= end)
  @requires(end <= _choiceValues.size)
  deleteRange(start: Int, end: Int) -> Example {
    if start == end {
      return self
    }

    const nextChoices = List.new()
    let position = 0
    while position < _choiceValues.size {
      if position < start or position >= end {
        nextChoices.add(_choiceValues.at(position))
      }
      position++
    }

    return Example.from(
      choices: nextChoices,
      spans: _ExampleSpans.adjustedAfterDeletion(
        spans: _spanValues,
        start: start,
        end: end
      ),
      generationSize: _generationSize
    )
  }

  deleteSpan(span: Span) -> Example {
    return self.deleteRange(start: span.start, end: span.end)
  }

  signature -> String {
    const parts = List.new()
    for choice in _choiceValues {
      parts.add(choice.signaturePart)
    }
    return _generationSize.toString + ":" + parts.join(",")
  }

  spanSignature -> String {
    const parts = List.new()
    for span in _spanValues {
      parts.add(
        span.id.toString + "," + span.label.toString + "," +
        span.start.toString + "," + span.end.toString + "," +
        _ExampleSpanText.parent(span.parent) + "," +
        span.discardable.toString
      )
    }
    return parts.join(";")
  }

  toString -> String {
    return "[" + _choiceValues.map |choice| { choice.value }.join(", ") + "]"
  }
}

class _ExampleCopies {
  @class
  choices(values: List<Choice>) -> List<Choice> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }

  @class
  spans(values: List<Span>) -> List<Span> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}

class _ExampleNumbers {
  @class
  min(left: Int, right: Int) -> Int {
    if left < right {
      return left
    }
    return right
  }

  @class
  clamp(value: Int, min: Int, max: Int) -> Int {
    if value < min {
      return min
    }
    if value > max {
      return max
    }
    return value
  }
}


class _ExampleSpans {
  @class
  adjustedAfterDeletion(
    spans: List<Span>,
    start: Int,
    end: Int
  ) -> List<Span> {
    const adjusted = List.new()
    const removed = end - start

    for span in spans {
      if span.end <= start {
        adjusted.add(span)
      } else if span.start >= end {
        adjusted.add(
          Span.create(
            id: span.id,
            label: span.label,
            start: span.start - removed,
            end: span.end - removed,
            parent: span.parent,
            discardable: span.discardable
          )
        )
      } else if span.start < start and span.end > end {
        adjusted.add(
          Span.create(
            id: span.id,
            label: span.label,
            start: span.start,
            end: span.end - removed,
            parent: span.parent,
            discardable: span.discardable
          )
        )
      } else if span.start < start and span.end > start {
        adjusted.add(
          Span.create(
            id: span.id,
            label: span.label,
            start: span.start,
            end: start,
            parent: span.parent,
            discardable: span.discardable
          )
        )
      } else if span.start < end and span.end > end {
        adjusted.add(
          Span.create(
            id: span.id,
            label: span.label,
            start: start,
            end: span.end - removed,
            parent: span.parent,
            discardable: span.discardable
          )
        )
      }
    }
    return adjusted
  }
}

class _ExampleSpanText {
  @class
  parent(value: Option<Int>) -> String {
    if value.isNone { return "-" }
    return value.unwrap.toString
  }
}

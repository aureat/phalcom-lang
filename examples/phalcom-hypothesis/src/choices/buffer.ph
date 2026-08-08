// Mutable construction buffer. Freezing copies all retained values into an
// immutable Example so later buffer mutation cannot alter prior examples.

import Choice from "choices/choice"
import Span from "choices/span"
import Example from "choices/example"
import errors from "core/errors"

@data
@immutable
class _OpenSpan {
  const _id: Int
  const _label: Symbol
  const _start: Int
  const _parent: Option<Int>
  const _discardable: Bool
}

class ChoiceBuffer {
  @constructor
  @requires(generationSize >= 0)
  new(generationSize: Int) {
    _generationSize = generationSize
    _choices = List.new()
    _closedSpans = List.new()
    _openSpans = List.new()
    _nextSpanId = 0
  }

  generationSize -> Int => _generationSize
  size -> Int => _choices.size

  add(choice: Choice) -> Choice {
    _choices.add(choice)
    return choice
  }

  beginSpan(label: Symbol, discardable: Bool) -> Int {
    const id = _nextSpanId
    _nextSpanId++

    let parent = None
    if _openSpans.size > 0 {
      parent = Some.new(_openSpans.at(_openSpans.size - 1).id)
    }

    _closedSpans.add(None)
    _openSpans.add(
      _OpenSpan.new(
        id: id,
        label: label,
        start: _choices.size,
        parent: parent,
        discardable: discardable
      )
    )
    return id
  }

  endSpan(id: Int) -> Span {
    if _openSpans.size == 0 {
      throw errors._UnclosedSpan.new("cannot close a span when none is open")
    }

    const open = _openSpans.at(_openSpans.size - 1)
    if open.id != id {
      throw errors._UnclosedSpan.new("semantic spans must close in LIFO order")
    }

    _openSpans.removeAt(_openSpans.size - 1)

    const closed = Span.create(
      id: open.id,
      label: open.label,
      start: open.start,
      end: _choices.size,
      parent: open.parent,
      discardable: open.discardable
    )
    _closedSpans.at(id, put: Some.new(closed))
    return closed
  }

  withSpan(label: Symbol, discardable: Bool, body: Block) -> Any {
    const id = self.beginSpan(label: label, discardable: discardable)
    return || {
      body.call()
    }.ensure || {
      self.endSpan(id)
    }
  }

  choices -> List<Choice> {
    const copied = List.new()
    for choice in _choices {
      copied.add(choice)
    }
    return copied
  }

  spans -> List<Span> {
    const ordered = List.new()
    for closed in _closedSpans {
      if closed.isSome || {
        ordered.add(closed.unwrap)
      }
    }
    return ordered
  }

  freeze -> Example {
    if _openSpans.size > 0 {
      throw errors._UnclosedSpan.new("cannot freeze an example with open spans")
    }
    return Example.from(
      choices: self.choices,
      spans: self.spans,
      generationSize: _generationSize
    )
  }
}

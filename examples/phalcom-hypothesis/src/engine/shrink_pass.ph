// Ordered immutable candidate generators used by the structural shrinker.

import Example from "choices/example"
import Choice from "choices/choice"
import Span from "choices/span"

protocol ShrinkPass {
  name -> Symbol
  candidates(current: Example) -> List<Example>
}

class _DeleteDiscardableSpans {
  name -> Symbol { #deleteDiscardableSpans }

  candidates(current: Example) -> List<Example> {
    const out = List.new()
    for span in current.spans || {
      if span.discardable and span.label != #recursiveBranch {
        const candidate = _SpanDeletion.delete(
          example: current,
          span: span
        )
        if candidate.isSome || {
          out.add(candidate.unwrap)
        }
      }
    }
    return out
  }
}

class _ShortenTrailingChoices {
  name -> Symbol { #shortenTrailingChoices }

  candidates(current: Example) -> List<Example> {
    const out = List.new()
    let size = current.size - 1
    while size >= 0 {
      out.add(current.prefix(size))
      size--
    }
    return out
  }
}

class _MinimizeBranchIndices {
  name -> Symbol { #minimizeBranchIndices }

  candidates(current: Example) -> List<Example> {
    const out = List.new()
    let index = 0
    while index < current.size || {
      const choice = current.at(index)
      choice.match(
        integer: |_| { None },
        boolean: |_| { None },
        index: |value| {
          if value.label == Some.new(#branch) or value.value != value.shrinkTowards || {
            for replacement in choice.simplifications || {
              out.add(current.replace(index, choice.withValue(replacement)))
            }
          }
        },
        bytes: |_| { None }
      )
      index++
    }
    return out
  }
}

class _MinimizeIntegerChoices {
  name -> Symbol { #minimizeIntegerChoices }

  candidates(current: Example) -> List<Example> {
    const out = List.new()
    let index = 0
    while index < current.size || {
      const choice = current.at(index)
      choice.match(
        integer: |_| {
          for replacement in choice.simplifications || {
            out.add(current.replace(index, choice.withValue(replacement)))
          }
        },
        boolean: |value| {
          if value.value != value.shrinkTowards || {
            out.add(current.replace(index, choice.withValue(value.shrinkTowards)))
          }
        },
        index: |_| { None },
        bytes: |_| { None }
      )
      index++
    }
    return out
  }
}

class _MinimizeIntegerBlocks {
  name -> Symbol { #minimizeIntegerBlocks }

  candidates(current: Example) -> List<Example> {
    const out = List.new()
    let start = 0
    while start < current.size || {
      let candidate = current
      let changed = false
      let index = start
      while index < current.size || {
        const choice = candidate.at(index)
        const replacement = choice.match(
          integer: |value| { Some.new(value.shrinkTowards) },
          boolean: |_| { None },
          index: |_| { None },
          bytes: |_| { None }
        )
        if replacement.isNone || {
          break
        }
        if choice.value != replacement.unwrap || {
          candidate = candidate.replace(
            index,
            choice.withValue(replacement.unwrap)
          )
          changed = true
        }
        index++
      }
      if changed and index - start > 1 {
        out.add(candidate)
      }
      start++
    }
    return out
  }
}

class _SimplifyBytesAndText {
  name -> Symbol { #simplifyBytesAndText }

  candidates(current: Example) -> List<Example> {
    const out = List.new()
    let index = 0
    while index < current.size || {
      const choice = current.at(index)
      choice.match(
        integer: |_| { None },
        boolean: |_| { None },
        index: |_| { None },
        bytes: |value| {
          if value.value != value.shrinkTowards || {
            out.add(
              current.replace(index, choice.withValue(value.shrinkTowards))
            )
          }

          let size = value.value.size - 1
          while size >= value.minSize || {
            out.add(
              current.replace(
                index,
                choice.withValue(_ShrinkBytes.prefix(value.value, size))
              )
            )
            size--
          }

          if value.value.size > 0 {
            out.add(
              current.replace(
                index,
                choice.withValue(Bytes.zeroed(value.value.size))
              )
            )
          }
        }
      )
      index++
    }
    return out
  }
}

class _MinimizeRecursiveStructures {
  name -> Symbol { #minimizeRecursiveStructures }

  candidates(current: Example) -> List<Example> {
    const out = List.new()
    for span in current.spans || {
      if span.label == #recursiveBranch and span.start > 0 {
        const decisionIndex = span.start - 1
        const decision = current.at(decisionIndex)
        decision.match(
          integer: |_| { None },
          boolean: |value| {
            if value.label == Some.new(#recursive) and value.value || {
              const collapsed = current.replace(
                decisionIndex,
                decision.withValue(false)
              )
              out.add(
                collapsed.deleteRange(start: span.start, end: span.end)
              )
            }
          },
          index: |_| { None },
          bytes: |_| { None }
        )
      }
    }
    return out
  }
}

class _SpanDeletion {
  @class
  delete(example: Example, span: Span) -> Option<Example> {
    let candidate = example
    const lengthIndex = self.lengthChoiceIndex(
      example: example,
      span: span
    )

    if lengthIndex.isSome || {
      const index = lengthIndex.unwrap
      const lengthChoice = candidate.at(index)
      const reduced = lengthChoice.match(
        integer: |value| {
          if value.value <= value.min || {
            return None
          }
          return Some.new(value.value - 1)
        },
        boolean: |_| { None },
        index: |_| { None },
        bytes: |_| { None }
      )
      if reduced.isNone || {
        return None
      }
      candidate = candidate.replace(
        index,
        lengthChoice.withValue(reduced.unwrap)
      )
    }

    return Some.new(
      candidate.deleteRange(start: span.start, end: span.end)
    )
  }

  @class
  lengthChoiceIndex(example: Example, span: Span) -> Option<Int> {
    if span.parent.isNone || {
      return None
    }

    const parent = example.spanWithId(span.parent.unwrap)
    if parent.isNone || {
      return None
    }

    let index = parent.unwrap.start
    while index < span.start || {
      const choice = example.at(index)
      if choice.label == Some.new(#length) {
        return Some.new(index)
      }
      index++
    }
    return None
  }
}

class _ShrinkBytes {
  @class
  prefix(value: Bytes, size: Int) -> Bytes {
    const copied = Bytes.zeroed(size)
    let index = 0
    while index < size {
      copied[index] = value[index]
      index++
    }
    return copied
  }
}

// Stable primitive-choice extension boundary.
//
// Providers supply one normalized typed choice for each semantic request.
// They do not shrink, open spans, or interpret strategies. Factories create a
// fresh provider for each generated example so consumption counters and script
// cursors never leak between examples.

import Choice from "choices/choice"
import ChoiceRequest from "choices/request"
import Example from "choices/example"
import errors from "core/errors"

protocol ChoiceProvider {
  choose(request: ChoiceRequest) -> Choice
  consumedChoices -> Int
}

protocol ChoiceProviderFactory {
  create(exampleIndex: Int, generationSize: Int) -> ChoiceProvider
}

class SystemRandomChoiceProvider {
  @constructor
  new() {
    self.init(Random.system)
  }

  @constructor
  new(random: Random) {
    self.init(random)
  }

  init(random: Random) -> None {
    _random = random
    _consumedChoices = 0
  }

  consumedChoices -> Int => _consumedChoices

  choose(request: ChoiceRequest) -> Choice {
    _consumedChoices++
    const source = request.match(
      integer: |value| {
        Choice.integer(
          value: _random.nextIntIn(value.min, value.max),
          min: value.min,
          max: value.max,
          shrinkTowards: value.shrinkTowards,
          label: value.label
        )
      },
      boolean: |value| {
        Choice.boolean(
          value: _random.nextIntIn(0, 1) == 1,
          shrinkTowards: value.shrinkTowards,
          label: value.label
        )
      },
      index: |value| {
        Choice.index(
          value: _random.nextIntIn(0, value.size - 1),
          size: value.size,
          shrinkTowards: value.shrinkTowards,
          label: value.label
        )
      },
      bytes: |value| {
        const length = _random.nextIntIn(value.minSize, value.maxSize)
        const bytes = Bytes.zeroed(length)
        let position = 0
        while position < length {
          bytes[position] = _random.nextIntIn(0, 255)
          position++
        }
        return Choice.bytes(
          value: bytes,
          minSize: value.minSize,
          maxSize: value.maxSize,
          shrinkTowards: value.shrinkTowards,
          label: value.label
        )
      }
    )
    return _ChoiceNormalization.normalize(request: request, source: source)
  }

}

class ScriptedChoiceProvider {
  @constructor
  new(choices: List<Choice>) {
    _choices = _ProviderCopies.choices(choices)
    _cursor = 0
  }

  consumedChoices -> Int => _cursor

  choose(request: ChoiceRequest) -> Choice {
    if _cursor >= _choices.size {
      throw errors._ScriptedProviderExhausted.new(
        "scripted provider ended before the draw sequence completed"
      )
    }
    const source = _choices.at(_cursor)
    _cursor++
    return _ChoiceNormalization.normalize(request: request, source: source)
  }
}

class SystemRandomProviderFactory {
  @constructor
  new(seed: Int) {
    _random = Random.new(seed: seed)
  }

  @constructor
  new(random: Random) {
    _random = random
  }

  create(exampleIndex: Int, generationSize: Int) -> ChoiceProvider {
    return SystemRandomChoiceProvider.new(_random)
  }
}

class ScriptedProviderFactory {
  @constructor
  new(scripts: List<List<Choice>>) {
    _scripts = _ProviderCopies.scripts(scripts)
  }

  create(exampleIndex: Int, generationSize: Int) -> ChoiceProvider {
    if exampleIndex < 0 or exampleIndex >= _scripts.size {
      throw errors._ScriptedProviderExhausted.new(
        "no scripted choice sequence exists for generated example " +
        exampleIndex.toString
      )
    }
    return ScriptedChoiceProvider.new(_scripts.at(exampleIndex))
  }
}

class _ReplayChoiceProvider {
  @constructor
  new(example: Example) {
    _example = example
    _cursor = 0
  }

  consumedChoices -> Int => _cursor

  choose(request: ChoiceRequest) -> Choice {
    if _cursor >= _example.size {
      throw errors._ReplayExhausted.new(
        "recorded example ended before the draw sequence completed"
      )
    }

    const source = _example.at(_cursor)
    _cursor++
    return _ChoiceNormalization.normalize(request: request, source: source)
  }
}

class _ChoiceNormalization {
  @class
  normalize(request: ChoiceRequest, source: Choice) -> Choice {
    return request.match(
      integer: |expected| {
        source.match(
          integer: |actual| {
            if actual.value < expected.min or actual.value > expected.max {
              throw errors._InvalidReplayChoice.new(
                "integer provider value is outside the current request bounds"
              )
            }
            return Choice.integer(
              value: actual.value,
              min: expected.min,
              max: expected.max,
              shrinkTowards: expected.shrinkTowards,
              label: expected.label
            )
          },
          boolean: |_| { self.typeMismatch(#integer) },
          index: |_| { self.typeMismatch(#integer) },
          bytes: |_| { self.typeMismatch(#integer) }
        )
      },
      boolean: |expected| {
        source.match(
          integer: |_| { self.typeMismatch(#boolean) },
          boolean: |actual| {
            return Choice.boolean(
              value: actual.value,
              shrinkTowards: expected.shrinkTowards,
              label: expected.label
            )
          },
          index: |_| { self.typeMismatch(#boolean) },
          bytes: |_| { self.typeMismatch(#boolean) }
        )
      },
      index: |expected| {
        source.match(
          integer: |_| { self.typeMismatch(#index) },
          boolean: |_| { self.typeMismatch(#index) },
          index: |actual| {
            if actual.value < 0 or actual.value >= expected.size {
              throw errors._InvalidReplayChoice.new(
                "index provider value is outside the current request domain"
              )
            }
            return Choice.index(
              value: actual.value,
              size: expected.size,
              shrinkTowards: expected.shrinkTowards,
              label: expected.label
            )
          },
          bytes: |_| { self.typeMismatch(#index) }
        )
      },
      bytes: |expected| {
        source.match(
          integer: |_| { self.typeMismatch(#bytes) },
          boolean: |_| { self.typeMismatch(#bytes) },
          index: |_| { self.typeMismatch(#bytes) },
          bytes: |actual| {
            if actual.value.size < expected.minSize or actual.value.size > expected.maxSize {
              throw errors._InvalidReplayChoice.new(
                "bytes provider value violates the current size bounds"
              )
            }
            return Choice.bytes(
              value: actual.value,
              minSize: expected.minSize,
              maxSize: expected.maxSize,
              shrinkTowards: expected.shrinkTowards,
              label: expected.label
            )
          }
        )
      }
    )
  }

  @class
  typeMismatch(expected: Symbol) -> Any {
    throw errors._InvalidReplayChoice.new(
      "provider choice kind does not match requested " + expected.toString
    )
  }
}

class _ProviderCopies {
  @class
  choices(values: List<Choice>) -> List<Choice> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }

  @class
  scripts(values: List<List<Choice>>) -> List<List<Choice>> {
    const copied = List.new()
    for value in values {
      copied.add(self.choices(value))
    }
    return copied
  }
}

// Compatibility alias for internal Phase 03 callers.
const _RandomChoiceProvider = SystemRandomChoiceProvider

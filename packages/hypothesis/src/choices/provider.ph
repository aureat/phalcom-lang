import .choice as Choices
import .request as Requests
import .example as Examples
from .choice import Choice

/// A deterministic generator used when the host does not provide one.
class SeededRandom {
  mut _state: Int

  @constructor
  new(seed: Int) {
    let normalizedSeed = seed % 2147483646
    _state = if normalizedSeed < 0 {
      normalizedSeed + 2147483646
    } else {
      normalizedSeed
    } + 1
  }

  /// Returns a pseudo-random integer from the inclusive range.
  nextIntIn(min: Int, max: Int) -> Int {
    if min > max {
      throw Error.new("invalid random integer range")
    }

    _state = (_state * 48271) % 2147483647

    min + (_state % (max - min + 1))
  }
}

/// Supplies one choice for each primitive request made by a strategy.
trait ChoiceProvider {
  /// Chooses a value satisfying the request's type and domain.
  choose(request: Requests.ChoiceRequest) -> Choices.Choice

  /// Number of requests already consumed from this provider.
  consumedChoices -> Int

  /// Remaining tape entries, or `None` for an unbounded provider.
  remainingChoices -> Option<Int>
}

/// Creates fresh, statically typed providers for generated examples.
class ChoiceProviderFactory<P> {
  _create: (Int) -> P

  @constructor
  new(create: (Int) -> P) {
    _create = create
  }

  /// Creates the provider for `exampleIndex`.
  create(exampleIndex: Int) -> P {
    _create(exampleIndex)
  }
}

/// Generates choices from a seeded Phalcom random source.
class SystemRandomChoiceProvider {
  _random: SeededRandom
  mut _consumedChoices: Int = 0

  @constructor
  new(seed: Int) {
    _random = SeededRandom.new(seed: seed)
  }

  consumedChoiceCount -> Int { _consumedChoices }

  recordChoice -> () {
    _consumedChoices += 1
  }

  randomIntIn(min: Int, max: Int) -> Int {
    _random.nextIntIn(min: min, max: max)
  }
}

impl ChoiceProvider for SystemRandomChoiceProvider {
  choose(request: Requests.ChoiceRequest) -> Choices.Choice {
    recordChoice

    match request {
      Requests.ChoiceRequest::Integer(min, max, target) => {
        if min > max or target < min or target > max {
          throw Error.new("invalid integer choice request")
        }

        Choice::Integer(randomIntIn(min: min, max: max), min, max, target)
      }

      Requests.ChoiceRequest::Boolean(target) =>
        Choice::Boolean(randomIntIn(min: 0, max: 1) == 1, target)

      Requests.ChoiceRequest::Index(size, target) => {
        if size <= 0 or target < 0 or target >= size {
          throw Error.new("invalid index choice request")
        }

        Choice::Index(randomIntIn(min: 0, max: size - 1), size, target)
      }

      Requests.ChoiceRequest::Bytes(minSize, maxSize, target) => {
        if minSize < 0 or minSize > maxSize or
          target.size < minSize or target.size > maxSize {
          throw Error.new("invalid byte choice request")
        }

        let length = randomIntIn(min: minSize, max: maxSize)
        let bytes = Bytes.new(length)
        let mut index = 0

        while index < length {
          bytes[index] = randomIntIn(min: 0, max: 255)
          index += 1
        }

        Choice::Bytes(bytes, minSize, maxSize, target).copy
      }
    }
  }

  consumedChoices -> Int { consumedChoiceCount }
  remainingChoices -> Option<Int> { None }
}

/// Supplies normalized choices from a fixed tape.
class ScriptedChoiceProvider {
  _script: List<Choices.Choice>
  mut _cursor: Int = 0

  @constructor
  new(script: List<Choices.Choice>) {
    _script = ProviderCopies.choices(values: script)
  }

  consumedChoiceCount -> Int { _cursor }

  scriptSize -> Int { _script.size }

  nextChoice -> Choices.Choice {
    let choice = _script[_cursor]
    _cursor += 1
    choice
  }
}

impl ChoiceProvider for ScriptedChoiceProvider {
  choose(request: Requests.ChoiceRequest) -> Choices.Choice {
    if consumedChoiceCount >= scriptSize {
      throw Error.new("scripted provider ran out of choices")
    }

    let source = nextChoice

    ChoiceNormalization.normalize(request: request, source: source)
  }

  consumedChoices -> Int { consumedChoiceCount }
  remainingChoices -> Option<Int> {
    Some(scriptSize - consumedChoiceCount)
  }
}

/// Replays choices from an immutable example tape.
class ReplayChoiceProvider {
  _example: Examples.Example
  mut _cursor: Int = 0

  @constructor
  new(example: Examples.Example) {
    _example = example
  }

  consumedChoiceCount -> Int { _cursor }

  hasNext -> Bool { _cursor < _example.size }

  exampleSize -> Int { _example.size }

  nextChoice -> Choices.Choice {
    let choice = _example.at(index: _cursor)
    _cursor += 1
    choice
  }
}

impl ChoiceProvider for ReplayChoiceProvider {
  choose(request: Requests.ChoiceRequest) -> Choices.Choice {
    if not hasNext {
      throw Error.new("replay tape ended before the draw sequence completed")
    }

    let source = nextChoice

    ChoiceNormalization.normalize(request: request, source: source)
  }

  consumedChoices -> Int { consumedChoiceCount }
  remainingChoices -> Option<Int> {
    Some(exampleSize - consumedChoiceCount)
  }
}

/// Produces a deterministic random stream per example from one seed.
class SystemRandomChoiceProviderFactory {
  _seed: Int

  @constructor
  new(seed: Int) {
    _seed = seed
  }

}

impl SystemRandomChoiceProviderFactory {
  providerFactory -> ChoiceProviderFactory<SystemRandomChoiceProvider> {
    // Each example gets a repeatable stream independent of earlier consumption.
    let seed = _seed
    ChoiceProviderFactory<SystemRandomChoiceProvider>.new(
      create: |exampleIndex| {
        SystemRandomChoiceProvider.new(seed: seed + exampleIndex)
      }
    )
  }
}

/// Reuses the supplied scripted tape for each example.
class ScriptedChoiceProviderFactory {
  _script: List<Choices.Choice>

  @constructor
  new(script: List<Choices.Choice>) {
    _script = ProviderCopies.choices(values: script)
  }

  scriptCopy -> List<Choices.Choice> {
    ProviderCopies.choices(values: _script)
  }
}

impl ScriptedChoiceProviderFactory {
  /// Creates providers with independent cursors over defensive tape copies.
  providerFactory -> ChoiceProviderFactory<ScriptedChoiceProvider> {
    let script = scriptCopy
    ChoiceProviderFactory<ScriptedChoiceProvider>.new(
      create: |exampleIndex| {
        ScriptedChoiceProvider.new(script: script)
      }
    )
  }
}

class ChoiceNormalization {
  @class
  normalize(
    request: Requests.ChoiceRequest,
    source: Choices.Choice
  ) -> Choices.Choice {
    // Replay must preserve the requested kind and its valid domain. A tape
    // that no longer satisfies the request is rejected instead of coerced.
    match request {
      Requests.ChoiceRequest::Integer(min, max, target) =>
        match source {
          Choices.Choice::Integer(value, _, _, _) => {
            if min > max or target < min or target > max or
              value < min or value > max {
              throw Error.new("integer choice is outside the requested bounds")
            }

            Choice::Integer(value, min, max, target)
          }

          _ => {
            throw Error.new("choice kind does not match its request")
          }
        }

      Requests.ChoiceRequest::Boolean(target) =>
        match source {
          Choices.Choice::Boolean(value, _) =>
            Choice::Boolean(value, target)

          _ => {
            throw Error.new("choice kind does not match its request")
          }
        }

      Requests.ChoiceRequest::Index(size, target) =>
        match source {
          Choices.Choice::Index(value, _, _) => {
            if size <= 0 or target < 0 or target >= size or
              value < 0 or value >= size {
              throw Error.new("index choice is outside the requested domain")
            }

            Choice::Index(value, size, target)
          }

          _ => {
            throw Error.new("choice kind does not match its request")
          }
        }

      Requests.ChoiceRequest::Bytes(minSize, maxSize, target) =>
        match source {
          Choices.Choice::Bytes(value, _, _, _) => {
            if minSize < 0 or minSize > maxSize or
              target.size < minSize or target.size > maxSize or
              value.size < minSize or value.size > maxSize {
              throw Error.new("byte choice is outside the requested size bounds")
            }

            Choice::Bytes(value, minSize, maxSize, target).copy
          }

          _ => {
            throw Error.new("choice kind does not match its request")
          }
        }
    }
  }
}

class ProviderCopies {
  @class
  choices(values: List<Choices.Choice>) -> List<Choices.Choice> {
    let mut copied = List.new()
    let mut index = 0

    while index < values.size {
      copied.append(values[index].copy)
      index += 1
    }

    copied
  }
}

export (
  ChoiceProvider,
  ChoiceProviderFactory,
  ScriptedChoiceProvider,
  ScriptedChoiceProviderFactory,
  ReplayChoiceProvider,
  SystemRandomChoiceProvider,
  SystemRandomChoiceProviderFactory
)

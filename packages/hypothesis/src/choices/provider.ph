import .choice as Choices
import .request as Requests
import .example as Examples

/// Supplies one choice for each primitive request made by a strategy.
trait ChoiceProvider {
  /// Chooses a value satisfying the request's type and domain.
  choose(request: Requests.ChoiceRequest) -> Choices.Choice

  /// Number of requests already consumed from this provider.
  consumedChoices -> Int
}

/// Creates a fresh provider for a generated example.
trait ChoiceProviderFactory {
  /// Creates the provider for `exampleIndex`.
  create(exampleIndex: Int) -> ChoiceProvider
}

/// Generates choices from a seeded Phalcom random source.
class SystemRandomChoiceProvider {
  _random: Random
  mut _consumedChoices: Int = 0

  @constructor
  new(random: Random) {
    _random = random
  }

  consumedChoices -> Int { _consumedChoices }
}

impl ChoiceProvider for SystemRandomChoiceProvider {
  choose(request: Requests.ChoiceRequest) -> Choices.Choice {
    _consumedChoices += 1

    match request {
      Requests.ChoiceRequest::Integer(min, max, target) => {
        if min > max or target < min or target > max {
          throw Error.new("invalid integer choice request")
        }

        Choices.Choice::Integer(_random.nextIntIn(min, max), min, max, target)
      }

      Requests.ChoiceRequest::Boolean(target) =>
        Choices.Choice::Boolean(_random.nextIntIn(0, 1) == 1, target)

      Requests.ChoiceRequest::Index(size, target) => {
        if size <= 0 or target < 0 or target >= size {
          throw Error.new("invalid index choice request")
        }

        Choices.Choice::Index(_random.nextIntIn(0, size - 1), size, target)
      }

      Requests.ChoiceRequest::Bytes(minSize, maxSize, target) => {
        if minSize < 0 or minSize > maxSize or
          target.size < minSize or target.size > maxSize {
          throw Error.new("invalid byte choice request")
        }

        let length = _random.nextIntIn(minSize, maxSize)
        let bytes = Bytes.zeroed(length)
        let mut index = 0

        while index < length {
          bytes[index] = _random.nextIntIn(0, 255)
          index += 1
        }

        Choices.Choice::Bytes(bytes, minSize, maxSize, target).copy
      }
    }
  }
}

/// Supplies normalized choices from a fixed tape.
class ScriptedChoiceProvider {
  _script: List<Choices.Choice>
  mut _cursor: Int = 0

  @constructor
  new(script: List<Choices.Choice>) {
    _script = ProviderCopies.choices(script)
  }

  consumedChoices -> Int { _cursor }
}

impl ChoiceProvider for ScriptedChoiceProvider {
  choose(request: Requests.ChoiceRequest) -> Choices.Choice {
    if _cursor >= _script.size {
      throw Error.new("scripted provider ran out of choices")
    }

    let source = _script[_cursor]
    _cursor += 1

    ChoiceNormalization.normalize(request, source)
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

  consumedChoices -> Int { _cursor }
}

impl ChoiceProvider for ReplayChoiceProvider {
  choose(request: Requests.ChoiceRequest) -> Choices.Choice {
    if _cursor >= _example.size {
      throw Error.new("replay tape ended before the draw sequence completed")
    }

    let source = _example.at(_cursor)
    _cursor += 1

    ChoiceNormalization.normalize(request, source)
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

impl ChoiceProviderFactory for SystemRandomChoiceProviderFactory {
  create(exampleIndex: Int) -> ChoiceProvider {
    // Each example gets a repeatable stream independent of earlier consumption.
    SystemRandomChoiceProvider.new(
      random: Random.new(seed: _seed + exampleIndex)
    )
  }
}

/// Reuses the supplied scripted tape for each example.
class ScriptedChoiceProviderFactory {
  _script: List<Choices.Choice>

  @constructor
  new(script: List<Choices.Choice>) {
    _script = ProviderCopies.choices(script)
  }
}

impl ChoiceProviderFactory for ScriptedChoiceProviderFactory {
  /// Creates a provider with an independent cursor over the stored script.
  create(exampleIndex: Int) -> ChoiceProvider {
    ScriptedChoiceProvider.new(script: _script)
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

            Choices.Choice::Integer(value, min, max, target)
          }

          _ => {
            throw Error.new("choice kind does not match its request")
          }
        }

      Requests.ChoiceRequest::Boolean(target) =>
        match source {
          Choices.Choice::Boolean(value, _) =>
            Choices.Choice::Boolean(value, target)

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

            Choices.Choice::Index(value, size, target)
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

            Choices.Choice::Bytes(value, minSize, maxSize, target).copy
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

    for value in values {
      copied.add(value.copy)
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

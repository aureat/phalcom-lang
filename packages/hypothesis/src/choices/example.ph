import .choice as Choices

/// A replayable, immutable snapshot of the choices consumed by one example.
class Example {
  _choices: List<Choices.Choice>

  @constructor
  new(choices: List<Choices.Choice>) {
    // Keep a private deep copy so mutating the source list or byte values
    // cannot change a previously recorded example.
    _choices = ExampleCopies.choices(choices)
  }

  /// Number of primitive choices in this example.
  size -> Int { _choices.size }

  /// Returns an isolated copy of the choice at `index`.
  at(index: Int) -> Choices.Choice { _choices[index].copy }

  /// Returns an isolated copy of the full choice tape.
  choices -> List<Choices.Choice> { ExampleCopies.choices(_choices) }

  /// Returns a new example with one choice replaced.
  replace(index: Int, choice: Choices.Choice) -> Example {
    let mut next = ExampleCopies.choices(_choices)
    next[index] = choice.copy

    Example.new(choices: next)
  }
}

class ExampleCopies {
  @class
  choices(values: List<Choices.Choice>) -> List<Choices.Choice> {
    let mut copied = List.new()

    for value in values {
      copied.add(value.copy)
    }

    copied
  }
}

export Example

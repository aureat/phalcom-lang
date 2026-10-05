import .choice as Choices
import .example as Examples

/// Mutable choice storage owned by one generation or replay attempt.
class ChoiceBuffer {
  mut _choices: List<Choices.Choice> = List.new()

  /// Number of choices currently stored.
  size -> Int { _choices.size }

  /// Appends an isolated copy of a choice to the active tape.
  append(_ choice: Choices.Choice) -> None {
    _choices.add(choice.copy)
  }

  /// Freezes the current choices into an independent example snapshot.
  snapshot -> Examples.Example {
    Examples.Example.new(choices: _choices)
  }
}

export ChoiceBuffer

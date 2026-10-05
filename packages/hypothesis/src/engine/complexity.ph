import ..choices.example as Examples
import ..choices.choice as Choices

/// The lexicographic size of an example tape.
data ExampleComplexity(choiceCount: Int, magnitude: Int)

impl ExampleComplexity {
  /// Measures choice count first, then distance from each shrink target.
  @class
  of(example: Examples.Example) -> ExampleComplexity {
    let mut magnitude = 0
    let choices = example.choices
    let mut index = 0

    while index < choices.size {
      let choice = choices[index]

      match choice {
        Choices.Choice::Integer(value, _, _, target) => {
          let distance = value - target

          if distance < 0 {
            magnitude += 0 - distance
          } else {
            magnitude += distance
          }
        }

        Choices.Choice::Boolean(value, target) => {
          if value != target {
            magnitude += 1
          }
        }

        Choices.Choice::Index(value, _, target) => {
          let distance = value - target

          if distance < 0 {
            magnitude += 0 - distance
          } else {
            magnitude += distance
          }
        }

        Choices.Choice::Bytes(value, _, _, target) => {
          // Byte contents are opaque for now, so compare their total lengths.
          magnitude += value.size + target.size
        }
      }

      index += 1
    }

    ExampleComplexity(choiceCount: example.size, magnitude: magnitude)
  }

  /// Orders examples by choice count and then by total shrink distance.
  lessThan(other: ExampleComplexity) -> Bool {
    choiceCount < other.choiceCount or
      (choiceCount == other.choiceCount and magnitude < other.magnitude)
  }
}

export ExampleComplexity

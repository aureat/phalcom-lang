import ..choices.choice as Choices

/// Proposes simpler replacements for one primitive choice.
trait ShrinkPass {
  candidates(choice: Choices.Choice) -> List<Choices.Choice>
}

/// Moves integer-like and boolean choices directly toward their targets.
class IntegerShrinkPass {
}

impl ShrinkPass for IntegerShrinkPass {
  /// Returns the shrink target when it differs from the current choice.
  candidates(choice: Choices.Choice) -> List<Choices.Choice> {
    let mut result = List.new()

    match choice {
      Choices.Choice::Integer(value, min, max, target) => {
        if value != target {
          result.add(Choices.Choice::Integer(target, min, max, target))
        }
      }

      Choices.Choice::Index(value, size, target) => {
        if value != target {
          result.add(Choices.Choice::Index(target, size, target))
        }
      }

      Choices.Choice::Boolean(value, target) => {
        if value != target {
          result.add(Choices.Choice::Boolean(target, target))
        }
      }

      Choices.Choice::Bytes(_, _, _, _) => ()
    }

    result
  }
}

export (ShrinkPass, IntegerShrinkPass)

import ..choices.choice as Choices
from ..choices.choice import Choice

/// Simplifies integer-like choices toward their targets in deterministic order.
class IntegerShrinkPass {
  integerCandidates(value: Int, min: Int, max: Int, target: Int) -> List<Int> {
    let mut result = List.new()
    self.appendCandidate(
      values: result,
      candidate: target,
      value: value,
      min: min,
      max: max,
      target: target
    )

    // Small offsets find common threshold boundaries without a long search.
    let mut offset = 1
    while offset <= 16 {
      self.appendCandidate(
        values: result,
        candidate: target + offset,
        value: value,
        min: min,
        max: max,
        target: target
      )
      self.appendCandidate(
        values: result,
        candidate: target - offset,
        value: value,
        min: min,
        max: max,
        target: target
      )
      offset += 1
    }

    // Halving probes make progress through large domains. Repeated shrink
    // passes then try the same small offsets around the remaining boundary.
    let mut probe = value
    while probe - target > 1 or target - probe > 1 {
      probe = target + ((probe - target) ~/ 2)
      self.appendCandidate(
        values: result,
        candidate: probe,
        value: value,
        min: min,
        max: max,
        target: target
      )
    }

    result
  }

  appendCandidate(
    values: List<Int>,
    candidate: Int,
    value: Int,
    min: Int,
    max: Int,
    target: Int
  ) -> () {
    let movesCloser =
      (value > target and candidate >= target and candidate < value) or
      (value < target and candidate <= target and candidate > value)

    if candidate >= min and candidate <= max and movesCloser {
      let mut index = 0
      let mut alreadyPresent = false
      while index < values.size and not alreadyPresent {
        alreadyPresent = values[index] == candidate
        index += 1
      }

      if not alreadyPresent {
        values.append(candidate)
      }
    }
  }

  /// Returns ordered, in-domain simplifications for integer-like choices.
  candidates(choice: Choices.Choice) -> List<Choices.Choice> {
    let mut result = List.new()

    match choice {
      Choices.Choice::Integer(value, min, max, target) => {
        let values = self.integerCandidates(
          value: value,
          min: min,
          max: max,
          target: target
        )
        let mut index = 0
        while index < values.size {
          result.append(Choice::Integer(values[index], min, max, target))
          index += 1
        }
      }

      Choices.Choice::Index(value, size, target) => {
        let values = self.integerCandidates(
          value: value,
          min: 0,
          max: size - 1,
          target: target
        )
        let mut index = 0
        while index < values.size {
          result.append(Choice::Index(values[index], size, target))
          index += 1
        }
      }

      Choices.Choice::Boolean(value, target) => {
        if value != target {
          result.append(Choice::Boolean(target, target))
        }
      }

      Choices.Choice::Bytes(_, _, _, _) => ()
    }

    result
  }
}

export IntegerShrinkPass

// Total, deterministic complexity ordering for immutable semantic examples.

import Example from "choices/example"
import Choice from "choices/choice"
import ordering from "_internal/ordering"
import fingerprints from "_internal/fingerprints"

@data
@immutable
class ExampleComplexity {
  const _choiceCount: Int
  const _structuralWeight: Int
  const _choiceWeight: Int
  const _signature: String

  @class
  of(example: Example) -> ExampleComplexity {
    let structuralWeight = example.generationSize
    for span in example.spans {
      structuralWeight += span.length
      if span.discardable {
        structuralWeight++
      }
    }

    let choiceWeight = 0
    for choice in example.choices {
      choiceWeight += _ComplexityWeights.choice(choice)
    }

    return ExampleComplexity.new(
      choiceCount: example.size,
      structuralWeight: structuralWeight,
      choiceWeight: choiceWeight,
      signature: fingerprints._Fingerprints.example(example)
    )
  }

  lessThan(other: ExampleComplexity) -> Bool {
    const choiceOrder = ordering._Ordering.integers(
      _choiceCount,
      other.choiceCount
    )
    if choiceOrder != 0 {
      return choiceOrder < 0
    }

    const structuralOrder = ordering._Ordering.integers(
      _structuralWeight,
      other.structuralWeight
    )
    if structuralOrder != 0 {
      return structuralOrder < 0
    }

    const valueOrder = ordering._Ordering.integers(
      _choiceWeight,
      other.choiceWeight
    )
    if valueOrder != 0 {
      return valueOrder < 0
    }

    return ordering._Ordering.strings(_signature, other.signature) < 0
  }
}

class _ComplexityWeights {
  @class
  choice(choice: Choice) -> Int {
    return choice.match(
      integer: { value => self.abs(value.value - value.shrinkTowards) },
      boolean: { value =>
        if value.value == value.shrinkTowards {
          return 0
        }
        return 1
      },
      index: { value => self.abs(value.value - value.shrinkTowards) },
      bytes: { value => self.bytes(value.value, target: value.shrinkTowards) }
    )
  }

  @class
  bytes(value: Bytes, target: Bytes) -> Int {
    let weight = self.abs(value.size - target.size) * 257
    let index = 0
    while index < value.size {
      weight += value[index]
      index++
    }
    return weight
  }

  @class
  abs(value: Int) -> Int {
    if value < 0 {
      return 0 - value
    }
    return value
  }
}

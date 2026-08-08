// DrawData is the sole strategy-facing access point to primitive choices.
// It records normalized choices and semantic spans in a mutable buffer, then
// exposes an immutable Example.

import Choice from "choices/choice"
import ChoiceRequest from "choices/request"
import Example from "choices/example"
import ChoiceBuffer from "choices/buffer"
import ChoiceProvider from "choices/provider"
import providers from "choices/provider"
import ExampleStatus from "core/status"
import Failure from "core/failure"
import errors from "core/errors"

class DrawData {
  @constructor
  @requires(generationSize >= 0)
  @requires(maxChoices > 0)
  new(
    provider: ChoiceProvider,
    generationSize: Int,
    maxChoices: Int
  ) {
    _provider = provider
    _buffer = ChoiceBuffer.new(generationSize: generationSize)
    _maxChoices = maxChoices
    _rejectionCount = 0
    _rejectionReasons = List.new()
    _labelStack = List.new()
    _sizeStack = List.new()
    _sizeStack.add(generationSize)
  }

  @class
  generate(provider: ChoiceProvider, generationSize: Int, maxChoices: Int) -> DrawData {
    return DrawData.new(
      provider: provider,
      generationSize: generationSize,
      maxChoices: maxChoices
    )
  }

  @class
  generate(
    random: Random,
    generationSize: Int,
    maxChoices: Int
  ) -> DrawData {
    return self.generate(
      provider: providers.SystemRandomChoiceProvider.new(random),
      generationSize: generationSize,
      maxChoices: maxChoices
    )
  }

  @class
  replay(example: Example, maxChoices: Int) -> DrawData {
    return DrawData.new(
      provider: providers._ReplayChoiceProvider.new(example),
      generationSize: example.generationSize,
      maxChoices: maxChoices
    )
  }

  generationSize -> Int => _buffer.generationSize
  size -> Int => _sizeStack.at(_sizeStack.size - 1)
  consumedChoices -> Int => _provider.consumedChoices
  rejectionCount -> Int => _rejectionCount

  rejectionReasons -> List<Any> {
    const copied = List.new()
    for reason in _rejectionReasons {
      copied.add(reason)
    }
    return copied
  }
  example -> Example => _buffer.freeze
  tape -> Example => self.example

  draw(request: ChoiceRequest) -> Any {
    if _buffer.size >= _maxChoices {
      throw errors._ChoiceBudgetExceeded.new(
        "generated example exceeded the configured choice budget"
      )
    }
    return _buffer.add(_provider.choose(request)).value
  }

  drawInt(
    min: Int,
    max: Int,
    shrinkTowards: Int,
    label: Option<Symbol>
  ) -> Int {
    return self.draw(
      ChoiceRequest.integer(
        min: min,
        max: max,
        shrinkTowards: shrinkTowards,
        label: self.resolvedLabel(label)
      )
    )
  }

  // Compatibility arity for the temporary strategy layer.
  drawInt(min: Int, max: Int, label: Option<Symbol>) -> Int {
    let target = 0
    if min > 0 {
      target = min
    }
    if max < 0 {
      target = max
    }
    return self.drawInt(
      min: min,
      max: max,
      shrinkTowards: target,
      label: label
    )
  }

  drawBool(
    shrinkTowards: Bool,
    label: Option<Symbol>
  ) -> Bool {
    return self.draw(
      ChoiceRequest.boolean(
        shrinkTowards: shrinkTowards,
        label: self.resolvedLabel(label)
      )
    )
  }

  drawIndex(
    size: Int,
    shrinkTowards: Int,
    label: Option<Symbol>
  ) -> Int {
    return self.draw(
      ChoiceRequest.index(
        size: size,
        shrinkTowards: shrinkTowards,
        label: self.resolvedLabel(label)
      )
    )
  }

  drawBytes(
    minSize: Int,
    maxSize: Int,
    shrinkTowards: Bytes,
    label: Option<Symbol>
  ) -> Bytes {
    return self.draw(
      ChoiceRequest.bytes(
        minSize: minSize,
        maxSize: maxSize,
        shrinkTowards: shrinkTowards,
        label: self.resolvedLabel(label)
      )
    )
  }

  recordRejection(reason: Any) -> None {
    _rejectionCount++
    _rejectionReasons.add(reason)
    return None
  }

  withLabel(label: Symbol, body: Block) -> Any {
    _labelStack.add(label)
    return || {
      body.call()
    }.ensure || {
      _labelStack.removeAt(_labelStack.size - 1)
    }
  }

  withGenerationSize(value: Int, body: Block) -> Any {
    if value < 0 {
      throw errors._InvalidStrategy.new(
        "strategy generation size cannot be negative"
      )
    }
    _sizeStack.add(value)
    return || {
      body.call()
    }.ensure || {
      _sizeStack.removeAt(_sizeStack.size - 1)
    }
  }

  resolvedLabel(label: Option<Symbol>) -> Option<Symbol> {
    if label.isSome or _labelStack.size == 0 {
      return label
    }
    return Some.new(_labelStack.at(_labelStack.size - 1))
  }


  withSpan(label: Symbol, discardable: Bool, body: Block) -> Any {
    return _buffer.withSpan(
      label: label,
      discardable: discardable,
      body: body
    )
  }

  attempt(body: [DrawData] -> Any) -> ExampleStatus {
    const outcome = || {
      body.call(self)
    }.attempt()

    const completed = self.example
    if outcome.isOk || {
      return ExampleStatus.valid(
        example: completed,
        arguments: const [outcome.unwrap],
        context: None
      )
    }

    const error = outcome.unwrapErr
    if error.isA(errors._RejectedExample) {
      return ExampleStatus.invalid(
        reason: error,
        example: completed,
        arguments: const [],
        context: None
      )
    }

    if error.isA(errors._EngineOverrun) {
      return ExampleStatus.overrun(
        reason: error,
        example: completed,
        arguments: const [],
        context: None
      )
    }

    return ExampleStatus.interesting(
      failure: Failure.from(error, completed, const []),
      example: completed,
      arguments: const [],
      context: None
    )
  }
}

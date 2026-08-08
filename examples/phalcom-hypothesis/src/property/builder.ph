// Explicit property API and fluent builder.

import Settings from "core/settings"
import PropertyResult from "core/status"
import coreContext from "core/context"
import errors from "core/errors"
import Strategy from "strategies/strategy"
import engineSpec from "engine/specification"
import engineSearch from "engine/engine"
import target from "property/target"
import assertion from "property/assertion"

class Property {
  @class
  current -> Any {
    return coreContext._propertyContexts.current.match(
      some: |context| { context },
      none: |_| {
        throw errors._MissingPropertyContext.new(
          "Property context used outside a running example"
        )
      }
    )
  }

  @class
  assume(condition: Bool) -> None {
    if not condition {
      throw errors._RejectedExample.new("assumption was false")
    }
  }

  @class
  note(value: Any) -> None {
    Property.current.note(value)
  }

  @class
  event(label: Symbol) -> None {
    Property.current.event(label)
  }

  @class
  classify(condition: Bool, as: Symbol) -> None {
    if condition {
      Property.event(as)
    }
  }

  @class
  given<T...>(*strategies: Strategy<Any>) -> PropertyBuilder<T...> {
    return PropertyBuilder.new(
      strategies: strategies,
      settings: Settings.standard
    )
  }

  @class
  forAll(*parts: Any) -> PropertyResult {
    if parts.size == 0 {
      throw errors.PropertyDiscoveryError.new(
        "Property.forAll requires strategies followed by a block"
      )
    }

    const body = parts.at(parts.size - 1)
    if not body.isA(Block) {
      throw errors.PropertyDiscoveryError.new(
        "Property.forAll final argument must be a block"
      )
    }

    const strategies = List.new()
    let index = 0
    while index < parts.size - 1 {
      const strategy = parts.at(index)
      if not strategy.respondsTo(#draw) {
        throw errors.PropertyDiscoveryError.new(
          "Property.forAll arguments before the block must be Strategy values"
        )
      }
      strategies.add(strategy)
      index++
    }

    return Property.given(*strategies).check(body)
  }

  @class
  find<T>(strategy: Strategy<T>, predicate: [T] -> Bool) -> Option<T> {
    return engineSearch.SearchEngine.new().find(
      strategy: strategy,
      predicate: predicate,
      settings: Settings.standard.examples(1000),
      reuseExamples: const []
    )
  }
}

@immutable
class PropertyBuilder<T...> {
  const _strategies: List<Strategy<Any>>
  const _settings: Settings

  @constructor
  new(strategies: List<Strategy<Any>>, settings: Settings) {
    _strategies = _BuilderCopies.list(strategies)
    _settings = settings
  }

  using(settings: Settings) -> PropertyBuilder<T...> {
    return PropertyBuilder.new(
      strategies: _strategies,
      settings: settings
    )
  }

  check(body: Block) -> PropertyResult {
    const spec = engineSpec.PropertySpec.check(
      id: #dynamicProperty,
      target: target._BlockTarget.new(body),
      strategies: _strategies,
      explicitExamples: const [],
      reuseExamples: const [],
      parameterNames: const [],
      settings: _settings
    )
    return engineSearch.SearchEngine.new().check(spec)
  }
}

class PropertySuite {
  assertEqual(expected: Any, actual: Any) -> None {
    assertion.Assert.equalAt(
      expected: expected,
      actual: actual,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }

  assertTrue(condition: Bool) -> None {
    assertion.Assert.trueAt(
      condition: condition,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }

  assertFalse(condition: Bool) -> None {
    assertion.Assert.falseAt(
      condition: condition,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }

  fail(message: String) -> None {
    assertion.Assert.failAt(
      message: message,
      location: SourceLocation.caller(skip: 1),
      label: None
    )
  }
}

class _BuilderCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}

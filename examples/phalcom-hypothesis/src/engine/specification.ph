// Immutable descriptions consumed by the Phase 05 search kernel.

import Settings from "core/settings"
import Strategy from "strategies/strategy"
import Example from "choices/example"

@data
@immutable
class PropertySpec<T...> {
  const _id: Any
  const _target: Any
  const _strategies: List<Strategy<Any>>
  const _explicitExamples: List<List<Any>>
  const _reuseExamples: List<Example>
  const _parameterNames: List<Symbol>
  const _settings: Settings

  @class
  check(
    id: Any,
    target: Any,
    strategies: List<Strategy<Any>>,
    explicitExamples: List<List<Any>>,
    reuseExamples: List<Example>,
    settings: Settings
  ) -> PropertySpec<T...> {
    return self.check(
      id: id,
      target: target,
      strategies: strategies,
      explicitExamples: explicitExamples,
      reuseExamples: reuseExamples,
      parameterNames: const [],
      settings: settings
    )
  }

  @class
  check(
    id: Any,
    target: Any,
    strategies: List<Strategy<Any>>,
    explicitExamples: List<List<Any>>,
    reuseExamples: List<Example>,
    parameterNames: List<Symbol>,
    settings: Settings
  ) -> PropertySpec<T...> {
    return PropertySpec.new(
      id: id,
      target: target,
      strategies: _SpecCopies.list(strategies),
      explicitExamples: _SpecCopies.nested(explicitExamples),
      reuseExamples: _SpecCopies.list(reuseExamples),
      parameterNames: _SpecCopies.list(parameterNames),
      settings: settings
    )
  }

  findMode -> Bool => false
  name -> Any => _id
  config -> Settings => _settings
}

@data
@immutable
class _FindSpec<T> {
  const _strategy: Strategy<T>
  const _predicate: [T] -> Bool
  const _reuseExamples: List<Example>
  const _parameterNames: List<Symbol>
  const _settings: Settings

  @class
  create(
    strategy: Strategy<T>,
    predicate: [T] -> Bool,
    reuseExamples: List<Example>,
    settings: Settings
  ) -> _FindSpec<T> {
    return _FindSpec.new(
      strategy: strategy,
      predicate: predicate,
      reuseExamples: _SpecCopies.list(reuseExamples),
      settings: settings
    )
  }

  id -> Symbol => #find
  target -> Any => None
  strategies -> List<Strategy<Any>> => const [_strategy]
  explicitExamples -> List<List<Any>> => const []
  parameterNames -> List<Symbol> => const [#value]
  findMode -> Bool => true
}

class _SpecCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }

  @class
  nested(values: List<List<Any>>) -> List<List<Any>> {
    const copied = List.new()
    for value in values {
      copied.add(self.list(value))
    }
    return copied
  }
}

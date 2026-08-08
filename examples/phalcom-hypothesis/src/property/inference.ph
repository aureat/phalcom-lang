// Reflected method parameters and @Given strategy resolution.

import Strategy from "strategies/strategy"
import StrategyRegistry from "strategies/registry"
import attributes from "property/attributes"
import errors from "core/errors"

@data
@immutable
class _ReflectedParameter {
  const _name: Symbol
  const _position: Int
  const _annotation: Option<Any>

  @class
  all(method: Method) -> List<_ReflectedParameter> {
    const reflected = List.new()
    let position = 0
    for parameter in method.parameters {
      reflected.add(
        _ReflectedParameter.new(
          name: parameter.name,
          position: position,
          annotation: parameter.type
        )
      )
      position++
    }
    return reflected
  }
}

class _StrategyInference {
  @class
  resolve(
    method: Method,
    given: attributes.Given,
    registry: StrategyRegistry,
    propertyName: String
  ) -> List<Strategy<Any>> {
    const parameters = _ReflectedParameter.all(method)

    return given.mode.match(
      inferred: |_| {
        self.inferAll(
          parameters: parameters,
          registry: registry,
          propertyName: propertyName
        )
      },
      explicit: |value| {
        self.explicit(
          strategies: value.strategies,
          parameters: parameters,
          propertyName: propertyName
        )
      },
      overrides: |value| {
        self.withOverrides(
          arguments: value.arguments,
          parameters: parameters,
          registry: registry,
          propertyName: propertyName
        )
      }
    )
  }

  @class
  parameterNames(method: Method) -> List<Symbol> {
    const names = List.new()
    for parameter in method.parameters {
      names.add(parameter.name)
    }
    return names
  }

  @class
  inferAll(
    parameters: List<_ReflectedParameter>,
    registry: StrategyRegistry,
    propertyName: String
  ) -> List<Strategy<Any>> {
    const strategies = List.new()
    for parameter in parameters {
      strategies.add(
        self.inferParameter(
          parameter: parameter,
          registry: registry,
          propertyName: propertyName
        )
      )
    }
    return strategies
  }

  @class
  explicit(
    strategies: List<Strategy<Any>>,
    parameters: List<_ReflectedParameter>,
    propertyName: String
  ) -> List<Strategy<Any>> {
    if strategies.size != parameters.size {
      throw errors.PropertyDiscoveryError.new(
        propertyName + " expected " + parameters.size.toString +
        " strategies, received " + strategies.size.toString
      )
    }
    return _InferenceCopies.list(strategies)
  }

  @class
  withOverrides(
    arguments: attributes.GivenArgs,
    parameters: List<_ReflectedParameter>,
    registry: StrategyRegistry,
    propertyName: String
  ) -> List<Strategy<Any>> {
    for name in arguments.names {
      let known = false
      for parameter in parameters {
        if parameter.name == name {
          known = true
        }
      }
      if not known {
        throw errors.PropertyDiscoveryError.new(
          propertyName + " has unknown @Given override '" +
          name.toString + "'"
        )
      }
    }

    const strategies = List.new()
    for parameter in parameters {
      arguments.strategyFor(parameter.name).match(
        some: |strategy| { strategies.add(strategy) },
        none: |_| {
          strategies.add(
            self.inferParameter(
              parameter: parameter,
              registry: registry,
              propertyName: propertyName
            )
          )
        }
      )
    }
    return strategies
  }

  @class
  inferParameter(
    parameter: _ReflectedParameter,
    registry: StrategyRegistry,
    propertyName: String
  ) -> Strategy<Any> {
    return parameter.annotation.match(
      some: |type| {
        {
          registry.forType(type)
        }.attempt().match(
          ok: |strategy| { strategy },
          error: |error| {
            throw errors.StrategyResolutionError.new(
              "cannot resolve parameter '" + parameter.name.toString +
              "' of " + propertyName + ": " +
              error.message.unwrapOr(error.toString)
            )
          }
        )
      },
      none: |_| {
        throw errors.StrategyResolutionError.new(
          "cannot resolve parameter '" + parameter.name.toString +
          "' of " + propertyName + ": no type annotation"
        )
      }
    )
  }
}

class _InferenceCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}

// Reflective discovery and validation of property methods.

import Settings from "core/settings"
import Strategy from "strategies/strategy"
import StrategyRegistry from "strategies/registry"
import engineSpec from "engine/specification"
import attributes from "property/attributes"
import inference from "property/inference"
import target from "property/target"
import errors from "core/errors"
import DatabaseKey from "database/key"
import ExampleCodec from "database/codec"

@data
@immutable
class PropertyId {
  const _package: Symbol
  const _module: Symbol
  const _suite: Symbol
  const _selector: Symbol

  @constructor
  new(module: Symbol, suite: Symbol, selector: Symbol) {
    _package = #unknown
    _module = module
    _suite = suite
    _selector = selector
  }

  @class
  from(suiteClass: Class, method: Method) -> PropertyId {
    let packageName = #unknown
    let moduleName = #unknown
    if suiteClass.respondsTo(#module) {
      const reflectedModule = suiteClass.module
      moduleName = reflectedModule.name
      if reflectedModule.respondsTo(#package) {
        packageName = reflectedModule.package.name
      }
    }
    return PropertyId.new(
      package: packageName,
      module: moduleName,
      suite: suiteClass.name,
      selector: method.selector
    )
  }

  toString -> String {
    return _suite.toString + "." + _selector.toString
  }
}

@data
@immutable
class PropertyDefinition {
  const _id: PropertyId
  const _target: Any
  const _strategies: List<Strategy<Any>>
  const _explicitExamples: List<List<Any>>
  const _parameterNames: List<Symbol>
  const _settings: Settings

  databaseKey -> DatabaseKey {
    return DatabaseKey.create(
      package: _id.package,
      module: _id.module,
      suite: _id.suite,
      selector: _id.selector,
      strategyFingerprint: _DiscoveryFingerprints.strategies(_strategies),
      engineFormatVersion: ExampleCodec.engineFormatVersion
    )
  }

  toSpec(reuseExamples: List<Any>) -> engineSpec.PropertySpec<Any> {
    return engineSpec.PropertySpec.check(
      id: _id,
      target: _target,
      strategies: _strategies,
      explicitExamples: _explicitExamples,
      reuseExamples: reuseExamples,
      parameterNames: _parameterNames,
      settings: _settings
    )
  }
}

class PropertyDiscovery {
  @class
  discover(
    suiteClass: Class,
    receiver: Any,
    defaults: Settings,
    registry: StrategyRegistry
  ) -> List<PropertyDefinition> {
    const definitions = List.new()

    for symbol in suiteClass.methods || {
      const method = receiver.methodFor(symbol)
      const givens = method.attributesOfType(attributes.Given).toList
      if givens.size > 0 {
        if givens.size != 1 {
          throw errors.PropertyDiscoveryError.new(
            suiteClass.name.toString + "." + method.selector.toString +
            " must have exactly one @Given"
          )
        }

        const id = PropertyId.from(suiteClass: suiteClass, method: method)
        const propertyName = id.toString
        const parameterNames = inference._StrategyInference.parameterNames(method)
        const strategies = inference._StrategyInference.resolve(
          method: method,
          given: givens.at(0),
          registry: registry,
          propertyName: propertyName
        )
        const explicitExamples = self.explicitExamples(
          method: method,
          arity: parameterNames.size,
          propertyName: propertyName
        )
        const settings = self.settingsFor(
          method: method,
          defaults: defaults,
          propertyName: propertyName
        )

        definitions.add(
          PropertyDefinition.new(
            id: id,
            target: target._MethodTarget.new(
              method: method,
              receiver: receiver
            ),
            strategies: _DiscoveryCopies.list(strategies),
            explicitExamples: _DiscoveryCopies.nested(explicitExamples),
            parameterNames: _DiscoveryCopies.list(parameterNames),
            settings: settings
          )
        )
      }
    }

    return definitions
  }

  @class
  explicitExamples(
    method: Method,
    arity: Int,
    propertyName: String
  ) -> List<List<Any>> {
    const examples = List.new()
    for example in method.attributesOfType(attributes.Case) {
      if example.values.size != arity {
        throw errors.PropertyDiscoveryError.new(
          propertyName + " @Case expected " + arity.toString +
          " values, received " + example.values.size.toString
        )
      }
      examples.add(example.values)
    }
    return examples
  }

  @class
  settingsFor(
    method: Method,
    defaults: Settings,
    propertyName: String
  ) -> Settings {
    const local = method.attributesOfType(attributes.WithSettings).toList
    if local.size > 1 {
      throw errors.PropertyDiscoveryError.new(
        propertyName + " may have at most one @WithSettings"
      )
    }
    if local.size == 0 {
      return defaults
    }

    const configured = local.at(0).settings
    if configured.databaseValue.isNone and defaults.databaseValue.isSome || {
      return configured.withDatabase(defaults.databaseValue.unwrap)
    }
    return configured
  }
}

class _DiscoveryFingerprints {
  @class
  strategies(values: List<Strategy<Any>>) -> String {
    let out = ""
    for strategy in values {
      const fingerprint = strategy.fingerprint
      out += fingerprint.codePoints.size.toString + ":" + fingerprint + "|"
    }
    return out
  }
}

class _DiscoveryCopies {
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

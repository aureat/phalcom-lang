// Reflective type-to-strategy registry with exact/custom/derived precedence.

import Strategy from "strategies/strategy"
import Gen from "strategies/gen"
import attributes from "strategies/attributes"
import derivation from "strategies/derivation"
import errors from "core/errors"

class StrategyRegistry {
  @constructor
  new() {
    _entries = Map.new()
    _derived = Map.new()
    _resolving = Map.new()
  }

  @class
  standard -> StrategyRegistry {
    const registry = StrategyRegistry.new()
    registry.register(type: Int, strategy: Gen.int)
    registry.register(type: Bool, strategy: Gen.bool)
    registry.register(type: Float, strategy: Gen.float)
    registry.register(type: Bytes, strategy: Gen.bytes)
    registry.register(type: String, strategy: Gen.text)
    return registry
  }

  register(type: Any, strategy: Strategy<Any>) -> StrategyRegistry {
    if not strategy.respondsTo(#draw) {
      throw errors.StrategyResolutionError.new(
        "registered value for " + type.toString + " is not a Strategy"
      )
    }
    _entries.at(type, put: strategy)
    if _derived.at(type) != None {
      _derived.remove(type)
    }
    return self
  }

  // Compatibility spelling retained for Phase 04 callers.
  register(type: Any, use: Strategy<Any>) -> StrategyRegistry {
    return self.register(type: type, strategy: use)
  }

  // Install passive @strategy(Type) provider methods. Installation
  // is explicit; the attribute does not mutate a global registry by itself.
  register(provider: Class) -> StrategyRegistry {
    const receiver = provider.new()
    const declaredTargets = Map.new()
    const selectors = provider.methods.toList.sorted { left, right =>
      left.toString < right.toString
    }
    for selector in selectors {
      const method = receiver.methodFor(selector)
      const declarations = method.attributesOfType(attributes.strategy).toList
      if declarations.size > 1 {
        throw errors.StrategyResolutionError.new(
          provider.name.toString + "." + method.selector.toString +
          " has duplicate @strategy declarations"
        )
      }
      if declarations.size == 1 {
        if method.parameters.toList.size != 0 {
          throw errors.StrategyResolutionError.new(
            "@strategy provider " + provider.name.toString + "." +
            method.selector.toString + " must have no parameters"
          )
        }
        const supplied = method.invokeOn(receiver, const [])
        if not supplied.respondsTo(#draw) {
          throw errors.StrategyResolutionError.new(
            "@strategy provider " + provider.name.toString + "." +
            method.selector.toString + " did not return a Strategy"
          )
        }
        const targetType = declarations.at(0).targetType
        if declaredTargets.at(targetType) != None {
          throw errors.StrategyResolutionError.new(
            provider.name.toString + " has duplicate @strategy providers for " +
            targetType.toString
          )
        }
        declaredTargets.at(targetType, put: true)
        self.register(type: targetType, strategy: supplied)
      }
    }
    return self
  }

  forType(type: Any) -> Strategy<Any> {
    const path = List.new()
    path.add(type.toString)
    return self.resolve(type: type, path: path)
  }

  @private
  resolve(type: Any, path: List<String>) -> Strategy<Any> {
    const exact = _entries.at(type)
    if exact != None {
      return exact
    }

    const cached = _derived.at(type)
    if cached != None {
      return cached
    }

    const applied = self.applied(type: type, path: path)
    if applied.isSome {
      return applied.unwrap
    }

    if _resolving.at(type) != None {
      throw errors.StrategyResolutionError.new(
        "unexpected eager derivation cycle for " + type.toString +
        "; resolution path: " + path.join(" -> ")
      )
    }

    _resolving.at(type, put: true)
    return {
      const generated = derivation._Derivation.derive(
        type: type,
        registry: self,
        path: path
      )
      _derived.at(type, put: generated)
      generated
    }.ensure {
      _resolving.remove(type)
    }
  }

  applied(type: Any, path: List<String>) -> Option<Strategy<Any>> {
    if not type.respondsTo(#origin) or not type.respondsTo(#arguments) {
      return None
    }

    const origin = type.origin
    const arguments = type.arguments

    if origin == Option {
      self.requireArity(type: type, arguments: arguments, expected: 1, path: path)
      return Some.new(
        Gen.option(
          self.resolve(
            type: arguments.at(0),
            path: self.extend(path: path, segment: "Option value " + arguments.at(0).toString)
          )
        )
      )
    }

    if origin == List {
      self.requireArity(type: type, arguments: arguments, expected: 1, path: path)
      return Some.new(
        Gen.list(
          of: self.resolve(
            type: arguments.at(0),
            path: self.extend(path: path, segment: "List element " + arguments.at(0).toString)
          )
        )
      )
    }

    if origin == Tuple {
      const strategies = List.new()
      let index = 0
      for argument in arguments {
        strategies.add(
          self.resolve(
            type: argument,
            path: self.extend(
              path: path,
              segment: "Tuple argument " + index.toString + " " + argument.toString
            )
          )
        )
        index++
      }
      return Some.new(Gen.tuple(*strategies))
    }

    if origin == Set {
      self.requireArity(type: type, arguments: arguments, expected: 1, path: path)
      return Some.new(
        Gen.set(
          of: self.resolve(
            type: arguments.at(0),
            path: self.extend(path: path, segment: "Set element " + arguments.at(0).toString)
          )
        )
      )
    }

    if origin == Map {
      self.requireArity(type: type, arguments: arguments, expected: 2, path: path)
      return Some.new(
        Gen.map(
          keys: self.resolve(
            type: arguments.at(0),
            path: self.extend(path: path, segment: "Map key " + arguments.at(0).toString)
          ),
          values: self.resolve(
            type: arguments.at(1),
            path: self.extend(path: path, segment: "Map value " + arguments.at(1).toString)
          )
        )
      )
    }

    if origin == Result {
      self.requireArity(type: type, arguments: arguments, expected: 2, path: path)
      return Some.new(
        Gen.result(
          ok: self.resolve(
            type: arguments.at(0),
            path: self.extend(path: path, segment: "Result ok " + arguments.at(0).toString)
          ),
          error: self.resolve(
            type: arguments.at(1),
            path: self.extend(path: path, segment: "Result error " + arguments.at(1).toString)
          )
        )
      )
    }

    return None
  }

  includes(type: Any) -> Bool {
    return {
      self.forType(type)
      true
    }.attempt().match(
      ok: { value => value },
      error: { _ => false }
    )
  }

  requireArity(
    type: Any,
    arguments: List<Any>,
    expected: Int,
    path: List<String>
  ) -> None {
    if arguments.size != expected {
      throw errors.StrategyResolutionError.new(
        type.toString + " expected " + expected.toString +
        " type arguments, received " + arguments.size.toString +
        "; resolution path: " + path.join(" -> ")
      )
    }
  }

  extend(path: List<String>, segment: String) -> List<String> {
    const copied = List.new()
    for item in path {
      copied.add(item)
    }
    copied.add(segment)
    return copied
  }
}
